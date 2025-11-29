#!/usr/bin/env bash
set -euo pipefail

# Deployment script for Kleos infrastructure

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
INFRA_DIR="$PROJECT_ROOT/infra"

# Colors
CYAN='\033[36m'
GREEN='\033[32m'
YELLOW='\033[33m'
RED='\033[31m'
RESET='\033[0m'

# Get environment from argument or ENV variable
ENV="${1:-${ENV:-}}"

print_usage() {
    echo -e "${CYAN}Usage:${RESET}"
    echo -e "  $0 [environment]"
    echo -e "  ENV=dev $0"
    echo -e ""
    echo -e "${CYAN}Environments:${RESET}"
    echo -e "  dev      - Development environment"
    echo -e "  staging  - Staging environment"
    echo -e "  prod     - Production environment"
    echo -e ""
    echo -e "${CYAN}Example:${RESET}"
    echo -e "  $0 dev"
    echo -e "  ENV=prod $0"
}

if [ -z "$ENV" ]; then
    echo -e "${YELLOW}No environment specified. Choose one:${RESET}"
    echo -e "  1) dev"
    echo -e "  2) staging"
    echo -e "  3) prod"
    read -p "Enter choice [1-3]: " choice

    case $choice in
        1) ENV="dev" ;;
        2) ENV="staging" ;;
        3) ENV="prod" ;;
        *)
            echo -e "${RED}Invalid choice${RESET}"
            exit 1
            ;;
    esac
fi

# Validate environment
if [[ ! "$ENV" =~ ^(dev|staging|prod)$ ]]; then
    echo -e "${RED}Error: Invalid environment '$ENV'${RESET}"
    print_usage
    exit 1
fi

echo -e "${CYAN}Deploying Kleos to ${YELLOW}$ENV${CYAN} environment...${RESET}"

# Check if SAM CLI is installed
if ! command -v sam &> /dev/null; then
    echo -e "${RED}Error: SAM CLI not found${RESET}"
    echo -e "${YELLOW}Install from: https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/install-sam-cli.html${RESET}"
    exit 1
fi

# Navigate to infrastructure directory
cd "$INFRA_DIR"

# Validate templates
echo -e "${YELLOW}Validating SAM templates...${RESET}"
if ! sam validate --lint; then
    echo -e "${RED}✗ Template validation failed${RESET}"
    exit 1
fi
echo -e "${GREEN}✓ Templates validated${RESET}"

# Build Lambda functions with cargo-lambda (more reliable than SAM's builder)
echo -e "${YELLOW}Building Lambda functions with cargo-lambda...${RESET}"
cd "$PROJECT_ROOT"
if ! cargo lambda build --release --arm64 --workspace; then
    echo -e "${RED}✗ Build failed${RESET}"
    exit 1
fi
echo -e "${GREEN}✓ Build complete${RESET}"

cd "$INFRA_DIR"

# Package for SAM (manually create build directory with pre-built binaries)
echo -e "${YELLOW}Packaging for deployment...${RESET}"
BUILD_DIR="$INFRA_DIR/.aws-sam/build"
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

# Copy templates
cp template.yaml "$BUILD_DIR/"
cp -r streams "$BUILD_DIR/"
cp -r api "$BUILD_DIR/"
cp -r observability "$BUILD_DIR/"
mkdir -p "$BUILD_DIR/functions/ingest" "$BUILD_DIR/functions/process"
cp functions/ingest/template.yaml "$BUILD_DIR/functions/ingest/"
cp functions/process/template.yaml "$BUILD_DIR/functions/process/"

# Copy pre-built Lambda binaries
mkdir -p "$BUILD_DIR/IngestFunctionStack" "$BUILD_DIR/ProcessFunctionStack"
cp "$PROJECT_ROOT/target/lambda/kleos-ingest-lambda/bootstrap" "$BUILD_DIR/IngestFunctionStack/"
cp "$PROJECT_ROOT/target/lambda/kleos-process-lambda/bootstrap" "$BUILD_DIR/ProcessFunctionStack/"

# Update CodeUri in built templates to point to the binary directories
sed -i '' 's|CodeUri: ../../../crates/kleos-ingest-lambda/|CodeUri: ../../IngestFunctionStack/|g' "$BUILD_DIR/functions/ingest/template.yaml"
sed -i '' 's|CodeUri: ../../../crates/kleos-process-lambda/|CodeUri: ../../ProcessFunctionStack/|g' "$BUILD_DIR/functions/process/template.yaml"

echo -e "${GREEN}✓ Packaging complete${RESET}"

# Deploy
echo -e "${YELLOW}Deploying to AWS...${RESET}"
AWS_REGION="${AWS_REGION:-us-east-1}"
STACK_NAME="kleos-pipeline-$ENV"

# Get shard count based on environment
case $ENV in
    dev) SHARD_COUNT=1 ;;
    staging) SHARD_COUNT=2 ;;
    prod) SHARD_COUNT=4 ;;
esac

if sam deploy \
    --template-file "$BUILD_DIR/template.yaml" \
    --stack-name "$STACK_NAME" \
    --parameter-overrides "Environment=$ENV KinesisShardCount=$SHARD_COUNT" \
    --capabilities CAPABILITY_IAM CAPABILITY_AUTO_EXPAND \
    --region "$AWS_REGION" \
    --resolve-s3; then
    echo -e "${GREEN}✓ Deployment successful!${RESET}"

    # Show stack outputs
    echo -e "${CYAN}Stack outputs:${RESET}"
    aws cloudformation describe-stacks \
        --stack-name "kleos-pipeline-$ENV" \
        --query 'Stacks[0].Outputs' \
        --output table 2>/dev/null || true
else
    echo -e "${RED}✗ Deployment failed${RESET}"
    exit 1
fi
