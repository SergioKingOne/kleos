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

# Build with SAM
echo -e "${YELLOW}Building with SAM...${RESET}"
if ! sam build --beta-features --parallel --cached; then
    echo -e "${RED}✗ Build failed${RESET}"
    exit 1
fi
echo -e "${GREEN}✓ Build complete${RESET}"

# Deploy
echo -e "${YELLOW}Deploying to AWS...${RESET}"
if sam deploy --config-env "$ENV" --beta-features; then
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
