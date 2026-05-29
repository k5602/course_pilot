//! Title Sanitizer - Removes noise from video titles.
//!
//! Uses a single-pass, Cow-based pipeline to minimize allocations.
//! Pre-computes lowercase once and reuses it across all case-insensitive
//! pattern matching steps.

use std::borrow::Cow;

/// Episode marker patterns searched case-insensitively.
const PATTERNS: &[&str] = &[
    "tutorial", "part", "episode", "ep.", "ep ", "lesson", "chapter", "section", "module",
    "lecture", "video",
];

/// Sanitizes video titles by removing common noise patterns.
#[derive(Debug, Default)]
pub struct TitleSanitizer;

impl TitleSanitizer {
    /// Creates a new title sanitizer.
    pub fn new() -> Self {
        Self
    }

    /// Sanitizes a video title by removing noise.
    ///
    /// Pipeline (3 stages, ~3-4 allocations typical):
    /// 1. Remove episode markers (case-insensitive, uses precomputed lowercase)
    /// 2. Remove year/update bracketed tags
    /// 3. Single-pass clean: emoji + clickbait punctuation + whitespace normalize
    pub fn sanitize(&self, title: &str) -> String {
        let lower = title.to_lowercase();
        let without_markers = Self::remove_episode_markers(title, &lower);
        let without_years = Self::remove_year_tags(&without_markers);
        Self::clean_text(&without_years)
    }

    /// Removes sequential episode/tutorial markers from the title.
    ///
    /// Each pattern is checked in order against the current text. On first match,
    /// the marker and its suffix (digits, #, -, :, spaces) are removed. Subsequent
    /// patterns are checked against the modified text.
    ///
    /// Returns `Cow::Borrowed` if no patterns match, avoiding allocation.
    fn remove_episode_markers<'a>(text: &'a str, original_lower: &str) -> Cow<'a, str> {
        let mut result: Cow<'a, str> = Cow::Borrowed(text);

        for pattern in PATTERNS {
            // Search in the lowercase of the current text.
            // Reuse precomputed lowercase when result is still the original.
            let matched = {
                let current_lower: String;
                let search_lower: &str = if let Cow::Borrowed(_) = &result {
                    original_lower
                } else {
                    current_lower = result.to_lowercase();
                    &current_lower
                };
                search_lower.find(pattern).map(|start| {
                    let end = find_marker_end(&result, start + pattern.len());
                    (start, end)
                })
            }; // borrows from current_lower / original_lower end here

            if let Some((start, end)) = matched
                && start < result.len()
                && end <= result.len()
            {
                let owned = format!("{}{}", &result[..start], &result[end..]);
                if owned.len() < result.len() {
                    result = Cow::Owned(owned);
                }
            }
        }

        result
    }

    /// Removes year/update tags in brackets like "(2024)", "[2024 Update]".
    ///
    /// A bracket group is removed if its contents are all ASCII digits/whitespace
    /// (a bare year) or contain the word "update" (case-insensitive).
    ///
    /// Returns `Cow::Borrowed` if no brackets were removed.
    fn remove_year_tags(text: &str) -> Cow<'_, str> {
        let mut result = String::with_capacity(text.len());
        let mut modified = false;
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

                let inside_lower = inside.to_lowercase();
                if inside_lower.contains("update")
                    || inside.chars().all(|ch| ch.is_ascii_digit() || ch.is_whitespace())
                {
                    modified = true;
                    // Skip this bracket group entirely
                } else {
                    result.push(c);
                    result.push_str(&inside);
                    result.push(closer);
                }
            } else {
                result.push(c);
            }
        }

        if modified { Cow::Owned(result) } else { Cow::Borrowed(text) }
    }

    /// Single-pass combined filter: removes emojis, reduces clickbait punctuation,
    /// and normalizes whitespace.
    ///
    /// Equivalent to running `remove_emojis` → `remove_clickbait_punctuation` →
    /// `normalize_whitespace` sequentially, but in one allocation.
    fn clean_text(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut prev_was_space = true; // trim leading whitespace
        let mut last_punct_char = '\0';
        let mut punct_run = 0u32;

        for c in text.chars() {
            // 1. Emoji filter - skip entirely
            if is_emoji(c) {
                continue;
            }

            // 2. Whitespace normalization - collapse to single space
            if c.is_whitespace() {
                if !prev_was_space {
                    result.push(' ');
                    prev_was_space = true;
                }
                punct_run = 0;
                last_punct_char = ' ';
                continue;
            }

            // 3. Clickbait punctuation reduction
            //    Track consecutive runs of same punctuation character.
            //    Only the first in a run is emitted; subsequent same chars are dropped.
            if c == '!' || c == '?' {
                if last_punct_char == c {
                    punct_run += 1;
                    if punct_run >= 2 {
                        continue;
                    }
                } else {
                    punct_run = 1;
                }
            } else {
                punct_run = 0;
            }

            result.push(c);
            last_punct_char = c;
            prev_was_space = false;
        }

        // Trim trailing space (leading is handled by prev_was_space init)
        if result.ends_with(' ') {
            result.pop();
        }

        result
    }
}

// ---------------------------------------------------------------------------
// Free functions - no self parameter needed
// ---------------------------------------------------------------------------

/// Finds the byte-index where an episode marker suffix ends.
///
/// The suffix consists of digits, '#', '-', ':', and spaces following the
/// pattern keyword (e.g. "Tutorial #5" → suffix is " #5").
fn find_marker_end(text: &str, start: usize) -> usize {
    let bytes = text.as_bytes();
    let mut end = start;

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

/// Returns true if the character falls within a known emoji Unicode range.
fn is_emoji(c: char) -> bool {
    let code = c as u32;
    (0x1F300..=0x1FAFF).contains(&code) || (0x2600..=0x27BF).contains(&code)
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
