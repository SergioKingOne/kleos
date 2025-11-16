//! Strongly-typed IDs using newtype pattern
//!
//! Prevents mixing semantically different IDs at compile time

use serde::{Deserialize, Serialize};
use std::fmt;

/// Request ID for tracking API requests through the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestId(String);

impl RequestId {
    /// Create a new RequestId from a string
    pub fn new(id: impl Into<String>) -> Self {
        RequestId(id.into())
    }

    /// Generate a new UUID-based RequestId
    pub fn generate() -> Self {
        RequestId(uuid_like())
    }

    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Stream record ID for Kinesis records
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StreamRecordId(String);

impl StreamRecordId {
    pub fn new(id: impl Into<String>) -> Self {
        StreamRecordId(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StreamRecordId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Partition key for Kinesis stream sharding
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PartitionKey(String);

impl PartitionKey {
    /// Create a partition key (max 256 bytes)
    pub fn create(key: String) -> Option<Self> {
        if key.is_empty() || key.len() > 256 {
            None
        } else {
            Some(PartitionKey(key))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PartitionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Simple UUID-like generation for demo purposes
fn uuid_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", timestamp)
}
