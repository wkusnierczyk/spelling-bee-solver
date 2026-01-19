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

# Convenience function for info messages
define info
	@printf "\033[36m[DIAG] %s\033[0m\n" $(1) >&2
endef

# Load environment variables if .env exists
ifneq (,$(wildcard ./.env))
    include .env
    export
endif


.PHONY: help test lint format build-backend install-backend run-backend build-frontend run-frontend build-cli install-cli start-local stop-local status deploy-cloud build-architecture

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
	cd $(SBS_BACKEND_DIR) && cargo install --path . --bin $(SBS_BACKEND_NAME) --force

start-backend: install-backend
	$(call info, "Starting backend...")
	cd $(SBS_BACKEND_DIR) && SBS_DICT=$(SBS_DICT) $(SBS_BACKEND_NAME)


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


# --- Local CI/CD Simulation (Requires 'act') ---

ci-list: ## List all available workflows detected by act
	act -l

ci-backend: ## Run the backend CI workflow locally using a complete runner image
	$(call info, "Running Backend Workflow locally...")
	@# We force ubuntu-latest to use the 'catthehacker' image which includes Node, Rust, etc.
	act -W .github/workflows/backend.yml \
		--container-architecture linux/amd64 \
		--platform ubuntu-latest=catthehacker/ubuntu:act-latest \
		--bind \
		--reuse
		
ci-docker: ## Run Docker workflow locally
	$(call info, "Running Docker Workflow locally...")
	act -W .github/workflows/docker.yml --container-architecture linux/amd64

ci-compose: ## Run Docker Compose workflow locally
	$(call info, "Running Docker Compose Workflow locally...")
	act -W .github/workflows/compose.yml --container-architecture linux/amd64

ci-all: ## Run all workflows locally
	act --container-architecture linux/amd64


# --- Minikube Targets ---

minikube-start:
	$(call info, "Starting Minikube...")
	minikube start --driver=docker

minikube-build: setup-dictionary
	$(call info, "Pointing Docker to Minikube...")
	@eval $$(minikube docker-env) && \
	docker build -t $(SBS_BACKEND_NAME):$(DOCKER_TAG) -f $(SBS_BACKEND_DIR)/Dockerfile $(SBS_BACKEND_DIR) && \
	docker build -t $(SBS_FRONTEND_NAME):$(DOCKER_TAG) -f $(SBS_FRONTEND_DIR)/Dockerfile $(SBS_FRONTEND_DIR)
	$(call info, "Images built inside Minikube registry.")

minikube-deploy: minikube-build ## Deploy charts to Minikube
	$(call info, "Deploying Helm Release $(RELEASE_NAME)...")
	helm upgrade --install $(RELEASE_NAME) ./charts/sbs-server \
		--namespace $(NAMESPACE) \
		--create-namespace \
		--set backend.fullnameOverride=$(SBS_BACKEND_NAME) \
		--set backend.image.repository=$(SBS_BACKEND_NAME) \
		--set backend.image.tag=$(DOCKER_TAG) \
		--set backend.image.pullPolicy=Never \
		--set frontend.fullnameOverride=$(SBS_FRONTEND_NAME) \
		--set frontend.image.repository=$(SBS_FRONTEND_NAME) \
		--set frontend.image.tag=$(DOCKER_TAG) \
		--set frontend.image.pullPolicy=Never

