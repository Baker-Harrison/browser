.PHONY: help build test check fmt clippy clean run benchmark install-hooks

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## Build the project
	cargo build --all-features

build-release: ## Build the project in release mode
	cargo build --release --all-features

test: ## Run all tests
	cargo test --all-features

test-coverage: ## Run tests with coverage
	cargo tarpaulin --out Html --all-features

check: ## Run cargo check
	cargo check --all-targets --all-features

fmt: ## Format the code
	cargo fmt --all

fmt-check: ## Check if code is formatted
	cargo fmt --all -- --check

clippy: ## Run clippy lints
	cargo clippy --all-targets --all-features -- -D warnings

clean: ## Clean build artifacts
	cargo clean

run: ## Run the project
	cargo run --all-features

run-release: ## Run the project in release mode
	cargo run --release --all-features

benchmark: ## Run benchmarks
	cargo bench --all-features

install-hooks: ## Install pre-commit hooks
	pre-commit install

update-deps: ## Update dependencies
	cargo update

audit: ## Audit dependencies for security vulnerabilities
	cargo audit

doc: ## Generate and open documentation
	cargo doc --open --all-features

doc-no-open: ## Generate documentation
	cargo doc --all-features
