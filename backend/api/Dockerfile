# syntax=docker/dockerfile:1
FROM rust:1.89-bookworm AS builder

WORKDIR /app

# Copy workspace manifests and source to leverage Docker cache.
COPY backend/Cargo.toml backend/Cargo.toml
COPY backend/sdk/Cargo.toml backend/sdk/Cargo.toml
COPY backend/api/Cargo.toml backend/api/Cargo.toml
COPY backend/sdk/src backend/sdk/src
COPY backend/api/src backend/api/src

# Build the API binary in release mode.
RUN cargo build --release --manifest-path backend/Cargo.toml -p trust-escrow-api

# Runtime stage: minimal Debian image with glibc.
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/backend/target/release/trust-escrow-api /usr/local/bin/trust-escrow-api

ENV PORT=3000
EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/trust-escrow-api"]