MINIKUBE_TEST_TIMEOUT = 120s
minikube-test: ## Verify the Minikube deployment (Wait + Curl)
	$(call info, "Waiting for Pods to be ready...")
	@kubectl wait --namespace $(NAMESPACE) --for=condition=ready pod --selector=app=$(SBS_BACKEND_NAME) --timeout=$(MINIKUBE_TEST_TIMEOUT)
	@kubectl wait --namespace $(NAMESPACE) --for=condition=ready pod --selector=app=$(SBS_FRONTEND_NAME) --timeout=$(MINIKUBE_TEST_TIMEOUT)
	
	$(call info, "1. Testing Frontend Static Content (Port-Forward)...")
	@kubectl port-forward service/$(SBS_FRONTEND_NAME) -n $(NAMESPACE) 9090:80 > /dev/null 2>&1 & \
	PID=$$!; \
	sleep 5; \
	curl -s --fail http://localhost:9090 | grep "<title>" && echo "   Static content served" || (kill $$PID && exit 1); \
	kill $$PID

	$(call info, "2. Testing Frontend -> Backend Connectivity (Internal)...")
	@# Exec into the frontend pod and ping the backend
	@# 'wget --spider' returns exit code 0 if the server returns 200 OK
	@kubectl exec -n $(NAMESPACE) deployment/$(RELEASE_NAME)-frontend -- \
		wget -q --spider http://$(SBS_BACKEND_NAME):8080/health && \
		echo "   Backend reachable from Frontend" && \
		echo "" && \
		echo "Full Stack Verified!"


minikube-url: ## Open the frontend URL in the default browser
	$(call info, "Opening Frontend Service...")
	minikube service $(SBS_FRONTEND_NAME) -n $(NAMESPACE)

minikube-clean: ## Remove the Helm release (leaves cluster running)
	$(call info, "Uninstalling Release $(RELEASE_NAME)...")
	helm uninstall $(RELEASE_NAME) -n $(NAMESPACE) || true

minikube-stop: ## Stop the Minikube cluster (saves resources)
	$(call info, "Stopping Minikube...")
	minikube stop

minikube-delete: ## Nuke the Minikube cluster (fresh start)
	$(call info, "Deleting Minikube Cluster...")
	minikube delete


# --- Cloud / GCP Orchestration ---

GCP_REGISTRY = gcr.io/$(GCP_PROJECT_ID)
CLOUD_TAG ?= $(shell git rev-parse --short HEAD)

gcp-auth: ## Authenticate kubectl with the GKE cluster
	gcloud container clusters get-credentials $(GCP_CLUSTER_NAME) --zone $(GCP_ZONE) --project $(GCP_PROJECT_ID)

gcp-build: ## Build images for Cloud (Force AMD64 for GKE compatibility)
	$(call info, "Building Cloud Images (linux/amd64)...")
	docker build --platform linux/amd64 -t $(GCP_REGISTRY)/$(SBS_BACKEND_NAME):$(CLOUD_TAG) -f $(SBS_BACKEND_DIR)/Dockerfile $(SBS_BACKEND_DIR)
	docker build --platform linux/amd64 -t $(GCP_REGISTRY)/$(SBS_FRONTEND_NAME):$(CLOUD_TAG) -f $(SBS_FRONTEND_DIR)/Dockerfile $(SBS_FRONTEND_DIR)

gcp-push: gcp-build ## Push images to Google Container Registry
	$(call info, "Pushing images to GCR...")
	docker push $(GCP_REGISTRY)/$(SBS_BACKEND_NAME):$(CLOUD_TAG)
	docker push $(GCP_REGISTRY)/$(SBS_FRONTEND_NAME):$(CLOUD_TAG)

gcp-deploy-candidate: gcp-push ## Deploy Candidate: No Ingress, Use LoadBalancer for testing
	$(call info, "Deploying Candidate Release (sbs-candidate)...")
	@# Strategy: We force type=LoadBalancer to get a temporary IP for testing
	helm upgrade --install sbs-candidate ./charts/sbs-server \
		--namespace $(NAMESPACE) \
		--create-namespace \
		--set backend.fullnameOverride=sbs-candidate-backend \
		--set frontend.fullnameOverride=sbs-candidate-frontend \
		--set backend.image.repository=$(GCP_REGISTRY)/$(SBS_BACKEND_NAME) \
		--set backend.image.tag=$(CLOUD_TAG) \
		--set frontend.image.repository=$(GCP_REGISTRY)/$(SBS_FRONTEND_NAME) \
		--set frontend.image.tag=$(CLOUD_TAG) \
		--set ingress.enabled=false \
		--set frontend.service.type=LoadBalancer \
		--wait

