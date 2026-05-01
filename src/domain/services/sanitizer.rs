//! Title Sanitizer - Removes noise from video titles.

/// Sanitizes video titles by removing common noise patterns.
#[derive(Debug, Default)]
pub struct TitleSanitizer;

impl TitleSanitizer {
    /// Creates a new title sanitizer.
    pub fn new() -> Self {
        Self
    }

    /// Sanitizes a video title by removing noise.
    pub fn sanitize(&self, title: &str) -> String {
        let mut result = title.to_string();

        // Remove common patterns
        result = self.remove_episode_markers(&result);
        result = self.remove_year_tags(&result);
        result = self.remove_emojis(&result);
        result = self.remove_clickbait_punctuation(&result);
        result = self.normalize_whitespace(&result);

        result.trim().to_string()
    }

    /// Removes episode/tutorial markers like "Tutorial #1", "Part 2", "Ep. 5"
    fn remove_episode_markers(&self, text: &str) -> String {
        // Simple pattern matching without regex for performance
        let patterns = [
            "tutorial", "part", "episode", "ep.", "ep ", "lesson", "chapter", "section", "module",
            "lecture", "video",
        ];

        let mut result = text.to_string();

        for pattern in patterns {
            let lower = result.to_lowercase();
            if let Some(start) = lower.find(pattern) {
                // Find the end of this marker (including any numbers)
                let end = self.find_marker_end(&result, start + pattern.len());
                if start < result.len() && end <= result.len() {
                    let new_result = format!("{}{}", &result[..start], &result[end..]);
                    if new_result.len() < result.len() {
                        result = new_result;
                    }
                }
            }
        }

        result
    }

    fn find_marker_end(&self, text: &str, start: usize) -> usize {
        let bytes = text.as_bytes();
        let mut end = start;

        // Skip whitespace and markers like #, -, :
        while end < bytes.len() {
            let c = bytes[end] as char;
            if c.is_ascii_digit() || c == '#' || c == '-' || c == ':' || c == ' ' {
                end += 1;
            } else {
                break;
            }
        }

        end
    }

    /// Removes year tags like "(2024)", "[2023 Update]"
    fn remove_year_tags(&self, text: &str) -> String {
        let mut result = String::new();
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '(' || c == '[' {
                let closer = if c == '(' { ')' } else { ']' };
                let mut inside = String::new();

                while let Some(&inner) = chars.peek() {
                    chars.next();
                    if inner == closer {
                        break;
                    }
                    inside.push(inner);
                }

                // Keep if not a year/update tag
                let inside_lower = inside.to_lowercase();
                if !inside_lower.contains("update")
                    && !inside.chars().all(|ch| ch.is_ascii_digit() || ch.is_whitespace())
                {
                    result.push(c);
                    result.push_str(&inside);
                    result.push(closer);
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    /// Removes emojis from text.
    fn remove_emojis(&self, text: &str) -> String {
        text.chars()
            .filter(|c| {
                let code = *c as u32;
                // Filter out common emoji ranges using contains
                !(0x1F300..=0x1FAFF).contains(&code) && !(0x2600..=0x27BF).contains(&code)
            })
            .collect()
    }

    /// Removes excessive punctuation like "!!!", "???"
    fn remove_clickbait_punctuation(&self, text: &str) -> String {
        let mut result = String::new();
        let mut last_char = ' ';
        let mut punct_count = 0;

        for c in text.chars() {
            if c == '!' || c == '?' {
                if last_char == c {
                    punct_count += 1;
                    if punct_count >= 2 {
                        continue; // Skip excessive punctuation
                    }
                } else {
                    punct_count = 1;
                }
            } else {
                punct_count = 0;
            }
            result.push(c);
            last_char = c;
        }

        result
    }

    /// Normalizes whitespace (multiple spaces to single).
    fn normalize_whitespace(&self, text: &str) -> String {
        let mut result = String::new();
        let mut last_was_space = false;

        for c in text.chars() {
            if c.is_whitespace() {
                if !last_was_space {
                    result.push(' ');
                    last_was_space = true;
                }
            } else {
                result.push(c);
                last_was_space = false;
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_tutorial_marker() {
        let sanitizer = TitleSanitizer::new();
        let result = sanitizer.sanitize("Tutorial #5 - Introduction to Rust");
        assert!(!result.to_lowercase().contains("tutorial"));
        assert!(!result.contains("#5"));
    }

    #[test]
    fn test_remove_year_tag() {
        let sanitizer = TitleSanitizer::new();
        let result = sanitizer.sanitize("Learn Python (2024 Update)");
        assert!(!result.contains("2024"));
        assert!(!result.contains("Update"));
    }

    #[test]
    fn test_remove_clickbait() {
        let sanitizer = TitleSanitizer::new();
        let result = sanitizer.sanitize("This is AMAZING!!!");
        assert!(!result.contains("!!!"));
        assert!(result.contains("AMAZING"));
    }

    #[test]
    fn test_normalize_whitespace() {
        let sanitizer = TitleSanitizer::new();
        let result = sanitizer.sanitize("Too   many    spaces");
        assert!(!result.contains("  "));
    }

    #[test]
    fn handles_multiple_pattern_removals() {
        let sanitizer = TitleSanitizer::new();
        let result =
            sanitizer.sanitize("Tutorial Part Episode Lesson Chapter Section Module Lecture Video");
        assert!(result.len() < 70);
    }

    #[test]
    fn handles_nested_brackets_safely() {
        let sanitizer = TitleSanitizer::new();
        let result = sanitizer.sanitize("Title [2024 Update] (with bonus)");
        assert!(!result.contains("2024"));
    }

    #[test]
    fn does_not_panic_on_edge_case_titles() {
        let sanitizer = TitleSanitizer::new();
        let edge_cases = vec![
            "",
            "   ",
            "A",
            "!@#$%^&*()",
            "Tutorial #1 - Introduction to Rust",
            "Episode 5: The One Where Everything Breaks",
            "Module 3 Part 2 Section 1.1",
            "Part 1 of 3: Getting Started",
        ];
        for title in edge_cases {
            let result = sanitizer.sanitize(title);
            assert!(result.chars().all(|c| c != '\u{FFFD}'));
        }
    }
}
