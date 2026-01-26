//! Boundary Detector - Groups videos into modules.

use std::collections::BTreeSet;

/// Groups videos into modules using title-aware patterns with a batch-size fallback.
/// - Detects hierarchical numbering: `1.5`, `1.5.1`
/// - Detects labeled patterns: `Module 2`, `Chapter 3.1`, `Week 4`
/// - Handles hybrid mixes (labels + dotted numbers + plain leading numbers)
#[derive(Debug)]
pub struct BoundaryDetector {
    batch_size: usize,
}

impl BoundaryDetector {
    /// Creates a boundary detector with default batch size (5 videos per module).
    pub fn new() -> Self {
        Self { batch_size: 5 }
    }

    /// Creates a boundary detector with a custom batch size.
    pub fn with_batch_size(batch_size: usize) -> Self {
        Self { batch_size: batch_size.max(1) }
    }

    /// Groups video indices into modules (each module has up to `batch_size` videos).
    /// Returns a vector of vectors, where each inner vector contains video indices for a module.
    pub fn group_into_modules(&self, video_count: usize) -> Vec<Vec<usize>> {
        if video_count == 0 {
            return vec![];
        }

        (0..video_count)
            .collect::<Vec<_>>()
            .chunks(self.batch_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }

    /// Groups videos into modules using title-aware boundary detection.
    /// Falls back to `group_into_modules` if signal is weak or ambiguous.
    pub fn group_by_titles<T: AsRef<str>>(&self, titles: &[T]) -> Vec<Vec<usize>> {
        if titles.is_empty() {
            return vec![];
        }

        let keys: Vec<Option<BoundaryKey>> =
            titles.iter().map(|t| boundary_key(t.as_ref())).collect();

        let matched = keys.iter().filter(|k| k.is_some()).count();
        let total = titles.len();
        let matched_ratio = matched as f32 / total as f32;

        let distinct_majors: BTreeSet<u32> =
            keys.iter().filter_map(|k| k.as_ref().map(|key| key.major)).collect();

        // Weak signal: fallback to batch grouping.
        if matched < 2 || matched_ratio < 0.5 {
            return self.group_into_modules(total);
        }

        // Only one major detected: fallback to batch grouping.
        if distinct_majors.len() <= 1 {
            return self.group_into_modules(total);
        }

        // Split on major changes in observed order.
        let mut groups: Vec<Vec<usize>> = Vec::new();
        let mut current_group: Vec<usize> = Vec::new();
        let mut current_major: Option<u32> = None;

        for (idx, key) in keys.iter().enumerate() {
            if let Some(key) = key {
                if let Some(active) = current_major {
                    if key.major != active && !current_group.is_empty() {
                        groups.push(current_group);
                        current_group = Vec::new();
                    }
                }
                current_major = Some(key.major);
            }

            current_group.push(idx);
        }

        if !current_group.is_empty() {
            groups.push(current_group);
        }

        groups
    }
}

impl Default for BoundaryDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BoundaryKey {
    major: u32,
    full: Vec<u32>,
}

fn boundary_key(title: &str) -> Option<BoundaryKey> {
    let title_trimmed = title.trim();
    if title_trimmed.is_empty() {
        return None;
    }

    if let Some(nums) = parse_labeled_sequence(title_trimmed) {
        return Some(BoundaryKey { major: nums[0], full: nums });
    }

    if let Some(nums) = parse_leading_sequence(title_trimmed) {
        // Avoid over-splitting on simple leading numbers like "1 Intro".
        if nums.len() == 1 {
            return None;
        }
        return Some(BoundaryKey { major: nums[0], full: nums });
    }

    None
}

fn parse_labeled_sequence(title: &str) -> Option<Vec<u32>> {
    let lower = title.to_lowercase();
    let labels = [
        "module", "chapter", "section", "part", "lesson", "lecture", "unit", "week", "day",
        "topic", "track", "stage",
    ];

    for label in labels {
        if let Some(pos) = find_word(&lower, label) {
            let start = pos + label.len();
            let mut idx = start;
            let bytes = lower.as_bytes();

            // Skip separators and whitespace after label.
            while idx < bytes.len() {
                let c = bytes[idx] as char;
                if c.is_whitespace() || matches!(c, ':' | '-' | '#' | '.' | '(' | '[') {
                    idx += 1;
                } else {
                    break;
                }
            }

            if let Some(nums) = parse_number_sequence(&lower, idx) {
                if !nums.is_empty() {
                    return Some(nums);
                }
            }
        }
    }

    None
}

fn parse_leading_sequence(title: &str) -> Option<Vec<u32>> {
    let mut idx = 0;
    let bytes = title.as_bytes();

    while idx < bytes.len() {
        let c = bytes[idx] as char;
        if c.is_whitespace() || matches!(c, '(' | '[' | '{' | '-' | '#') {
            idx += 1;
        } else {
            break;
        }
    }

    parse_number_sequence(title, idx)
}

fn parse_number_sequence(text: &str, start: usize) -> Option<Vec<u32>> {
    let bytes = text.as_bytes();
    let mut idx = start;
    let mut numbers: Vec<u32> = Vec::new();
    let mut current: Option<u32> = None;
    let mut saw_digit = false;

    while idx < bytes.len() {
        let c = bytes[idx] as char;
        if c.is_ascii_digit() {
            saw_digit = true;
            let digit = (c as u8 - b'0') as u32;
            current = Some(current.unwrap_or(0).saturating_mul(10).saturating_add(digit));
            idx += 1;
            continue;
        }

        let is_sep = matches!(c, '.' | '-' | '_' | '·');
        if is_sep {
            if let Some(num) = current.take() {
                numbers.push(num);
            } else if saw_digit {
                // e.g., "1..2" -> stop on malformed sequence
                break;
            }
            idx += 1;
            continue;
        }

        // Stop on non-separator, non-digit once a sequence has started
        if saw_digit {
            break;
        }
        idx += 1;
    }

    if let Some(num) = current.take() {
        numbers.push(num);
    }

    if numbers.is_empty() { None } else { Some(numbers) }
}

fn find_word(haystack: &str, needle: &str) -> Option<usize> {
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(needle) {
        let abs = start + pos;
        let before = abs.saturating_sub(1);
        let after = abs + needle.len();

        let before_ok = abs == 0 || !haystack.as_bytes()[before].is_ascii_alphabetic();
        let after_ok = after >= haystack.len() || !haystack.as_bytes()[after].is_ascii_alphabetic();

        if before_ok && after_ok {
            return Some(abs);
        }
        start = abs + needle.len();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_by_major_from_dotted_numbers() {
        let titles =
            vec!["1.1 Intro", "1.2 Basics", "1.3 Ownership", "2.1 Lifetimes", "2.2 Borrowing"];
        let detector = BoundaryDetector::new();
        let groups = detector.group_by_titles(&titles);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0], vec![0, 1, 2]);
        assert_eq!(groups[1], vec![3, 4]);
    }

