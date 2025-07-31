# Course Pilot Video Player System

## Overview

The Course Pilot video player system provides cross-platform video playback capabilities for both local video files and YouTube videos. It's built on top of **FFmpeg** for robust, high-performance media handling with excellent format support and cross-platform compatibility.

## Architecture

### Core Components

1. **VideoPlayerManager** - Main interface for managing video playback
2. **LocalVideoPlayer** - FFmpeg-based player for local video files
3. **YouTubeEmbeddedPlayer** - Placeholder for YouTube embedded player (Task 12)
4. **VideoPlayerControls** - Unified control interface
5. **VideoPlayer UI Component** - Dioxus component for video display

### Why FFmpeg Over GStreamer?

After comprehensive research, we chose **FFmpeg** as our primary video backend because:

- **Better Desktop Integration**: Direct control over rendering and UI integration
- **Simpler Dependencies**: Easier to install and bundle FFmpeg across platforms
- **Superior Performance**: Direct access to optimized codecs without pipeline overhead
- **Format Support**: Excellent support for all video formats with consistent behavior
- **Flexibility**: More control over custom UI controls and video processing
- **Stability**: Fewer version compatibility issues compared to GStreamer Rust bindings

### Supported Formats

#### Local Video Files
- MP4 (H.264/H.265)
- AVI
- MOV (QuickTime)
- MKV (Matroska)
- WebM
- FLV
- WMV
- M4V

#### YouTube Videos
- All YouTube video formats (via embedded player - Task 12)
- YouTube playlists
- YouTube live streams

## Usage

### Basic Video Playback

```rust
use crate::video_player::{VideoPlayerManager, VideoSource};

// Initialize the video player manager
let mut manager = VideoPlayerManager::new()?;

// Play a local video
let local_source = VideoSource::Local {
    path: PathBuf::from("/path/to/video.mp4"),
    title: "My Video".to_string(),
};
manager.play_video(local_source)?;

// Play a YouTube video
let youtube_source = VideoSource::YouTube {
    video_id: "dQw4w9WgXcQ".to_string(),
    playlist_id: None,
    title: "YouTube Video".to_string(),
};
manager.play_video(youtube_source)?;
```

### Using Controls

```rust
// Get controls for the current player
if let Some(controls) = manager.get_current_controls()? {
    // Play/pause
    controls.toggle_play_pause()?;
    
    // Seek to position (in seconds)
    controls.seek(30.0)?;
    
    // Set volume (0.0 to 1.0)
    controls.set_volume(0.8)?;
    
    // Toggle fullscreen
    controls.toggle_fullscreen()?;
}
```

### UI Component

```rust
use crate::ui::components::VideoPlayer;

rsx! {
    VideoPlayer {
        video_source: Some(video_source),
        width: Some("800px".to_string()),
        height: Some("450px".to_string()),
        show_controls: Some(true),
        autoplay: Some(false),
        on_error: move |error| {
            log::error!("Video player error: {}", error);
        }
    }
}
```

## Platform Support

### Windows
- Uses FFmpeg with DirectShow/Media Foundation backends
- Supports hardware acceleration via DirectX, NVENC, QSV
- FFmpeg can be bundled or installed via vcpkg/chocolatey

### Linux
- Uses FFmpeg with native Linux backends (ALSA, V4L2)
- Supports hardware acceleration via VA-API, VDPAU, NVENC
- Available through system package managers (apt, yum, pacman)

### macOS
- Uses FFmpeg with AVFoundation backend
- Supports hardware acceleration via VideoToolbox
- Available via Homebrew or official FFmpeg builds

## Installation Requirements

### FFmpeg Dependencies

#### Ubuntu/Debian
```bash
# Install FFmpeg with development headers
sudo apt-get install ffmpeg libavcodec-dev libavformat-dev libavutil-dev \
    libavfilter-dev libavdevice-dev libswscale-dev libswresample-dev

# For hardware acceleration (optional)
sudo apt-get install libva-dev libvdpau-dev
```

