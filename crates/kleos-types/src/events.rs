//! Domain events for the stream processing pipeline
//!
//! Demonstrates type-driven design:
//! - Each event type contains only relevant data
//! - Illegal states are unrepresentable
//! - State transitions are explicit

use crate::{RequestId, String100, String1000};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Envelope for all events sent through the stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    /// Unique identifier for tracing this event through the system
    pub request_id: RequestId,

    /// When this event was created
    pub timestamp: DateTime<Utc>,

    /// The actual event payload
    pub payload: EventPayload,
}

impl StreamEvent {
    pub fn new(request_id: RequestId, payload: EventPayload) -> Self {
        Self {
            request_id,
            timestamp: Utc::now(),
            payload,
        }
    }
}

/// All possible event types in the system
///
/// Using enum ensures exhaustive pattern matching - adding a new event type
/// will cause compilation errors everywhere events are processed, forcing
/// developers to handle the new case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum EventPayload {
    /// A user action occurred
    UserAction(UserActionEvent),

    /// A system event occurred
    SystemEvent(SystemEventData),

    /// Data was processed
    DataProcessed(DataProcessedEvent),
}

/// User-initiated action event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActionEvent {
    pub user_id: String100,
    pub action_type: ActionType,
    pub details: String1000,
}

/// Types of user actions
///
/// Enum ensures only valid action types can be created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Create,
    Update,
    Delete,
    View,
}

/// System-generated event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEventData {
    pub source: String100,
    pub severity: Severity,
    pub message: String1000,
}

/// Event severity levels
///
/// Ordered enum allows comparison: Critical > Error > Warning > Info
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Data processing result event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProcessedEvent {
    pub original_request_id: RequestId,
    pub result: ProcessingResult,
}

/// Processing result - makes illegal states unrepresentable
///
/// Cannot have both success and error data simultaneously.
/// Pattern matching forces handling of both cases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingResult {
    Success {
        records_processed: u32,
        duration_ms: u64,
    },
    Failed {
        error_message: String1000,
        retry_count: u8,
    },
}

impl ProcessingResult {
    /// Helper to check if processing succeeded
    pub fn is_success(&self) -> bool {
        matches!(self, ProcessingResult::Success { .. })
    }

    /// Helper to check if processing failed
    pub fn is_failed(&self) -> bool {
        matches!(self, ProcessingResult::Failed { .. })
    }
}
