# Spelling Bee Solver

[Spelling Bee](https://en.wikipedia.org/wiki/Spelling_bee) is a word game where players build a list of valid words from a fixed set of letters, typically with one required center letter and length constraints. Solutions are scored by length, with a bonus for pangrams that use every letter at least once. There are implementations available online, e.g., at [SpellBee.org](https://spellbee.org/) or at [New York Times](https://www.nytimes.com/puzzles/spelling-bee).

This repository does not provide yet another implementation of the puzzle.
It offers a _solver_: a tool for solving Spelling Bee challenges.
The purpose, of course, is not the cheat the game by copying words over from an automated solver.
The tool was built as a proof of concept, a BWYN (build what you need) tool for educational purposes, and as an exercise in bui;ding and deploying such tools.

## Project structure

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

## Deployment options

The tool can be used as:

* A rust library, that can be used in client rust code.
* A CLI (command-line interface) tool that can be installed on a local machine and executed in the terminal.
* A locally deployed backend and frontend GUI.
* A locally deployed Docker compose cluster.
* A local k8s (kubernetes) cluster, deployed with minikube.
* A cloud service, deployed to GCP (Google Cloud Platform).

## Using the Rust library

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

## Using the CLI

Build the CLI binary:

```bash
(cd sbs-solver && cargo build --bin sbs)
```

Run it from the repo root (debug build):

```bash
./sbs-solver/target/debug/sbs --letters abcdefg --present a
```

Install globally:

```bash
cargo install --path sbs-solver
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

Used for the fastest iteration loop.

```bash
# Backend:
make build-backend && ./sbs-solver/target/debug/sbs-server

# Frontend:
cd sbs-gui && npm install && npm run dev
```


### Local kubernetes (minikube) deployment

Used to test the full containerized stack locally.

Setup the environment:

```bash
minikube start --driver=docker
eval $(minikube docker-env)  
```

Build and deploy:

```bash
cd {project root directory}

docker build -t sbs-solver:latest -f sbs-solver/Dockerfile sbs-solver/
docker build -t sbs-gui:v2-react sbs-gui/

make deploy-minikube
```

Access the service:

```bash
minikube service sbs-prod-frontend -n sbs-namespace
```
**Note**  
If the page fails to load, run `minikube tunnel` in a separate terminal.


### Cloud deployment (GCP)

Requires `GCP_PROJECT_ID`, `GCP_CLUSTER_NAME`, and `GCP_ZONE` environment variables to be set to appropriate values.
Check your GCP project for details.

Initial infrastructure setup:

```bash
# Provision global IP, SSL certificates, and ingress
make install-infra 
```

Deploying updates:

```bash
# Rebuild images, push to GCR, and update Helm
make deploy         
```

---

### Cloud cost management

To avoid excessive charges when the cloud service is not in use:

**Option A**  
Scale to zero (stops _compute_ costs).
The Load Balancer remains active (~$18/mo), but CPU/RAM incur no costs.

```bash
# STOP
kubectl scale deployment sbs-prod-backend --replicas=0 -n sbs-namespace
kubectl scale deployment sbs-prod-frontend --replicas=0 -n sbs-namespace

# START (Back Up)
kubectl scale deployment sbs-prod-backend --replicas=1 -n sbs-namespace
kubectl scale deployment sbs-prod-frontend --replicas=1 -n sbs-namespace
```

**Option B**  
Delete ingress (stops _load balancer_ costs).
Use for long-term breaks.

```bash
# STOP
kubectl delete -f infra/sbs-ingress.yaml

# START
kubectl apply -f infra/sbs-ingress.yaml
```


## Development

Makefile targets:

- Print available Makefile targets and a short description; no state changes.
  ```bash
  make help
  ```

- Provision GKE infrastructure such as ingress and SSL; create/update cloud resources.
  ```bash
  make install-infra
  ```

- Build and pushe images to GCP and update the running deployment; change live cloud state.
  ```bash
  make deploy
  ```

- Start backend and frontend locally via the launcher; run services on local machine only.
  ```bash
  make run-local
  ```

- Query current ingress/pod/cert status in the cluster; read-only.
  ```bash
  make status
  ```

- Compile the Rust backend locally; produce local build artifacts only.
  ```bash
  make build-backend
  ```

- Deploy to local Minikube using local images; affects only your local cluster.
  ```bash
  make deploy-minikube
  ```

## About

```bash
sbs --about

sbs: Spelling Bee Solver tool
├─ version:   0.1.2
├─ developer: mailto:waclaw.kusnierczyk@gmail.com
├─ source:    https://github.com/wkusnierczyk/ips-sampler
├─ licence:   MIT https://opensource.org/licenses/MIT
└─ usage:     sbs --help
```