#### Windows (vcpkg - Recommended)
```bash
# Install FFmpeg with all features
vcpkg install ffmpeg[core,avcodec,avformat,avfilter,avdevice,swresample,swscale,nvcodec,qsv,amf,x264]:x64-windows

# For static linking
vcpkg install ffmpeg[core,avcodec,avformat,avfilter,avdevice,swresample,swscale,nvcodec,qsv,amf,x264]:x64-windows-static-md
```

#### Windows (Chocolatey - Alternative)
```bash
choco install ffmpeg
```

#### macOS (Homebrew)
```bash
# Install FFmpeg with all features
brew install ffmpeg

# With additional codecs and hardware acceleration
brew install ffmpeg --with-libvpx --with-libvorbis --with-opus --with-x264 --with-x265
```

#### Arch Linux
```bash
sudo pacman -S ffmpeg
```

#### CentOS/RHEL/Fedora
```bash
# Enable RPM Fusion repository first
sudo dnf install ffmpeg ffmpeg-devel
```

## Features

### Current Implementation (Task 11)
- ✅ Cross-platform video player architecture
- ✅ Local video file support (MP4, AVI, MOV, etc.)
- ✅ GStreamer-based playback engine
- ✅ Unified control interface
- ✅ Basic UI component with controls
- ✅ Integration with session list play buttons
- ✅ Fullscreen support
- ✅ Volume control
- ✅ Seek functionality

### Future Implementation (Task 12)
- ⏳ YouTube embedded player integration
- ⏳ YouTube API integration
- ⏳ YouTube playlist navigation
- ⏳ YouTube player controls synchronization

### Future Enhancements
- 🔮 Subtitle support
- 🔮 Multiple audio track support
- 🔮 Video filters and effects
- 🔮 Streaming protocol support (RTSP, HLS)
- 🔮 Hardware acceleration optimization
- 🔮 Picture-in-picture mode
- 🔮 Video thumbnails and previews

## Error Handling

The video player system uses comprehensive error handling:

```rust
match manager.play_video(source) {
    Ok(()) => {
        log::info!("Video started successfully");
    }
    Err(e) => {
        log::error!("Failed to play video: {}", e);
        // Show user-friendly error message
        toast_helpers::error("Unable to play video. Please check the file format.");
    }
}
```

## Testing

Run the video player integration tests:

```bash
cargo test video_player
```

Note: Tests require GStreamer to be installed and available on the system.

## Troubleshooting

### Common Issues

1. **GStreamer not found**
   - Ensure GStreamer is installed and in PATH
   - Check PKG_CONFIG_PATH environment variable

2. **Video format not supported**
   - Install additional GStreamer plugins
   - Check if codec is available

3. **Audio/Video sync issues**
   - Update GStreamer to latest version
   - Check system audio configuration

4. **Performance issues**
   - Enable hardware acceleration
   - Reduce video resolution/quality
   - Check system resources

### Debug Logging

Enable debug logging for video player issues:

```rust
env_logger::init();
log::set_max_level(log::LevelFilter::Debug);
```

## Integration Points

### Session List Integration
- Play buttons in session list trigger video playback
- Video context is passed to the contextual panel
- Progress tracking integration (future)

### Contextual Panel Integration
- Video player embedded in Player tab
- Video controls accessible from panel
- Notes integration with video timestamps (future)

### Course Management Integration
- Video sources determined from course structure
- Local folder imports map to local video sources
- YouTube imports map to YouTube video sources

## Performance Considerations

- Video decoding is handled by GStreamer's optimized pipelines
- Hardware acceleration is used when available
- Memory usage is managed by GStreamer's buffer management
- UI remains responsive during video operations through async handling

## Security Considerations

- Local video files are accessed with appropriate file permissions
- YouTube videos are played through official embedded player (Task 12)
- No direct network access for video streaming
- User data privacy maintained through local processing