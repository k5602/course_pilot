//! Exam entity - A validation test for a video.

use crate::domain::value_objects::{ExamId, VideoId};

/// Pass threshold for an exam (70%).
const PASS_THRESHOLD: f32 = 0.70;

/// An exam represents an AI-generated MCQ test for a video.
#[derive(Debug, Clone, PartialEq)]
pub struct Exam {
    id: ExamId,
    video_id: VideoId,
    question_json: String,
    score: Option<f32>,
    passed: Option<bool>,
    user_answers_json: Option<String>,
}

impl Exam {
    /// Creates a new exam with questions but no score yet.
    pub fn new(id: ExamId, video_id: VideoId, question_json: String) -> Self {
        Self { id, video_id, question_json, score: None, passed: None, user_answers_json: None }
    }

    pub fn id(&self) -> &ExamId {
        &self.id
    }

    pub fn video_id(&self) -> &VideoId {
        &self.video_id
    }

    pub fn question_json(&self) -> &str {
        &self.question_json
    }

    pub fn score(&self) -> Option<f32> {
        self.score
    }

    pub fn passed(&self) -> Option<bool> {
        self.passed
    }

    pub fn user_answers_json(&self) -> Option<&str> {
        self.user_answers_json.as_deref()
    }

    /// Records the exam result with a score between 0.0 and 1.0.
    pub fn record_result(&mut self, score: f32, answers_json: Option<String>) {
        let clamped = score.clamp(0.0, 1.0);
        self.score = Some(clamped);
        self.passed = Some(clamped >= PASS_THRESHOLD);
        self.user_answers_json = answers_json;
    }

    /// Returns true if the exam has been taken.
    pub fn is_taken(&self) -> bool {
        self.score.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exam_pass() {
        let mut exam =
            Exam::new(ExamId::new(), VideoId::new(), r#"[{"question": "test"}]"#.to_string());
        exam.record_result(0.8, None);
        assert!(exam.passed().unwrap());
    }

    #[test]
    fn test_exam_fail() {
        let mut exam =
            Exam::new(ExamId::new(), VideoId::new(), r#"[{"question": "test"}]"#.to_string());
        exam.record_result(0.5, None);
        assert!(!exam.passed().unwrap());
    }
}
