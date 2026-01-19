//! Strongly-typed identifiers for domain entities.

use std::str::FromStr;
use uuid::Uuid;

/// Unique identifier for a Course.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CourseId(Uuid);

impl CourseId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for CourseId {
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for CourseId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(Self)
    }
}

impl std::fmt::Display for CourseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a Module.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleId(Uuid);

impl ModuleId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for ModuleId {
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for ModuleId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(Self)
    }
}

impl std::fmt::Display for ModuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a Video.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VideoId(Uuid);

impl VideoId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for VideoId {
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for VideoId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(Self)
    }
}

impl std::fmt::Display for VideoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for an Exam.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExamId(Uuid);

impl ExamId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for ExamId {
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for ExamId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(Self)
    }
}

impl std::fmt::Display for ExamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
