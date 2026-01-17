# Build System for Spelling Bee Solver

PROJECT_NAME = sbs
CARGO = cargo
DOCKER = docker
IMAGE_NAME = sbs-solver

.PHONY: all build setup test clean format lint doc release help run-cli run-server docker-build docker-run docker-stop

all: setup format build test

setup: ## Download default dictionary data
	@echo "Setting up dictionary..."
	@./scripts/setup_dictionary.sh

build: ## Build all binaries (CLI and Server)
	$(CARGO) build

release: ## Build release versions
	$(CARGO) build --release

test: ## Run tests
	$(CARGO) test

format: ## Format code
	$(CARGO) fmt

lint: ## Lint code
	$(CARGO) clippy -- -D warnings

clean: ## Clean artifacts
	$(CARGO) clean
	rm -f data/dictionary.txt

doc: ## Generate documentation
	$(CARGO) doc --open

run-cli: ## Run the CLI tool (pass args like: ARGS="-l abc -p a")
	$(CARGO) run --bin sbs -- $(ARGS)

run-server: ## Run the API server locally
	$(CARGO) run --bin sbs-server

docker-build: ## Build the Docker image
	$(DOCKER) build -t $(IMAGE_NAME) .

docker-run: ## Run the service using Docker Compose
	$(DOCKER) compose up -d

docker-stop: ## Stop the Docker service
	$(DOCKER) compose down

help: ## Display help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'
