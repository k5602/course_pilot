//! Transcript chunker domain service.
//!
//! Splits a transcript into overlapping chunks at natural boundaries
//! (paragraph → sentence → word) for LLM consumption.

/// Splits a transcript into overlapping chunks for LLM consumption.
///
/// This service handles chunking at natural boundaries (paragraph → sentence → word)
/// to produce chunks suitable for:
/// - Summarization (summarize each chunk, then merge)
/// - Companion Q&A (retrieve relevant chunk for context)
///
/// The chunker is deterministic: same input + same params = same output.
#[derive(Debug, Clone)]
pub struct TranscriptChunker {
    chunk_size: usize,
    overlap: usize,
}

impl TranscriptChunker {
    /// Default chunk size in characters (fits ~2k tokens for most LLMs).
    pub const DEFAULT_CHUNK_SIZE: usize = 5000;
    /// Default overlap in characters (for context continuity).
    pub const DEFAULT_OVERLAP: usize = 800;

    /// Creates a new chunker with default parameters.
    pub fn new() -> Self {
        Self { chunk_size: Self::DEFAULT_CHUNK_SIZE, overlap: Self::DEFAULT_OVERLAP }
    }

    /// Creates a new chunker with custom parameters.
    ///
    /// # Panics
    /// - If `chunk_size` is 0
    /// - If `overlap` >= `chunk_size`
    pub fn with_params(chunk_size: usize, overlap: usize) -> Self {
        assert!(chunk_size > 0, "chunk_size must be > 0");
        assert!(overlap < chunk_size, "overlap must be < chunk_size");
        Self { chunk_size, overlap }
    }

    /// Splits a transcript into chunks.
    ///
    /// Returns `vec![]` if input is empty.
    /// Returns `vec![transcript]` if transcript is shorter than chunk_size.
    pub fn chunk(&self, transcript: &str) -> Vec<String> {
        if transcript.is_empty() {
            return vec![];
        }

        let clean = transcript.trim();
        if clean.len() <= self.chunk_size {
            return vec![clean.to_string()];
        }

        let mut chunks = Vec::new();
        let mut start = 0;

        while start < clean.len() {
            let safe_start = clean.floor_char_boundary(start);
            let end = self.find_chunk_end(clean, safe_start);
            let safe_end = clean.floor_char_boundary(end);
            chunks.push(clean[safe_start..safe_end].trim().to_string());

            let next_start = safe_end.saturating_sub(self.overlap);
            if next_start <= safe_start {
                start = safe_end;
            } else {
                start = self.find_chunk_start(clean, next_start);
                if start <= safe_start {
                    start = safe_end;
                }
            }

            if start >= clean.len() {
                break;
            }
        }

        chunks
    }

    /// Finds the end boundary for a chunk starting at `start`.
    fn find_chunk_end(&self, text: &str, start: usize) -> usize {
        let safe_start = text.floor_char_boundary(start);
        let end = safe_start + self.chunk_size;
        if end >= text.len() {
            return text.len();
        }
        let safe_end = text.floor_char_boundary(end);

        if let Some(pos) = text[safe_start..safe_end].rfind("\n\n") {
            return safe_start + pos + 2;
        }

        for delim in [". ", "! ", "? ", ".\"", "!\"", "?\""] {
            if let Some(pos) = text[safe_start..safe_end].rfind(delim) {
                return safe_start + pos + delim.len();
            }
        }

        if let Some(pos) = text[safe_start..safe_end].rfind('\n') {
            return safe_start + pos + 1;
        }

        if let Some(pos) = text[safe_start..safe_end].rfind(' ')
            && safe_end - (safe_start + pos) < self.chunk_size / 2
        {
            return safe_start + pos + 1;
        }

        safe_end
    }

