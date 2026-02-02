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

.PHONY: \
	setup-dictionary \
	test \
	lint \
	format \
	help \
	test \
	lint \
	format \
	build-backend \
	install-backend \
	run-backend \
	build-frontend \
	run-frontend \
	build-cli \
	install-cli \
	start-local \
	stop-local \
	status \
	deploy-cloud \
	build-architecture \
	version \
	version-set \
	bump-patch \
	bump-minor \
	bump-major \
	setup-android \
	build-android \
	clean-android \
	setup-mobile \
	build-mobile \
	check-mobile \
	run-mobile \
	clean-mobile \
	test-mobile \
	clean-docker \
	start-docker stop-docker test-docker open-docker \
	docker-start docker-stop docker-test docker-clean docker-open \
	start-compose stop-compose test-compose open-compose \
	compose-start compose-stop compose-test compose-open \
	start-minikube build-minikube deploy-minikube test-minikube open-minikube clean-minikube stop-minikube delete-minikube \
	minikube-start minikube-build minikube-deploy minikube-test minikube-open minikube-clean minikube-stop minikube-delete \
	auth-gcp build-gcp push-gcp deploy-gcp-candidate test-gcp-candidate promote-gcp-candidate cleanup-gcp-candidate deploy-gcp \
	gcp-auth gcp-build gcp-push gcp-deploy-candidate gcp-test-candidate gcp-promote-candidate gcp-cleanup-candidate gcp-deploy \
	status-gcp logs-gcp-backend logs-gcp-frontend rollback-gcp destroy-gcp hibernate-gcp wake-gcp \
	gcp-status gcp-logs-backend gcp-logs-frontend gcp-rollback gcp-destroy gcp-hibernate gcp-wake

help: ## Show help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'


# --- Data Management ---

setup-dictionary: ## Force download of the dictionary (overwrites if exists)
	$(call info, "Downloading fresh dictionary to $(SBS_DICT)...")
	@mkdir -p $(dir $(SBS_DICT))
	@curl -L -o $(SBS_DICT) $(DICT_URL) || (rm -f $(SBS_DICT) && exit 1)


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

check: format lint test ## Run format, lint, and test

# --- Version Management ---

VERSION = $(shell sed -n 's/^version = "\(.*\)"/\1/p' sbs-backend/Cargo.toml | head -1)

version: ## Print current version
	@echo $(VERSION)

version-set: ## Set version across all files: make version-set V=x.y.z
	@test -n "$(V)" || (echo "Usage: make version-set V=x.y.z" && exit 1)
	@sed -i '' 's/^version = ".*"/version = "$(V)"/' sbs-backend/Cargo.toml
	@sed -i '' 's/^version = ".*"/version = "$(V)"/' sbs-ffi/Cargo.toml
	@sed -i '' 's/"version": ".*"/"version": "$(V)"/' sbs-frontend/package.json
	@sed -i '' 's/"version": ".*"/"version": "$(V)"/' sbs-mobile/package.json
	@sed -i '' 's/versionName ".*"/versionName "$(V)"/' sbs-mobile/android/app/build.gradle
	@sed -i '' 's/^appVersion: ".*"/appVersion: "$(V)"/' charts/minikube/Chart.yaml
	@sed -i '' 's/^appVersion: ".*"/appVersion: "$(V)"/' charts/gcp/Chart.yaml
	@sed -i '' 's/├─ version:   .*/├─ version:   $(V)/' README.md
	$(call info, "Version set to $(V)")

bump-patch: ## Bump patch version (0.1.2 → 0.1.3)
	@$(MAKE) version-set V=$(shell echo $(VERSION) | awk -F. '{printf "%d.%d.%d", $$1, $$2, $$3+1}')

bump-minor: ## Bump minor version (0.1.2 → 0.2.0)
	@$(MAKE) version-set V=$(shell echo $(VERSION) | awk -F. '{printf "%d.%d.0", $$1, $$2+1}')

