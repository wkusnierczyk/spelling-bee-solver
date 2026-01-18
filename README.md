# Spelling Bee Solver (SBS)

**Developer:** [Waclaw Kusnierczyk](mailto:waclaw.kusnierczyk@gmail.com)  
**License:** MIT

## Overview
SBS is a high-performance system for solving Spelling Bee puzzles. It consists of:
1.  **Backend (Root)**: Rust-based engine (Actix-web) using a Trie data structure. Source files are in `src/`.
2.  **Frontend (`sbs-gui/`)**: React-based web interface (Vite + TypeScript).
3.  **CLI**: Command-line interface included in the Rust binary.

---

## Prerequisites
* **Rust** (1.75+) & **Cargo**
* **Node.js** (18+) & **npm**
* **Docker** & **Docker Compose**
* **Google Cloud CLI** (For cloud deployment)
* **Minikube** & **Helm** (For local Kubernetes testing)

---

## 1. Local Native Deployment (No Docker)

### Backend (Rust)
Run the API server directly from the project root.
```bash
cargo run -- --dictionary assets/dictionary.txt --port 8080
```

### Frontend (React)
The frontend uses Vite. Navigate to the `sbs-gui` directory to start the development server.
```bash
cd sbs-gui
npm install
npm run dev
```
*The frontend will be available at the URL provided in the terminal (usually [http://localhost:5173](http://localhost:5173)).*

---

## 2. Local Docker Deployment (Docker Compose)
Tests the Nginx proxy to Rust bridge. This is the most accurate local representation of the production environment.

```bash
docker-compose up --build
```
*Access the application at [http://localhost](http://localhost).*

---

## 3. Local Kubernetes Deployment (Minikube)
1.  **Start Minikube and configure shell:**
    ```bash
    minikube start
    eval $(minikube docker-env)
    ```
2.  **Build Images:**
    ```bash
    # Backend image from root
    docker build -t sbs-solver:local .
    
    # Frontend image from sbs-gui
    docker build -t sbs-gui:local ./sbs-gui
    ```
3.  **Deploy via Helm:**
    ```bash
    ./scripts/deploy_k8s.sh
    ```

---

## 4. Cloud Deployment (GCP/GKE)

### Infrastructure Setup
```bash
gcloud container clusters get-credentials sbs-cluster --zone europe-west6-a
```

### Deployment
```bash
./scripts/deploy_gcp.sh
```

---

## Development & Maintenance

### Backend (`make`)
The `Makefile` at the root manages Rust tasks.
* `make test`: Run unit/integration tests.
* `make format`: Apply `rustfmt`.
* `make lint`: Run `clippy`.

### Frontend (`npm`)
All frontend commands must be run inside the `sbs-gui` directory.
* `npm run dev`: Start Vite development server.
* `npm run build`: Create production assets in the `dist/` folder.
* `npm run preview`: Locally preview the production build.