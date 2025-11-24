#!/usr/bin/env bash
set -euo pipefail

# Local development script - starts SAM local API

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
INFRA_DIR="$PROJECT_ROOT/infra"

# Colors
CYAN='\033[36m'
GREEN='\033[32m'
YELLOW='\033[33m'
RED='\033[31m'
RESET='\033[0m'

echo -e "${CYAN}Starting Kleos local development environment...${RESET}"

# Check if SAM CLI is installed
if ! command -v sam &> /dev/null; then
    echo -e "${RED}Error: SAM CLI not found${RESET}"
    echo -e "${YELLOW}Install from: https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/install-sam-cli.html${RESET}"
    exit 1
fi

# Build functions
echo -e "${YELLOW}Building Lambda functions...${RESET}"
cd "$PROJECT_ROOT"
if ! cargo lambda build --release --arm64 --workspace; then
    echo -e "${RED}✗ Build failed${RESET}"
    exit 1
fi
echo -e "${GREEN}✓ Build complete${RESET}"

# Start local API
cd "$INFRA_DIR"
echo -e "${GREEN}Starting local API Gateway...${RESET}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e "${GREEN}API endpoint: ${YELLOW}http://127.0.0.1:3000${RESET}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e ""
echo -e "${CYAN}Test the ingest endpoint:${RESET}"
echo -e "${YELLOW}curl -X POST http://127.0.0.1:3000/ingest \\${RESET}"
echo -e "${YELLOW}  -H 'Content-Type: application/json' \\${RESET}"
echo -e "${YELLOW}  -d '{\"action\":\"test\",\"timestamp\":\"2024-01-01T00:00:00Z\"}'${RESET}"
echo -e ""
echo -e "${CYAN}Press Ctrl+C to stop${RESET}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e ""

sam local start-api --beta-features
