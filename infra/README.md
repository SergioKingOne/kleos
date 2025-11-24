# Kleos Infrastructure

AWS infrastructure for the Kleos event-driven data pipeline using AWS SAM (Serverless Application Model).

## Architecture

The infrastructure is organized into modular stacks by infrastructure layer:

```
┌─────────────────────────────────────────────────────────────┐
│                      API Gateway                             │
│                  (REST API + CORS)                           │
└──────────────────────┬──────────────────────────────────────┘
                       │ POST /ingest
                       ▼
┌─────────────────────────────────────────────────────────────┐
│              Ingest Lambda (ARM64)                           │
│         Validates & publishes events                         │
└──────────────────────┬──────────────────────────────────────┘
                       │ PutRecord
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                 Kinesis Stream                               │
│         (Encrypted, configurable shards)                     │
└──────────────────────┬──────────────────────────────────────┘
                       │ Event Source Mapping
                       ▼
┌─────────────────────────────────────────────────────────────┐
│             Process Lambda (ARM64)                           │
│      Processes events with retry & DLQ                       │
└──────────────────────┬──────────────────────────────────────┘
                       │ On failure
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                 Dead Letter Queue                            │
│              (14-day retention)                              │
└─────────────────────────────────────────────────────────────┘
```

## Directory Structure

```
infra/
├── template.yaml              # Root template (orchestrates all stacks)
├── samconfig.toml            # Environment configurations
├── streams/                  # Kinesis stream + DLQ
│   └── template.yaml
├── api/                      # API Gateway
│   └── template.yaml
├── functions/                # Lambda functions
│   ├── ingest/
│   │   └── template.yaml    # Ingest Lambda + API integration
│   └── process/
│       └── template.yaml    # Process Lambda + Kinesis trigger
├── observability/            # CloudWatch logs + alarms
│   └── template.yaml
└── events/                   # Sample events for testing
    ├── apigw-request.json
    ├── apigw-click.json
    ├── apigw-purchase.json
    └── kinesis-record.json
```

## Environment Variables

All infrastructure commands use environment variables for flexibility:

- **ENV** - Environment name (`dev`, `staging`, `prod`). Default: `dev`
- **FUNC** - Function name (`ingest`, `process`). Default: `ingest`
- **AWS_REGION** - AWS region. Default: `us-east-1`

**Usage patterns:**

```bash
# Explicit environment variable
make deploy ENV=prod
make logs FUNC=process ENV=staging

# Using defaults
make deploy        # Uses ENV=dev
make logs          # Uses FUNC=ingest ENV=dev

# Combining with scripts
ENV=prod ./scripts/deploy.sh
FUNC=process ENV=prod ./scripts/logs.sh
```

This approach provides maximum flexibility without requiring environment-specific make targets.

## Prerequisites

1. **AWS CLI** - [Installation guide](https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html)
2. **SAM CLI** - [Installation guide](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/install-sam-cli.html)
3. **cargo-lambda** - Install with: `cargo install cargo-lambda`
4. **Docker** - Required for SAM local testing

## Quick Start

### 1. Install Dependencies

```bash
make install     # Install cargo-lambda and verify SAM CLI
make deps        # Fetch Cargo dependencies
```

### 2. Build Lambda Functions

```bash
make build       # Build all Lambda functions with ARM64 target
```

Or build individual functions:

```bash
make build-ingest    # Build only ingest Lambda
make build-process   # Build only process Lambda
```

### 3. Validate Templates

```bash
make validate    # Validate SAM templates
```

### 4. Deploy to AWS

Deploy to **dev** environment (default):

```bash
make deploy              # Uses ENV=dev by default
make deploy ENV=dev      # Explicit dev deployment
```

Deploy to **staging** or **prod**:

```bash
make deploy ENV=staging
make deploy ENV=prod
```

Or use the script directly:

```bash
./scripts/deploy.sh          # Interactive - will prompt for environment
./scripts/deploy.sh dev      # Deploy to specific environment
ENV=prod ./scripts/deploy.sh # Deploy using ENV variable
```

## Local Development

### Start Local API Gateway

```bash
make local-api
```

This starts a local API Gateway at `http://127.0.0.1:3000`.

Test with curl:

```bash
# PageView event
curl -X POST http://127.0.0.1:3000/ingest \
  -H 'Content-Type: application/json' \
  -d '{"type":"PageView","data":{"user_id":"550e8400-e29b-41d4-a716-446655440000","url":"/products/laptop"}}'

# Click event
curl -X POST http://127.0.0.1:3000/ingest \
  -H 'Content-Type: application/json' \
  -d '{"type":"Click","data":{"user_id":"550e8400-e29b-41d4-a716-446655440000","element_id":"add-to-cart-btn","url":"/products/laptop"}}'

# Purchase event
curl -X POST http://127.0.0.1:3000/ingest \
  -H 'Content-Type: application/json' \
  -d '{"type":"Purchase","data":{"user_id":"550e8400-e29b-41d4-a716-446655440000","product_id":"650e8400-e29b-41d4-a716-446655440001","amount":1299.99}}'
```

### Invoke Functions Locally

```bash
make invoke-ingest    # Invoke ingest function with sample event
make invoke-process   # Invoke process function with sample event
```

## Monitoring

### View CloudWatch Logs

Tail logs using environment and function variables:

```bash
make logs                        # Uses FUNC=ingest ENV=dev (defaults)
make logs FUNC=ingest ENV=dev    # Tail ingest logs in dev
make logs FUNC=process ENV=prod  # Tail process logs in prod
```

Or use the script directly:

```bash
./scripts/logs.sh                  # Interactive - will prompt
./scripts/logs.sh ingest dev       # Tail specific function in environment
FUNC=process ENV=prod ./scripts/logs.sh # Using environment variables
```

### View Stack Outputs

```bash
make outputs              # Show dev outputs (default)
make outputs ENV=staging  # Show staging outputs
make outputs ENV=prod     # Show production outputs
```

This shows:
- API Gateway endpoint URL
- Kinesis stream name and ARN
- Lambda function names and ARNs
- Dead Letter Queue URL

## Testing

```bash
make test        # Run all Rust tests
make fmt         # Format code
make lint        # Run clippy linter
```

## Deployment Strategies

### Plan Before Deploy (Like Terraform)

Preview infrastructure changes before applying them using **CloudFormation Change Sets**:

```bash
make plan ENV=dev      # Show what will change (like terraform plan)
make deploy ENV=dev    # Apply changes after review
```

**Interactive workflow:**

1. Creates a change set without deploying
2. Shows a table of changes (Add/Modify/Remove resources)
3. Prompts you to:
   - Execute the change set (deploy)
   - Delete the change set (cancel)
   - View detailed changes in AWS Console
   - Exit and decide later

**Example output:**
```
┌──────────┬────────────────────────────────┬───────────────────────┬─────────────┐
│ Action   │ ResourceType                   │ LogicalResourceId     │ Replacement │
├──────────┼────────────────────────────────┼───────────────────────┼─────────────┤
│ Modify   │ AWS::Lambda::Function          │ IngestFunction        │ False       │
│ Add      │ AWS::CloudWatch::Alarm         │ IngestErrorAlarm      │ N/A         │
└──────────┴────────────────────────────────┴───────────────────────┴─────────────┘
```

### Full Stack Deployment

Deploys all nested stacks (streams, API, functions, observability):

```bash
make deploy ENV=dev       # Deploy to dev
make deploy ENV=staging   # Deploy to staging
make deploy ENV=prod      # Deploy to production
```

### Fast Function Updates

For rapid iteration on a single Lambda function:

```bash
make deploy-function FUNC=ingest ENV=dev   # Update only ingest function
make deploy-function FUNC=process ENV=dev  # Update only process function

# Or use script directly
./scripts/deploy-function.sh ingest dev
```

This bypasses CloudFormation and directly updates the Lambda code, making it much faster (~10 seconds vs ~2 minutes).

## Environment Configuration

Environments are configured in `samconfig.toml`:

- **dev**: 1 Kinesis shard, no changeset confirmation
- **staging**: 2 Kinesis shards, requires changeset confirmation
- **prod**: 4 Kinesis shards, requires changeset confirmation, fails on empty changesets

## Stack Dependencies

The root template orchestrates deployment in the correct order:

1. **StreamsStack** - Kinesis stream + DLQ (foundation)
2. **ApiStack** - API Gateway (parallel with streams)
3. **IngestFunctionStack** - Depends on API + Streams
4. **ProcessFunctionStack** - Depends on Streams
5. **ObservabilityStack** - Depends on both functions

## Resource Tagging

All resources are tagged with:
- `Environment`: dev/staging/prod
- `Project`: Kleos
- `ManagedBy`: SAM

## Cost Optimization

- **ARM64 architecture**: 20-30% cheaper than x86_64
- **128MB memory**: Rust's efficiency allows minimal memory allocation
- **Reserved concurrency**: Process Lambda limited to 10 concurrent executions
- **Log retention**: 7 days (configurable in observability/template.yaml)

## Infrastructure Teardown

### Destroy Infrastructure (Stop AWS Costs)

To completely tear down the infrastructure and avoid ongoing costs (especially Kinesis):

```bash
make destroy              # Destroy dev stack (default, with confirmation)
make destroy ENV=staging  # Destroy staging stack
make destroy ENV=prod     # Destroy production stack
```

Or use the script directly:

```bash
./scripts/destroy.sh          # Interactive - will prompt for environment
./scripts/destroy.sh dev      # Destroy specific environment
ENV=prod ./scripts/destroy.sh # Destroy using ENV variable
```

**What gets deleted:**
- Kinesis Data Stream (stops per-shard hourly charges)
- Lambda Functions (no ongoing cost, but removes them)
- API Gateway (no ongoing cost for REST APIs)
- CloudWatch Log Groups and Alarms
- Dead Letter Queue
- IAM Roles and Policies

**⚠️ Important:**
- Requires typing `yes` to confirm
- Deletion is **irreversible** - all data will be lost
- Waits for deletion to complete before exiting
- Useful for dev/staging environments when not in use

### Clean Local Build Artifacts

Remove local build artifacts (does not affect AWS resources):

```bash
make clean
```

## Troubleshooting

### Build fails with "cargo-lambda not found"

Install cargo-lambda:

```bash
cargo install cargo-lambda
```

### SAM deploy fails with "No changes to deploy"

This is expected if no resources have changed. Use `--no-fail-on-empty-changeset` flag or set `fail_on_empty_changeset = false` in samconfig.toml.

### Lambda function times out

Increase timeout in the function template:

```yaml
Properties:
  Timeout: 60  # Increase from default 30
```

### Kinesis records not being processed

Check:
1. Lambda function logs: `make logs FUNC=process ENV=dev`
2. Dead Letter Queue: Check SQS console for failed records
3. Event source mapping: Verify it's enabled in Lambda console

## Additional Resources

- [AWS SAM Documentation](https://docs.aws.amazon.com/serverless-application-model/)
- [cargo-lambda Documentation](https://www.cargo-lambda.info/)
- [AWS Lambda Rust Runtime](https://github.com/awslabs/aws-lambda-rust-runtime)
