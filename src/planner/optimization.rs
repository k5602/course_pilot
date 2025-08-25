/*!
Planner optimization pipeline

This module provides the post-generation optimization passes applied to a Plan:
- add_review_sessions
- balance_cognitive_load
- add_adaptive_buffer_days
- optimize_session_timing
- add_consolidation_breaks
- validate_plan_structure

These refinements aim to improve retention, balance session effort, respect
temporal spacing heuristics, and keep a valid, sorted schedule.
*/

use crate::PlanError;
use crate::planner::calendar::next_session_date;
use crate::types::{Plan, PlanItem};
use chrono::{Datelike, Utc, Weekday};
use std::time::Duration;

// Reuse adaptive helpers for content-based cognitive load estimation
use crate::planner::strategies::adaptive::calculate_cognitive_load;

/// Enhanced plan optimization with advanced learning science principles.
///
/// Pipeline:
/// 1) Add periodic review sessions
/// 2) Balance cognitive load across neighboring sessions
/// 3) Insert adaptive buffer days for complex content or high-density sessions
/// 4) Optimize timing (e.g., avoid difficult sessions on Mondays)
/// 5) Add consolidation (rest) breaks in long plans
/// 6) Validate and normalize the resulting plan (sort by date, cleanup)
pub fn optimize_plan(plan: &mut Plan) -> Result<(), PlanError> {
    add_review_sessions(plan)?;
    balance_cognitive_load(plan)?;
    add_adaptive_buffer_days(plan)?;
    optimize_session_timing(plan)?;
    add_consolidation_breaks(plan)?;
    validate_plan_structure(plan)?;
    Ok(())
}

/// Balance cognitive load across sessions by moving a small amount of content
/// from overloaded sessions towards underloaded ones (first-fit heuristic).
fn balance_cognitive_load(plan: &mut Plan) -> Result<(), PlanError> {
    if plan.items.len() < 2 {
        return Ok(());
    }

    // Compute per-session load: number of videos + content-based estimate
    let mut session_loads: Vec<f32> = Vec::with_capacity(plan.items.len());
    for item in &plan.items {
        let mut load = 0.0;
        // Base load: number of videos
        load += item.video_indices.len() as f32 * 0.2;
        // Content-based load from titles (duration-insensitive estimate here)
        let title_load = calculate_cognitive_load(&item.section_title, Duration::from_secs(0));
        load += title_load;
        session_loads.push(load);
    }

    // Target average load
    let total_load: f32 = session_loads.iter().sum();
    let target_load = total_load / session_loads.len() as f32;

    // Greedy redistribution: move 1 video from overloaded session to next underloaded session
    let mut i = 0;
    while i + 1 < plan.items.len() {
        if session_loads[i] > target_load * 1.5 {
            for j in (i + 1)..plan.items.len() {
                if session_loads[j] < target_load * 0.7 && !plan.items[i].video_indices.is_empty() {
                    if let Some(video_index) = plan.items[i].video_indices.pop() {
                        plan.items[j].video_indices.push(video_index);
                        // Adjust loads by a modest constant; precise recalculation is unnecessary here.
                        session_loads[i] -= 0.3;
                        session_loads[j] += 0.3;
                        break;
                    }
                }
            }
        }
        i += 1;
    }

    Ok(())
}

/// Insert buffer days to give learners recovery/consolidation time based on content complexity
/// and high-density sessions.
fn add_adaptive_buffer_days(plan: &mut Plan) -> Result<(), PlanError> {
    let complexity_threshold = 0.7;

    for item in plan.items.iter_mut() {
        let mut needs_buffer = false;
        let mut buffer_days = 0;

        // High video count sessions need buffer
        if item.video_indices.len() > 5 {
            needs_buffer = true;
            buffer_days = 1;
        }

        // Complex content (estimated from title semantics) needs buffer
        let title_load = calculate_cognitive_load(&item.section_title, Duration::from_secs(0));
        if title_load > complexity_threshold {
            needs_buffer = true;
            buffer_days = buffer_days.max(1);
        }

        // Expert/advanced content gets extra buffer
        let tl = item.section_title.to_lowercase();
        if tl.contains("advanced") || tl.contains("expert") {
            needs_buffer = true;
            buffer_days = buffer_days.max(2);
        }

        if needs_buffer {
            item.date += chrono::Duration::days(buffer_days);
        }
    }

    // Re-sort after applying buffers
    plan.items.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(())
}

