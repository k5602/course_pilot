use crate::domain::ports::FetchError;
use crate::domain::value_objects::VideoQuality;

/// Port for resolving a streaming URL from a video source identifier.
///
/// Implementations use yt-dlp (or equivalent) to get a direct,
/// playable stream URL respecting the requested quality.
#[allow(async_fn_in_trait)]
pub trait StreamResolver: Send + Sync {
    /// Resolve a YouTube video ID to a direct stream URL at the
    /// requested quality. Returns the URL as a String.
    async fn resolve_youtube_stream(
        &self,
        youtube_id: &str,
        quality: VideoQuality,
    ) -> Result<String, FetchError>;
}
