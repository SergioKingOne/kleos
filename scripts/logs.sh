#!/usr/bin/env bash
set -euo pipefail

# CloudWatch logs tailing script

# Colors
CYAN='\033[36m'
GREEN='\033[32m'
YELLOW='\033[33m'
RED='\033[31m'
RESET='\033[0m'

# Get function and environment from arguments or environment variables
FUNCTION="${1:-${FUNC:-}}"
ENV="${2:-${ENV:-dev}}"
AWS_REGION="${AWS_REGION:-us-east-1}"

print_usage() {
    echo -e "${CYAN}Usage:${RESET}"
    echo -e "  $0 <function> [environment]"
    echo -e "  FUNC=ingest ENV=dev $0"
    echo -e ""
    echo -e "${CYAN}Functions:${RESET}"
    echo -e "  ingest   - Ingest Lambda function logs"
    echo -e "  process  - Process Lambda function logs"
    echo -e ""
    echo -e "${CYAN}Environments:${RESET}"
    echo -e "  dev      - Development environment (default)"
    echo -e "  staging  - Staging environment"
    echo -e "  prod     - Production environment"
    echo -e ""
    echo -e "${CYAN}Example:${RESET}"
    echo -e "  $0 ingest dev"
    echo -e "  FUNC=process ENV=prod $0"
}

if [ -z "$FUNCTION" ]; then
    echo -e "${YELLOW}Select function to tail:${RESET}"
    echo -e "  1) ingest"
    echo -e "  2) process"
    read -p "Enter choice [1-2]: " choice

    case $choice in
        1) FUNCTION="ingest" ;;
        2) FUNCTION="process" ;;
        *)
            echo -e "${RED}Invalid choice${RESET}"
            exit 1
            ;;
    esac
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

LOG_GROUP="/aws/lambda/kleos-$FUNCTION-$ENV"

echo -e "${CYAN}Tailing logs for ${YELLOW}$FUNCTION${CYAN} in ${YELLOW}$ENV${CYAN} environment...${RESET}"
echo -e "${CYAN}Log group: ${YELLOW}$LOG_GROUP${RESET}"
echo -e "${CYAN}Region: ${YELLOW}$AWS_REGION${RESET}"
echo -e "${CYAN}Press Ctrl+C to stop${RESET}"
echo -e ""

# Check if log group exists
if ! aws logs describe-log-groups \
    --log-group-name-prefix "$LOG_GROUP" \
    --region "$AWS_REGION" \
    --query "logGroups[?logGroupName=='$LOG_GROUP']" \
    --output text | grep -q "$LOG_GROUP"; then
    echo -e "${YELLOW}Warning: Log group '$LOG_GROUP' not found. It may not exist yet.${RESET}"
    echo -e "${YELLOW}Logs will appear once the function is invoked.${RESET}"
fi

# Tail logs
aws logs tail "$LOG_GROUP" \
    --follow \
    --region "$AWS_REGION" \
    --format short