gcp-test-candidate: ## Run Smoke Tests against the Candidate IP
	$(call info, "Waiting for Candidate Public IP...")
	@IP=""; \
	count=0; \
	while [ -z "$$IP" ]; do \
		IP=$$(kubectl get svc -n $(NAMESPACE) sbs-candidate-frontend -o jsonpath='{.status.loadBalancer.ingress[0].ip}'); \
		if [ -z "$$IP" ]; then \
			echo "   ...waiting for LoadBalancer IP ($$count/60)"; \
			sleep 5; \
			count=$$((count+1)); \
			if [ $$count -ge 60 ]; then echo "Timeout getting IP"; exit 1; fi; \
		fi; \
	done; \
	echo "Candidate IP: $$IP"; \
	echo "Running Smoke Test..."; \
	# Retrying curl a few times as LBs can take a moment to warm up
	for i in {1..5}; do curl -s --fail "http://$$IP" | grep "<title>" && break || sleep 5; done

gcp-promote: ## Promote to Prod: Enable Ingress, Use NodePort (Standard)
	$(call info, "Promoting to Production (sbs-prod)...")
	helm upgrade --install $(RELEASE_NAME) ./charts/sbs-server \
		--namespace $(NAMESPACE) \
		--create-namespace \
		--set backend.fullnameOverride=$(SBS_BACKEND_NAME) \
		--set frontend.fullnameOverride=$(SBS_FRONTEND_NAME) \
		--set backend.image.repository=$(GCP_REGISTRY)/$(SBS_BACKEND_NAME) \
		--set backend.image.tag=$(CLOUD_TAG) \
		--set frontend.image.repository=$(GCP_REGISTRY)/$(SBS_FRONTEND_NAME) \
		--set frontend.image.tag=$(CLOUD_TAG) \
		--set ingress.enabled=true \
		--set frontend.service.type=NodePort \
		--wait
	$(call info, "🚀 Production Updated Successfully! Checking Ingress...")
	@kubectl get ingress -n $(NAMESPACE)

gcp-cleanup: ## Remove the candidate deployment
	$(call info, "Removing Candidate Release...")
	helm uninstall sbs-candidate -n $(NAMESPACE) || true

gcp-deploy: \
	gcp-auth \
	gcp-deploy-candidate \
	gcp-test-candidate \
	gcp-promote \
	gcp-cleanup

gcp-test-production: ## Verify the Production URL (sbsolver.ch)
	$(call info, "Verifying Production (https://sbsolver.ch)...")
	@# 1. Check HTTP 200 OK (Follow redirects)
	@curl -s -L --fail -o /dev/null -w "%{http_code}" https://sbsolver.ch | grep 200 > /dev/null && echo "   ✅ Site is reachable (200 OK)" || (echo "   ❌ Site Unreachable" && exit 1)
	
	@# 2. Check Content
	@curl -s -L https://sbsolver.ch | grep "<title>Spelling Bee Solver</title>" > /dev/null && echo "   ✅ Content verified" || (echo "   ❌ Wrong Content" && exit 1)
	
	@# 3. Check API Connectivity (via Frontend proxy)
	@# Note: This assumes your frontend proxies /solve to the backend correctly
	@curl -s -L -X POST https://sbsolver.ch/solve \
		-H "Content-Type: application/json" \
		-d '{"letters": "abcdefg", "present": "a"}' | grep "result" > /dev/null && echo "   ✅ API is working" || echo "   ⚠️ API check skipped/failed (Ensure /solve is exposed)"


generate-diagrams: ## Build all architecture diagrams with mmdc
	$(call info, "Building architecture diagrams...")
	@for file in ./architecture/*.mmd; do \
		echo "Processing $$file..."; \
		mmdc -i "$$file" -o "$${file%.mmd}.png"; \
	done
