# SBS GCP Helm Chart Deployment Guide

## Prerequisites
- `gcloud` CLI authenticated
- `kubectl` configured for your cluster
- `helm` installed
- Docker images pushed to GCR

## Quick Deploy

### 1. Authenticate with GKE
```bash
gcloud container clusters get-credentials sbs-cluster --zone europe-west6-a --project sbs-solver
```

### 2. Build and Push Images
```bash
# Set the tag (using git short hash)
export CLOUD_TAG=$(git rev-parse --short HEAD)

# Build for linux/amd64 (required for GKE)
docker build --platform linux/amd64 \
  -t gcr.io/sbs-solver/sbs-backend:$CLOUD_TAG \
  -f sbs-backend/Dockerfile sbs-backend

docker build --platform linux/amd64 \
  -t gcr.io/sbs-solver/sbs-frontend:$CLOUD_TAG \
  -f sbs-frontend/Dockerfile sbs-frontend

# Push to GCR
docker push gcr.io/sbs-solver/sbs-backend:$CLOUD_TAG
docker push gcr.io/sbs-solver/sbs-frontend:$CLOUD_TAG
```

### 3. Deploy with Helm
```bash
helm upgrade --install sbs-prod ./charts/gcp \
  --namespace sbs-namespace \
  --create-namespace \
  --set backend.image.tag=$CLOUD_TAG \
  --set frontend.image.tag=$CLOUD_TAG \
  --wait
```

### 4. Verify Deployment
```bash
# Check pods are running
kubectl get pods -n sbs-namespace

# Check services
kubectl get svc -n sbs-namespace

# Check ingress (may take 5-10 minutes to get an IP)
kubectl get ingress -n sbs-namespace

# Check managed certificate status (can take 15-60 minutes to provision)
kubectl get managedcertificate -n sbs-namespace
```

## Certificate Provisioning

The GCP managed certificate can take **15-60 minutes** to provision. You can check its status:

```bash
kubectl describe managedcertificate sbs-managed-cert -n sbs-namespace
```

Status progression:
1. `Provisioning` - Certificate is being created
2. `Active` - Certificate is ready

**Important:** The certificate will only provision successfully if:
- Your domain `sbsolver.ch` has DNS pointing to the static IP `35.244.163.14`
- The ingress is healthy and responding

## DNS Configuration

Make sure your domain points to the static IP:

| Type | Name | Value |
|------|------|-------|
| A | sbsolver.ch | 35.244.163.14 |
| A | www.sbsolver.ch | 35.244.163.14 (optional) |

## Troubleshooting

### Pods not starting
```bash
kubectl describe pod -n sbs-namespace -l app=sbs-backend
kubectl logs -n sbs-namespace -l app=sbs-backend
```

### Ingress not getting IP
```bash
kubectl describe ingress -n sbs-namespace
```

### Backend health check failing
The backend must respond to `GET /health` with HTTP 200. Verify locally:
```bash
curl http://localhost:8080/health
```

### Certificate stuck in Provisioning
- Verify DNS is correctly configured
- Check ingress health: `kubectl describe ingress -n sbs-namespace`
- GCP load balancer health checks must pass first

## Architecture

```
Internet
    │
    ▼
┌─────────────────────────────────────┐
│  GCP Global Load Balancer           │
│  (Static IP: 35.244.163.14)         │
│  (Managed SSL Certificate)          │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  GKE Ingress                        │
│  Routes all traffic to frontend     │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  Frontend Service (NodePort)        │
│  sbs-frontend:80                    │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  Frontend Pods (nginx)              │
│  - Serves static React app          │
│  - Proxies /solve to backend        │
└─────────────────────────────────────┘
    │ (internal proxy)
    ▼
┌─────────────────────────────────────┐
│  Backend Service (ClusterIP)        │
│  sbs-backend:8080                   │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  Backend Pods (Rust API)            │
│  - /health endpoint                 │
│  - /solve endpoint                  │
└─────────────────────────────────────┘
```