bump-major: ## Bump major version (0.1.2 → 1.0.0)
	@$(MAKE) version-set V=$(shell echo $(VERSION) | awk -F. '{printf "%d.0.0", $$1+1}')


setup-hooks: ## Configure git to use the repo's .githooks directory
	$(call info, "Setting git hooks path to .githooks/...")
	@git config core.hooksPath .githooks
	$(call info, "Git hooks configured.")


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

start-docker: setup-dictionary build-backend-image build-frontend-image start-backend-container start-frontend-container
	@sleep 2
	@# @make test-frontend-container
	$(call info, "Stack running!")

test-docker: setup-dictionary build-backend-image build-frontend-image start-backend-container start-frontend-container
	$(call info, "Verifying Full Docker Stack...")
	@make fullstack-smoke-test
	$(call info, "Full Docker Stack is verified and running!")

stop-docker: stop-frontend-container stop-backend-container
	$(call info, "Stack stopped.")

clean-docker: ## Remove all Docker containers (manual and Compose), images, and build cache
	$(call info, "Stopping and removing Docker containers...")
	@docker stop $(SBS_BACKEND_NAME) $(SBS_FRONTEND_NAME) 2>/dev/null || true
	@docker rm $(SBS_BACKEND_NAME) $(SBS_FRONTEND_NAME) 2>/dev/null || true
	@docker compose down --rmi local 2>/dev/null || true
	@docker builder prune -f
	$(call info, "Docker clean complete.")

open-docker: ## Open the Docker stack frontend URL in the default browser
	$(call info, "Opening Frontend at http://localhost:5173...")
	open http://localhost:5173

# Aliases
docker-start: start-docker ## Alias for start-docker
docker-test: test-docker ## Alias for test-docker
docker-stop: stop-docker ## Alias for stop-docker
docker-clean: clean-docker ## Alias for clean-docker
docker-open: open-docker ## Alias for open-docker


# --- Docker Compose Orchestration ---

start-compose: setup-dictionary
	$(call info, "Starting stack with Docker Compose...")
	@docker compose up -d --build
	$(call info, "Stack is running.")
	$(call info, "Frontend: http://localhost:5173")
	$(call info, "Backend:  http://localhost:8080")

test-compose:
	$(call info, "Verifying Docker Compose stack...")
	@make fullstack-smoke-test

stop-compose:
	$(call info, "Stopping Docker Compose stack...")
	@docker compose down

open-compose: ## Open the Docker Compose stack frontend URL in the default browser
	$(call info, "Opening Frontend at http://localhost:5173...")
	open http://localhost:5173

# Aliases
compose-start: start-compose ## Alias for start-compose
compose-test: test-compose ## Alias for test-compose
compose-stop: stop-compose ## Alias for stop-compose
compose-open: open-compose ## Alias for open-compose



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

ci-minikube: ## Run Minikube workflow locally
	$(call info, "Running Minikube Workflow locally...")
	act -W .github/workflows/minikube.yml \
		--container-architecture linux/amd64 \
		--platform ubuntu-latest=catthehacker/ubuntu:act-latest \
		--bind \
		--reuse

ci-gcp: ## Run GCP workflow locally
	$(call info, "Running GCP Workflow locally...")
	act -W .github/workflows/gcp.yml \
		--container-architecture linux/amd64 \
		--platform ubuntu-latest=catthehacker/ubuntu:act-latest \
		--bind \
		--reuse

ci-all: ## Run all workflows locally
	act --container-architecture linux/amd64


# --- Minikube Targets ---

start-minikube:
	$(call info, "Starting Minikube...")
	minikube start --driver=docker

build-minikube: setup-dictionary
	$(call info, "Pointing Docker to Minikube...")
	@eval $$(minikube docker-env) && \
		docker build -t $(SBS_BACKEND_NAME):$(DOCKER_TAG) -f $(SBS_BACKEND_DIR)/Dockerfile $(SBS_BACKEND_DIR) && \
		docker build -t $(SBS_FRONTEND_NAME):$(DOCKER_TAG) -f $(SBS_FRONTEND_DIR)/Dockerfile $(SBS_FRONTEND_DIR)
	$(call info, "Images built inside Minikube registry.")

