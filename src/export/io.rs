use anyhow::Result;
use std::path::{Path, PathBuf};

/// Centralized file saving for export results.
pub async fn save_bytes_atomic(path: impl AsRef<Path>, data: &[u8]) -> Result<PathBuf> {
    let path = path.as_ref();
    let parent = path.parent().unwrap_or(Path::new("."));
    tokio::fs::create_dir_all(parent).await.ok();

    // Write to a temp file first, then rename.
    let tmp_path = path.with_extension("tmp");
    tokio::fs::write(&tmp_path, data).await?;
    // Best-effort fsync on parent dir is not easily available cross-platform in async; rename is fine for desktop.
    tokio::fs::rename(&tmp_path, path).await?;
    Ok(path.to_path_buf())
}

/// Choose a default output path if user didn't provide one.
pub fn default_output_path(filename: &str) -> PathBuf {
    // Save to current working directory by default
    PathBuf::from(filename)
}
