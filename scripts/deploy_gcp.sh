#!/bin/bash
# scripts/deploy_gcp.sh
# Automates deployment to Google Kubernetes Engine (GKE)
# FORCES linux/amd64 architecture to match Standard GKE Nodes

set -e

PROJECT_ID="${GCP_PROJECT_ID:-sbs-solver}"
CLUSTER_NAME="${GCP_CLUSTER_NAME:-sbs-cluster}"
ZONE="${GCP_ZONE:-europe-west6-a}"
RELEASE_NAME="sbs-prod"
NAMESPACE="sbs-namespace"

echo "--- GCP Unified Deployment ---"
echo "Target: $PROJECT_ID | Cluster: $CLUSTER_NAME | Arch: linux/amd64"

# 1. Config Context
gcloud container clusters get-credentials "$CLUSTER_NAME" --zone "$ZONE" --project "$PROJECT_ID"

# 2. Build & Push BACKEND (Explicitly targeting Cloud Architecture)
echo "[Backend] Building for linux/amd64..."
docker build --platform linux/amd64 -t "sbs-solver:latest" .
docker tag "sbs-solver:latest" "gcr.io/$PROJECT_ID/sbs-solver:latest"
echo "[Backend] Pushing..."
docker push "gcr.io/$PROJECT_ID/sbs-solver:latest"

# 3. Build & Push FRONTEND (Explicitly targeting Cloud Architecture)
echo "[Frontend] Building for linux/amd64..."
cd sbs-gui
docker build --platform linux/amd64 -f Dockerfile -t "sbs-gui:latest" .
docker tag "sbs-gui:latest" "gcr.io/$PROJECT_ID/sbs-gui:latest"
echo "[Frontend] Pushing..."
docker push "gcr.io/$PROJECT_ID/sbs-gui:latest"
cd ..

# 4. Deploy Helm Chart
echo "[Helm] Deploying Stack..."
# --wait ensures we pause until the architecture mismatch is resolved and pods are Running
helm upgrade --install "$RELEASE_NAME" ./charts/sbs-server \
  --namespace "$NAMESPACE" \
  --create-namespace \
  --set backend.image.repository="gcr.io/$PROJECT_ID/sbs-solver" \
  --set backend.image.tag="latest" \
  --set frontend.image.repository="gcr.io/$PROJECT_ID/sbs-gui" \
  --set frontend.image.tag="latest" \
  --wait

echo "--- Deployment Complete ---"
echo "Application URL:"
kubectl get svc -n $NAMESPACE $RELEASE_NAME-frontend -o jsonpath='http://{.status.loadBalancer.ingress[0].ip}:{.spec.ports[0].port}'
echo ""
