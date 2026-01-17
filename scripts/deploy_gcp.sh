#!/bin/bash
# scripts/deploy_gcp.sh
# Automates deployment to Google Kubernetes Engine (GKE)

set -e

# --- Configuration ---
# You can override these via environment variables
PROJECT_ID="${GCP_PROJECT_ID:-sbs-solver-prod}"
CLUSTER_NAME="${GCP_CLUSTER_NAME:-sbs-cluster}"
ZONE="${GCP_ZONE:-us-central1-a}"
IMAGE_NAME="sbs-solver"
RELEASE_NAME="sbs-prod"
NAMESPACE="sbs-namespace"

echo "--- GCP Deployment for $PROJECT_ID ---"

# 1. Prerequisites Check
if ! command -v gcloud &> /dev/null; then echo "Error: gcloud CLI not found."; exit 1; fi
if ! command -v helm &> /dev/null; then echo "Error: helm not found."; exit 1; fi

# 2. Authentication & Context
echo "[1/5] Configuring kubectl context..."
gcloud container clusters get-credentials "$CLUSTER_NAME" --zone "$ZONE" --project "$PROJECT_ID"

# 3. Build & Tag Image
echo "[2/5] Building and Tagging Docker Image..."
# We must use the standard local docker daemon here, not minikube's
docker build -t "$IMAGE_NAME:latest" .
docker tag "$IMAGE_NAME:latest" "gcr.io/$PROJECT_ID/$IMAGE_NAME:latest"

# 4. Push to GCR
echo "[3/5] Pushing to Google Container Registry..."
docker push "gcr.io/$PROJECT_ID/$IMAGE_NAME:latest"

# 5. Deploy with Helm
echo "[4/5] Deploying to GKE..."
# We override the image repository to point to GCR
# We set service.type=LoadBalancer to get a public Static IP
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
