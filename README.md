# Kleos - Type-Driven AWS Serverless Event Streaming

A Rust-based AWS serverless architecture for event streaming using API Gateway, Lambda, and Kinesis Data Streams, built with **type-driven development** principles.

## Architecture

```
┌──────────────┐     ┌──────────────────┐     ┌─────────────────┐     ┌───────────────────────┐
│              │     │                  │     │                 │     │                       │
│ API Gateway  │────▶│ Producer Lambda  │────▶│ Kinesis Stream  │────▶│ Consumer Lambda       │
│              │     │  (API Handler)   │     │  (Event Store)  │     │  (Event Processor)    │
│              │     │                  │     │                 │     │                       │
└──────────────┘     └──────────────────┘     └─────────────────┘     └───────────────────────┘
                              │                                                   │
                              │                                                   │
                              └────── Validated StreamEvent ────────────────────┘
                                                                                 │
                                                                                 ▼
                                                                    Type-safe processing
                                                                    Exhaustive matching
```

## Type-Driven Development Philosophy

This project demonstrates **type-driven development** principles from F# for Fun and Profit, applied to Rust:

1. **Make illegal states unrepresentable** - Use enums and types to prevent invalid data
2. **Construction-time validation** - Validate once at creation, never at use
3. **Newtype pattern** - Wrap primitives for semantic type safety
4. **Exhaustive pattern matching** - Compiler ensures all cases handled
5. **Self-documenting code** - Types reveal business rules at a glance

### Type Safety Examples

```rust
// Prevent mixing semantically different IDs
let request_id = RequestId::generate();
let stream_id = StreamRecordId::new("123");
// request_id == stream_id  // Won't compile! Different types

// Construction-time validation
let user_id = String100::create("user@example.com".to_string())?;
// Now guaranteed to be non-empty and <= 100 chars

// Make illegal states unrepresentable
enum ProcessingResult {
    Success { records: u32, duration_ms: u64 },
    Failed { error: String1000, retry_count: u8 },
}
// Cannot have both success and failure data!

// Exhaustive matching enforced by compiler
match event.payload {
    EventPayload::UserAction(e) => process_user_action(e),
    EventPayload::SystemEvent(e) => process_system_event(e),
    EventPayload::DataProcessed(e) => process_data_processed(e),
    // Compiler error if new variant added but not handled
}
```

## Project Structure

```
kleos/
├── Cargo.toml                              # Workspace configuration
├── crates/
│   ├── kleos-types/                        # Shared domain types (type-driven)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ids.rs                      # Newtype IDs (RequestId, etc.)
│   │       ├── validated.rs                # Constrained types (String100, etc.)
│   │       └── events.rs                   # Domain events (StreamEvent, etc.)
│   │
│   ├── kleos-api-producer/                 # API Gateway Lambda
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs                     # HTTP handler + Kinesis producer
│   │
│   └── kleos-stream-consumer/              # Kinesis Consumer Lambda
│       ├── Cargo.toml
│       └── src/
│           └── main.rs                     # Stream processor
└── README.md
```

## Components

### 1. kleos-types (Shared Library)

Domain types with construction-time validation:

**IDs** (Newtype pattern):
- `RequestId` - Unique request identifier
- `StreamRecordId` - Kinesis record identifier
- `PartitionKey` - Stream partition key (validated 1-256 bytes)

**Validated Types** (Construction-time validation):
- `String100` - Non-empty string, max 100 chars, auto-trimmed
- `String1000` - Non-empty string, max 1000 chars
- `NonNegativeInt` - Integer >= 0
- `PositiveInt` - Integer > 0

**Events** (Illegal states unrepresentable):
- `StreamEvent` - Envelope with request_id, timestamp, payload
- `EventPayload` - Enum of all event types (exhaustive matching)
  - `UserAction` - User-initiated actions
  - `SystemEvent` - System events with severity
  - `DataProcessed` - Processing results

### 2. kleos-api-producer (API Gateway Lambda)

HTTP handler that:
- Accepts JSON requests via API Gateway
- Validates input using domain types (fails fast on bad data)
- Creates type-safe `StreamEvent`
- Pushes to Kinesis Data Stream
- Returns HTTP 200 with request ID or error response

**Key Features:**
- Kinesis client initialized once in `main()` (reused across invocations)
- Type-driven validation at API boundary
- Structured logging with `tracing`

### 3. kleos-stream-consumer (Kinesis Lambda)

