use async_trait::async_trait;
use crate::events::{Event, Created};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IngestError {
    #[error("Failed to parse payload: {0}")]
    ParseError(String),
    #[error("Ingestion failed: {0}")]
    SystemError(String),
}

/// Defines how to receive data and convert it into an Event.
#[async_trait]
pub trait Ingestor<T>: Send + Sync {
    /// Ingests raw data and produces an Event in the Created state.
    async fn ingest(&self, data: T) -> Result<Event<T, Created>, IngestError>;
}