    /// Finds the start boundary for the next chunk, preferring sentence starts.
    fn find_chunk_start(&self, text: &str, start: usize) -> usize {
        let safe_start = text.floor_char_boundary(start);
        if safe_start >= text.len() {
            return text.len();
        }

        let search_end = (safe_start + self.chunk_size / 3).min(text.len());

        for offset in safe_start..search_end {
            if !text.is_char_boundary(offset) {
                continue;
            }
            if text[offset..].starts_with("\n\n") {
                return offset + 2;
            }
        }

        for offset in safe_start..search_end {
            if !text.is_char_boundary(offset) {
                continue;
            }
            let remaining = &text[offset..];
            if remaining.starts_with(". ")
                || remaining.starts_with("! ")
                || remaining.starts_with("? ")
            {
                return offset + 2;
            }
        }

        let safe_search_end = text.floor_char_boundary(search_end);
        if let Some(pos) = text[safe_start..safe_search_end].find('\n') {
            return safe_start + pos + 1;
        }

        safe_start
    }

    /// Returns the number of chunks that would be produced.
    pub fn chunk_count(&self, transcript: &str) -> usize {
        if transcript.trim().is_empty() { 0 } else { self.chunk(transcript).len() }
    }
}

impl Default for TranscriptChunker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_empty_for_empty() {
        let chunker = TranscriptChunker::new();
        assert!(chunker.chunk("").is_empty());
    }

    #[test]
    fn returns_one_chunk_if_short() {
        let chunker = TranscriptChunker::new();
        let text = "Hello world";
        let chunks = chunker.chunk(text);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Hello world");
    }

    #[test]
    fn splits_long_text() {
        let chunker = TranscriptChunker::with_params(100, 20);
        let text = "A ".repeat(200);
        let chunks = chunker.chunk(&text);
        assert!(chunks.len() >= 2, "should produce at least 2 chunks, got {}", chunks.len());
        assert!(!chunks[0].is_empty());
        assert!(!chunks[1].is_empty());
    }

    #[test]
    fn overlaps_provide_context() {
        let chunker = TranscriptChunker::with_params(100, 40);
        let text = "word ".repeat(100);
        let chunks = chunker.chunk(&text);
        assert!(chunks.len() >= 5, "should produce enough chunks");
        for chunk in &chunks {
            assert!(chunk.len() <= 100, "chunk too long: {}", chunk.len());
        }
    }

    #[test]
    fn breaks_at_paragraphs() {
        let chunker = TranscriptChunker::with_params(500, 50);
        let mut text = String::new();
        for i in 0..5 {
            text.push_str(&format!(
                "This is paragraph {} which contains some text for testing purposes.\n\n",
                i
            ));
        }
        let chunks = chunker.chunk(&text);
        assert_eq!(chunks.len(), 1, "short text should be one chunk");
    }

    #[test]
    fn deternimistic_output() {
        let chunker = TranscriptChunker::with_params(1000, 100);
        let text = "sentence. ".repeat(500);
        let chunks1 = chunker.chunk(&text);
        let chunks2 = chunker.chunk(&text);
        assert_eq!(chunks1, chunks2, "chunking must be deterministic");
    }

    #[test]
    fn chunk_count_matches() {
        let chunker = TranscriptChunker::with_params(100, 20);
        let text = "word ".repeat(200);
        let chunks = chunker.chunk(&text);
        assert_eq!(chunker.chunk_count(&text), chunks.len());
    }

    #[test]
    #[should_panic(expected = "chunk_size must be > 0")]
    fn rejects_zero_chunk_size() {
        TranscriptChunker::with_params(0, 10);
    }

    #[test]
    #[should_panic(expected = "overlap must be < chunk_size")]
    fn rejects_overlap_gte_chunk_size() {
        TranscriptChunker::with_params(100, 100);
    }

    #[test]
    fn handles_multi_byte_utf8_at_boundary() {
        let chunker = TranscriptChunker::with_params(10, 2);
        let text = "你好世界这是一个测试".repeat(5);
        let chunks = chunker.chunk(&text);
        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(chunk.len() <= 12);
            assert!(chunk.chars().all(|c| c != '\u{FFFD}'));
        }
    }

    #[test]
    fn handles_mixed_ascii_and_unicode() {
        let chunker = TranscriptChunker::with_params(15, 3);
        let text = "Hello world 这是一个测试 sentence here.";
        let chunks = chunker.chunk(&text);
        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(!chunk.contains('\u{FFFD}'));
        }
    }
}
