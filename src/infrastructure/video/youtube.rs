/// Extract a direct stream URL from YouTube using yt-dlp.
///
/// This runs yt-dlp as an external process and parses its output.
/// yt-dlp must be installed on the system or bundled with the app.
use std::sync::OnceLock;

static YT_DLP_PATH: OnceLock<String> = OnceLock::new();

pub fn set_yt_dlp_path(path: String) {
    let _ = YT_DLP_PATH.set(path);
}

pub async fn get_stream_url(youtube_url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let yt_dlp = YT_DLP_PATH.get().map(|s| s.as_str()).unwrap_or("yt-dlp");

    let output = tokio::process::Command::new(yt_dlp)
        .arg("--format")
        .arg("best[ext=mp4]")
        .arg("--get-url")
        .arg(youtube_url)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp failed: {}", stderr).into());
    }

    let url = String::from_utf8(output.stdout)?.trim().to_string();

    if url.is_empty() {
        return Err("yt-dlp returned empty URL".into());
    }

    Ok(url)
}

pub async fn get_best_stream(
    youtube_url: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let yt_dlp = YT_DLP_PATH.get().map(|s| s.as_str()).unwrap_or("yt-dlp");

    let output =
        tokio::process::Command::new(yt_dlp).arg("--dump-json").arg(youtube_url).output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp metadata fetch failed: {}", stderr).into());
    }

    let info: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let url = info["url"].as_str().ok_or("No stream URL found")?.to_string();
    let title = info["title"].as_str().unwrap_or("Untitled").to_string();

    Ok((url, title))
}
