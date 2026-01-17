#!/bin/bash
# scripts/deploy_gcp.sh
# Automates deployment to Google Kubernetes Engine (GKE)

set -e

# --- Configuration ---
# UPDATED: Default Project ID is now 'sbs-solver'
PROJECT_ID="${GCP_PROJECT_ID:-sbs-solver}"
CLUSTER_NAME="${GCP_CLUSTER_NAME:-sbs-cluster}"
# UPDATED: Default Zone is Zurich
ZONE="${GCP_ZONE:-europe-west6-a}"

IMAGE_NAME="sbs-solver"
RELEASE_NAME="sbs-prod"
NAMESPACE="sbs-namespace"

echo "--- GCP Deployment for $PROJECT_ID ($ZONE) ---"

if ! command -v gcloud &> /dev/null; then echo "Error: gcloud CLI not found."; exit 1; fi
if ! command -v helm &> /dev/null; then echo "Error: helm not found."; exit 1; fi

echo "[1/5] Configuring kubectl context..."
gcloud container clusters get-credentials "$CLUSTER_NAME" --zone "$ZONE" --project "$PROJECT_ID"

echo "[2/5] Building and Tagging Docker Image..."
# Use local docker daemon
docker build -t "$IMAGE_NAME:latest" .
docker tag "$IMAGE_NAME:latest" "gcr.io/$PROJECT_ID/$IMAGE_NAME:latest"

echo "[3/5] Pushing to Google Container Registry..."
docker push "gcr.io/$PROJECT_ID/$IMAGE_NAME:latest"

echo "[4/5] Deploying to GKE..."
helm upgrade --install "$RELEASE_NAME" ./charts/sbs-server \
  --namespace "$NAMESPACE" \
  --create-namespace \
  --set image.repository="gcr.io/$PROJECT_ID/$IMAGE_NAME" \
  --set image.tag="latest" \
  --set service.type=LoadBalancer \
  --wait

echo "[5/5] Deployment Complete!"
echo "---"
echo "To get your public IP, run:"
echo "  kubectl get svc -n $NAMESPACE $RELEASE_NAME -o jsonpath='{.status.loadBalancer.ingress[0].ip}'"
