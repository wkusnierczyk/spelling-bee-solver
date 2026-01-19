# SBS Root Orchestrator
NAMESPACE = sbs-namespace
RELEASE_NAME = sbs-prod

# Structure
SBS_BACKEND_DIR = sbs-backend
SBS_FRONTEND_DIR = sbs-frontend

# Binary Configuration
SBS_CLI_NAME ?= sbs
SBS_BACKEND_NAME ?= sbs-backend
SBS_FRONTEND_NAME ?= sbs-frontend

# Data Configuration
DICT_URL = https://raw.githubusercontent.com/dwyl/english-words/master/words_alpha.txt
SBS_DICT ?= $(SBS_BACKEND_DIR)/data/dictionary.txt

# PID files for background process management
BACKEND_PID = .backend.pid
FRONTEND_PID = .frontend.pid

# Containerisation
DOCKER_TAG ?= latest

# Conenience function for info messages
define info
	@printf "\033[36m[DIAG] %s\033[0m\n" $(1) >&2
endef


.PHONY: help test lint format build-backend install-backend run-backend build-frontend run-frontend build-cli install-cli start-local stop-local status

help: ## Show help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'



# --- Data Management ---

setup-dictionary: ## Force download of the dictionary (overwrites if exists)
	$(call info, "Downloading fresh dictionary to $(SBS_DICT)...")
	@mkdir -p $(dir $(SBS_DICT))
	@curl -L -o $(SBS_DICT) $(DICT_URL) || (rm -f $(SBS_DICT) && exit 1)
	$(call info, "Dictionary downloaded successfully.")


# --- Hygiene & Testing ---

test: ## Run backend unit and integration tests
	$(call info, "Running backend tests...")
	cd $(SBS_BACKEND_DIR) && cargo test

lint: ## Run clippy linter on backend
	$(call info, "Running backend linter (clippy)...")
	cd $(SBS_BACKEND_DIR) && cargo clippy -- -D warnings

format: ## Format backend code using rustfmt
	$(call info, "Formatting backend code...")
	cd $(SBS_BACKEND_DIR) && cargo fmt


# --- CLI Management ---

build-cli:
	$(call info, "Building CLI...")
	cd $(SBS_BACKEND_DIR) && cargo build --bin $(SBS_CLI_NAME)

install-cli:
	$(call info, "Installing CLI...")
	cd $(SBS_BACKEND_DIR) && cargo install --path . --bin $(SBS_CLI_NAME) --force


# --- Backend Management ---

build-backend:
	$(call info, "Building backend...")
	cd $(SBS_BACKEND_DIR) && cargo build --bin $(SBS_BACKEND_NAME)

install-backend: build-backend
	$(call info, "Installing backend...")
	d $(SBS_BACKEND_DIR) && cargo install --path . --bin $(SBS_BACKEND_NAME) --force

start-backend: install-backend
	$(call info, "Starting backend...")
	cd $(SBS_BACKEND_DIR) && BS_DICT=$(SBS_DICT) $(SBS_BACKEND_NAME)


# --- Frontend Management ---

build-frontend:
	$(call info, "Building frontend...")
	cd $(SBS_FRONTEND_DIR) && npm install && npm run build

start-frontend: ## Run the frontend dev server in the foreground
	$(call info, "Starting frontend...")
	cd $(SBS_FRONTEND_DIR) && npm run dev

# --- Local Orchestration ---

start-local: stop-local ## Start Backend and Frontend in background
	$(call info, "Starting Backend Server...")
	@SBS_DICT=$(SBS_DICT) $(SBS_BACKEND_NAME) > backend.log 2>&1 & echo $$! > $(BACKEND_PID)
	$(call info, "Starting Frontend (Vite)...")
	@cd $(SBS_FRONTEND_DIR) && npm run dev > ../frontend.log 2>&1 & echo $$! > ../$(FRONTEND_PID)
	@sleep 2
	$(call info, "\nSERVICES STARTED")
	$(call info, "Frontend URL: http://localhost:5173")
	$(call info, "Logs: tail -f backend.log frontend.log")
	$(call info, "To stop services, run: make stop-local\n")

stop-local: ## Stop background services and verify
	$(call info, "Stopping local services...")
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
	$(call info, "Cleanup complete.")


# --- Backend Docker Targets ---

build-backend-image:
	$(call info, "Building backend image...")
	docker build \
		-t $(SBS_BACKEND_NAME):$(DOCKER_TAG) \
		-f $(SBS_BACKEND_DIR)/Dockerfile $(SBS_BACKEND_DIR)

start-backend-container: remove-backend-container
	$(call info, "Starting backend container...")
	docker run -d --name $(SBS_BACKEND_NAME) \
		-p 8080:8080 \
		-v $(PWD)/$(SBS_BACKEND_DIR)/data:/app/data \
		$(SBS_BACKEND_NAME):$(DOCKER_TAG)
	$(call info, "Backend container started on http://localhost:8080")