deploy-minikube: build-minikube ## Deploy charts to Minikube
	$(call info, "Deploying Helm Release $(RELEASE_NAME)...")
	helm upgrade --install $(RELEASE_NAME) ./charts/minikube \
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
test-minikube: ## Verify the Minikube deployment (Wait + Curl)
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

open-minikube: ## Open the frontend URL in the default browser
	$(call info, "Opening Frontend Service...")
	minikube service $(SBS_FRONTEND_NAME) -n $(NAMESPACE)

clean-minikube: ## Remove the Helm release (leaves cluster running)
	$(call info, "Uninstalling Release $(RELEASE_NAME)...")
	helm uninstall $(RELEASE_NAME) -n $(NAMESPACE) || true

stop-minikube: ## Stop the Minikube cluster (saves resources)
	$(call info, "Stopping Minikube...")
	minikube stop

delete-minikube: ## Nuke the Minikube cluster (fresh start)
	$(call info, "Deleting Minikube Cluster...")
	minikube delete

# Aliases (reverse compatibility)
minikube-start: start-minikube ## Alias for start-minikube
minikube-build: build-minikube ## Alias for build-minikube
minikube-deploy: deploy-minikube ## Alias for deploy-minikube
minikube-test: test-minikube ## Alias for test-minikube
minikube-open: open-minikube ## Alias for open-minikube
minikube-clean: clean-minikube ## Alias for clean-minikube
minikube-stop: stop-minikube ## Alias for stop-minikube
minikube-delete: delete-minikube ## Alias for delete-minikube


# --- Cloud / GCP Orchestration ---

GCP_REGISTRY = gcr.io/$(GCP_PROJECT_ID)
CLOUD_TAG ?= $(shell git rev-parse --short HEAD)
NAMESPACE = sbs-namespace
STAGING_NAMESPACE = sbs-staging
RELEASE_NAME = sbs-prod
STAGING_RELEASE_NAME = sbs-staging

auth-gcp: ## Authenticate kubectl with the GKE cluster
	gcloud container clusters get-credentials $(GCP_CLUSTER_NAME) --zone $(GCP_ZONE) --project $(GCP_PROJECT_ID)

build-gcp: ## Build images for Cloud (Force AMD64 for GKE compatibility)
	$(call info, "Building Cloud Images (linux/amd64) with tag $(CLOUD_TAG)...")
	docker build --platform linux/amd64 -t $(GCP_REGISTRY)/$(SBS_BACKEND_NAME):$(CLOUD_TAG) -f $(SBS_BACKEND_DIR)/Dockerfile $(SBS_BACKEND_DIR)
	docker build --platform linux/amd64 -t $(GCP_REGISTRY)/$(SBS_FRONTEND_NAME):$(CLOUD_TAG) -f $(SBS_FRONTEND_DIR)/Dockerfile $(SBS_FRONTEND_DIR)

push-gcp: build-gcp ## Push images to Google Container Registry
	$(call info, "Pushing images to GCR...")
	docker push $(GCP_REGISTRY)/$(SBS_BACKEND_NAME):$(CLOUD_TAG)
	docker push $(GCP_REGISTRY)/$(SBS_FRONTEND_NAME):$(CLOUD_TAG)

deploy-gcp-candidate: push-gcp ## Deploy to staging namespace for testing
	$(call info, "Deploying candidate to staging namespace...")
	helm upgrade --install $(STAGING_RELEASE_NAME) ./charts/gcp \
		--namespace $(STAGING_NAMESPACE) \
		--create-namespace \
		--set backend.image.tag=$(CLOUD_TAG) \
		--set frontend.image.tag=$(CLOUD_TAG) \
		--set ingress.enabled=false \
		--set certificate.enabled=false \
		--set frontend.service.type=LoadBalancer \
		--wait --timeout=5m
	$(call info, "Candidate deployed to staging namespace")

