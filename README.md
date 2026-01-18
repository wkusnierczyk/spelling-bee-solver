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
* [WIP] A locally deployed Docker compose cluster.
* [WIP] A local k8s (kubernetes) cluster, deployed with minikube.
* [WIP] A cloud service, deployed to GCP (Google Cloud Platform).

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

### Local containerised deployment

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

# Stop all containers
make stop-docker-stack

# Remove all containers
make remove-docker-stack
```

Deploy the stack using Docker Compose

```bash
# Start the whole stack
make docker-compose-up

# Stop the whole stack
make docker-compose-down
```

## Development

Makefile targets:

- Print available Makefile targets and a short description; no state changes.
  ```bash
  make help
  ```

- Run backend unit and integration tests; local only.
  ```bash
  make test
  ```

- Run clippy linter on backend; local only.
  ```bash
  make lint
  ```

- Format backend code using rustfmt; local only.
  ```bash
  make format
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
