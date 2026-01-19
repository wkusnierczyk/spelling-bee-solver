# Spelling Bee Solver

[Spelling Bee](https://en.wikipedia.org/wiki/Spelling_bee) is a word game where players build a list of valid words from a fixed set of letters, typically with one required center letter and length constraints. Solutions are scored by length, with a bonus for pangrams that use every letter at least once. There are implementations available online, e.g., at [SpellBee.org](https://spellbee.org/) or at [New York Times](https://www.nytimes.com/puzzles/spelling-bee).

This repository does not provide yet another implementation of the puzzle.
It offers a _solver_: a tool for solving Spelling Bee challenges.
The purpose, of course, is not the cheat the game by copying words over from an automated solver.
The tool was built as a proof of concept, a BWYN (build what you need) tool for educational purposes, and as an exercise in bui;ding and deploying such tools.

## Contents

- [Deployment options](#deployment-options)
  - [Using the Rust library](#using-the-rust-library)
  - [Using the CLI](#using-the-cli)
  - [Local native deployment](#local-native-deployment)
  - [Local deployment with Docker](#local-deployment-with-docker)
  - [Local deployment with Docker Compose](#local-deployment-with-docker-compose)
  - [Local deployment with Kubernetes and Minikube](#local-deployment-with-kubernetes-and-minikube)
- [Development](#development)
  - [Project structure](#project-structure)
  - [Workflows](#workflows)
  - [Code health](#code-health)
- [About](#about)

## Deployment options

The tool can be used as:

* A rust library, that can be used in client rust code.
* A CLI (command-line interface) tool that can be installed on a local machine and executed in the terminal.
* A locally deployed backend and frontend GUI.
* A locally deployed Docker compose cluster.
* A local k8s (kubernetes) cluster, deployed with minikube.
* [**WIP**] A cloud service, deployed to GCP (Google Cloud Platform).

### Using the Rust library

Add the library as a local dependency from this repo:

```toml
[dependencies]
sbs = { path = "sbs-solver" }
```

Minimal example:

```rust
use sbs::{Config, Dictionary, Solver};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()
        .with_letters("abcdefg")
        .with_present("a");

    let dictionary = Dictionary::from_words(&[
        "face", "cafe", "bead", "feed", "decaf", "badge",
    ]);

    let solver = Solver::new(config);
    let solutions = solver.solve(&dictionary)?;

    println!("{} words", solutions.len());
    Ok(())
}
```

### Using the CLI

Build the CLI binary:

```bash
make build-cli
```

Run it from the repo root (debug build):

```bash
./sbs-solver/target/debug/sbs --letters abcdefg --present a
```

Install globally:

```bash
make install-cli
```

Optional flags:

```bash
sbs \
  --letters abcdefg \
  --present a \
  --dictionary sbs-solver/data/dictionary.txt \
  --output /tmp/solutions.txt
```

You can also provide a JSON config file and override specific fields via flags:

```json
{
  "letters": "abcdefg",
  "present": "a",
  "dictionary": "sbs-solver/data/dictionary.txt",
  "minimal-word-length": 4
}
```

```bash
sbs --config /path/to/config.json --present a
```

### Local native deployment

![Native](architecture/native.png)

Deploy the service natively using the followng make targets.

Build and start the backend and the frontend.

```bash
# Backend:
make build-backend
make start-backend

# Frontend:
make build-frontend
make start-frontend
```

Both components of the service can also be started in one go.

```bash
make start-local

# stop afterwards
make stop-local
```

### Local deployment with Docker

![Docker](architecture/docker.png)

Deploy the service with Docker using the followng make targets.

Build and use backend image.

```bash
# Build, deploy backend
make build-backend-container
make start-backend-container

# Test backend
make test-backend-container

# Stop, clean up backend
make stop-backend-container
make remove-backend-container
```

Build and use frontend image.

```bash
# Build, deploy frontend
make build-frontend-container
make start-frontend-container

# Test frontend (requires a running backend)
make test-frontend-container

# Stop, clean up frontend
make stop-frontend-container
make remove-frontend-container
```

Build and use containerised components in one go.

```bash
# Build and start all containers
make start-docker-stack

# Test the stack
make test-docker-stack

# Stop all containers
make stop-docker-stack

# Remove all containers
make remove-docker-stack
```

### Local deployment with Docker Compose

![Compose](architecture/compose.png)

Deploy the service with Docker Compose using the followng make targets.

```bash
# Start the whole stack
make docker-compose-up

# Stop the whole stack
make docker-compose-down
```

### Local deployment with Kubernetes and Minikube

![Minikube](architecture/minikube.png)

Deploy the stack to a local Minikube cluster using the followng make targets.

```bash
# Start a local cluster (Docker driver)
make minikube-start

# Build images inside Minikube registry and deploy via Helm
make minikube-deploy

# Verify pods and service connectivity
make minikube-test

# Open the frontend service in your browser
make minikube-url

# Clean up the Helm release (cluster stays running)
make minikube-clean

# Stop or delete the Minikube cluster
make minikube-stop
make minikube-delete
```

### Cloud deployment (GCP)

![Cloud](architecture/cloud.png)

Deploy the stack to a GCP (Google Clopud Platform) Kubernetes cluster using the followng make targets.

**Note**  
Requires `GCP_PROJECT_ID`, `GCP_CLUSTER_NAME`, and `GCP_ZONE` environment variables to be set to appropriate values.
Check your GCP project for details.

```bash
# Authenticate kubectl with the GKE cluster
make gcp-auth

# Build images for Cloud (Force AMD64 for GKE compatibility)
make gcp-build

# Push images to Google Container Registry
make gcp-push

# Deploy to staging namespace for testing
make gcp-deploy-candidate

# Test the candidate deployment in staging
make gcp-test-candidate

# Promote candidate to production (rolling update)
make gcp-promote-candidate

# Remove the staging deployment
make gcp-cleanup-candidate

# Full deployment pipeline
make gcp-deploy

# Show current deployment status
make gcp-status

# Tail backend logs from production
make gcp-logs-backend

# Tail frontend logs from production
make gcp-logs-frontend

# Rollback to previous production release
make gcp-rollback

# Remove all GCP deployments (DANGEROUS)
make gcp-destroy
```



## Development

The project is open source and welcomes contributions.
Use [issues](https://github.com/wkusnierczyk/spelling-bee-solver/issues) and [discussions](https://github.com/wkusnierczyk/spelling-bee-solver/discussions) to provide feedback and contributions.

Use [pull requests](https://github.com/wkusnierczyk/spelling-bee-solver/pulls) to contribute content.
No pushing to `main` is allowed.

### Project structure

```text
.
|-- LICENSE
|-- Makefile
|-- README.md
|-- charts/                # Helm charts
|   `-- ...
|-- infra/                 # GKE manifests (Ingress, SSL, certs)
|   `-- ...
|-- sbs-gui/               # React frontend (Vite + TypeScript)
|   `-- ...
|-- sbs-solver/            # Rust backend & trie engine (Actix-web)
|   `-- ...
|-- scripts/               # Automation for builds and deployments
|   `-- ...
`-- ...                    # Other files
```

### Workflows

When developing locally, run the GitHub test and integration workflows locally to speed up debugging in case of failure, and saving the remote from polluting commits.

Prerequisites:

* [act](https://github.com/nektos/act).

```bash
# List available workflows
make ci-list

# Run individual workflows
make ci-backend
make ci-docker
make ci-compose

# Run all workflows
make ci-all
```

### Code health

Use these make targets to maintain clean and healthy code.

```bash
# Print available Makefile targets and a short description; no state changes.
make help

# Run backend unit and integration tests; local only.
make test

# Run clippy linter on backend; local only.
make lint

# Format backend code using rustfmt; local only.
make format
```
---

## About

```bash
sbs --about

sbs: Spelling Bee Solver tool
├─ version:   0.1.0
├─ developer: mailto:waclaw.kusnierczyk@gmail.com
├─ source:    https://github.com/wkusnierczyk/ips-sampler
├─ licence:   MIT https://opensource.org/licenses/MIT
└─ usage:     sbs --help
```