    #[test]
    fn groups_by_labeled_numbers() {
        let titles = vec![
            "Module 1 - Setup",
            "Lesson 1.1 Installing",
            "Module 2 - Basics",
            "Lesson 2.1 Variables",
        ];
        let detector = BoundaryDetector::new();
        let groups = detector.group_by_titles(&titles);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0], vec![0, 1]);
        assert_eq!(groups[1], vec![2, 3]);
    }

    #[test]
    fn keeps_single_major_as_one_group() {
        let titles = vec!["1.1 Intro", "1.2 Basics", "1.3 Ownership"];
        let detector = BoundaryDetector::new();
        let groups = detector.group_by_titles(&titles);

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0], vec![0, 1, 2]);
    }

    #[test]
    fn falls_back_on_weak_signal() {
        let titles = vec!["Intro", "Deep Dive", "Advanced Topics"];
        let detector = BoundaryDetector::with_batch_size(2);
        let groups = detector.group_by_titles(&titles);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0], vec![0, 1]);
        assert_eq!(groups[1], vec![2]);
    }

    #[test]
    fn parses_hybrid_separators() {
        assert_eq!(parse_number_sequence("1-5-1 Intro", 0), Some(vec![1, 5, 1]));
        assert_eq!(parse_number_sequence("1_5 Intro", 0), Some(vec![1, 5]));
    }
}
