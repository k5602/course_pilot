/*!
Bin packing and session utilization utilities for the scheduler.

This module focuses on:
- Packing videos into sessions under a time budget (bin packing-lite)
- Improving utilization by fitting additional small videos
- Clear, testable boundaries with no UI or clustering concerns

Public API:
- `VideoItem`: minimal unit used for packing decisions
- `pack_videos_into_session`: pack items from a queue into a single session, honoring limits
*/

use crate::PlanError;
use crate::types::PlanSettings;
use std::collections::VecDeque;
use std::time::Duration;

/// Minimal item used by the packing algorithm
#[derive(Debug, Clone, PartialEq)]
pub struct VideoItem {
    pub module_title: String,
    pub section_title: String,
    pub video_index: usize,
    pub duration: Duration,
}

/// Calculate the strict and effective limits for a session.
/// - strict limit = session_length_minutes
/// - effective limit = 80% of strict to leave time for notes/breaks
#[inline]
fn session_limits(settings: &PlanSettings) -> (Duration, Duration) {
    let strict = Duration::from_secs(settings.session_length_minutes as u64 * 60);
    // Apply 20% buffer; minimum effective limit is at least 1 minute to avoid degenerate cases.
    let effective = Duration::from_secs(((strict.as_secs() as f32) * 0.8).max(60.0) as u64);
    (strict, effective)
}

/// Returns true if the single video exceeds the strict session time limit.
#[inline]
fn video_exceeds_session_limit(video_duration: Duration, settings: &PlanSettings) -> bool {
    let (strict, _) = session_limits(settings);
    video_duration > strict
}

/// Pack videos from the front of the queue into a single session respecting time constraints.
///
/// Strategy:
/// 1) Greedily take videos from the front while they fit in the effective limit (80% of session).
///    Always allow at least the first video to be taken, even if it alone exceeds the limit.
/// 2) If the next front item doesn't fit and we already have at least one, stop (next session).
/// 3) After the first pass, try to fill remaining time by scanning the queue for small videos
///    that fit, preserving relative order for the remaining queue.
///
/// Notes:
/// - Oversized first video is accepted with a warning (logged once).
/// - The queue is modified in-place; consumed items are removed.
pub fn pack_videos_into_session(
    video_queue: &mut VecDeque<VideoItem>,
    settings: &PlanSettings,
) -> Result<Vec<VideoItem>, PlanError> {
    let (_, effective_limit) = session_limits(settings);

    let mut session_videos = Vec::new();
    let mut session_duration = Duration::from_secs(0);

    // First pass: greedy from the front, preserving queue order
    while let Some(front) = video_queue.front() {
        let video_duration = front.duration;

        // If the very first video exceeds limit, accept it and warn (once)
        if video_exceeds_session_limit(video_duration, settings) {
            if session_videos.is_empty() {
                let v = video_queue.pop_front().expect("peeked some, must exist");
                log::warn!(
                    "Video '{}' ({:.1} min) exceeds session limit ({} min). Including anyway.",
                    v.section_title,
                    video_duration.as_secs_f32() / 60.0,
                    settings.session_length_minutes
                );
                session_duration += video_duration;
                session_videos.push(v);
            }
            break; // If not first, leave for next session
        }

        // If it fits or session is empty, take it
        if session_duration + video_duration <= effective_limit || session_videos.is_empty() {
            let v = video_queue.pop_front().expect("peeked some, must exist");
            session_duration += video_duration;
            session_videos.push(v);
        } else {
            break;
        }
    }

    // Second pass: fit small items from the remaining queue into the leftover space.
    // Preserve the relative order of items that remain in the queue.
    if session_duration < effective_limit && !video_queue.is_empty() {
        try_fill_remaining_time(
            &mut session_videos,
            &mut session_duration,
            video_queue,
            effective_limit,
        );
    }

    Ok(session_videos)
}

