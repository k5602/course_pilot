//! Take Exam Use Case
//!
//! Generates MCQ, scores answers, and updates video completion.

use std::sync::Arc;

use crate::domain::{
    entities::Exam,
    ports::{ExamRepository, ExaminerAI, MCQuestion, VideoRepository},
    value_objects::{ExamId, VideoId},
};

/// Error type for exam operations.
#[derive(Debug, thiserror::Error)]
pub enum ExamError {
    #[error("Video not found")]
    VideoNotFound,
    #[error("Exam not found")]
    ExamNotFound,
    #[error("AI error: {0}")]
    AI(String),
    #[error("Repository error: {0}")]
    Repository(String),
}

/// Input for generating an exam.
pub struct GenerateExamInput {
    pub video_id: VideoId,
    pub num_questions: u8,
}

/// Output of exam generation.
pub struct GenerateExamOutput {
    pub exam_id: ExamId,
    pub questions: Vec<MCQuestion>,
}

/// Input for submitting exam answers.
pub struct SubmitExamInput {
    pub exam_id: ExamId,
    pub answers: Vec<usize>, // User's selected answer indices
}

/// Output of exam submission.
#[derive(Debug)]
pub struct SubmitExamOutput {
    pub score: f32,
    pub passed: bool,
    pub video_marked_complete: bool,
}

/// Use case for taking exams.
pub struct TakeExamUseCase<AI, VR, ER>
where
    AI: ExaminerAI,
    VR: VideoRepository,
    ER: ExamRepository,
{
    examiner: Arc<AI>,
    video_repo: Arc<VR>,
    exam_repo: Arc<ER>,
}

impl<AI, VR, ER> TakeExamUseCase<AI, VR, ER>
where
    AI: ExaminerAI,
    VR: VideoRepository,
    ER: ExamRepository,
{
    pub fn new(examiner: Arc<AI>, video_repo: Arc<VR>, exam_repo: Arc<ER>) -> Self {
        Self { examiner, video_repo, exam_repo }
    }

    /// Generates an exam for a video.
    pub async fn generate(
        &self,
        input: GenerateExamInput,
    ) -> Result<GenerateExamOutput, ExamError> {
        // Get video
        let video = self
            .video_repo
            .find_by_id(&input.video_id)
            .map_err(|e| ExamError::Repository(e.to_string()))?
            .ok_or(ExamError::VideoNotFound)?;

        // Generate questions via AI
        let questions = self
            .examiner
            .generate_mcq(video.title(), None, input.num_questions)
            .await
            .map_err(|e| ExamError::AI(e.to_string()))?;

        // Create and save exam
        let exam_id = ExamId::new();
        let question_json =
            serde_json::to_string(&questions).map_err(|e| ExamError::AI(e.to_string()))?;

        let exam = Exam::new(exam_id.clone(), input.video_id, question_json);
        self.exam_repo.save(&exam).map_err(|e| ExamError::Repository(e.to_string()))?;

        Ok(GenerateExamOutput { exam_id, questions })
    }

    /// Retrieves an exam and its questions.
    pub fn get_exam(&self, exam_id: &ExamId) -> Result<(Exam, Vec<MCQuestion>), ExamError> {
        let exam = self
            .exam_repo
            .find_by_id(exam_id)
            .map_err(|e| ExamError::Repository(e.to_string()))?
            .ok_or(ExamError::ExamNotFound)?;

        let questions: Vec<MCQuestion> = serde_json::from_str(exam.question_json())
            .map_err(|e| ExamError::AI(format!("Failed to parse exam questions: {}", e)))?;

        Ok((exam, questions))
    }

    /// Submits exam answers and calculates score.
    pub fn submit(&self, input: SubmitExamInput) -> Result<SubmitExamOutput, ExamError> {
        let (exam, questions) = self.get_exam(&input.exam_id)?;

        // Calculate score
        let correct_count = input
            .answers
            .iter()
            .zip(questions.iter())
            .filter(|(answer, q)| **answer == q.correct_index)
            .count();

        let score =
            if questions.is_empty() { 0.0 } else { correct_count as f32 / questions.len() as f32 };

        let passed = score >= 0.7;
        let user_answers_json = serde_json::to_string(&input.answers).ok();

        // Update exam with result
        self.exam_repo
            .update_result(&input.exam_id, score, passed, user_answers_json)
            .map_err(|e| ExamError::Repository(e.to_string()))?;

        // Mark video as complete if passed
        let mut video_marked_complete = false;
        if passed {
            self.video_repo
                .update_completion(exam.video_id(), true)
                .map_err(|e| ExamError::Repository(e.to_string()))?;
            video_marked_complete = true;
        }

        Ok(SubmitExamOutput { score, passed, video_marked_complete })
    }
}
