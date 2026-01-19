//! Local media scanner adapter.
//!
//! Scans a local folder recursively and extracts lightweight metadata for MP4 and MKV files.

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::domain::ports::{LocalMediaError, LocalMediaScanner, RawLocalMediaMetadata};

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

        let mut results = Vec::new();
        if root_abs.is_file() {
            if let Some(item) = scan_file(&root_abs) {
                results.push(item);
            }
            return Ok(results);
        }

        for entry in WalkDir::new(&root_abs)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if let Some(item) = scan_file(path) {
                results.push(item);
            }
        }

        Ok(results)
    }
}

fn scan_file(path: &Path) -> Option<RawLocalMediaMetadata> {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    let supported = matches!(ext.as_str(), "mp4" | "mkv" | "webm");
    if !supported {
        return None;
    }

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

    let absolute = path.to_string_lossy().to_string();

    Some(RawLocalMediaMetadata { path: absolute, title, duration_secs })
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
