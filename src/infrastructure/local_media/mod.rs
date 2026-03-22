//! Local media scanner adapter.
//!
//! Scans a local folder recursively and extracts lightweight metadata for video files.
//! Duration extraction uses GStreamer Discoverer for universal container support.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use gst_pbutils::Discoverer;
use walkdir::WalkDir;

use crate::domain::ports::{
    LocalMediaError, LocalMediaScanner, RawLocalMediaMetadata, RawSubtitleMetadata,
};

/// Local media scanner implementation backed by GStreamer Discoverer.
#[derive(Debug, Clone)]
pub struct LocalMediaScannerAdapter {
    discoverer: Discoverer,
}

impl Default for LocalMediaScannerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalMediaScannerAdapter {
    /// Creates a new adapter with a GStreamer discoverer (5-second timeout).
    pub fn new() -> Self {
        gst::init().ok();
        let discoverer = Discoverer::new(5 * gst::ClockTime::SECOND)
            .expect("Failed to create GStreamer discoverer");
        Self { discoverer }
    }

    fn scan_video_file(
        &self,
        path: &Path,
        subtitle: Option<&PathBuf>,
    ) -> Option<RawLocalMediaMetadata> {
        if !is_video_file(path) {
            return None;
        }

        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "Untitled".to_string());

        let duration_secs = self.get_video_duration(path).unwrap_or(0);

        let mut subtitles: Vec<RawSubtitleMetadata> = subtitle
            .map(|p| RawSubtitleMetadata { path: p.to_string_lossy().to_string() })
            .into_iter()
            .collect();

        if subtitles.is_empty() {
            let embedded = extract_embedded_subtitles_ffmpeg(path);
            subtitles = embedded
                .into_iter()
                .map(|p| RawSubtitleMetadata { path: p.to_string_lossy().to_string() })
                .collect();
        }

        let absolute = path.to_string_lossy().to_string();

        Some(RawLocalMediaMetadata { path: absolute, title, duration_secs, subtitles })
    }

    fn get_video_duration(&self, path: &Path) -> Result<u32, LocalMediaError> {
        let url = format!("file://{}", path.to_string_lossy());
        let info = self
            .discoverer
            .discover_uri(&url)
            .map_err(|e| LocalMediaError::Metadata(e.to_string()))?;
        let duration =
            info.duration().ok_or_else(|| LocalMediaError::Metadata("no duration".into()))?;
        Ok(duration.seconds() as u32)
    }
}

#[allow(async_fn_in_trait)]
impl LocalMediaScanner for LocalMediaScannerAdapter {
    async fn scan(&self, root: &str) -> Result<Vec<RawLocalMediaMetadata>, LocalMediaError> {
        let root_path = Path::new(root);
        if !root_path.exists() {
            return Err(LocalMediaError::Io(format!("path does not exist: {root}")));
        }

        let root_abs = canonicalize_path(root_path)?;

        let mut video_paths: Vec<PathBuf> = Vec::new();
        let mut subtitle_paths: Vec<PathBuf> = Vec::new();

        if root_abs.is_file() {
            if is_video_file(&root_abs) {
                video_paths.push(root_abs.clone());
            } else if is_subtitle_file(&root_abs) {
                subtitle_paths.push(root_abs.clone());
            }
        } else {
            for entry in WalkDir::new(&root_abs)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                let path = entry.path();
                if is_video_file(path) {
                    video_paths.push(path.to_path_buf());
                } else if is_subtitle_file(path) {
                    subtitle_paths.push(path.to_path_buf());
                }
            }
        }

        let assignments = match_subtitles_greedy(&video_paths, &subtitle_paths);

        let mut results = Vec::new();
        for path in video_paths {
            if let Some(item) = self.scan_video_file(&path, assignments.get(&path)) {
                results.push(item);
            }
        }

        Ok(results)
    }
}

