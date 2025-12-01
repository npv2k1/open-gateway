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

clippy: ## Run clippy linter
	cargo clippy -- -D warnings

fmt: ## Format code
	cargo fmt

fmt-check: ## Check code formatting
	cargo fmt --all -- --check

clean: ## Clean build artifacts
	cargo clean

docker-build: ## Build Docker image
	docker build -t template-rust:latest .

docker-run: ## Run Docker container in TUI mode
	docker run --rm -it -v $(PWD)/data:/app/data template-rust:latest tui

docker-compose-up: ## Start services with docker-compose
	docker compose up

docker-compose-down: ## Stop services with docker-compose
	docker compose down

docker-compose-dev: ## Start development service with docker-compose
	docker compose up dev

all: fmt clippy test build ## Run format, clippy, test, and build