test-gcp-candidate: ## Test the candidate deployment in staging
	$(call info, "Waiting for staging LoadBalancer IP...")
	@IP=""; \
	count=0; \
	while [ -z "$$IP" ]; do \
		IP=$$(kubectl get svc -n $(STAGING_NAMESPACE) sbs-frontend -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null); \
		if [ -z "$$IP" ]; then \
			echo "   ...waiting for LoadBalancer IP ($$count/60)"; \
			sleep 5; \
			count=$$((count+1)); \
			if [ $$count -ge 60 ]; then echo "Timeout getting IP"; exit 1; fi; \
		fi; \
	done; \
	echo "Staging IP: $$IP"; \
	echo ""; \
	echo "1. Testing Frontend Static Content..."; \
	for i in 1 2 3 4 5; do \
		if curl -s --fail "http://$$IP" | grep -q "<title>"; then \
			echo "   ✅ Frontend serving content"; \
			break; \
		fi; \
		echo "   ...retrying ($$i/5)"; \
		sleep 5; \
	done; \
	echo ""; \
	echo "2. Testing Backend API via Frontend Proxy..."; \
	RESULT=$$(curl -s -X POST "http://$$IP/solve" \
		-H "Content-Type: application/json" \
		-d '{"letters": "pelniga", "present": "a"}'); \
	if echo "$$RESULT" | grep -q "appeal\|alpine"; then \
		echo "   ✅ Backend API responding correctly"; \
	else \
		echo "   ❌ Backend API test failed"; \
		echo "   Response: $$RESULT"; \
		exit 1; \
	fi; \
	echo ""; \
	echo "✅ All candidate tests passed!"

promote-gcp-candidate: ## Promote candidate to production (rolling update)
	$(call info, "Promoting candidate to production...")
	helm upgrade --install $(RELEASE_NAME) ./charts/gcp \
		--namespace $(NAMESPACE) \
		--create-namespace \
		--set backend.image.tag=$(CLOUD_TAG) \
		--set frontend.image.tag=$(CLOUD_TAG) \
		--set ingress.enabled=true \
		--set certificate.enabled=true \
		--set frontend.service.type=NodePort \
		--wait --timeout=5m
	$(call info, "Production updated with rolling deployment")
	@kubectl get ingress -n $(NAMESPACE)

cleanup-gcp-candidate: ## Remove the staging deployment
	$(call info, "Cleaning up staging namespace...")
	helm uninstall $(STAGING_RELEASE_NAME) -n $(STAGING_NAMESPACE) 2>/dev/null || true
	kubectl delete namespace $(STAGING_NAMESPACE) --ignore-not-found=true --wait=true
	$(call info, "Staging cleaned up")

deploy-gcp: auth-gcp deploy-gcp-candidate test-gcp-candidate promote-gcp-candidate cleanup-gcp-candidate ## Full deployment pipeline
	$(call info, "🚀 Deployment complete!")

status-gcp: ## Show current deployment status
	$(call info, "Production namespace ($(NAMESPACE)):")
	@kubectl get pods,svc,ingress -n $(NAMESPACE) 2>/dev/null || echo "   No resources found"
	@echo ""
	$(call info, "Staging namespace ($(STAGING_NAMESPACE)):")
	@kubectl get pods,svc -n $(STAGING_NAMESPACE) 2>/dev/null || echo "   No resources found"
	@echo ""
	$(call info, "Managed Certificate:")
	@kubectl get managedcertificate -n $(NAMESPACE) 2>/dev/null || echo "   No certificate found"

logs-gcp-backend: ## Tail backend logs from production
	kubectl logs -f -n $(NAMESPACE) -l app=sbs-backend --all-containers