Event processor that:
- Receives batches from Kinesis (automatic polling)
- Deserializes to strongly-typed domain events
- Pattern matches on event type (compiler-enforced exhaustiveness)
- Processes each event type appropriately
- Returns partial batch failures for retry

**Key Features:**
- Partial batch response (retry only failed records)
- Type-safe event deserialization
- Exhaustive pattern matching ensures all events handled

## Prerequisites

- **Rust 2024 edition** or later
- **AWS SAM CLI** for building and deploying
- **AWS CLI** configured with credentials
- AWS account with permissions for Lambda, API Gateway, Kinesis, CloudFormation, IAM, S3

### Installing AWS SAM CLI

```bash
# macOS
brew install aws-sam-cli

# Linux
pip install aws-sam-cli

# Or download from:
# https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/install-sam-cli.html
```

### Verify Installation

```bash
sam --version
aws --version
rustc --version
```

## Quick Start with Makefile

The project includes a comprehensive Makefile for easy development and deployment. Run `make help` to see all available commands.

### Most Useful Commands

```bash
# Show all available commands
make help

# Build and deploy to dev
make deploy-dev

# Test the API
make test-api

# Check stack status and costs
make status

# View logs in real-time
make logs-consumer

# Pause infrastructure to save costs (~$11/month)
make pause

# Resume infrastructure
make resume

# Clean build artifacts
make clean
```

### Cost Management

The Makefile includes commands to manage costs:

```bash
# Pause: Delete the stack when not in use (saves ~$11/month)
make pause

# Resume: Redeploy the stack (takes ~2-3 minutes)
make resume

# Check current status and estimated costs
make status

# Scale Kinesis shards
make scale-down  # 1 shard (~$11/month)
make scale-up    # 5 shards (~$55/month)
```

## Building

### Local Development Build

```bash
# Build all crates
cargo build

# Build with tests
cargo test

# Build specific Lambda
cargo build -p kleos-api-producer
cargo build -p kleos-stream-consumer
```

### Production Build for AWS Lambda (SAM)

SAM CLI handles cross-compilation automatically:

```bash
# Enable Rust support (beta feature)
export SAM_CLI_BETA_RUST_CARGO_LAMBDA=1

# Build all Lambda functions for deployment
sam build

# Build with parallel processing
sam build --parallel

# For local testing on macOS (use debug mode)
SAM_CLI_BUILD_MODE=debug sam build
```

Output: Compiled functions in `.aws-sam/build/`

## Testing

### Unit Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p kleos-types
cargo test -p kleos-api-producer
cargo test -p kleos-stream-consumer

# Run with output
cargo test -- --nocapture
```

### Local Lambda Testing with SAM

```bash
# Enable Rust support
export SAM_CLI_BETA_RUST_CARGO_LAMBDA=1

# Build first
SAM_CLI_BUILD_MODE=debug sam build

# Invoke Producer Lambda locally with test event
sam local invoke KleosApiProducer --event test-events/api-request.json

# Start local API Gateway
sam local start-api

# Test the local endpoint
curl -X POST http://127.0.0.1:3000/events \
  -H "Content-Type: application/json" \
  -d '{"user_id":"user123","action":"create","details":"Test event"}'

# View logs in real-time
sam logs --stack-name kleos-dev --tail
```

### Example Test Event

The repository includes a test event at `test-events/api-request.json`:

```json
{
  "version": "2.0",
  "routeKey": "POST /events",
  "rawPath": "/events",
  "body": "{\"user_id\":\"user-123\",\"action\":\"create\",\"details\":\"Test event from local testing\"}",
  "isBase64Encoded": false
}
```

## Deployment

### Quick Start Deployment

Deploy the entire stack with a single command:

```bash
# Enable Rust support (beta feature)
export SAM_CLI_BETA_RUST_CARGO_LAMBDA=1

# Validate the SAM template
sam validate

# Build
sam build

# Deploy to dev (interactive)
sam deploy --guided --config-env dev

# Or deploy to dev (non-interactive after first time)
sam deploy --config-env dev
```

The guided deployment will ask:
- Stack Name: `kleos-dev`
- AWS Region: `us-east-1` (or your preference)
- Confirm changes before deploy: `Y`
- Allow SAM CLI IAM role creation: `Y`
- Disable rollback: `N`
- Save arguments to config: `Y`

### Deployment to Different Environments

The project includes pre-configured environments in `samconfig.toml`:

```bash
# Deploy to development
sam build && sam deploy --config-env dev

