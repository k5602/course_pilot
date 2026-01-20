//! Local media scanner adapter.
//!
//! Scans a local folder recursively and extracts lightweight metadata for MP4 and MKV files.

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::domain::ports::{
    LocalMediaError, LocalMediaScanner, RawLocalMediaMetadata, RawSubtitleMetadata,
};

/// Local media scanner implementation.
#[derive(Debug, Default, Clone)]
pub struct LocalMediaScannerAdapter;

impl LocalMediaScannerAdapter {
    /// Creates a new adapter.
    pub fn new() -> Self {
        Self
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
            if let Some(item) = scan_video_file(&path, assignments.get(&path)) {
                results.push(item);
            }
        }

        Ok(results)
    }
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

fn scan_video_file(path: &Path, subtitle: Option<&PathBuf>) -> Option<RawLocalMediaMetadata> {
    if !is_video_file(path) {
        return None;
    }

    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();

    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Untitled".to_string());

    let duration_secs = match ext.as_str() {
        "mp4" => read_mp4_duration(path).unwrap_or(0),
        "mkv" | "webm" => read_matroska_duration(path).unwrap_or(0),
        _ => 0,
    };

    let subtitles = subtitle
        .map(|path| RawSubtitleMetadata { path: path.to_string_lossy().to_string() })
        .into_iter()
        .collect();

    let absolute = path.to_string_lossy().to_string();

    Some(RawLocalMediaMetadata { path: absolute, title, duration_secs, subtitles })
}

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

fn read_mp4_duration(path: &Path) -> Result<u32, LocalMediaError> {
    let file = File::open(path).map_err(|e| LocalMediaError::Io(e.to_string()))?;
    let size = file.metadata().map_err(|e| LocalMediaError::Io(e.to_string()))?.len();
    let reader = BufReader::new(file);

    let mp4 = mp4::Mp4Reader::read_header(reader, size)
        .map_err(|e| LocalMediaError::Metadata(e.to_string()))?;

    let duration = mp4.duration();
    Ok(u32::try_from(duration.as_secs()).unwrap_or(u32::MAX))
}

fn read_matroska_duration(path: &Path) -> Result<u32, LocalMediaError> {
    let info = matroska::get_from::<_, matroska::Info>(path)
        .map_err(|e| LocalMediaError::Metadata(e.to_string()))?;

    let duration = info.and_then(|i| i.duration).map(|d| d.as_secs()).unwrap_or(0);
    Ok(u32::try_from(duration).unwrap_or(u32::MAX))
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

        let assignments = match_subtitles_greedy(&[video.clone()], &[subtitle.clone(), other]);
        assert_eq!(assignments.get(&video), Some(&subtitle));
    }

    #[test]
    fn ignores_low_similarity_subtitles() {
        let video = PathBuf::from("/root/Chapter 01 - Basics.mp4");
        let subtitle = PathBuf::from("/root/Completely Different Topic.srt");

        let assignments = match_subtitles_greedy(&[video.clone()], &[subtitle]);
        assert!(assignments.get(&video).is_none());
    }
}
