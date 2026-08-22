# Dioxus fullstack — Render Web Service (free)
FROM rust:1.82-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY assets ./assets
COPY tailwind.css ./
RUN cargo build --release --features server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/trust-work-escrow-app /usr/local/bin/app
COPY assets ./assets
ENV PORT=10000
EXPOSE 10000
CMD ["app"]
