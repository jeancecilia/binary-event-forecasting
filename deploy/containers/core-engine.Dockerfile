# Core Engine Dockerfile

FROM rust:1.85-slim-bookworm AS builder

WORKDIR /app

# Copy workspace manifests (all members needed for workspace resolution)
COPY Cargo.toml Cargo.lock ./
COPY crates/domain-types/Cargo.toml crates/domain-types/
COPY crates/protocol/Cargo.toml crates/protocol/
COPY crates/market-state/Cargo.toml crates/market-state/
COPY crates/forecast-policy/Cargo.toml crates/forecast-policy/
COPY crates/matching/Cargo.toml crates/matching/
COPY crates/ledger/Cargo.toml crates/ledger/
COPY crates/journal/Cargo.toml crates/journal/
COPY crates/replay/Cargo.toml crates/replay/
COPY crates/experiment-control/Cargo.toml crates/experiment-control/
COPY crates/telemetry/Cargo.toml crates/telemetry/
COPY services/core-engine/Cargo.toml services/core-engine/
COPY services/mock-gateway/Cargo.toml services/mock-gateway/

# Build dependencies only (layer caching)
RUN mkdir -p crates/domain-types/src crates/protocol/src crates/market-state/src \
    crates/forecast-policy/src crates/matching/src crates/ledger/src \
    crates/journal/src crates/replay/src crates/experiment-control/src \
    crates/telemetry/src services/core-engine/src services/mock-gateway/src
RUN echo 'fn main() {}' > services/core-engine/src/main.rs
RUN echo 'fn main() {}' > services/mock-gateway/src/main.rs
RUN cargo build --release --bin core-engine || true
RUN rm services/core-engine/src/main.rs services/mock-gateway/src/main.rs

# Copy actual source
COPY crates/ crates/
COPY services/ services/

RUN cargo build --release --bin core-engine

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/core-engine /usr/local/bin/core-engine
COPY config/core.toml /app/config/core.toml
COPY data/traces/golden/ /app/data/traces/golden/
RUN mkdir -p /app/var/journal /app/var/spool /app/var/artifacts
RUN mkdir -p /run/binary-event-research

EXPOSE 0

ENTRYPOINT ["/usr/local/bin/core-engine"]
CMD ["--config", "/app/config/core.toml"]