# Deploy to staging
sam build && sam deploy --config-env staging

# Deploy to production (requires confirmation)
sam build && sam deploy --config-env prod
```

Each environment has different settings:

| Environment | Shards | Log Level | Log Retention | Throttle Rate | Concurrency |
|-------------|--------|-----------|---------------|---------------|-------------|
| dev         | 1      | DEBUG     | 7 days        | 100 req/sec   | 5           |
| staging     | 2      | INFO      | 14 days       | 500 req/sec   | 10          |
| prod        | 5      | WARN      | 30 days       | 2000 req/sec  | 50          |

### What Gets Deployed

The SAM template creates:

1. **Kinesis Data Stream** (`kleos-stream-{env}`)
   - KMS encryption enabled
   - Configurable shard count
   - Environment-specific retention

2. **HTTP API Gateway** (`kleos-api-{env}`)
   - POST /events endpoint
   - CORS configured
   - Throttling limits

3. **Producer Lambda** (`kleos-api-producer-{env}`)
   - ARM64 architecture
   - 256 MB memory
   - IAM role with Kinesis write permissions
   - X-Ray tracing enabled

4. **Consumer Lambda** (`kleos-stream-consumer-{env}`)
   - ARM64 architecture
   - 512 MB memory
   - Event source mapping to Kinesis
   - Partial batch response enabled
   - IAM role with Kinesis read permissions

5. **Dead Letter Queue** (`kleos-failed-records-{env}`)
   - SQS queue for failed records
   - 14-day retention

6. **CloudWatch Resources**
   - Log groups with retention policies
   - Alarms for errors and lag (staging/prod only)

7. **IAM Roles and Policies**
   - Least-privilege policies
   - Managed by CloudFormation

### Viewing Deployment Outputs

After deployment, view the stack outputs:

```bash
# Get API endpoint
aws cloudformation describe-stacks \
  --stack-name kleos-dev \
  --query 'Stacks[0].Outputs[?OutputKey==`ApiEndpoint`].OutputValue' \
  --output text

# Or use SAM CLI
sam list endpoints --stack-name kleos-dev
```

### Testing the Deployment

```bash
# Get the API endpoint from outputs
API_ENDPOINT=$(aws cloudformation describe-stacks \
  --stack-name kleos-dev \
  --query 'Stacks[0].Outputs[?OutputKey==`ApiEndpoint`].OutputValue' \
  --output text)

# Send a test request
curl -X POST ${API_ENDPOINT}/events \
  -H "Content-Type: application/json" \
  -d '{"user_id":"user123","action":"create","details":"Hello from deployed API!"}'

# Check logs
sam logs --stack-name kleos-dev --tail

# Or use AWS CLI
aws logs tail /aws/lambda/kleos-api-producer-dev --follow
aws logs tail /aws/lambda/kleos-stream-consumer-dev --follow
```

### Monitoring Deployment

```bash
# Watch CloudFormation events during deployment
sam deploy --config-env dev --no-execute-changeset
aws cloudformation describe-stack-events --stack-name kleos-dev

# View stack resources
aws cloudformation describe-stack-resources --stack-name kleos-dev

# Check Lambda function status
aws lambda get-function --function-name kleos-api-producer-dev
aws lambda get-function --function-name kleos-stream-consumer-dev

# Check Kinesis stream status
aws kinesis describe-stream --stream-name kleos-stream-dev
```

### Updating the Stack

```bash
# Make code changes, then rebuild and deploy
sam build && sam deploy --config-env dev

# Deploy with change preview
sam deploy --config-env dev --no-execute-changeset

# If satisfied, execute the changeset
aws cloudformation execute-change-set \
  --change-set-name <changeset-name> \
  --stack-name kleos-dev
```

### Deleting the Stack

```bash
# Delete development stack (removes ALL resources)
sam delete --stack-name kleos-dev --no-prompts

# Or use CloudFormation CLI
aws cloudformation delete-stack --stack-name kleos-dev

