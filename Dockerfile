# syntax=docker/dockerfile:1.6

FROM rust:1.90-bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
 && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo update
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN useradd -m -u 10001 appuser \
 && apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/chimp-chaos-agent /usr/local/bin/chimp-chaos-agent

USER appuser
EXPOSE 50051
ENV RUST_LOG=info

ENTRYPOINT ["/usr/local/bin/chimp-chaos-agent"]

