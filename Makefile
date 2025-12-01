.PHONY: help build test clean docker-build docker-run docker-compose-up docker-compose-down

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## Build the project in debug mode
	cargo build

build-release: ## Build the project in release mode
	cargo build --release

test: ## Run all tests
	cargo test

test-verbose: ## Run tests with verbose output
	cargo test -- --nocapture

test-e2e: ## Run end-to-end tests
	cargo test --test e2e_test

clippy: ## Run clippy linter
	cargo clippy -- -D warnings

fmt: ## Format code
	cargo fmt

fmt-check: ## Check code formatting
	cargo fmt --all -- --check

clean: ## Clean build artifacts
	cargo clean

run: ## Run the gateway with example config
	cargo run -- start -c config.example.toml

monitor: ## Start TUI monitor with example config
	cargo run -- monitor -c config.example.toml

init: ## Generate sample configuration
	cargo run -- init -o config.toml

validate: ## Validate example configuration
	cargo run -- validate -c config.example.toml

docker-build: ## Build Docker image
	docker build -t open-gateway:latest .

docker-run: ## Run Docker container
	docker run --rm -it -p 8080:8080 -v $(PWD)/config:/app/config:ro open-gateway:latest start -c /app/config/config.toml

docker-compose-up: ## Start services with docker-compose
	docker compose up

docker-compose-down: ## Stop services with docker-compose
	docker compose down

docker-compose-dev: ## Start development service with docker-compose
	docker compose up dev

all: fmt clippy test build ## Run format, clippy, test, and build
