# Spelling Bee Solver

[Spelling Bee](https://en.wikipedia.org/wiki/Spelling_bee) is a word game where players build a list of valid words from a fixed set of letters, typically with one required center letter and length constraints. Solutions are scored by length, with a bonus for pangrams that use every letter at least once. There are implementations available online, e.g., at [SpellBee.org](https://spellbee.org/) or at [New York Times](https://www.nytimes.com/puzzles/spelling-bee).

This repository does not provide yet another implementation of the puzzle.
It offers a _solver_: a tool for solving Spelling Bee challenges.

The tool _generalizes_ beyond the original NYT puzzle though.

| Constraint        | Solver                                                     | Original puzzle            |
| ----------------- | ---------------------------------------------------------- | -------------------------- |
| Available letters | Any number of letters                                      | Fixed at 7 letters         |
| Required letters  | Any number of required letters, including _zero_ and _all_ | Fixed at 1 required letter |
| Letter repetition | Configurable repetition limits                             | Unbounded repetition       |
| Word length       | [WIP] Configurable lower and upper bounds                  | Lower bound of 4           |

## Dictionary validation

The solver uses a _seed dictionary_ (a local word list) to generate candidate words.
Optionally, results can be validated against an external dictionary API.
When a validator is enabled:
* Only words confirmed by the external dictionary are retained.
* Each word is enriched with a short definition and a hyperlink to the dictionary entry.

### Supported validators

| Validator | API Key | API Documentation |
| --- | --- | --- |
| [Free Dictionary](https://dictionaryapi.dev/) | Not required | `https://api.dictionaryapi.dev/api/v2/entries/en/{word}` |
| [Merriam-Webster](https://dictionaryapi.com/) | Required (free tier) | `https://dictionaryapi.com/api/v3/references/collegiate/json/{word}?key=KEY` |
| [Wordnik](https://developer.wordnik.com/) | Required (free tier) | `https://api.wordnik.com/v4/word.json/{word}/definitions?api_key=KEY` |
| Custom URL | Not required | User-provided URL (must be Free Dictionary API-compatible) |

### Custom validator

You can provide your own dictionary API URL. The system will probe it (by looking up the word "test") and verify it returns a compatible JSON response. If the probe fails, an error is reported and the custom URL is not used.

## Future features

The Spelling Bee Solver simply lists all the generated words in the GUI.
A future release will allow the user to download those words as a text file.

The Spelling Bee Solver frontend talks to the backend via a RESTful API.
A future release may expose the API publicly, to make interaction with the frontend an optional convenience.

> **Note**  
> The tool was built as a **proof of concept**, a **BWYN** (build what you need) tool for educational purposes, and as an exercise in building and deploying such tools.
> Running a service on the cloud implies non-negligible costs. 
> Deploying further extensions and making the tool available 24/7 depends on external funding. 
> See _Sponsor this project_ ([patreon](https://www.patreon.com/c/wkusnierczyk), [buymeacoffee](https://buymeacoffee.com/wkusnierczyk)).


## Contents

- [Dictionary validation](#dictionary-validation)
  - [Supported validators](#supported-validators)
  - [Custom validator](#custom-validator)
- [Deployment options](#deployment-options)
  - [Using the Rust library](#using-the-rust-library)
  - [Using the CLI](#using-the-cli)
  - [Local native deployment](#local-native-deployment)
  - [Local deployment with Docker](#local-deployment-with-docker)
  - [Local deployment with Docker Compose](#local-deployment-with-docker-compose)
  - [Local deployment with Kubernetes and Minikube](#local-deployment-with-kubernetes-and-minikube)
  - [Cloud deployment (GCP)](#cloud-deployment-gcp)
    - [Cloud cost management](#cloud-cost-management)
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
* A cloud service, deployed to GCP (Google Cloud Platform).

### Using the Rust library

Add the library as a local dependency from this repo:

```toml
[dependencies]
sbs = { path = "sbs-backend" }
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
./sbs-backend/target/debug/sbs --letters abcdefg --present a
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
  --dictionary sbs-backend/data/dictionary.txt \
  --output /tmp/solutions.txt
```

With dictionary validation (results include definitions and URLs):

```bash
sbs \
  --letters abcdefg \
  --present a \
  --validator free-dictionary
```

Validators that require an API key:

```bash
sbs \
  --letters abcdefg \
  --present a \
  --validator merriam-webster \
  --api-key YOUR_KEY
```

Custom validator URL:

```bash
sbs \
  --letters abcdefg \
  --present a \
  --validator custom \
  --validator-url https://your-dictionary-api.example.com/api/v2/entries/en
```

You can also provide a JSON config file and override specific fields via flags:

```json
{
  "letters": "abcdefg",
  "present": "a",
  "dictionary": "sbs-backend/data/dictionary.txt",
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

# Remove containers, images, and build cache for a fresh rebuild
make clean-compose-stack
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

Deploy the stack to a GCP (Google Cloud Platform) Kubernetes cluster using the followng make targets.

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

#### Cloud cost management

To avoid excessive charges when the cloud service is not in use, scale down or stop the cluster.

**Option A: Scale to zero**  
Stops _compute_ costs.  
The Load Balancer remains active (~$18/mo), but CPU/RAM incur no costs.

```bash
# Scale down
make gcp-hibernate

# Scale back up
make gcp-wake
```

**Option B: Full teardown**  
Stops _all_ costs.  
Use for long-term breaks. Removes deployments, ingress, and load balancer.

```bash
# Remove everything
make gcp-destroy

# Redeploy later
make gcp-deploy
```

**Check current state**

```bash
make gcp-status
```


## Development

The project is open source and welcomes contributions.
Use [issues](https://github.com/wkusnierczyk/spelling-bee-solver/issues) and [discussions](https://github.com/wkusnierczyk/spelling-bee-solver/discussions) to provide feedback and contributions.

Use [pull requests](https://github.com/wkusnierczyk/spelling-bee-solver/pulls) to contribute content.
No pushing to `main` is allowed.

After cloning, set up the git hooks:

```bash
make setup-hooks
```

This enables a pre-push hook that runs `make check` (format, lint, test) before each push.

### Project structure

```text
.
|-- .github/              # GitHub workflows and templates
|-- LICENSE
|-- Makefile
|-- README.md
|-- architecture/         # Diagrams
|-- charts/               # Helm charts
|-- infra/                # GKE manifests (Ingress, SSL, certs)
|-- sbs-backend/          # Rust backend & trie engine (Actix-web)
|-- sbs-frontend/         # React frontend (Vite + TypeScript)
|-- docker-compose.yml    # Full stack Docker compose
|-- target/               # Local build artifacts
`-- ...                   # Other files
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

# Run format, lint, and test in sequence; local only.
make check
```
---

## About

```bash
sbs --about

sbs: Spelling Bee Solver tool
├─ version:   0.1.0
├─ developer: mailto:waclaw.kusnierczyk@gmail.com
├─ source:    https://github.com/wkusnierczyk/spelling-bee-solver
├─ licence:   MIT https://opensource.org/licenses/MIT
└─ usage:     sbs --help
```
