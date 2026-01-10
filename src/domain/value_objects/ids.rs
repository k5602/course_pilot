//! Strongly-typed identifiers for domain entities.

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

    pub fn from_str(s: &str) -> Result<Self, uuid::Error> {
        Uuid::parse_str(s).map(Self)
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

    pub fn from_str(s: &str) -> Result<Self, uuid::Error> {
        Uuid::parse_str(s).map(Self)
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
