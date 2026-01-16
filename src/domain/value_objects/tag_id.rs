//! TagId value object.

use std::str::FromStr;
use uuid::Uuid;

/// Unique identifier for a Tag.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagId(Uuid);

impl TagId {
    /// Creates a new random TagId.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Returns the underlying UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for TagId {
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for TagId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl std::fmt::Display for TagId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