fn extract_embedded_subtitles_ffmpeg(path: &Path) -> Vec<PathBuf> {
    let ffmpeg_check = Command::new("ffmpeg").arg("-version").output();
    if ffmpeg_check.is_err() {
        return vec![];
    }

    let mut paths = Vec::new();
    let base_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("subtitle");
    let temp_dir = std::env::temp_dir();

    for stream_idx in 0..10 {
        let output_path = temp_dir.join(format!("{}_{}.srt", base_stem, stream_idx));

        let result = Command::new("ffmpeg")
            .args([
                "-i",
                &path.to_string_lossy(),
                "-map",
                &format!("0:s:{}", stream_idx),
                "-f",
                "srt",
                &output_path.to_string_lossy(),
            ])
            .output();

        match result {
            Ok(output) => {
                if output.status.success()
                    && output_path.exists()
                    && output_path.metadata().map(|m| m.len() > 0).unwrap_or(false)
                {
                    paths.push(output_path);
                } else {
                    let _ = std::fs::remove_file(&output_path);
                    break;
                }
            },
            Err(_) => break,
        }
    }

    paths
}

fn is_video_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    matches!(ext.as_str(), "mp4" | "mkv" | "webm")
}

fn is_subtitle_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    matches!(ext.as_str(), "srt" | "vtt" | "txt")
}

const MIN_MATCH_SCORE: f32 = 0.75;

/// Greedy, folder-level subtitle matching.
/// Each subtitle can be assigned to at most one video, and only within the same folder.
fn match_subtitles_greedy(
    video_paths: &[PathBuf],
    subtitle_paths: &[PathBuf],
) -> HashMap<PathBuf, PathBuf> {
    let mut assignments: HashMap<PathBuf, PathBuf> = HashMap::new();

    let mut videos_by_folder: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    let mut subs_by_folder: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

    for video in video_paths {
        let folder = video.parent().map(Path::to_path_buf).unwrap_or_else(PathBuf::new);
        videos_by_folder.entry(folder).or_default().push(video.clone());
    }

    for sub in subtitle_paths {
        let folder = sub.parent().map(Path::to_path_buf).unwrap_or_else(PathBuf::new);
        subs_by_folder.entry(folder).or_default().push(sub.clone());
    }

    for (folder, videos) in videos_by_folder {
        let subtitles = match subs_by_folder.get(&folder) {
            Some(subs) => subs.clone(),
            None => continue,
        };

        let mut used_subs: HashSet<PathBuf> = HashSet::new();
        let mut candidates: Vec<(f32, PathBuf, PathBuf)> = Vec::new();

        for video in &videos {
            let video_stem = video.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let video_norm = normalize_name(video_stem);
            let video_tokens = tokenize(&video_norm);

            for subtitle in &subtitles {
                let sub_stem = subtitle.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let sub_norm = normalize_name(sub_stem);
                let score = similarity_score(&video_norm, &video_tokens, &sub_norm);
                if score >= MIN_MATCH_SCORE {
                    candidates.push((score, video.clone(), subtitle.clone()));
                }
            }
        }

        candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        for (_, video, subtitle) in candidates {
            if assignments.contains_key(&video) {
                continue;
            }
            if used_subs.contains(&subtitle) {
                continue;
            }
            assignments.insert(video, subtitle.clone());
            used_subs.insert(subtitle);
        }
    }

    // Pass 2: Walk up parent directories for unmatched videos
    let mut used_subs_global: HashSet<PathBuf> = assignments.values().cloned().collect();

    for video in video_paths {
        if assignments.contains_key(video) {
            continue;
        }

        let video_stem = video.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let video_norm = normalize_name(video_stem);
        let video_tokens = tokenize(&video_norm);

        let mut parent = video.parent();
        while let Some(parent_path) = parent {
            if let Some(parent_subs) = subs_by_folder.get(parent_path) {
                let mut best_score = MIN_MATCH_SCORE;
                let mut best_sub: Option<PathBuf> = None;

                for sub in parent_subs {
                    if used_subs_global.contains(sub) {
                        continue;
                    }
                    let sub_stem = sub.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                    let sub_norm = normalize_name(sub_stem);
                    let score = similarity_score(&video_norm, &video_tokens, &sub_norm);
                    if score >= best_score {
                        best_score = score;
                        best_sub = Some(sub.clone());
                    }
                }

                if let Some(sub) = best_sub {
                    used_subs_global.insert(sub.clone());
                    assignments.insert(video.clone(), sub);
                    break;
                }
            }
            parent = parent_path.parent();
        }
    }

    assignments
}

fn normalize_name(name: &str) -> String {
    let mut normalized = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
        } else {
            normalized.push(' ');
        }
    }

    let tokens =
        normalized.split_whitespace().filter(|token| !is_stopword(token)).collect::<Vec<_>>();

    tokens.join(" ")
}

