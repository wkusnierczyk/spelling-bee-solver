# Stage 1: Build
FROM rust:1.84-slim AS builder

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --bin sbs-backend


# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder \
    /app/target/release/sbs-backend \
    /usr/local/bin/sbs-backend

RUN mkdir -p /app/data
ENV SBS_DICT=/app/data/dictionary.txt
ENV RUST_LOG=info

EXPOSE 8080

CMD ["sbs-backend"]