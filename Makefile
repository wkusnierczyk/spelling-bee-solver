# SBS Root Orchestrator
NAMESPACE = sbs-namespace
RELEASE_NAME = sbs-prod

.PHONY: help install-infra deploy status logs-backend logs-frontend build-backend run-local

help: ## Show help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

install-infra: ## 1. Setup GKE (Ingress, SSL)
	kubectl apply -f infra/

deploy: ## 2. Build and Push to Cloud
	./scripts/deploy_gcp.sh

run-local: ## Run both via local launcher
	./scripts/run_local.sh

status: ## Check Cloud Health
	kubectl get ingress,pods,managedcertificate -n $(NAMESPACE)

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