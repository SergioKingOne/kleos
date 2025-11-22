use async_trait::async_trait;
use crate::events::{Event, Created, Consumed, EventId};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StreamError {
    #[error("Failed to publish to stream: {0}")]
    PublishError(String),
    #[error("Failed to consume from stream: {0}")]
    ConsumeError(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
}

/// Defines how to push an Event to a stream.
#[async_trait]
pub trait StreamPublisher<T>: Send + Sync {
    /// Publishes a Created event to the stream.
    async fn publish(&self, event: &Event<T, Created>) -> Result<(), StreamError>;
}

/// Defines how to consume Events from a stream.
#[async_trait]
pub trait StreamConsumer<T>: Send + Sync {
    /// Consumes the next event from the stream, returning it in the Consumed state.
    async fn consume(&self) -> Result<Option<Event<T, Consumed>>, StreamError>;
    
    /// Acknowledges that an event has been successfully processed.
    async fn ack(&self, event_id: &EventId) -> Result<(), StreamError>;
}