logs-gcp-frontend: ## Tail frontend logs from production
	kubectl logs -f -n $(NAMESPACE) -l app=sbs-frontend --all-containers

rollback-gcp: ## Rollback to previous production release
	$(call info, "Rolling back production deployment...")
	helm rollback $(RELEASE_NAME) -n $(NAMESPACE)
	$(call info, "Rollback complete")

destroy-gcp: ## Remove all GCP deployments (DANGEROUS)
	$(call info, "Destroying all deployments...")
	helm uninstall $(RELEASE_NAME) -n $(NAMESPACE) 2>/dev/null || true
	helm uninstall $(STAGING_RELEASE_NAME) -n $(STAGING_NAMESPACE) 2>/dev/null || true
	kubectl delete namespace $(STAGING_NAMESPACE) --ignore-not-found=true
	$(call info, "All deployments removed")


# --- Cloud Cost Management ---

hibernate-gcp: ## Scale deployments to zero (stops compute costs, keeps LB)
	$(call info, "Scaling deployments to zero...")
	kubectl scale deployment sbs-backend --replicas=0 -n $(NAMESPACE)
	kubectl scale deployment sbs-frontend --replicas=0 -n $(NAMESPACE)
	$(call info, "Cluster hibernated. Run 'make wake-gcp' to restore.")

wake-gcp: ## Restore deployments from hibernation
	$(call info, "Waking up deployments...")
	kubectl scale deployment sbs-backend --replicas=1 -n $(NAMESPACE)
	kubectl scale deployment sbs-frontend --replicas=1 -n $(NAMESPACE)
	kubectl rollout status deployment/sbs-backend -n $(NAMESPACE) --timeout=120s
	kubectl rollout status deployment/sbs-frontend -n $(NAMESPACE) --timeout=120s
	$(call info, "Cluster is awake and ready.")

# Aliases (reverse compatibility)
gcp-auth: auth-gcp ## Alias for auth-gcp
gcp-build: build-gcp ## Alias for build-gcp
gcp-push: push-gcp ## Alias for push-gcp
gcp-deploy-candidate: deploy-gcp-candidate ## Alias for deploy-gcp-candidate
gcp-test-candidate: test-gcp-candidate ## Alias for test-gcp-candidate
gcp-promote-candidate: promote-gcp-candidate ## Alias for promote-gcp-candidate
gcp-cleanup-candidate: cleanup-gcp-candidate ## Alias for cleanup-gcp-candidate
gcp-deploy: deploy-gcp ## Alias for deploy-gcp
gcp-status: status-gcp ## Alias for status-gcp
gcp-logs-backend: logs-gcp-backend ## Alias for logs-gcp-backend
gcp-logs-frontend: logs-gcp-frontend ## Alias for logs-gcp-frontend
gcp-rollback: rollback-gcp ## Alias for rollback-gcp
gcp-destroy: destroy-gcp ## Alias for destroy-gcp
gcp-hibernate: hibernate-gcp ## Alias for hibernate-gcp
gcp-wake: wake-gcp ## Alias for wake-gcp


# --- Architecture Diagram Generation ---

# --- Android Cross-Compilation ---

ANDROID_JNILIBS = sbs-mobile/android/app/src/main/jniLibs

setup-android: ## Install Android cross-compilation toolchains
	$(call info, "Adding Android targets...")
	rustup target add aarch64-linux-android x86_64-linux-android armv7-linux-androideabi
	$(call info, "Installing cargo-ndk...")
	cargo install cargo-ndk
	$(call info, "Android setup complete.")

ANDROID_NDK_VERSION ?= 27.1.12297006
ANDROID_NDK_HOME_OVERRIDE = $(ANDROID_HOME)/ndk/$(ANDROID_NDK_VERSION)

ANDROID_JNI_SRC = sbs-mobile/android/app/src/main/jni/sbs_jni.c
NDK_TOOLCHAIN = $(ANDROID_NDK_HOME_OVERRIDE)/toolchains/llvm/prebuilt/darwin-x86_64/bin

