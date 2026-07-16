# Mock Gateway Dockerfile

FROM rust:1.85-slim-bookworm AS builder

WORKDIR /app

# Copy workspace manifests
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

# Build dependencies only
RUN mkdir -p crates/domain-types/src crates/protocol/src crates/market-state/src \
    crates/forecast-policy/src crates/matching/src crates/ledger/src \
    crates/journal/src crates/replay/src crates/experiment-control/src \
    crates/telemetry/src services/core-engine/src services/mock-gateway/src
RUN echo 'fn main() {}' > services/mock-gateway/src/main.rs
RUN echo 'fn main() {}' > services/core-engine/src/main.rs
RUN cargo build --release --bin mock-gateway || true
RUN rm services/mock-gateway/src/main.rs services/core-engine/src/main.rs

# Copy actual source
COPY crates/ crates/
COPY services/ services/

RUN cargo build --release --bin mock-gateway

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/mock-gateway /usr/local/bin/mock-gateway
COPY config/mock-gateway.toml /app/config/mock-gateway.toml

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/mock-gateway"]
CMD ["--config", "/app/config/mock-gateway.toml"]
