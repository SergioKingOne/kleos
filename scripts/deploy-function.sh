#!/usr/bin/env bash
set -euo pipefail

# Fast deployment script for individual Lambda functions

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
CYAN='\033[36m'
GREEN='\033[32m'
YELLOW='\033[33m'
RED='\033[31m'
RESET='\033[0m'

FUNCTION="${1:-}"
ENV="${2:-dev}"

print_usage() {
    echo -e "${CYAN}Usage:${RESET}"
    echo -e "  $0 <function> [environment]"
    echo -e ""
    echo -e "${CYAN}Functions:${RESET}"
    echo -e "  ingest   - Ingest Lambda function"
    echo -e "  process  - Process Lambda function"
    echo -e ""
    echo -e "${CYAN}Environments:${RESET}"
    echo -e "  dev      - Development environment (default)"
    echo -e "  staging  - Staging environment"
    echo -e "  prod     - Production environment"
    echo -e ""
    echo -e "${CYAN}Example:${RESET}"
    echo -e "  $0 ingest dev"
}

if [ -z "$FUNCTION" ]; then
    echo -e "${RED}Error: Function name required${RESET}"
    print_usage
    exit 1
fi

# Validate function name
if [[ ! "$FUNCTION" =~ ^(ingest|process)$ ]]; then
    echo -e "${RED}Error: Invalid function '$FUNCTION'${RESET}"
    print_usage
    exit 1
fi

# Validate environment
if [[ ! "$ENV" =~ ^(dev|staging|prod)$ ]]; then
    echo -e "${RED}Error: Invalid environment '$ENV'${RESET}"
    print_usage
    exit 1
fi

FUNCTION_NAME="kleos-$FUNCTION-$ENV"
PACKAGE_NAME="kleos-$FUNCTION-lambda"

echo -e "${CYAN}Fast deploying ${YELLOW}$FUNCTION${CYAN} function to ${YELLOW}$ENV${CYAN}...${RESET}"

# Build the specific function
echo -e "${YELLOW}Building $FUNCTION Lambda...${RESET}"
cd "$PROJECT_ROOT"

if ! cargo lambda build --release --arm64 -p "$PACKAGE_NAME"; then
    echo -e "${RED}✗ Build failed${RESET}"
    exit 1
fi
echo -e "${GREEN}✓ Build complete${RESET}"

# Create deployment package
TEMP_DIR=$(mktemp -d)
BOOTSTRAP_PATH="$PROJECT_ROOT/target/lambda/$PACKAGE_NAME/bootstrap"

if [ ! -f "$BOOTSTRAP_PATH" ]; then
    echo -e "${RED}✗ Bootstrap binary not found at $BOOTSTRAP_PATH${RESET}"
    exit 1
fi

cp "$BOOTSTRAP_PATH" "$TEMP_DIR/bootstrap"

# Create zip file
ZIP_FILE="$TEMP_DIR/function.zip"
(cd "$TEMP_DIR" && zip -q "$ZIP_FILE" bootstrap)

echo -e "${YELLOW}Updating Lambda function code...${RESET}"

# Update function code
if aws lambda update-function-code \
    --function-name "$FUNCTION_NAME" \
    --zip-file "fileb://$ZIP_FILE" \
    --publish \
    > /dev/null; then
    echo -e "${GREEN}✓ Function code updated!${RESET}"
else
    echo -e "${RED}✗ Function code update failed${RESET}"
    rm -rf "$TEMP_DIR"
    exit 1
fi

# Wait for update to complete
echo -e "${YELLOW}Waiting for update to complete...${RESET}"
aws lambda wait function-updated --function-name "$FUNCTION_NAME"

# Clean up
rm -rf "$TEMP_DIR"

echo -e "${GREEN}✓ Deployment complete!${RESET}"
echo -e "${CYAN}Function: $FUNCTION_NAME${RESET}"