/// Avoid scheduling difficult content immediately after weekends (e.g., Mondays),
/// and ensure light heuristics for assessments.
fn optimize_session_timing(plan: &mut Plan) -> Result<(), PlanError> {
    for item in plan.items.iter_mut() {
        let tl = item.section_title.to_lowercase();

        // Avoid heavy content on Mondays
        if (tl.contains("advanced") || tl.contains("complex"))
            && item.date.weekday() == Weekday::Mon
        {
            item.date += chrono::Duration::days(1);
        }

        // Placeholder for assessment spacing (could check neighbors if needed)
        if tl.contains("test") || tl.contains("exam") {
            // Ensure not back-to-back with other assessments — advanced logic omitted for now.
        }
    }

    plan.items.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(())
}

/// Add consolidation (rest & reflection) break days for long plans.
fn add_consolidation_breaks(plan: &mut Plan) -> Result<(), PlanError> {
    let total_sessions = plan.items.len();
    if total_sessions < 10 {
        return Ok(()); // Only add for longer courses
    }

    let break_interval = total_sessions / 4; // Every 25% of course
    let mut break_items = Vec::new();

    for i in (break_interval..total_sessions).step_by(break_interval) {
        if i < plan.items.len() {
            let break_date = plan.items[i].date + chrono::Duration::days(1);
            let break_duration = Duration::from_secs(0);
            let estimated_completion_time = Duration::from_secs(30 * 60); // 30 minutes reflection

            break_items.push(PlanItem {
                date: break_date,
                module_title: "Consolidation".to_string(),
                section_title: "Rest & Reflection Day".to_string(),
                video_indices: Vec::new(),
                completed: false,
                total_duration: break_duration,
                estimated_completion_time,
                overflow_warnings: Vec::new(),
            });
        }
    }

    plan.items.extend(break_items);
    plan.items.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(())
}

/// Remove invalid/empty sessions, ensure dates are reasonable (not in the past),
/// and finally sort the plan items chronologically.
fn validate_plan_structure(plan: &mut Plan) -> Result<(), PlanError> {
    // Keep sessions that have content or that are explicitly review/rest sessions
    plan.items.retain(|item| {
        !item.video_indices.is_empty()
            || item.section_title.contains("Review")
            || item.section_title.contains("Rest")
    });

    // Ensure dates are not in the past
    let now = Utc::now();
    for item in plan.items.iter_mut() {
        if item.date < now {
            item.date = now + chrono::Duration::days(1);
        }
    }

    plan.items.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(())
}

/// Insert review sessions periodically to reinforce learning.
///
/// Heuristics:
/// - Review every max(5, total_sessions / 4)
/// - Duration 45 minutes with buffer-aware completion estimate
fn add_review_sessions(plan: &mut Plan) -> Result<(), PlanError> {
    let total_sessions = plan.items.len();
    if total_sessions == 0 {
        return Ok(());
    }

    let review_interval = std::cmp::max(5, total_sessions / 4);

    let mut review_items = Vec::new();
    for (i, item) in plan.items.iter().enumerate() {
        if (i + 1) % review_interval == 0 && i < plan.items.len() - 1 {
            let review_date = next_session_date(item.date, &plan.settings);

            let review_duration = Duration::from_secs(45 * 60); // 45 minutes
            let estimated_completion_time =
                crate::types::duration_utils::calculate_completion_time_with_buffer(
                    review_duration,
                    0.25,
                );

            review_items.push(PlanItem {
                date: review_date,
                module_title: "Review".to_string(),
                section_title: format!("Review: Modules 1-{}", (i / review_interval) + 1),
                video_indices: vec![],
                completed: false,
                total_duration: review_duration,
                estimated_completion_time,
                overflow_warnings: Vec::new(),
            });
        }
    }

    plan.items.extend(review_items);
    plan.items.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(())
}
