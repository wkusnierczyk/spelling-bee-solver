#!/bin/bash
# scripts/deploy_gcp.sh
set -e

# --- SECURITY CHECK ---
# We do NOT hardcode Project IDs here to prevent leaking secrets in git.
# You must source a .env file or set these variables in your shell.

if [ -z "$GCP_PROJECT_ID" ]; then
    echo "ERROR: GCP_PROJECT_ID is not set."
    echo "Please create a .env file (see .env.template) or export this variable."
    exit 1
fi

if [ -z "$GCP_CLUSTER_NAME" ]; then
    echo "ERROR: GCP_CLUSTER_NAME is not set."
    exit 1
fi

if [ -z "$GCP_ZONE" ]; then
    echo "ERROR: GCP_ZONE is not set."
    exit 1
fi

# Variables
PROJECT_ID="$GCP_PROJECT_ID"
CLUSTER_NAME="$GCP_CLUSTER_NAME"
ZONE="$GCP_ZONE"
RELEASE_NAME="sbs-prod"
NAMESPACE="sbs-namespace"
TAG="v4-react-final"

echo "--- GCP Deployment ---"
echo "Target: $PROJECT_ID | Cluster: $CLUSTER_NAME | Zone: $ZONE"

# 1. Authenticate (fail fast if no perm)
gcloud container clusters get-credentials "$CLUSTER_NAME" --zone "$ZONE" --project "$PROJECT_ID"

# 2. Backend
echo "[Backend] Building..."
docker build --platform linux/amd64 -t "sbs-solver:$TAG" .
docker tag "sbs-solver:$TAG" "gcr.io/$PROJECT_ID/sbs-solver:$TAG"
docker push "gcr.io/$PROJECT_ID/sbs-solver:$TAG"

# 3. Frontend
echo "[Frontend] Building..."
cd sbs-gui
docker build --platform linux/amd64 -t "sbs-gui:$TAG" .
docker tag "sbs-gui:$TAG" "gcr.io/$PROJECT_ID/sbs-gui:$TAG"
docker push "gcr.io/$PROJECT_ID/sbs-gui:$TAG"
cd ..

# 4. Helm
echo "[Helm] Deploying..."
helm upgrade --install "$RELEASE_NAME" ./charts/sbs-server \
  --namespace "$NAMESPACE" \
  --create-namespace \
  --set backend.image.repository="gcr.io/$PROJECT_ID/sbs-solver" \
  --set backend.image.tag="$TAG" \
  --set frontend.image.repository="gcr.io/$PROJECT_ID/sbs-gui" \
  --set frontend.image.tag="$TAG" \
  --set frontend.image.pullPolicy="Always" \
  --wait

echo "--- Success ---"
kubectl get svc -n $NAMESPACE $RELEASE_NAME-frontend -o jsonpath='http://{.status.loadBalancer.ingress[0].ip}:{.spec.ports[0].port}'
echo ""
