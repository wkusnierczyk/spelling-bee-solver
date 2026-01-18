# SBS Root Orchestrator
NAMESPACE = sbs-namespace
RELEASE_NAME = sbs-prod

.PHONY: help install-infra deploy status logs-backend logs-frontend build-backend run-local deploy-minikube test lint format

help: ## Show help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\\033[36m%-20s\\033[0m %s\\n", $$1, $$2}'

# --- Hygiene & Testing (Delegated to sbs-solver) ---

test: ## Run backend unit and integration tests
	cd sbs-solver && cargo test

lint: ## Run clippy linter on backend
	cd sbs-solver && cargo clippy -- -D warnings

format: ## Format backend code using rustfmt
	cd sbs-solver && cargo fmt

# --- Infrastructure & Cloud ---

install-infra: ## 1. Setup GKE Infrastructure (Ingress, SSL, Config)
	kubectl apply -f infra/

deploy: ## 2. Build and Push to Cloud (GCP)
	./scripts/deploy_gcp.sh

status: ## Check Cloud Health
	kubectl get ingress,pods,managedcertificate -n $(NAMESPACE)

# --- Local Development ---

run-local: ## Run both via local launcher (Native Rust/Node)
	./scripts/run_local.sh

build-backend: ## Build Rust backend locally
	cd sbs-solver && cargo build

deploy-minikube: ## Deploy to local Minikube using local images
	@echo "Updating Helm charts in Minikube..."
	helm upgrade --install $(RELEASE_NAME) ./charts/sbs-server \
		--namespace $(NAMESPACE) \
		--create-namespace \
		--set backend.image.repository=sbs-solver \
		--set backend.image.tag=latest \
		--set frontend.image.repository=sbs-gui \
		--set frontend.image.tag=v2-react \
		--set frontend.service.type=NodePort \
		--set backend.image.pullPolicy=Never \
		--set frontend.image.pullPolicy=Never