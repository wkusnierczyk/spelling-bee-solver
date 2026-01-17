# Spelling Bee Solver (SBS)

**Developer:** [Waclaw Kusnierczyk](mailto:waclaw.kusnierczyk@gmail.com)  
**License:** MIT

## Overview
SBS is a high-performance system for solving Spelling Bee puzzles. It consists of:
1.  **Core Library (`sbs-lib`)**: Rust-based engine using a Trie data structure.
2.  **CLI (`sbs`)**: Command-line interface.
3.  **Service API (`sbs-server`)**: REST API server (Actix-web).
4.  **GUI (`sbs-gui`)**: Kotlin Multiplatform Desktop client.

---

## Prerequisites
* **Rust** (1.75+) & **Cargo**
* **JDK** (17+)
* **Docker** (Optional, for containerization)
* **Google Cloud CLI** (Optional, for cloud deployment)
* **Minikube** & **Helm** (Optional, for Kubernetes)

### Initial Setup
Before running any component, download the dictionary data:
```bash
make setup
```

---

## 1. Using the Core Library (Rust)
To use the solver logic directly in your own Rust code:

```rust
use sbs::{Config, Dictionary, Solver};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load dictionary (shared reference)
    let dictionary = Dictionary::from_file("data/dictionary.txt")?;

    // 2. Configure puzzle
    let config = Config::new()
        .with_letters("abcdefg")
        .with_present("a");

    // 3. Solve
    let solver = Solver::new(config);
    let words = solver.solve(&dictionary)?;

    for word in words {
        println!("{}", word);
    }
    Ok(())
}
```

---

## 2. Using the CLI (Local Binary)
Run the tool directly from the terminal without starting a server.

**Build:**
```bash
make release
```

**Usage:**
```bash
# Basic Usage
./target/release/sbs --letters "abcdefg" --present "a"

# Write to file
./target/release/sbs -l "abcdefg" -p "a" -o output.txt

# Display Developer Info
./target/release/sbs --about

# Help
./target/release/sbs --help
```

---

## 3. Deployment Scenario A: Local Native (No Docker)
Run the API server and GUI directly on your host machine.

1.  **Start the Server:**
    ```bash
    make run-server
    # Server listens on [http://0.0.0.0:8080](http://0.0.0.0:8080)
    ```
2.  **Start the GUI:**
    Open a new terminal:
    ```bash
    cd sbs-gui
    ./gradlew run
    ```

---

## 4. Deployment Scenario B: Local Docker (Docker Compose)
Run the service in a container, matching the production environment.

1.  **Build & Run:**
    ```bash
    make docker-build
    make docker-run
    ```
2.  **Access:**
    The service is exposed on `localhost:8080`.
3.  **Start the GUI:**
    ```bash
    cd sbs-gui
    ./gradlew run
    ```
4.  **Stop:**
    ```bash
    make docker-stop
    ```

---

## 5. Deployment Scenario C: Local Kubernetes (Minikube)
Simulate a cluster environment locally.

1.  **Start Minikube:**
    ```bash
    minikube start
    # Point shell to Minikube's Docker daemon
    eval $(minikube docker-env)
    ```
2.  **Build Image (inside Minikube):**
    ```bash
    make docker-build
    ```
3.  **Deploy via Helm:**
    ```bash
    ./scripts/deploy_k8s.sh
    ```
4.  **Connect GUI:**
    The GUI expects `localhost:8080`. Use port-forwarding to bridge the gap:
    ```bash
    kubectl port-forward -n sbs-namespace svc/sbs-prod 8080:80
    ```
5.  **Run GUI:**
    ```bash
    cd sbs-gui
    ./gradlew run
    ```

---

## 6. Deployment Scenario D: Cloud (Google Cloud Platform)

### A. GCP Infrastructure Setup
Perform these steps once to create the environment.

1.  **Install Tools:**
    * Install [Google Cloud SDK](https://cloud.google.com/sdk/docs/install).
    * Install `kubectl`.
    * **CRITICAL:** Install the Auth Plugin:
        ```bash
        gcloud components install gke-gcloud-auth-plugin
        ```

2.  **Authenticate & Project Setup:**
    ```bash
    gcloud auth login
    
    export PROJECT_ID=sbs-solver-prod
    gcloud projects create $PROJECT_ID --name="SBS Solver"
    gcloud config set project $PROJECT_ID
    ```

3.  **Enable APIs:**
    ```bash
    gcloud services enable container.googleapis.com artifactregistry.googleapis.com
    ```

4.  **Create Cluster:**
    Creating a cluster in Zurich (`europe-west6`) with 2 nodes.
    ```bash
    gcloud container clusters create sbs-cluster \
        --zone europe-west6-a \
        --num-nodes 2 \
        --machine-type e2-small
    ```
    *Note: If `kubectl` commands fail later, verify credentials:*
    ```bash
    gcloud container clusters get-credentials sbs-cluster --zone europe-west6-a
    ```

### B. Deployment
Use the automated script to build, push, and deploy.

```bash
# 1. Set Environment Variables
export GCP_PROJECT_ID="sbs-solver-prod"
export GCP_CLUSTER_NAME="sbs-cluster"
export GCP_ZONE="europe-west6-a"

# 2. Run Deployment Script
./scripts/deploy_gcp.sh
```

### C. Accessing the GUI
The script will provision a Public IP (LoadBalancer). However, the GUI currently connects to `localhost:8080`. You have two options:

**Option 1 (Recommended): Port Forwarding**
This is secure and requires no code changes.
```bash
# Forward remote cluster traffic to your local machine
kubectl port-forward -n sbs-namespace svc/sbs-prod 8080:80
```
Then run the GUI locally:
```bash
cd sbs-gui && ./gradlew run
```

**Option 2: Direct Public IP**
1. Get the IP: `kubectl get svc -n sbs-namespace`
2. Update `sbs-gui/composeApp/src/commonMain/kotlin/App.kt` to use the external IP instead of `localhost`.
3. Re-run `./gradlew run`.

---

## Development & Maintenance

### Build System (`make`)
The `Makefile` wraps standard Cargo commands.

* `make setup`: Download dictionary.
* `make build`: Compile in debug mode.
* `make release`: Compile in release mode.
* `make test`: Run unit/integration tests.
* `make format`: Apply `rustfmt`.
* `make lint`: Run `clippy` checks.
* `make doc`: Generate and open documentation.

### GUI (`gradle`)
* `./gradlew run`: Run the desktop application.
* `./gradlew check`: Run Kotlin tests/lints.
