#!/usr/bin/env bash
set -euo pipefail

# Build script for Kleos Lambda functions using cargo-lambda

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
CYAN='\033[36m'
GREEN='\033[32m'
YELLOW='\033[33m'
RED='\033[31m'
RESET='\033[0m'

echo -e "${CYAN}Building Kleos Lambda functions...${RESET}"

# Check if cargo-lambda is installed
if ! command -v cargo-lambda &> /dev/null; then
    echo -e "${RED}Error: cargo-lambda not found${RESET}"
    echo -e "${YELLOW}Install with: cargo install cargo-lambda${RESET}"
    exit 1
fi

cd "$PROJECT_ROOT"

# Build all workspace Lambda functions
echo -e "${YELLOW}Building all Lambda functions with ARM64 target...${RESET}"
cargo lambda build --release --arm64 --workspace

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Build successful!${RESET}"
    echo -e "${CYAN}Binaries located in: target/lambda/${RESET}"
    ls -lh target/lambda/*/bootstrap 2>/dev/null || true
else
    echo -e "${RED}✗ Build failed${RESET}"
    exit 1
fi
