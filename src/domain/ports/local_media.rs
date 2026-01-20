//! Local media scanner port.

/// Raw subtitle metadata from filesystem scan.
#[derive(Debug, Clone)]
pub struct RawSubtitleMetadata {
    pub path: String,
}

/// Raw local media metadata from filesystem scan.
#[derive(Debug, Clone)]
pub struct RawLocalMediaMetadata {
    pub path: String,
    pub title: String,
    pub duration_secs: u32,
    /// Candidate subtitle files matching the video.
    pub subtitles: Vec<RawSubtitleMetadata>,
}

/// Error type for local media scanning.
#[derive(Debug, thiserror::Error)]
pub enum LocalMediaError {
    #[error("I/O error: {0}")]
    Io(String),
    #[error("Unsupported file: {0}")]
    Unsupported(String),
    #[error("Metadata parse error: {0}")]
    Metadata(String),
}

/// Port for scanning local media libraries.
#[allow(async_fn_in_trait)]
pub trait LocalMediaScanner: Send + Sync {
    /// Recursively scans a root directory for supported media.
    async fn scan(&self, root: &str) -> Result<Vec<RawLocalMediaMetadata>, LocalMediaError>;
}
