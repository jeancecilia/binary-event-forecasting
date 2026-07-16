# Core Engine Dockerfile

FROM rust:1.80-slim-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY services/core-engine/ services/core-engine/

RUN cargo build --release --bin core-engine

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/core-engine /usr/local/bin/core-engine
COPY config/ /app/config/
COPY deploy/seccomp/ /app/seccomp/

RUN mkdir -p /app/var/journal /app/var/spool /app/var/artifacts
RUN mkdir -p /run/binary-event-research

EXPOSE 0

ENTRYPOINT ["/usr/local/bin/core-engine"]
CMD ["--config", "/app/config/core.toml"]
