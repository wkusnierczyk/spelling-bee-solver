#!/bin/bash
# scripts/deploy_k8s.sh
# Deploys the SBS Server to a Kubernetes cluster using Helm.

set -e

NAMESPACE="sbs-namespace"
RELEASE_NAME="sbs-prod"
CHART_PATH="./charts/sbs-server"

# Check dependencies
if ! command -v helm &> /dev/null; then
    echo "Error: helm is not installed."
    exit 1
fi

if ! command -v kubectl &> /dev/null; then
    echo "Error: kubectl is not installed."
    exit 1
fi

echo "--- Deploying Spelling Bee Solver to K8s ---"

# 1. Create Namespace if it doesn't exist
kubectl create namespace $NAMESPACE --dry-run=client -o yaml | kubectl apply -f -

# 2. (Optional) Build and Push Image
# In a real cloud scenario (AWS EKS, etc.), you must push the image to a registry here.
# Example:
# docker tag sbs-solver:latest myregistry.com/sbs-solver:v1
# docker push myregistry.com/sbs-solver:v1
# For local testing (Minikube/Kind), ensure the image is loaded into the cluster VM.
echo "Note: Ensure 'sbs-solver:latest' is available to your cluster."

# 3. Deploy/Upgrade using Helm
echo "Upgrading Helm release..."
helm upgrade --install $RELEASE_NAME $CHART_PATH \
  --namespace $NAMESPACE \
  --set image.repository=sbs-solver \
  --set image.tag=latest \
  --wait

echo "--- Deployment Successful ---"
echo "To access the service locally (port-forward):"
echo "  kubectl port-forward -n $NAMESPACE svc/$RELEASE_NAME 8080:80"
