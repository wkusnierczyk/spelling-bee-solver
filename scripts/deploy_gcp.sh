#!/bin/bash
# scripts/deploy_gcp.sh
# Automates deployment of Backend and Frontend to GKE

set -e

# --- Configuration ---
# CORRECTED: Default Project ID is 'sbs-solver'
PROJECT_ID="${GCP_PROJECT_ID:-sbs-solver}"
CLUSTER_NAME="${GCP_CLUSTER_NAME:-sbs-cluster}"
ZONE="${GCP_ZONE:-europe-west6-a}"
RELEASE_NAME="sbs-prod"
NAMESPACE="sbs-namespace"

echo "--- GCP Full Stack Deployment ---"
echo "Project: $PROJECT_ID | Cluster: $CLUSTER_NAME | Zone: $ZONE"

# 1. Config Context
gcloud container clusters get-credentials "$CLUSTER_NAME" --zone "$ZONE" --project "$PROJECT_ID"

# 2. Build & Push BACKEND
echo "[Backend] Building..."
docker build -t "sbs-solver:latest" .
docker tag "sbs-solver:latest" "gcr.io/$PROJECT_ID/sbs-solver:latest"
echo "[Backend] Pushing..."
docker push "gcr.io/$PROJECT_ID/sbs-solver:latest"

# 3. Build & Push FRONTEND
echo "[Frontend] Building (this may take a minute for Gradle)..."
cd sbs-gui
docker build -f Dockerfile -t "sbs-gui:latest" .
docker tag "sbs-gui:latest" "gcr.io/$PROJECT_ID/sbs-gui:latest"
echo "[Frontend] Pushing..."
docker push "gcr.io/$PROJECT_ID/sbs-gui:latest"
cd ..

# 4. Deploy Helm Chart
echo "[Helm] Deploying Stack..."
helm upgrade --install "$RELEASE_NAME" ./charts/sbs-server \
  --namespace "$NAMESPACE" \
  --create-namespace \
  --set backend.image.repository="gcr.io/$PROJECT_ID/sbs-solver" \
  --set backend.image.tag="latest" \
  --set frontend.image.repository="gcr.io/$PROJECT_ID/sbs-gui" \
  --set frontend.image.tag="latest" \
  --wait

echo "--- Deployment Complete ---"
echo "Fetching External IPs..."
echo ""
echo "API URL (Backend):"
kubectl get svc -n $NAMESPACE $RELEASE_NAME-backend -o jsonpath='http://{.status.loadBalancer.ingress[0].ip}:{.spec.ports[0].port}'
echo ""
echo "GUI URL (Frontend):"
kubectl get svc -n $NAMESPACE $RELEASE_NAME-frontend -o jsonpath='http://{.status.loadBalancer.ingress[0].ip}:{.spec.ports[0].port}'
echo ""
echo "Note: It may take a minute for the LoadBalancer IPs to be assigned."
