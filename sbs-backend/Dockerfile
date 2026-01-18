# Stage 1: Builder
# UPDATED: Using Rust 1.84 to support Lockfile v4
FROM rust:1.84-slim-bookworm AS builder

WORKDIR /usr/src/sbs

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy manifests first to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create dummy main files to compile dependencies
# Ensure src/bin exists before writing server.rs
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/bin/server.rs && \
    touch src/lib.rs && \
    cargo build --release --bin sbs-server && \
    rm -rf src

# Copy source code
COPY . .

# Build the actual application
# We need to touch the main files to force rebuild of the app code, not deps
RUN touch src/main.rs src/bin/server.rs src/lib.rs
RUN cargo build --release --bin sbs-server

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y libssl3 ca-certificates curl && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /usr/src/sbs/target/release/sbs-server /usr/local/bin/sbs-server

# Copy dictionary setup script
COPY scripts/setup_dictionary.sh ./scripts/setup_dictionary.sh
RUN chmod +x ./scripts/setup_dictionary.sh

# Environment variables
ENV RUST_LOG=info
ENV SBS_DICT=/app/data/dictionary.txt

# Create data dir and download dictionary during build
RUN ./scripts/setup_dictionary.sh

EXPOSE 8080

CMD ["sbs-server"]
