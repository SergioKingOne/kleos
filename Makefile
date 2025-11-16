# Kleos - AWS Serverless Event Streaming Makefile
# Type 'make help' to see all available commands

.PHONY: help build deploy-dev deploy-staging deploy-prod validate clean test-api test-all-actions logs-producer logs-consumer check-dlq pause resume status delete-dev delete-all outputs resources local-api test-local install-deps scale-down scale-up

# Load environment variables from .env file
ifneq (,$(wildcard .env))
    include .env
    export
endif

# Colors for output
CYAN := \033[0;36m
GREEN := \033[0;32m
YELLOW := \033[0;33m
RED := \033[0;31m
NC := \033[0m # No Color

# Configuration
STACK_NAME_DEV := kleos-dev
STACK_NAME_STAGING := kleos-staging
STACK_NAME_PROD := kleos-prod
REGION := us-east-1

# Enable Rust support for SAM
export SAM_CLI_BETA_RUST_CARGO_LAMBDA=1

# ========================================
# Help
# ========================================

help: ## Show this help message
	@echo "$(CYAN)Kleos - AWS Serverless Event Streaming$(NC)"
	@echo ""
	@echo "$(GREEN)Build & Deploy:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E '(build|deploy|validate)' | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(CYAN)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Local Development:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E '(local|logs)' | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(CYAN)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Testing:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E '(test|check)' | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(CYAN)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Cost Management:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E '(pause|resume|status|scale)' | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(CYAN)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Infrastructure:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E '(delete|outputs|resources)' | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(CYAN)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Utilities:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -E '(clean|install|help)' | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(CYAN)%-20s$(NC) %s\n", $$1, $$2}'

# ========================================
# Build & Deploy
# ========================================

build: ## Build all Lambda functions
	@echo "$(CYAN)Building Lambda functions...$(NC)"
	sam build --parallel
	@echo "$(GREEN)✓ Build complete$(NC)"

validate: ## Validate SAM template
	@echo "$(CYAN)Validating SAM template...$(NC)"
	sam validate
	@echo "$(GREEN)✓ Template is valid$(NC)"

deploy-dev: build ## Deploy to dev environment
	@echo "$(CYAN)Deploying to dev environment...$(NC)"
	sam deploy --config-env dev --no-confirm-changeset --no-fail-on-empty-changeset
	@echo "$(GREEN)✓ Deployed to dev$(NC)"
	@$(MAKE) outputs

deploy-staging: build ## Deploy to staging environment
	@echo "$(CYAN)Deploying to staging environment...$(NC)"
	sam deploy --config-env staging --no-confirm-changeset --no-fail-on-empty-changeset
	@echo "$(GREEN)✓ Deployed to staging$(NC)"

deploy-prod: build ## Deploy to production (requires confirmation)
	@echo "$(YELLOW)⚠ Warning: Deploying to PRODUCTION$(NC)"
	@read -p "Are you sure? (yes/no): " confirm && [ "$$confirm" = "yes" ] || (echo "$(RED)Deployment cancelled$(NC)" && exit 1)
	sam deploy --config-env prod
	@echo "$(GREEN)✓ Deployed to production$(NC)"

# ========================================
# Local Development
# ========================================

local-api: build ## Start local API Gateway
	@echo "$(CYAN)Starting local API Gateway on http://127.0.0.1:3000$(NC)"
	SAM_CLI_BUILD_MODE=debug sam local start-api

test-local: build ## Test producer Lambda locally
	@echo "$(CYAN)Testing producer Lambda locally...$(NC)"
	sam local invoke KleosApiProducer --event test-events/api-request.json

logs-producer: ## Tail producer Lambda logs
	@echo "$(CYAN)Tailing producer logs (Ctrl+C to stop)...$(NC)"
	sam logs --stack-name $(STACK_NAME_DEV) --name KleosApiProducer --tail

logs-consumer: ## Tail consumer Lambda logs
	@echo "$(CYAN)Tailing consumer logs (Ctrl+C to stop)...$(NC)"
	sam logs --stack-name $(STACK_NAME_DEV) --name KleosStreamConsumer --tail

# ========================================
# Testing
# ========================================

test-api: ## Send test request to deployed API
	@echo "$(CYAN)Sending test request to API...$(NC)"
	@API_ENDPOINT=$$(aws cloudformation describe-stacks --stack-name $(STACK_NAME_DEV) --query 'Stacks[0].Outputs[?OutputKey==`ApiEndpoint`].OutputValue' --output text 2>/dev/null); \
	if [ -z "$$API_ENDPOINT" ]; then \
		echo "$(RED)✗ Stack not deployed. Run 'make deploy-dev' first.$(NC)"; \
		exit 1; \
	fi; \
	echo "API Endpoint: $$API_ENDPOINT"; \
	curl -X POST $$API_ENDPOINT/events \
		-H "Content-Type: application/json" \
		-d '{"user_id":"test-user","action":"create","details":"Test from Makefile"}' \
		-w "\n$(GREEN)✓ Request sent (Status: %{http_code})$(NC)\n"

test-all-actions: ## Test all event types (create, update, delete, view)
	@echo "$(CYAN)Testing all action types...$(NC)"
	@API_ENDPOINT=$$(aws cloudformation describe-stacks --stack-name $(STACK_NAME_DEV) --query 'Stacks[0].Outputs[?OutputKey==`ApiEndpoint`].OutputValue' --output text 2>/dev/null); \
	if [ -z "$$API_ENDPOINT" ]; then \
		echo "$(RED)✗ Stack not deployed. Run 'make deploy-dev' first.$(NC)"; \
		exit 1; \
	fi; \
	echo "Testing CREATE action..."; \
	curl -s -X POST $$API_ENDPOINT/events -H "Content-Type: application/json" -d '{"user_id":"alice","action":"create","details":"Creating resource"}' && echo ""; \
	echo "Testing UPDATE action..."; \
	curl -s -X POST $$API_ENDPOINT/events -H "Content-Type: application/json" -d '{"user_id":"bob","action":"update","details":"Updating resource"}' && echo ""; \
	echo "Testing DELETE action..."; \
	curl -s -X POST $$API_ENDPOINT/events -H "Content-Type: application/json" -d '{"user_id":"charlie","action":"delete","details":"Deleting resource"}' && echo ""; \
	echo "Testing VIEW action..."; \
	curl -s -X POST $$API_ENDPOINT/events -H "Content-Type: application/json" -d '{"user_id":"david","action":"view","details":"Viewing resource"}' && echo ""; \
	echo "$(GREEN)✓ All actions tested. Wait 10 seconds, then run 'make logs-consumer' to see results.$(NC)"

check-dlq: ## Check dead letter queue for failed messages
	@echo "$(CYAN)Checking DLQ for failed messages...$(NC)"
	@DLQ_URL=$$(aws cloudformation describe-stacks --stack-name $(STACK_NAME_DEV) --query 'Stacks[0].Outputs[?OutputKey==`DLQUrl`].OutputValue' --output text 2>/dev/null); \
	if [ -z "$$DLQ_URL" ]; then \
		echo "$(RED)✗ Stack not deployed. Run 'make deploy-dev' first.$(NC)"; \
		exit 1; \
	fi; \
	COUNT=$$(aws sqs get-queue-attributes --queue-url $$DLQ_URL --attribute-names ApproximateNumberOfMessages --query 'Attributes.ApproximateNumberOfMessages' --output text); \
	if [ "$$COUNT" = "0" ]; then \
		echo "$(GREEN)✓ DLQ is empty (no failed messages)$(NC)"; \
	else \
		echo "$(YELLOW)⚠ DLQ has $$COUNT message(s)$(NC)"; \
	fi

# ========================================
# Cost Management
# ========================================

pause: ## Delete dev stack to save costs (~$11/month)
	@echo "$(YELLOW)⚠ This will delete the dev stack to save Kinesis costs (~\$$11/month)$(NC)"
	@echo "$(CYAN)You can redeploy anytime with 'make resume'$(NC)"
	@read -p "Continue? (yes/no): " confirm && [ "$$confirm" = "yes" ] || (echo "$(RED)Cancelled$(NC)" && exit 1)
	@echo "$(CYAN)Deleting stack...$(NC)"
	sam delete --stack-name $(STACK_NAME_DEV) --no-prompts
	@echo "$(GREEN)✓ Stack deleted. Run 'make resume' to redeploy.$(NC)"

resume: deploy-dev ## Redeploy dev stack (alias for deploy-dev)
	@echo "$(GREEN)✓ Stack resumed and ready to use$(NC)"

status: ## Show stack status and estimated costs
	@echo "$(CYAN)Stack Status:$(NC)"
	@STACK_STATUS=$$(aws cloudformation describe-stacks --stack-name $(STACK_NAME_DEV) --query 'Stacks[0].StackStatus' --output text 2>/dev/null || echo "NOT_DEPLOYED"); \
	if [ "$$STACK_STATUS" = "NOT_DEPLOYED" ]; then \
		echo "  Status: $(RED)NOT DEPLOYED$(NC)"; \
		echo "  Estimated Cost: $(GREEN)\$$0/month (stack is paused)$(NC)"; \
		echo "  Run 'make resume' to deploy"; \
	elif [ "$$STACK_STATUS" = "CREATE_COMPLETE" ] || [ "$$STACK_STATUS" = "UPDATE_COMPLETE" ]; then \
		echo "  Status: $(GREEN)$$STACK_STATUS$(NC)"; \
		STREAM_STATUS=$$(aws kinesis describe-stream --stream-name kleos-stream-dev --query 'StreamDescription.StreamStatus' --output text 2>/dev/null || echo "N/A"); \
		echo "  Kinesis Stream: $$STREAM_STATUS"; \
		SHARD_COUNT=$$(aws kinesis describe-stream --stream-name kleos-stream-dev --query 'length(StreamDescription.Shards)' --output text 2>/dev/null || echo "0"); \
		echo "  Kinesis Shards: $$SHARD_COUNT"; \
		COST=$$((11 * SHARD_COUNT)); \
		echo "  Estimated Cost: $(YELLOW)\$$$$COST/month$(NC) (Kinesis only)"; \
		echo ""; \
		echo "$(CYAN)API Endpoint:$(NC)"; \
		API_ENDPOINT=$$(aws cloudformation describe-stacks --stack-name $(STACK_NAME_DEV) --query 'Stacks[0].Outputs[?OutputKey==`ApiEndpoint`].OutputValue' --output text 2>/dev/null); \
		echo "  $$API_ENDPOINT"; \
	else \
		echo "  Status: $(YELLOW)$$STACK_STATUS$(NC)"; \
	fi

scale-down: ## Reduce Kinesis to 1 shard (minimal cost)
	@echo "$(CYAN)Scaling down to 1 shard (minimal cost: ~\$$11/month)...$(NC)"
	sam deploy --config-env dev --parameter-overrides "KinesisShardCount=1" --no-confirm-changeset
	@echo "$(GREEN)✓ Scaled down to 1 shard$(NC)"

scale-up: ## Increase Kinesis to 5 shards (production capacity)
	@echo "$(CYAN)Scaling up to 5 shards (higher capacity: ~\$$55/month)...$(NC)"
	sam deploy --config-env dev --parameter-overrides "KinesisShardCount=5" --no-confirm-changeset
	@echo "$(GREEN)✓ Scaled up to 5 shards$(NC)"

# ========================================
# Infrastructure Management
# ========================================

delete-dev: ## Delete dev stack
	@echo "$(YELLOW)⚠ Warning: Deleting dev stack$(NC)"
	@read -p "Are you sure? (yes/no): " confirm && [ "$$confirm" = "yes" ] || (echo "$(RED)Cancelled$(NC)" && exit 1)
	sam delete --stack-name $(STACK_NAME_DEV) --no-prompts
	@echo "$(GREEN)✓ Dev stack deleted$(NC)"

delete-staging: ## Delete staging stack
	@echo "$(YELLOW)⚠ Warning: Deleting staging stack$(NC)"
	@read -p "Are you sure? (yes/no): " confirm && [ "$$confirm" = "yes" ] || (echo "$(RED)Cancelled$(NC)" && exit 1)
	sam delete --stack-name $(STACK_NAME_STAGING) --no-prompts
	@echo "$(GREEN)✓ Staging stack deleted$(NC)"

delete-prod: ## Delete production stack
	@echo "$(RED)⚠⚠⚠ WARNING: DELETING PRODUCTION STACK ⚠⚠⚠$(NC)"
	@read -p "Type 'DELETE PRODUCTION' to confirm: " confirm && [ "$$confirm" = "DELETE PRODUCTION" ] || (echo "$(RED)Cancelled$(NC)" && exit 1)
	sam delete --stack-name $(STACK_NAME_PROD) --no-prompts
	@echo "$(GREEN)✓ Production stack deleted$(NC)"

delete-all: ## Delete all stacks (dev, staging, prod)
	@echo "$(RED)⚠⚠⚠ WARNING: DELETING ALL STACKS ⚠⚠⚠$(NC)"
	@read -p "Type 'DELETE ALL' to confirm: " confirm && [ "$$confirm" = "DELETE ALL" ] || (echo "$(RED)Cancelled$(NC)" && exit 1)
	@for stack in $(STACK_NAME_DEV) $(STACK_NAME_STAGING) $(STACK_NAME_PROD); do \
		if aws cloudformation describe-stacks --stack-name $$stack >/dev/null 2>&1; then \
			echo "$(CYAN)Deleting $$stack...$(NC)"; \
			sam delete --stack-name $$stack --no-prompts; \
		else \
			echo "$(YELLOW)Stack $$stack does not exist, skipping$(NC)"; \
		fi \
	done
	@echo "$(GREEN)✓ All stacks deleted$(NC)"

outputs: ## Show stack outputs (API endpoint, stream ARN, etc.)
	@echo "$(CYAN)Stack Outputs:$(NC)"
	@aws cloudformation describe-stacks --stack-name $(STACK_NAME_DEV) --query 'Stacks[0].Outputs[*].[OutputKey,OutputValue]' --output table 2>/dev/null || echo "$(RED)Stack not deployed$(NC)"

resources: ## List all deployed resources
	@echo "$(CYAN)Deployed Resources:$(NC)"
	@aws cloudformation describe-stack-resources --stack-name $(STACK_NAME_DEV) --query 'StackResources[*].[ResourceType,LogicalResourceId,ResourceStatus]' --output table 2>/dev/null || echo "$(RED)Stack not deployed$(NC)"

# ========================================
# Utilities
# ========================================

clean: ## Clean build artifacts
	@echo "$(CYAN)Cleaning build artifacts...$(NC)"
	rm -rf .aws-sam
	@echo "$(GREEN)✓ Clean complete$(NC)"

install-deps: ## Install required tools (SAM CLI, AWS CLI)
	@echo "$(CYAN)Checking dependencies...$(NC)"
	@command -v sam >/dev/null 2>&1 || { echo "$(YELLOW)Installing SAM CLI...$(NC)"; brew install aws-sam-cli; }
	@command -v aws >/dev/null 2>&1 || { echo "$(YELLOW)Installing AWS CLI...$(NC)"; brew install awscli; }
	@command -v rustc >/dev/null 2>&1 || { echo "$(YELLOW)Installing Rust...$(NC)"; curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh; }
	@echo "$(GREEN)✓ All dependencies installed$(NC)"

# ========================================
# Quick Commands (Shortcuts)
# ========================================

dev: deploy-dev ## Shortcut for deploy-dev

test: test-all-actions ## Shortcut for test-all-actions

logs: logs-consumer ## Shortcut for logs-consumer

.DEFAULT_GOAL := help
