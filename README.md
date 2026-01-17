
# Spelling Bee Solver (SBS)

## Overview
SBS is a high-performance system for solving Spelling Bee puzzles. It generates all valid words from a dictionary given a set of allowed letters and a center (obligatory) letter.

**Developer:** [Waclaw Kusnierczyk](mailto:waclaw.kusnierczyk@gmail.com)  
**License:** MIT

## Architecture
1.  **Core Library (`sbs-lib`)**: Rust-based engine using a Trie data structure.
2.  **CLI (`sbs`)**: Command-line interface.
3.  **Service API (`sbs-server`)**: REST API server (Actix-web), deployable via Docker/K8s.
4.  **GUI (`sbs-gui`)**: Kotlin Multiplatform Desktop client.

---

## 1. Rust Backend (CLI & Service)

### Setup & Build
```bash
make setup      # Download dictionary
make build      # Build debug
make release    # Build release
make test       # Run tests
```

### Usage: CLI
```bash
./target/release/sbs --letters "abcdefg" --present "a"
./target/release/sbs --about
```

### Usage: Local Service
```bash
make run-server
# API: POST http://localhost:8080/solve
```

---

## 2. Kubernetes Deployment

### Option A: Local (Minikube)
Ideal for development without cloud costs.

1.  **Start Minikube**:
    ```bash
    minikube start
    eval $(minikube docker-env)  # Use Minikube's Docker daemon
    ```
2.  **Build & Deploy**:
    ```bash
    make docker-build
    ./scripts/deploy_k8s.sh
    ```
3.  **Access**:
    Minikube exposes the internal service via a tunnel:
    ```bash
    minikube service sbs-prod -n sbs-namespace --url
    ```
    Use the URL provided by this command (e.g., `http://127.0.0.1:52635`) in your browser or curl.

### Option B: Cloud (Google Cloud Platform)
Automated deployment script included.

1.  **Prerequisites**:
    * `gcloud` CLI installed and authenticated.
    * A GKE cluster created.
2.  **Deploy**:
    ```bash
    # Set your project details
    export GCP_PROJECT_ID="your-project-id"
    export GCP_CLUSTER_NAME="sbs-cluster"
    export GCP_ZONE="us-central1-a"

    # Run the deployment script
    ./scripts/deploy_gcp.sh
    ```
3.  **Access**:
    The script will deploy a LoadBalancer. Get the external IP:
    ```bash
    kubectl get svc -n sbs-namespace
    ```

---

## 3. GUI Client (Kotlin)

Located in `sbs-gui/`.

1.  **Run**:
    ```bash
    cd sbs-gui
    ./gradlew run
    ```
    *(If connecting to Minikube, you may need to update the URL in `App.kt` to match the `minikube service` URL)*.

---

## CI/CD
* GitHub Actions (`.github/workflows/ci.yml`) validates Rust code and builds Docker images on push.

