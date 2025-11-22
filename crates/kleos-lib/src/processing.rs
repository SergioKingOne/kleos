use async_trait::async_trait;
use crate::events::{Event, Consumed};
use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Processing failed: {0}")]
    ExecutionError(String),
    #[error("Transient error, retriable: {0}")]
    TransientError(String),
}

/// The result of processing an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingResult {
    Success,
    Failure(String),
    Skipped(String),
}

/// Defines how to process an Event.
#[async_trait]
pub trait Processor<T>: Send + Sync {
    /// Processes a Consumed event and returns a result.
    async fn process(&self, event: &Event<T, Consumed>) -> Result<ProcessingResult, ProcessingError>;
}
