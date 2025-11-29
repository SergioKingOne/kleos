# kleos-ingest-lambda

API Gateway Lambda that ingests user actions and publishes to Kinesis.

## Flow

```
POST /ingest → Validate (TryFrom) → Ingest (trait) → Publish (trait) → Kinesis
```

## Modules

| Module | Description |
|--------|-------------|
| `lib.rs` | `UserAction` enum + domain types (`UserId`, `PageUrl`, `Amount`) |
| `ingestor.rs` | `Ingestor<UserAction>` implementation |
| `publisher.rs` | `StreamPublisher<UserAction>` → Kinesis |
| `config.rs` | Environment config (`KINESIS_STREAM_NAME`) |

## Domain Types

```rust
enum UserAction {
    PageView { user_id: UserId, url: PageUrl },
    Click { user_id: UserId, element_id: String, url: PageUrl },
    Purchase { user_id: UserId, product_id: ProductId, amount: Amount },
}
```

Constrained types with validation:
- `PageUrl` - must start with `http` or `/`
- `Amount` - must be >= 0

## Trait Implementations

```rust
// Ingestor<UserAction>
impl Ingestor<UserAction> for UserActionIngestor {
    async fn ingest(&self, data: UserAction) -> Result<Event<UserAction, Created>, IngestError>;
}

// StreamPublisher<UserAction>
impl StreamPublisher<UserAction> for KinesisPublisher {
    async fn publish(&self, event: &Event<UserAction, Created>) -> Result<(), StreamError>;
}
```

## Request Format

```json
{
  "type": "PageView",
  "data": {
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "url": "/home"
  }
}
```

```json
{
  "type": "Click",
  "data": {
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "element_id": "buy-button",
    "url": "/product/123"
  }
}
```

```json
{
  "type": "Purchase",
  "data": {
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "product_id": "660e8400-e29b-41d4-a716-446655440001",
    "amount": 99.99
  }
}
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `KINESIS_STREAM_NAME` | Target Kinesis stream |
| `ENVIRONMENT` | dev/staging/prod |
| `RUST_LOG` | Log level (default: info) |

## Build

```bash
cargo lambda build --release --arm64 -p kleos-ingest-lambda
```
