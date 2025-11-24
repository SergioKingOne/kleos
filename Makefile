.PHONY: help install deps build build-ingest build-process test fmt lint \
        validate plan deploy deploy-function local-api logs outputs destroy clean

# Environment variables with defaults
ENV ?= dev
FUNC ?= ingest
AWS_REGION ?= us-east-1

# Colors for output
CYAN := \033[36m
GREEN := \033[32m
YELLOW := \033[33m
RESET := \033[0m

help: ## Show this help message
	@echo "$(CYAN)Kleos Infrastructure Management$(RESET)"
	@echo ""
	@echo "$(GREEN)Available targets:$(RESET)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(CYAN)%-20s$(RESET) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(YELLOW)Environment variables:$(RESET)"
	@echo "  ENV=dev|staging|prod    (default: dev)"
	@echo "  FUNC=ingest|process     (default: ingest)"
	@echo "  AWS_REGION=us-east-1    (default: us-east-1)"
	@echo ""
	@echo "$(CYAN)Examples:$(RESET)"
	@echo "  make plan ENV=dev              # Preview changes"
	@echo "  make deploy ENV=dev            # Deploy changes"
	@echo "  make logs FUNC=process ENV=prod"
	@echo "  make destroy ENV=staging"

install: ## Install required tools (cargo-lambda, SAM CLI)
	cargo install cargo-lambda

deps: ## Fetch Cargo dependencies
	cargo fetch

build: ## Build all Lambda functions
	@./scripts/build.sh

build-ingest: ## Build only the ingest Lambda
	cargo lambda build --release --arm64 -p kleos-ingest-lambda

build-process: ## Build only the process Lambda
	cargo lambda build --release --arm64 -p kleos-process-lambda

test: ## Run all tests
	cargo test --workspace

fmt: ## Format code
	cargo fmt --all

lint: ## Run clippy linter
	cargo clippy --all-targets --all-features -- -D warnings

validate: ## Validate SAM templates
	cd infra && sam validate --lint

plan: ## Show infrastructure changes without deploying (usage: make plan ENV=dev)
	@ENV=$(ENV) ./scripts/plan.sh

deploy: ## Deploy to environment (usage: make deploy ENV=dev)
	@ENV=$(ENV) ./scripts/deploy.sh

deploy-function: ## Deploy single function (usage: make deploy-function FUNC=ingest ENV=dev)
	@./scripts/deploy-function.sh $(FUNC) $(ENV)

local-api: build ## Start local API Gateway
	@./scripts/local-dev.sh

logs: ## Tail CloudWatch logs (usage: make logs FUNC=ingest ENV=dev)
	@./scripts/logs.sh $(FUNC) $(ENV)

outputs: ## Show stack outputs (usage: make outputs ENV=dev)
	@aws cloudformation describe-stacks \
		--stack-name kleos-pipeline-$(ENV) \
		--region $(AWS_REGION) \
		--query 'Stacks[0].Outputs' \
		--output table

destroy: ## Destroy infrastructure (usage: make destroy ENV=dev)
	@ENV=$(ENV) ./scripts/destroy.sh

clean: ## Clean build artifacts
	cargo clean
	rm -rf .aws-sam infra/.aws-sam

.DEFAULT_GOAL := help
