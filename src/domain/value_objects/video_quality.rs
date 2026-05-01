use std::fmt;

/// Target video quality for stream resolution at play-time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VideoQuality {
    P240,
    P360,
    P480,
    P720,
    P1080,
    Best,
}

impl VideoQuality {
    pub fn ytdlp_format(self) -> &'static str {
        match self {
            Self::P240 => "bestvideo[height<=240]+bestaudio/best[height<=240]",
            Self::P360 => "bestvideo[height<=360]+bestaudio/best[height<=360]",
            Self::P480 => "bestvideo[height<=480]+bestaudio/best[height<=480]",
            Self::P720 => "bestvideo[height<=720]+bestaudio/best[height<=720]",
            Self::P1080 => "bestvideo[height<=1080]+bestaudio/best[height<=1080]",
            Self::Best => "best[ext=mp4]/best",
        }
    }

    pub fn height(self) -> Option<u16> {
        match self {
            Self::P240 => Some(240),
            Self::P360 => Some(360),
            Self::P480 => Some(480),
            Self::P720 => Some(720),
            Self::P1080 => Some(1080),
            Self::Best => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::P240 => "240p",
            Self::P360 => "360p",
            Self::P480 => "480p",
            Self::P720 => "720p",
            Self::P1080 => "1080p",
            Self::Best => "Best",
        }
    }

    pub fn variants() -> &'static [VideoQuality] {
        const VARIANTS: &[VideoQuality] = &[
            VideoQuality::P240,
            VideoQuality::P360,
            VideoQuality::P480,
            VideoQuality::P720,
            VideoQuality::P1080,
            VideoQuality::Best,
        ];
        VARIANTS
    }
}

impl fmt::Display for VideoQuality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_match_format_suffix() {
        for v in VideoQuality::variants() {
            let label = v.label();
            if *v == VideoQuality::Best {
                assert_eq!(label, "Best");
            } else {
                assert!(label.ends_with('p'), "{label} should end with p");
            }
        }
    }

    #[test]
    fn ytdlp_format_best_is_original_default() {
        assert_eq!(VideoQuality::Best.ytdlp_format(), "best[ext=mp4]/best");
    }

    #[test]
    fn ytdlp_format_bounded_includes_height() {
        let fmt = VideoQuality::P720.ytdlp_format();
        assert!(fmt.contains("height<=720"));
    }

    #[test]
    fn height_returns_some_for_bounded() {
        assert_eq!(VideoQuality::P480.height(), Some(480));
        assert_eq!(VideoQuality::Best.height(), None);
    }

    #[test]
    fn display_matches_label() {
        for v in VideoQuality::variants() {
            assert_eq!(format!("{v}"), v.label());
        }
    }

    #[test]
    fn variants_are_ordered_from_lowest_to_highest() {
        let variants = VideoQuality::variants();
        for i in 1..variants.len() {
            let prev = variants[i - 1].height().unwrap_or(u16::MAX);
            let cur = variants[i].height().unwrap_or(u16::MAX);
            assert!(prev <= cur, "variants not sorted");
        }
    }
}