build-android: ## Cross-compile sbs-ffi and JNI bridge for Android (arm64, x86_64, armv7)
	$(call info, "Building sbs-ffi for Android targets...")
	cd sbs-ffi && ANDROID_NDK_HOME=$(ANDROID_NDK_HOME_OVERRIDE) cargo ndk -t arm64-v8a -t x86_64 -t armeabi-v7a -P 24 -o ../$(ANDROID_JNILIBS) build --release
	$(call info, "Building JNI bridge for Android targets...")
	$(NDK_TOOLCHAIN)/aarch64-linux-android24-clang -shared -fPIC -o $(ANDROID_JNILIBS)/arm64-v8a/libsbs_jni.so $(ANDROID_JNI_SRC) \
		-I$(ANDROID_NDK_HOME_OVERRIDE)/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/include \
		-L$(ANDROID_JNILIBS)/arm64-v8a -lsbs_ffi -llog
	$(NDK_TOOLCHAIN)/x86_64-linux-android24-clang -shared -fPIC -o $(ANDROID_JNILIBS)/x86_64/libsbs_jni.so $(ANDROID_JNI_SRC) \
		-I$(ANDROID_NDK_HOME_OVERRIDE)/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/include \
		-L$(ANDROID_JNILIBS)/x86_64 -lsbs_ffi -llog
	$(NDK_TOOLCHAIN)/armv7a-linux-androideabi24-clang -shared -fPIC -o $(ANDROID_JNILIBS)/armeabi-v7a/libsbs_jni.so $(ANDROID_JNI_SRC) \
		-I$(ANDROID_NDK_HOME_OVERRIDE)/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/include \
		-L$(ANDROID_JNILIBS)/armeabi-v7a -lsbs_ffi -llog
	$(call info, "Android build complete. Output in $(ANDROID_JNILIBS)")

clean-android: ## Remove Android JNI libraries
	$(call info, "Cleaning Android JNI libraries...")
	rm -rf $(ANDROID_JNILIBS)
	$(call info, "Android clean complete.")


# --- React Native Mobile ---

SBS_MOBILE_DIR = sbs-mobile

setup-mobile: setup-android ## Install all mobile dependencies (Rust targets, cargo-ndk, npm)
	$(call info, "Installing React Native dependencies...")
	cd $(SBS_MOBILE_DIR) && npm install
	$(call info, "Mobile setup complete. Ensure ANDROID_HOME is set in .env or environment.")

build-mobile: ## Build the Android debug APK
	$(call info, "Building Android debug APK...")
	cd $(SBS_MOBILE_DIR)/android && ./gradlew assembleDebug
	$(call info, "APK built at $(SBS_MOBILE_DIR)/android/app/build/outputs/apk/debug/")

run-mobile: ## Run the React Native app on a connected Android device/emulator
	$(call info, "Starting React Native for Android...")
	cd $(SBS_MOBILE_DIR) && npx react-native run-android

check-mobile: build-android build-mobile ## Verify mobile builds (requires Android SDK + NDK)

test-mobile: ## Run mobile unit tests
	$(call info, "Running mobile tests...")
	cd $(SBS_MOBILE_DIR) && npx jest --ci --forceExit

clean-mobile: ## Clean mobile build artifacts (Gradle, bundled JS)
	$(call info, "Cleaning mobile build artifacts...")
	cd $(SBS_MOBILE_DIR)/android && ./gradlew clean
	rm -rf $(SBS_MOBILE_DIR)/android/app/build
	rm -f $(SBS_MOBILE_DIR)/android/app/src/main/assets/index.android.bundle
	$(call info, "Mobile clean complete.")


generate-diagrams: ## Build all architecture diagrams with mmdc
	$(call info, "Building architecture diagrams...")
	@for file in ./architecture/*.mmd; do \
		echo "Processing $$file..."; \
		mmdc -i "$$file" -o "$${file%.mmd}.png"; \
	done