test-backend-container: 
	$(call info, "Testing backend container...")
	curl -v http://localhost:8080/health
	curl -X POST http://localhost:8080/solve \
  		-H "Content-Type: application/json" \
  		-d '{"letters": "pelniga", "present": "a"}' | jq '{ size: length, head: .[:10] }'

stop-backend-container:
	$(call info, "Stopping backend container...")
	docker stop $(SBS_BACKEND_NAME) >/dev/null 2>&1 || true

remove-backend-container: stop-backend-container
	$(call info, "Removing backend container...")
	docker rm $(SBS_BACKEND_NAME) >/dev/null 2>&1 || true


# --- Frontend Docker Targets ---

build-frontend-image:
	$(call info, "Building frontend image $(SBS_FRONTEND_NAME):$(DOCKER_TAG)...")
	docker build \
		-t $(SBS_FRONTEND_NAME):$(DOCKER_TAG) \
		-f $(SBS_FRONTEND_DIR)/Dockerfile $(SBS_FRONTEND_DIR)
	$(call info, "Frontend build complete.")

start-frontend-container: remove-frontend-container
	$(call info, "Launching container $(SBS_FRONTEND_NAME)...")
	docker run -d --name $(SBS_FRONTEND_NAME) \
		-p 5173:80 \
		--link $(SBS_BACKEND_NAME):$(SBS_BACKEND_NAME) \
		$(SBS_FRONTEND_NAME):$(DOCKER_TAG)
	$(call info, "Frontend container reachable at http://localhost:5173")

test-frontend-container: ## Test the frontend's ability to proxy to the backend
	$(call info, "Testing Frontend Proxy: http://localhost:5173/solve...")
	@curl --fail -s -X POST http://localhost:5173/solve \
		-H "Content-Type: application/json" \
		-d '{"letters": "pelniga", "present": "a"}' > /dev/null || \
		(echo "Proxy Test Failed: Frontend cannot reach Backend" && exit 1)
	$(call info, "Proxy Test Passed: Frontend successfully routed request to Backend.")

stop-frontend-container:
	$(call info, "Stopping container $(SBS_FRONTEND_NAME)...")
	@docker stop $(SBS_FRONTEND_NAME) >/dev/null 2>&1 || true

remove-frontend-container: stop-frontend-container ## Stop and then remove the frontend test container
	$(call info, "Cleaning up container $(SBS_FRONTEND_NAME)...")
	@docker rm $(SBS_FRONTEND_NAME) >/dev/null 2>&1 || true


# --- Docker Orchestration ---

start-docker-stack: setup-dictionary build-backend-image build-frontend-image start-backend-container start-frontend-container
	@sleep 2
	@# @make test-frontend-container
	$(call info, "Stack running!")

test-docker-stack: setup-dictionary build-backend-image build-frontend-image start-backend-container start-frontend-container
	$(call info, "Verifying Full Docker Stack...")
	@make fullstack-smoke-test
	$(call info, "Full Docker Stack is verified and running!")

stop-docker-stack: stop-frontend-container stop-backend-container
	$(call info, "Stack stopped.")

remove-docker-stack: remove-frontend-container remove-backend-container
	$(call info, "Stack removed.")


# --- Docker Compose Orchestration ---

start-compose-stack: setup-dictionary 
	$(call info, "Starting stack with Docker Compose...")
	@docker compose up -d --build
	$(call info, "Stack is running.")
	$(call info, "Frontend: http://localhost:5173")
	$(call info, "Backend:  http://localhost:8080")

test-compose-stack: 
	$(call info, "Verifying Docker Compose stack...")
	@make fullstack-smoke-test

stop-compose-stack:
	$(call info, "Stopping Docker Compose stack...")
	@docker compose down


# --- Full Stack Testing ---

fullstack-smoke-test:
	$(call info, "Testing full stack...")
	$(call info, "Waiting for backend...")
	@timeout=30; \
	while ! curl -s --fail http://localhost:8080/health > /dev/null; do \
		if [ $$timeout -le 0 ]; then \
			echo "Timed out waiting for Backend to start."; \
			exit 1; \
		fi; \
		echo "   ...waiting for backend ($${timeout}s remaining)"; \
		sleep 1; \
		timeout=$$((timeout - 1)); \
	done
	$(call info, "Backend is up! Running full checks...")
	@make test-backend-container
	@make test-frontend-container




# --- Cloud/Infra (Preserved) ---

status: ## Check Cloud Health
	kubectl get ingress,pods -n $(NAMESPACE)

deploy: ## Build and Push to Cloud (GCP)
	./scripts/deploy_gcp.sh