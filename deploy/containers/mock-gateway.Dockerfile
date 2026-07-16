# Mock Gateway Dockerfile

FROM rust:1.80-slim-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY services/mock-gateway/ services/mock-gateway/

RUN cargo build --release --bin mock-gateway

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/mock-gateway /usr/local/bin/mock-gateway
COPY config/ /app/config/

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/mock-gateway"]
CMD ["--config", "/app/config/mock-gateway.toml"]