# Monitor deletion
aws cloudformation wait stack-delete-complete --stack-name kleos-dev
```

## Configuration

### Environment Variables

**kleos-api-producer:**
- `KINESIS_STREAM_NAME` - Kinesis stream name (default: `kleos-stream`)

**kleos-stream-consumer:**
- None required (event source mapping provides stream data)

### Recommended Lambda Settings

| Setting | API Producer | Stream Consumer |
|---------|--------------|-----------------|
| Memory | 256 MB | 512 MB |
| Timeout | 30 sec | 60 sec |
| Architecture | ARM64 | ARM64 |
| Runtime | `provided.al2023` | `provided.al2023` |

## Monitoring

### CloudWatch Logs

Structured JSON logging with `tracing`:

```json
{
  "level": "INFO",
  "request_id": "abc123",
  "message": "Processing user action",
  "user_id": "user123",
  "action": "Create"
}
```

Log groups:
- `/aws/lambda/kleos-api-producer`
- `/aws/lambda/kleos-stream-consumer`

### CloudWatch Metrics

Monitor:
- **Lambda**: Invocations, Errors, Duration, Concurrent Executions
- **Kinesis**: IncomingRecords, IncomingBytes, IteratorAge (lag indicator)
- **API Gateway**: Count, 4XXError, 5XXError, Latency

### Alarms

Recommended CloudWatch alarms:
- Lambda error rate > 5%
- Kinesis iterator age > 60 seconds (processing lag)
- API Gateway 5XX errors > 10

## Performance

### Cold Start

- **Rust Lambda**: ~150-300ms cold start
- **Warm execution**: <10ms

Optimization techniques:
- ARM64 architecture
- Release build with LTO
- Binary stripping
- Minimal dependencies

### Throughput

- **API Producer**: Limited by API Gateway (10,000 req/sec default)
- **Stream Consumer**: Limited by Kinesis shards (1000 records/sec per shard)
- **Scaling**: Use `ParallelizationFactor` (1-10) for higher throughput per shard

## Type-Driven Development Benefits

### 1. Compiler-Enforced Correctness

Adding a new event type? The compiler will error everywhere it needs handling:

```rust
// Add new variant to EventPayload
enum EventPayload {
    UserAction(UserActionEvent),
    SystemEvent(SystemEventData),
    DataProcessed(DataProcessedEvent),
    NewEventType(NewEvent),  // <-- Add this
}

// Compiler error in stream-consumer/src/main.rs:
// "non-exhaustive patterns: `NewEventType(_)` not covered"
```

### 2. Self-Documenting Types

Function signatures reveal business rules:

```rust
// Old way (stringly-typed, anything goes)
fn process(user_id: &str, count: i32) -> Result<String, String>

// Type-driven way (constraints clear from signature)
fn process(user_id: &String100, count: PositiveInt)
  -> Result<RequestId, ValidationError>
```

### 3. No Defensive Checks

Once created, types are valid everywhere:

```rust
// Validation at boundary (API layer)
let user_id = String100::create(input)?;

// Domain logic: no validation needed!
fn save_to_db(user_id: String100) {
    // user_id guaranteed to be:
    // - Non-empty
    // - <= 100 chars
    // - Trimmed
    // No need to check!
}
```

### 4. Breaking Changes as Compiler Errors

Business rule changes become type changes:

```rust
// Change: User IDs now max 50 chars instead of 100
type UserId = String50;  // Change this

// Compiler finds every place this affects:
// - API validation
// - Database schemas
// - Serialization
// - Display logic
```

## Extending the Skeleton

### Add New Event Type

1. **Define in `kleos-types/src/events.rs`:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentEvent {
    pub amount: NonNegativeInt,
    pub currency: String100,
}

// Add to EventPayload enum
pub enum EventPayload {
    // ... existing variants
    Payment(PaymentEvent),
}
```

2. **Handle in Consumer:**
```rust
// Compiler will error until you add this:
match event.payload {
    EventPayload::Payment(payment) => {
        process_payment(&request_id, payment).await?
    }
    // ... other variants
}
```

3. **Add to Producer API:**
```rust
// Accept payment requests
if let Some(payment_data) = api_request.payment {
    let payload = EventPayload::Payment(PaymentEvent {
        amount: NonNegativeInt::create(payment_data.amount)?,
        currency: String100::create(payment_data.currency)?,
    });
}
```

### Add Downstream Processing

Stream Consumer skeleton includes TODO markers:

```rust
async fn process_user_action(...) -> Result<(), anyhow::Error> {
    match event.action_type {
        ActionType::Create => {
            // TODO: Implement create logic
            // - Save to DynamoDB
            // - Send to SNS
            // - Trigger Step Function
        }
        // ...
    }
}
```

