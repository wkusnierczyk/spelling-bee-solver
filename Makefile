# SBS Root Orchestrator
NAMESPACE = sbs-namespace
RELEASE_NAME = sbs-prod

# Binary Configuration
SBS_CLI_NAME ?= sbs
SBS_SERVER_NAME ?= sbs-server

# Data Configuration
SBS_DICT ?= sbs-solver/data/dictionary.txt

# PID files for background process management
BACKEND_PID = .backend.pid
FRONTEND_PID = .frontend.pid

.PHONY: help test lint format build-backend install-backend run-backend build-frontend run-frontend build-cli install-cli start-local stop-local status

help: ## Show help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# --- Hygiene & Testing ---

test: ## Run backend unit and integration tests
	cd sbs-solver && cargo test

lint: ## Run clippy linter on backend
	cd sbs-solver && cargo clippy -- -D warnings

format: ## Format backend code using rustfmt
	cd sbs-solver && cargo fmt

# --- Backend Management ---

build-cli:
	cd sbs-solver && cargo build --bin $(SBS_CLI_NAME)

install-cli:
	cd sbs-solver && cargo install --path . --bin $(SBS_CLI_NAME) --force

# --- Backend Management ---

build-backend:
	cd sbs-solver && cargo build --bin $(SBS_SERVER_NAME)

install-backend:
	cd sbs-solver && cargo install --path . --bin $(SBS_SERVER_NAME) --force

run-backend: ## Run the backend server in the foreground
	SBS_DICT=$(SBS_DICT) $(SBS_SERVER_NAME)

# --- Frontend Management ---

build-frontend:
	cd sbs-gui && npm install && npm run build

run-frontend: ## Run the frontend dev server in the foreground
	cd sbs-gui && npm run dev

# --- Local Orchestration ---

start-local: stop-local ## Start Backend and Frontend in background
	@echo "🚀 Starting Backend Server..."
	@SBS_DICT=$(SBS_DICT) $(SBS_SERVER_NAME) > backend.log 2>&1 & echo $$! > $(BACKEND_PID)
	@echo "🚀 Starting Frontend (Vite)..."
	@cd sbs-gui && npm run dev > ../frontend.log 2>&1 & echo $$! > ../$(FRONTEND_PID)
	@sleep 2
	@echo "\n✅ SERVICES STARTED"
	@echo "🔗 Frontend URL: http://localhost:5173"
	@echo "📝 Logs: tail -f backend.log frontend.log"
	@echo "🛑 To stop services, run: make stop-local\n"

stop-local: ## Stop background services and verify
	@echo "🛑 Stopping local services..."
	@if [ -f $(BACKEND_PID) ]; then \
		K_PID=$$(cat $(BACKEND_PID)); \
		kill $$K_PID 2>/dev/null && echo "  ...Backend (PID $$K_PID) stopped" || echo "  ...Backend already stopped"; \
		rm $(BACKEND_PID); \
	fi
	@if [ -f $(FRONTEND_PID) ]; then \
		K_PID=$$(cat $(FRONTEND_PID)); \
		kill $$K_PID 2>/dev/null && echo "  ...Frontend (PID $$K_PID) stopped" || echo "  ...Frontend already stopped"; \
		rm $(FRONTEND_PID); \
	fi
	@# Cleanup potential orphaned vite/node processes
	@lsof -ti:5173 | xargs kill -9 >/dev/null 2>&1 || true
	@echo "✅ Cleanup complete."

# --- Cloud/Infra (Preserved) ---

status: ## Check Cloud Health
	kubectl get ingress,pods -n $(NAMESPACE)

deploy: ## Build and Push to Cloud (GCP)
	./scripts/deploy_gcp.sh