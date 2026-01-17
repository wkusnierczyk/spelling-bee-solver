# Build System for Spelling Bee Solver

PROJECT_NAME = sbs
CARGO = cargo

.PHONY: all build setup test clean format lint doc release help

all: setup format build test

setup: ## Download default dictionary data
	@echo "Setting up dictionary..."
	@./scripts/setup_dictionary.sh

build: ## Build the project in debug mode
	$(CARGO) build

release: ## Build the project in release mode
	$(CARGO) build --release

test: ## Run unit and integration tests
	$(CARGO) test

format: ## Format the code using rustfmt
	$(CARGO) fmt

lint: ## Lint the code using clippy
	$(CARGO) clippy -- -D warnings

clean: ## Clean build artifacts and data
	$(CARGO) clean
	rm -f data/dictionary.txt

doc: ## Generate documentation
	$(CARGO) doc --open

help: ## Display this help screen
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'
