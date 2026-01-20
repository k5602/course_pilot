//! Exam difficulty value object.

use std::fmt;
use std::str::FromStr;

/// Difficulty level for generated exams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[derive(Default)]
pub enum ExamDifficulty {
    Easy,
    #[default]
    Medium,
    Hard,
}

impl ExamDifficulty {
    /// Returns the canonical string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Easy => "easy",
            Self::Medium => "medium",
            Self::Hard => "hard",
        }
    }
}


impl fmt::Display for ExamDifficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Error returned when parsing an invalid difficulty.
#[derive(Debug, thiserror::Error)]
pub enum ExamDifficultyParseError {
    #[error("Invalid exam difficulty: {0}")]
    Invalid(String),
}

impl FromStr for ExamDifficulty {
    type Err = ExamDifficultyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "easy" => Ok(Self::Easy),
            "medium" => Ok(Self::Medium),
            "hard" => Ok(Self::Hard),
            _ => Err(ExamDifficultyParseError::Invalid(s.to_string())),
        }
    }
}
