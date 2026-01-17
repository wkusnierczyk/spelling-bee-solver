
# Spelling Bee Solver (SBS)

## Overview
SBS is a high-performance system for solving Spelling Bee puzzles. It generates all valid words from a dictionary given a set of allowed letters and a center (obligatory) letter.

**Developer:** [Waclaw Kusnierczyk](mailto:waclaw.kusnierczyk@gmail.com)  
**License:** MIT

## Architecture
The system consists of four main components:
1.  **Core Library (`sbs-lib`)**: Rust-based engine using a Trie data structure for efficient recursive backtracking and pruning.
2.  **CLI (`sbs`)**: Command-line interface for local execution.
3.  **Service API (`sbs-server`)**: REST API server (Actix-web) for remote access, deployable via Docker.
4.  **GUI (`sbs-gui`)**: Kotlin Multiplatform Desktop client.

### Algorithm
The solver loads a dictionary into a **Prefix Trie**. During the search:
1.  It iterates through allowed letters to form candidate prefixes.
2.  It traverses the Trie. If a prefix doesn't exist in the Trie, that branch is pruned immediately (fail-fast).
3.  Valid words (nodes marked `is_end_of_word`) containing the "center" letter are collected.

---

## 1. Rust Backend (CLI & Service)

### Prerequisites
* Rust (1.75+)
* Docker (optional, for containerization)
* Make

### Setup
Initialize the dictionary (downloads `words_alpha.txt`):
```bash
make setup
```

### Build
```bash
make build      # Debug build
make release    # Release build
```

### Testing
```bash
make test
```

### Usage: CLI
```bash
# Run directly
./target/release/sbs --letters "abcdefg" --present "a"

# Or via Cargo
cargo run --bin sbs -- --letters "abcdefg" --present "a"

# Save output to file
./target/release/sbs -l "abcdefg" -p "a" -o results.txt

# About
./target/release/sbs --about
```

### Usage: Service (REST API)
Start the server locally:
```bash
make run-server
# Listens on [http://0.0.0.0:8080](http://0.0.0.0:8080)
```

Test the endpoint:
```bash
curl -X POST http://localhost:8080/solve \
     -H "Content-Type: application/json" \
     -d '{"letters": "abcdefg", "present": "a"}'
```

---

## 2. Docker Deployment

The service is containerized using a multi-stage Docker build.

### Build Image
```bash
make docker-build
```

### Run Container
```bash
make docker-run
# The API is now available at http://localhost:8080
```

### Stop Container
```bash
make docker-stop
```

---

## 3. GUI Client (Kotlin Multiplatform)

Located in `sbs-gui/`.

### Prerequisites
* JDK 17+
* IntelliJ IDEA or Android Studio (recommended)

### Running the Desktop App
1.  Ensure the Backend Service is running (locally or via Docker).
2.  Navigate to the GUI directory:
    ```bash
    cd sbs-gui
    ```
3.  Run the application:
    ```bash
    # MacOS/Linux
    ./gradlew run

    # Windows
    gradlew.bat run
    ```
    *(Note: If the wrapper script `gradlew` is not executable, run `chmod +x gradlew` first).*

### Configuration
The GUI currently defaults to connecting to `http://localhost:8080`.

## CI/CD
A GitHub Actions workflow (`.github/workflows/ci.yml`) is provided to:
1.  Lint and Test the Rust code.
2.  Build the Docker image on push to `main`.

