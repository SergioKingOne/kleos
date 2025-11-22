# kleos-lib

Type-safe event pipeline library using compile-time state tracking.

## Core Traits

```rust
#[async_trait]
pub trait Ingestor<T>: Send + Sync {
    async fn ingest(&self, data: T) -> Result<Event<T, Created>, IngestError>;
}

#[async_trait]
pub trait StreamPublisher<T>: Send + Sync {
    async fn publish(&self, event: &Event<T, Created>) -> Result<(), StreamError>;
}

#[async_trait]
pub trait StreamConsumer<T>: Send + Sync {
    async fn consume(&self) -> Result<Option<Event<T, Consumed>>, StreamError>;
    async fn ack(&self, event_id: &EventId) -> Result<(), StreamError>;
}

#[async_trait]
pub trait Processor<T>: Send + Sync {
    async fn process(&self, event: &Event<T, Consumed>) -> Result<ProcessingResult, ProcessingError>;
}
```

## Type States

- `Event<T, Created>` - Created, can be published
- `Event<T, Consumed>` - Consumed from stream, can be processed

State transitions via `event.transition()` prevent invalid operations at compile time.

## Example

See `examples/basic_flow.rs` for complete implementation.

## License

MIT OR Apache-2.0
