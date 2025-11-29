# Kleos

Rust library and AWS Lambda pipeline for event-driven data processing.

## Architecture

```
POST /ingest → Ingest Lambda → Kinesis → Process Lambda → (your logic)
                                              ↓ failure
                                           SQS DLQ
```

## Crates

| Crate | Description |
|-------|-------------|
| `kleos-lib` | Core traits: `Ingestor`, `StreamPublisher`, `StreamConsumer`, `Processor` |
| `kleos-ingest-lambda` | API Gateway → Kinesis publisher |
| `kleos-process-lambda` | Kinesis consumer → processor |

## Type-State Pattern

Events enforce correct lifecycle at compile time:

```rust
Event<T, Created>  // Fresh event, ready to publish
Event<T, Consumed> // From stream, ready to process
```

Transitions are explicit via `event.transition()`.

## Core Traits

```rust
trait Ingestor<T> {
    async fn ingest(&self, data: T) -> Result<Event<T, Created>, IngestError>;
}

trait StreamPublisher<T> {
    async fn publish(&self, event: &Event<T, Created>) -> Result<(), StreamError>;
}

trait StreamConsumer<T> {
    async fn load(&self, records: Vec<&[u8]>) -> Result<(), StreamError>;
    async fn consume(&self) -> Result<Option<Event<T, Consumed>>, StreamError>;
    async fn ack(&self, event_id: &EventId) -> Result<(), StreamError>;
}

trait Processor<T> {
    async fn process(&self, event: &Event<T, Consumed>) -> Result<ProcessingResult, ProcessingError>;
}
```

## Quick Start

```bash
# Prerequisites
cargo install cargo-lambda
brew install aws-sam-cli  # or pip install aws-sam-cli

# Build
make build

# Test
make test

# Deploy
make deploy ENV=dev

# Tail logs
make logs FUNC=ingest ENV=dev

# Destroy
make destroy ENV=dev
```

## Example Request

```bash
curl -X POST https://<api-id>.execute-api.us-east-1.amazonaws.com/dev/ingest \
  -H "Content-Type: application/json" \
  -d '{"PageView": {"path": "/home"}}'
```

Event types: `PageView`, `Click`, `Purchase`

## Infrastructure

SAM nested stacks in `infra/`:

- `api/` - API Gateway REST API
- `streams/` - Kinesis + SQS DLQ
- `functions/ingest/` - Ingest Lambda + API method
- `functions/process/` - Process Lambda + Kinesis trigger

## Development

```bash
make help         # Show all commands
make fmt          # Format
make lint         # Clippy
make validate     # SAM template validation
make plan ENV=dev # Preview changes
```

## License

MIT OR Apache-2.0
