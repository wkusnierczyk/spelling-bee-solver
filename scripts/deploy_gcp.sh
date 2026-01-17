#!/bin/bash
# scripts/deploy_gcp.sh
set -e

PROJECT_ID="${GCP_PROJECT_ID:-sbs-solver}"
CLUSTER_NAME="${GCP_CLUSTER_NAME:-sbs-cluster}"
ZONE="${GCP_ZONE:-europe-west6-a}"
RELEASE_NAME="sbs-prod"
NAMESPACE="sbs-namespace"

echo "--- GCP Unified Deployment (Auto-Fix Wasm) ---"
echo "Target: $PROJECT_ID | Cluster: $CLUSTER_NAME | Arch: linux/amd64"

gcloud container clusters get-credentials "$CLUSTER_NAME" --zone "$ZONE" --project "$PROJECT_ID"

echo "[Backend] Building & Pushing..."
docker build --platform linux/amd64 -t "sbs-solver:latest" .
docker tag "sbs-solver:latest" "gcr.io/$PROJECT_ID/sbs-solver:latest"
docker push "gcr.io/$PROJECT_ID/sbs-solver:latest"

echo "[Frontend] 1. Building Locally..."
cd sbs-gui
# Running jsBrowserDistribution will now trigger our custom 'copyWasmToDist' task
./gradlew clean :composeApp:jsBrowserDistribution --no-daemon -Dorg.gradle.jvmargs="-Xmx2g"

echo "[Frontend] 2. Packaging Cloud Image..."
docker build --platform linux/amd64 -f Dockerfile.cloud -t "sbs-gui:latest" .
docker tag "sbs-gui:latest" "gcr.io/$PROJECT_ID/sbs-gui:latest"

echo "[Frontend] 3. Pushing..."
docker push "gcr.io/$PROJECT_ID/sbs-gui:latest"
cd ..

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
echo "Application URL:"
kubectl get svc -n $NAMESPACE $RELEASE_NAME-frontend -o jsonpath='http://{.status.loadBalancer.ingress[0].ip}:{.spec.ports[0].port}'
echo ""
