# kleos-process-lambda

Kinesis consumer Lambda that processes user action events.

## Flow

```
Kinesis → Load batch (trait) → Consume (trait) → Process (trait) → Ack
                                                       ↓ failure
                                                    SQS DLQ
```

## Modules

| Module | Description |
|--------|-------------|
| `lib.rs` | `ProcessableRecord<T>` wrapper + metadata extraction |
| `consumer.rs` | `StreamConsumer<UserAction>` implementation |
| `processor.rs` | `Processor<UserAction>` implementation |
| `config.rs` | Environment config |

## Trait Implementations

```rust
// StreamConsumer<UserAction>
impl StreamConsumer<UserAction> for KinesisBatchConsumer {
    async fn load(&self, records: Vec<&[u8]>) -> Result<(), StreamError>;
    async fn consume(&self) -> Result<Option<Event<UserAction, Consumed>>, StreamError>;
    async fn ack(&self, event_id: &EventId) -> Result<(), StreamError>;
}

// Processor<UserAction>
impl Processor<UserAction> for UserActionProcessor {
    async fn process(&self, event: &Event<UserAction, Consumed>) -> Result<ProcessingResult, ProcessingError>;
}
```

## Processing Logic

Current implementation logs events. Extend `UserActionProcessor` for real business logic:

```rust
match &event.payload.data {
    UserAction::PageView { user_id, url } => { /* analytics */ }
    UserAction::Click { user_id, element_id, url } => { /* tracking */ }
    UserAction::Purchase { user_id, product_id, amount } => { /* order processing */ }
}
```

## Kinesis Event Source Config

- Batch size: 100 records
- Batching window: 5 seconds
- Retry attempts: 3
- Bisect on error: enabled
- Failed batches: SQS DLQ

## Environment Variables

| Variable | Description |
|----------|-------------|
| `ENVIRONMENT` | dev/staging/prod |
| `RUST_LOG` | Log level (default: info) |

## Build

```bash
cargo lambda build --release --arm64 -p kleos-process-lambda
```

## Dependencies

Imports `UserAction` from `kleos-ingest-lambda` to share domain types.
