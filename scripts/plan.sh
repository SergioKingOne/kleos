#!/usr/bin/env bash
set -euo pipefail

# Plan script - shows infrastructure changes without deploying (like terraform plan)

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
ENV="${1:-${ENV:-dev}}"
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

# Validate environment
if [[ ! "$ENV" =~ ^(dev|staging|prod)$ ]]; then
    echo -e "${RED}Error: Invalid environment '$ENV'${RESET}"
    print_usage
    exit 1
fi

STACK_NAME="kleos-pipeline-$ENV"
CHANGE_SET_NAME="kleos-changeset-$(date +%s)"

echo -e "${CYAN}Planning infrastructure changes for ${YELLOW}$ENV${CYAN} environment...${RESET}"
echo -e "${CYAN}(Similar to 'terraform plan')${RESET}"
echo -e ""

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
echo -e ""

# Build with SAM
echo -e "${YELLOW}Building with SAM...${RESET}"
if ! sam build --beta-features --parallel --cached; then
    echo -e "${RED}✗ Build failed${RESET}"
    exit 1
fi
echo -e "${GREEN}✓ Build complete${RESET}"
echo -e ""

# Create change set (without executing)
echo -e "${YELLOW}Creating change set...${RESET}"
if sam deploy \
    --config-env "$ENV" \
    --beta-features \
    --no-execute-changeset \
    --no-confirm-changeset 2>&1 | tee /tmp/sam-deploy-output.txt; then
    echo -e "${GREEN}✓ Change set created${RESET}"
else
    # Check if it's just "no changes" error
    if grep -q "No changes to deploy" /tmp/sam-deploy-output.txt; then
        echo -e "${GREEN}✓ No infrastructure changes detected${RESET}"
        echo -e "${CYAN}Stack is up to date with the template${RESET}"
        exit 0
    else
        echo -e "${RED}✗ Failed to create change set${RESET}"
        exit 1
    fi
fi

echo -e ""
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e "${CYAN}Planned Changes (Change Set Preview)${RESET}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e ""

# Wait a moment for change set to be created
sleep 2

# Get the most recent change set
CHANGE_SET=$(aws cloudformation list-change-sets \
    --stack-name "$STACK_NAME" \
    --region "$AWS_REGION" \
    --query 'Summaries[0].ChangeSetName' \
    --output text 2>/dev/null || echo "")

if [ -z "$CHANGE_SET" ] || [ "$CHANGE_SET" == "None" ]; then
    echo -e "${YELLOW}No change set found. Stack may not exist yet (first deployment).${RESET}"
    echo -e "${CYAN}Run 'make deploy ENV=$ENV' to create the stack.${RESET}"
    exit 0
fi

# Describe the change set
echo -e "${YELLOW}Change Set:${RESET} $CHANGE_SET"
echo -e ""

# Show changes in a readable format
aws cloudformation describe-change-set \
    --stack-name "$STACK_NAME" \
    --change-set-name "$CHANGE_SET" \
    --region "$AWS_REGION" \
    --query 'Changes[*].[ResourceChange.Action,ResourceChange.ResourceType,ResourceChange.LogicalResourceId,ResourceChange.Replacement]' \
    --output table 2>/dev/null || {
        echo -e "${YELLOW}Could not retrieve change set details${RESET}"
        echo -e "${CYAN}Check AWS Console for details:${RESET}"
        echo -e "https://console.aws.amazon.com/cloudformation/home?region=$AWS_REGION#/stacks"
    }

echo -e ""
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e ""

# Ask if user wants to execute the change set
echo -e "${YELLOW}What would you like to do?${RESET}"
echo -e "  1) Execute change set (deploy changes)"
echo -e "  2) Delete change set (cancel)"
echo -e "  3) View detailed changes in AWS Console"
echo -e "  4) Exit (leave change set for later)"
echo -e ""
read -p "Enter choice [1-4]: " choice

case $choice in
    1)
        echo -e ""
        echo -e "${YELLOW}Executing change set...${RESET}"
        if aws cloudformation execute-change-set \
            --stack-name "$STACK_NAME" \
            --change-set-name "$CHANGE_SET" \
            --region "$AWS_REGION"; then
            echo -e "${GREEN}✓ Change set execution initiated${RESET}"
            echo -e "${YELLOW}Monitoring stack update...${RESET}"

            # Wait for stack update
            aws cloudformation wait stack-update-complete \
                --stack-name "$STACK_NAME" \
                --region "$AWS_REGION" && \
                echo -e "${GREEN}✓ Stack update complete!${RESET}" || \
                echo -e "${RED}Stack update failed or timed out. Check AWS Console.${RESET}"
        else
            echo -e "${RED}✗ Failed to execute change set${RESET}"
            exit 1
        fi
        ;;
    2)
        echo -e ""
        echo -e "${YELLOW}Deleting change set...${RESET}"
        if aws cloudformation delete-change-set \
            --stack-name "$STACK_NAME" \
            --change-set-name "$CHANGE_SET" \
            --region "$AWS_REGION"; then
            echo -e "${GREEN}✓ Change set deleted${RESET}"
        else
            echo -e "${RED}✗ Failed to delete change set${RESET}"
            exit 1
        fi
        ;;
    3)
        echo -e ""
        echo -e "${CYAN}Opening AWS Console...${RESET}"
        CONSOLE_URL="https://console.aws.amazon.com/cloudformation/home?region=$AWS_REGION#/stacks/changesets/changes?stackId=$STACK_NAME&changeSetId=$CHANGE_SET"
        echo -e "${CYAN}$CONSOLE_URL${RESET}"

        # Try to open in browser (macOS/Linux)
        if command -v open &> /dev/null; then
            open "$CONSOLE_URL"
        elif command -v xdg-open &> /dev/null; then
            xdg-open "$CONSOLE_URL"
        fi
        ;;
    4)
        echo -e ""
        echo -e "${CYAN}Change set preserved. Execute later with:${RESET}"
        echo -e "${YELLOW}aws cloudformation execute-change-set \\${RESET}"
        echo -e "${YELLOW}  --stack-name $STACK_NAME \\${RESET}"
        echo -e "${YELLOW}  --change-set-name $CHANGE_SET${RESET}"
        ;;
    *)
        echo -e "${RED}Invalid choice${RESET}"
        exit 1
        ;;
esac

echo -e ""
echo -e "${GREEN}Done!${RESET}"