fn tokenize(name: &str) -> Vec<String> {
    name.split_whitespace().filter(|token| !is_stopword(token)).map(|s| s.to_string()).collect()
}

fn is_stopword(token: &str) -> bool {
    matches!(
        token,
        "the"
            | "a"
            | "an"
            | "and"
            | "or"
            | "of"
            | "to"
            | "in"
            | "on"
            | "for"
            | "with"
            | "by"
            | "from"
            | "at"
            | "as"
            | "is"
            | "are"
            | "be"
            | "this"
            | "that"
            | "subtitle"
            | "subtitles"
            | "sub"
            | "subs"
            | "caption"
            | "captions"
            | "captioned"
            | "cc"
            | "closed"
            | "sdh"
            | "hi"
            | "hearing"
            | "impaired"
            | "srt"
            | "vtt"
            | "english"
            | "eng"
            | "en"
            | "spanish"
            | "es"
            | "french"
            | "fr"
            | "german"
            | "de"
            | "arabic"
            | "ar"
            | "italian"
            | "it"
            | "portuguese"
            | "pt"
            | "russian"
            | "ru"
            | "japanese"
            | "ja"
            | "korean"
            | "ko"
            | "chinese"
            | "zh"
            | "1080p"
            | "720p"
            | "480p"
            | "2160p"
            | "4k"
            | "uhd"
            | "hdr"
            | "dv"
            | "dolby"
            | "vision"
            | "x264"
            | "x265"
            | "h264"
            | "h265"
            | "hevc"
            | "avc"
            | "aac"
            | "ac3"
            | "dts"
            | "bluray"
            | "bdrip"
            | "brrip"
            | "webrip"
            | "webdl"
            | "web"
            | "proper"
            | "repack"
            | "extended"
            | "remastered"
            | "uncut"
            | "hdr10"
            | "hdr10plus"
    )
}

fn similarity_score(video_norm: &str, video_tokens: &[String], sub_norm: &str) -> f32 {
    if video_norm.is_empty() || sub_norm.is_empty() {
        return 0.0;
    }

    if sub_norm.contains(video_norm) || video_norm.contains(sub_norm) {
        return 1.0;
    }

    let a: HashSet<String> = video_tokens.iter().cloned().collect();
    let b: HashSet<String> = tokenize(sub_norm).into_iter().collect();

    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let intersection = a.intersection(&b).count() as f32;
    let union = a.union(&b).count() as f32;

    if union == 0.0 { 0.0 } else { intersection / union }
}

fn canonicalize_path(path: &Path) -> Result<PathBuf, LocalMediaError> {
    std::fs::canonicalize(path).map_err(|e| LocalMediaError::Io(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greedy_allows_partial_matches() {
        let videos: Vec<PathBuf> =
            (1..=11).map(|i| PathBuf::from(format!("/root/lesson_{i}.mp4"))).collect();
        let subtitles: Vec<PathBuf> =
            (1..=10).map(|i| PathBuf::from(format!("/root/lesson_{i}.srt"))).collect();

        let assignments = match_subtitles_greedy(&videos, &subtitles);
        assert_eq!(assignments.len(), 10);
    }

    #[test]
    fn matches_subtitle_with_language_suffix() {
        let video = PathBuf::from("/root/Lesson 01 - Intro.mp4");
        let subtitle = PathBuf::from("/root/Lesson 01 - Intro [English] CC.srt");
        let other = PathBuf::from("/root/Unrelated.srt");

        let assignments =
            match_subtitles_greedy(std::slice::from_ref(&video), &[subtitle.clone(), other]);
        assert_eq!(assignments.get(&video), Some(&subtitle));
    }

    #[test]
    fn ignores_low_similarity_subtitles() {
        let video = PathBuf::from("/root/Chapter 01 - Basics.mp4");
        let subtitle = PathBuf::from("/root/Completely Different Topic.srt");

        let assignments = match_subtitles_greedy(std::slice::from_ref(&video), &[subtitle]);
        assert!(!assignments.contains_key(&video));
    }

    #[test]
    fn ffmpeg_absent_does_not_crash() {
        let result = extract_embedded_subtitles_ffmpeg(Path::new("/nonexistent/video.mkv"));
        assert!(result.is_empty());
    }
}
