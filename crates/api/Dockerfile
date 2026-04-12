# Stage 1: Build
FROM rust:1.85-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

# Build only the API binary in release mode.
RUN cargo build --release -p rustok-api && \
    strip target/release/rustok-api

# Stage 2: Minimal runtime
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rustok-api /usr/local/bin/rustok-api

ENV RUST_LOG=info
EXPOSE 3000

CMD ["rustok-api"]
