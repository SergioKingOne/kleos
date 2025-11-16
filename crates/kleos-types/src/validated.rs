//! Validated types with construction-time guarantees
//!
//! Once created, these types are guaranteed to be valid.
//! No defensive checks needed in domain logic.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("String too long: max {max}, got {actual}")]
    TooLong { max: usize, actual: usize },

    #[error("String too short: min {min}, got {actual}")]
    TooShort { min: usize, actual: usize },

    #[error("String is empty")]
    Empty,

    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

/// String with maximum length constraint (100 chars)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct String100(String);

impl String100 {
    /// Create with validation. Returns None if string is empty or > 100 chars
    pub fn create(s: String) -> Result<Self, ValidationError> {
        let trimmed = s.trim().to_string();

        if trimmed.is_empty() {
            Err(ValidationError::Empty)
        } else if trimmed.len() > 100 {
            Err(ValidationError::TooLong {
                max: 100,
                actual: trimmed.len(),
            })
        } else {
            Ok(String100(trimmed))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for String100 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// String with maximum length constraint (1000 chars)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct String1000(String);

impl String1000 {
    pub fn create(s: String) -> Result<Self, ValidationError> {
        let trimmed = s.trim().to_string();

        if trimmed.is_empty() {
            Err(ValidationError::Empty)
        } else if trimmed.len() > 1000 {
            Err(ValidationError::TooLong {
                max: 1000,
                actual: trimmed.len(),
            })
        } else {
            Ok(String1000(trimmed))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for String1000 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Non-negative integer
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NonNegativeInt(i32);

impl NonNegativeInt {
    pub fn create(value: i32) -> Option<Self> {
        if value >= 0 {
            Some(NonNegativeInt(value))
        } else {
            None
        }
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}

impl fmt::Display for NonNegativeInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Positive integer (> 0)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PositiveInt(i32);

impl PositiveInt {
    pub fn create(value: i32) -> Option<Self> {
        if value > 0 {
            Some(PositiveInt(value))
        } else {
            None
        }
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}

impl fmt::Display for PositiveInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