Add your logic:
- Write to DynamoDB
- Publish to SNS/SQS
- Invoke other Lambdas
- Call external APIs

## Troubleshooting

### Build Errors

**Error: edition "2024" is unstable**
```bash
# Update Rust to latest stable
rustup update stable
```

**Error: Cargo Lambda not found**
```bash
cargo install cargo-lambda
```

### Deployment Errors

**Error: Lambda role lacks permissions**
- Verify IAM role has `AWSLambdaBasicExecutionRole`
- For producer: Add Kinesis `PutRecord` permission
- For consumer: Add `AWSLambdaKinesisExecutionRole`

### Runtime Errors

**Consumer Lambda not triggering:**
- Check event source mapping status: `aws lambda list-event-source-mappings`
- Verify stream has data: `aws kinesis get-records`
- Check CloudWatch Logs for errors

**API Gateway 502 errors:**
- Increase Lambda timeout
- Check CloudWatch Logs for exceptions
- Verify Lambda has proper IAM role

**Kinesis PutRecord fails:**
- Verify stream exists and is ACTIVE
- Check partition key is 1-256 bytes
- Verify IAM role has `kinesis:PutRecord`

## Cost Estimation

For 1M requests/day:

| Service | Usage | Monthly Cost |
|---------|-------|--------------|
| API Gateway | 1M requests | ~$3.50 |
| Lambda (Producer) | 1M invocations, 256MB, 100ms | ~$0.20 |
| Kinesis | 1 shard, 1M records | ~$11 |
| Lambda (Consumer) | 10K invocations (batched), 512MB, 500ms | ~$0.50 |
| CloudWatch Logs | 1GB | ~$0.50 |
| **Total** | | **~$16/month** |

Optimization:
- Use ARM64 for 20% savings on Lambda
- Increase batch size to reduce invocations
- Set log retention to 7 days

## Infrastructure as Code

### AWS SAM Template

The project includes a production-ready SAM template (`template.yaml`) that defines all infrastructure:

**Key Resources:**
- Kinesis Data Stream with KMS encryption
- HTTP API Gateway with throttling
- Producer Lambda (API → Kinesis)
- Consumer Lambda (Kinesis → Processing)
- Event Source Mapping with partial batch response
- Dead Letter Queue for failed records
- CloudWatch Log Groups with retention
- CloudWatch Alarms (staging/prod)
- IAM Roles with least-privilege policies

**Template Features:**
- Environment-specific parameters (dev/staging/prod)
- Conditional resource creation (alarms only for prod/staging)
- Mappings for environment-specific config
- Comprehensive outputs for easy reference
- Tags for cost tracking

**Configuration Files:**
- `template.yaml` - Main SAM template (350+ lines)
- `samconfig.toml` - Deployment profiles for each environment
- `parameters/dev.json` - Development parameters
- `parameters/staging.json` - Staging parameters
- `parameters/prod.json` - Production parameters

### Template Customization

Edit `template.yaml` to customize:

```yaml
# Change Lambda memory sizes
Globals:
  Function:
    MemorySize: 512  # Change from 256

# Adjust Kinesis retention
Mappings:
  EnvironmentConfig:
    prod:
      KinesisRetentionHours: 8760  # 1 year instead of 7 days

# Add new resources
Resources:
  MyDynamoDBTable:
    Type: AWS::DynamoDB::Table
    Properties:
      TableName: kleos-data
      # ... table definition
```

### Alternative IaC Tools

The architecture can also be implemented with:
- **Terraform** - Use `aws_kinesis_stream`, `aws_lambda_function`, `aws_api_gateway_v2_api`
- **AWS CDK** - Use `@aws-cdk/aws-kinesis`, `@aws-cdk/aws-lambda`, `@aws-cdk/aws-apigatewayv2`
- **Pulumi** - Use `@pulumi/aws` package

## License

MIT OR Apache-2.0

## Resources

- [AWS Lambda Rust Runtime](https://github.com/aws/aws-lambda-rust-runtime)
- [Cargo Lambda](https://www.cargo-lambda.info/)
- [Type-Driven Development (F# for Fun and Profit)](https://fsharpforfunandprofit.com/posts/designing-with-types-intro/)
- [AWS Kinesis Data Streams](https://docs.aws.amazon.com/kinesis/)
- [AWS SDK for Rust](https://docs.rs/aws-sdk-kinesis/)
