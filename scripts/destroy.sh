#!/usr/bin/env bash
set -euo pipefail

# Infrastructure teardown script - destroys CloudFormation stack and all resources

# Colors
CYAN='\033[36m'
GREEN='\033[32m'
YELLOW='\033[33m'
RED='\033[31m'
RESET='\033[0m'

ENV="${1:-${ENV:-}}"
AWS_REGION="${AWS_REGION:-us-east-1}"

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
    echo -e "${YELLOW}No environment specified. Choose one to destroy:${RESET}"
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

STACK_NAME="kleos-pipeline-$ENV"

echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e "${RED}WARNING: Infrastructure Teardown${RESET}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e ""
echo -e "${YELLOW}Stack:${RESET} $STACK_NAME"
echo -e "${YELLOW}Region:${RESET} $AWS_REGION"
echo -e ""

# Check if stack exists
if ! aws cloudformation describe-stacks \
    --stack-name "$STACK_NAME" \
    --region "$AWS_REGION" \
    &>/dev/null; then
    echo -e "${YELLOW}Stack '$STACK_NAME' does not exist or has already been deleted${RESET}"
    exit 0
fi

# Show stack resources
echo -e "${YELLOW}This will delete the following stack and ALL its resources:${RESET}"
aws cloudformation describe-stacks \
    --stack-name "$STACK_NAME" \
    --region "$AWS_REGION" \
    --query 'Stacks[0].[StackName,StackStatus,CreationTime]' \
    --output table 2>/dev/null || true

echo -e ""
echo -e "${RED}Resources that will be DELETED:${RESET}"
echo -e "  - Kinesis Data Stream (kleos-stream-$ENV)"
echo -e "  - API Gateway REST API (kleos-api-$ENV)"
echo -e "  - Ingest Lambda Function (kleos-ingest-$ENV)"
echo -e "  - Process Lambda Function (kleos-process-$ENV)"
echo -e "  - Dead Letter Queue (kleos-dlq-$ENV)"
echo -e "  - CloudWatch Log Groups"
echo -e "  - CloudWatch Alarms"
echo -e "  - IAM Roles and Policies"
echo -e ""
echo -e "${RED}This action CANNOT be undone!${RESET}"
echo -e ""

# Confirmation
read -p "Are you sure you want to destroy '$STACK_NAME'? Type 'yes' to confirm: " confirmation

if [ "$confirmation" != "yes" ]; then
    echo -e "${YELLOW}Destruction cancelled${RESET}"
    exit 0
fi

echo -e ""
echo -e "${YELLOW}Initiating stack deletion...${RESET}"

# Delete the stack
if aws cloudformation delete-stack \
    --stack-name "$STACK_NAME" \
    --region "$AWS_REGION"; then
    echo -e "${GREEN}✓ Stack deletion initiated${RESET}"
else
    echo -e "${RED}✗ Failed to initiate stack deletion${RESET}"
    exit 1
fi

# Wait for deletion
echo -e "${YELLOW}Waiting for stack deletion to complete...${RESET}"
echo -e "${CYAN}This may take several minutes${RESET}"

if aws cloudformation wait stack-delete-complete \
    --stack-name "$STACK_NAME" \
    --region "$AWS_REGION" 2>/dev/null; then
    echo -e ""
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
    echo -e "${GREEN}✓ Stack '$STACK_NAME' successfully deleted!${RESET}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
else
    echo -e ""
    echo -e "${YELLOW}Checking deletion status...${RESET}"

    # Check if stack still exists
    if aws cloudformation describe-stacks \
        --stack-name "$STACK_NAME" \
        --region "$AWS_REGION" \
        &>/dev/null; then

        STACK_STATUS=$(aws cloudformation describe-stacks \
            --stack-name "$STACK_NAME" \
            --region "$AWS_REGION" \
            --query 'Stacks[0].StackStatus' \
            --output text 2>/dev/null)

        echo -e "${RED}✗ Stack deletion failed${RESET}"
        echo -e "${YELLOW}Current status: $STACK_STATUS${RESET}"
        echo -e ""
        echo -e "${CYAN}Check the AWS Console for details:${RESET}"
        echo -e "https://console.aws.amazon.com/cloudformation/home?region=$AWS_REGION#/stacks"
        exit 1
    else
        echo -e "${GREEN}✓ Stack deleted successfully${RESET}"
    fi
fi

echo -e ""
echo -e "${CYAN}All resources have been removed. No ongoing AWS costs for this environment.${RESET}"
