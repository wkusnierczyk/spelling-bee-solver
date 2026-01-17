
# Spelling Bee Solver (SBS)

## Overview
A high-performance library and tool to solve Spelling Bee puzzles.

## Architecture
The core logic relies on a Trie-based recursive backtracking algorithm to efficiently filter valid words from a seed dictionary.

### Data Flow
1. **Bootstrap**: System loads a local dictionary (default: `data/dictionary.txt`) into a Trie.
2. **Input**: User provides config (letters, center letter).
3. **Process**: Recursive search on the Trie.
4. **Validation (Optional)**: Candidates checked against external API if configured.

## Usage

### Setup
Download the default dictionary:
```bash
make setup
```

### Building
```bash
make build
```

### Testing
```bash
make test
```

## Configuration
Configuration is handled via JSON/YAML files. See `tests/fixtures` for examples.

## License
MIT
