# Spelling Bee Solver (SBS) - Monorepo

**Developer:** Waclaw Kusnierczyk  
**License:** MIT

## 🛠 Project Structure
- `sbs-solver/`: Rust Backend & Trie Engine (Actix-web).
- `sbs-gui/`: React Frontend (Vite + TypeScript).
- `infra/`: GKE Infrastructure manifests (Ingress, SSL, ManagedCerts).
- `charts/`: Helm charts for Kubernetes deployment.
- `scripts/`: Automation for builds and cloud deployments.

---

## 🏗 Local Development (Native)
Use this for the fastest iteration loop.
1. **Backend**: `make build-backend && ./sbs-solver/target/debug/sbs-server`
2. **Frontend**: `cd sbs-gui && npm install && npm run dev`

---

## 📦 Local Kubernetes (Minikube)
Use this to test the full containerized stack locally.

### 1. Environment Setup
```bash
minikube start --driver=docker
eval $(minikube docker-env)  # Point Docker to Minikube's registry
```

### 2. Build & Deploy
```bash
# Build from project root
docker build -t sbs-solver:latest -f sbs-solver/Dockerfile sbs-solver/
docker build -t sbs-gui:v2-react sbs-gui/

# Deploy using the custom Makefile target
make deploy-minikube
```

### 3. Accessing the Service
```bash
# The most reliable way to open the UI
minikube service sbs-prod-frontend -n sbs-namespace
```
*Note: If the page fails to load, run `minikube tunnel` in a separate terminal.*

---

## 🚀 Cloud Deployment (GCP)
Requires `GCP_PROJECT_ID`, `GCP_CLUSTER_NAME`, and `GCP_ZONE` environment variables.

### First-Time Infrastructure Setup
```bash
make install-infra  # Provisions Global IP, SSL Certs, and Ingress
```

### Deploying Updates
```bash
make deploy         # Rebuilds images, pushes to GCR, and updates Helm
```

---

## 💰 Cost Management (Cloud)
To avoid unnecessary Google Cloud charges when the service is not in use:

### Option A: Scale to Zero (Stops Compute Costs)
The Load Balancer remains active (~$18/mo), but you stop paying for CPU/RAM.
```bash
# STOP
kubectl scale deployment sbs-prod-backend --replicas=0 -n sbs-namespace
kubectl scale deployment sbs-prod-frontend --replicas=0 -n sbs-namespace

# START (Back Up)
kubectl scale deployment sbs-prod-backend --replicas=1 -n sbs-namespace
kubectl scale deployment sbs-prod-frontend --replicas=1 -n sbs-namespace
```

### Option B: Delete Ingress (Stops Load Balancer Costs)
Use this for long-term breaks.
```bash
# STOP
kubectl delete -f infra/sbs-ingress.yaml

# START
kubectl apply -f infra/sbs-ingress.yaml
```

---

## 🧪 Testing
- **Backend**: `cd sbs-solver && cargo test`
- **CLI Mode**: `cd sbs-solver && cargo run --bin sbs -- -l letters -p mandatory`