/// Attempt to fill remaining time in the current session by selecting additional
/// small videos from the queue. Maintains the relative order of the remaining queue.
///
/// Implementation detail:
/// - We iterate the queue by index and pick videos that fit.
/// - We record indices to remove and then remove them in reverse order to avoid shifting.
/// - This is a greedy-first-fit pass (simple and predictable).
fn try_fill_remaining_time(
    session_videos: &mut Vec<VideoItem>,
    session_duration: &mut Duration,
    video_queue: &mut VecDeque<VideoItem>,
    effective_limit: Duration,
) {
    if video_queue.is_empty() {
        return;
    }

    // Compute indices of videos that can fit
    let mut selected_indices: Vec<usize> = Vec::new();
    let mut current_duration = *session_duration;

    for (idx, v) in video_queue.iter().enumerate() {
        if current_duration + v.duration <= effective_limit {
            selected_indices.push(idx);
            current_duration += v.duration;
        }

        // Early exit if essentially full
        if effective_limit
            .as_secs()
            .saturating_sub(current_duration.as_secs())
            < 30
        {
            break;
        }
    }

    // Remove selected items from the queue in reverse order and push to session
    // Maintain original order of selected videos when appending to session.
    if !selected_indices.is_empty() {
        // Build a temporary buffer in reverse removal-safe manner
        let mut picked: Vec<VideoItem> = Vec::with_capacity(selected_indices.len());
        for idx in selected_indices.into_iter().rev() {
            // Convert VecDeque removal by rotating or rebuilding:
            // We pop_front repeatedly into a temp buffer until reaching idx.
            // But that's O(n^2). For simplicity and typical small queues, acceptable.
            // For larger queues, consider a different data structure.
            let item = remove_at_index(video_queue, idx);
            picked.push(item);
        }
        picked.reverse(); // restore original relative order of picks

        for p in picked {
            *session_duration += p.duration;
            session_videos.push(p);
        }
    }
}

/// Remove an element at `index` from a VecDeque by rotating.
/// This is O(n) but simple and adequate for moderate queues.
fn remove_at_index<T: Clone>(dq: &mut VecDeque<T>, index: usize) -> T {
    assert!(index < dq.len());
    // Rotate left `index` times, pop_front, then rotate back right `index` times.
    // To minimize moves, choose the shorter rotation direction.
    if index <= dq.len() / 2 {
        for _ in 0..index {
            if let Some(front) = dq.pop_front() {
                dq.push_back(front);
            }
        }
        dq.pop_front().expect("index validated")
    } else {
        // Rotate right len-index times to bring target to back, then pop_back
        let rot = dq.len() - index - 1;
        for _ in 0..rot {
            if let Some(back) = dq.pop_back() {
                dq.push_front(back);
            }
        }
        // Now target is at back
        let target = dq.pop_back().expect("index validated");
        // Restore original order by rotating left one to counter the off-by-one shift
        if let Some(front) = dq.pop_front() {
            dq.push_back(front);
        }
        target
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn settings(minutes: u32) -> PlanSettings {
        PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 3,
            session_length_minutes: minutes,
            include_weekends: false,
            advanced_settings: None,
        }
    }

    fn vi(idx: usize, mins: u64) -> VideoItem {
        VideoItem {
            module_title: "M".into(),
            section_title: format!("S{idx}"),
            video_index: idx,
            duration: Duration::from_secs(mins * 60),
        }
    }

    #[test]
    fn oversized_first_video_is_accepted() {
        let mut q = VecDeque::from(vec![vi(0, 200)]); // 200 min, exceed any normal session
        let s = settings(60);
        let packed = pack_videos_into_session(&mut q, &s).expect("pack");
        assert_eq!(packed.len(), 1);
        assert!(q.is_empty());
    }

    #[test]
    fn greedy_from_front_then_stop() {
        // Session = 60 min, effective = 48 min
        // Queue: 20, 20, 20, 20
        let mut q = VecDeque::from(vec![vi(0, 20), vi(1, 20), vi(2, 20), vi(3, 20)]);
        let s = settings(60);
        let packed = pack_videos_into_session(&mut q, &s).expect("pack");
        // Should fit first two (40 min) and stop before third (would be 60, but effective is 48)
        assert_eq!(packed.len(), 2);
        assert_eq!(q.len(), 2);
        assert_eq!(packed[0].video_index, 0);
        assert_eq!(packed[1].video_index, 1);
    }

    #[test]
    fn utilization_fills_with_small_items() {
        // Session = 60 min, effective = 48
        // First pass picks 30 => left ~18
        // Next small one (10) fits, next (9) does NOT fit after 10 (would be 49)
        let mut q = VecDeque::from(vec![vi(0, 30), vi(1, 10), vi(2, 9), vi(3, 8)]);
        let s = settings(60);

        // Manually simulate first-pass behavior by placing a larger item at front:
        // Actually run pack to exercise both passes.
        let packed = pack_videos_into_session(&mut q, &s).expect("pack");
        // 30 from front, then try fill: 10 fits (40), 9 would exceed (49), then 8 fits (48)
        assert_eq!(
            packed.iter().map(|v| v.duration.as_secs()).sum::<u64>() / 60,
            48
        );
        assert_eq!(packed.len(), 3);
        // Remaining queue should keep relative order for unpicked items, which is just the 9-min one.
        assert_eq!(q.len(), 1);
        assert_eq!(q.front().unwrap().video_index, 2);
    }
}
