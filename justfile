# Binary Event Forecasting — Task Runner
# Run with: just <recipe>

# Default: build everything
default: build

# ─── Bootstrap ───

bootstrap:
    @echo "==> Installing Rust toolchain..."
    rustup show
    @echo "==> Installing Python dependencies..."
    uv sync
    @echo "==> Setting up pre-commit hooks..."
    pre-commit install
    @echo "==> Creating var directories..."
    mkdir -p var/journal var/spool var/artifacts/sha256
    @echo "==> Bootstrap complete."

# ─── Build ───

build: build-rust build-python

build-rust:
    cargo build --workspace

build-rust-release:
    cargo build --workspace --release

build-python:
    uv sync --all-packages

# ─── Test ───

test: test-rust test-python

test-rust:
    cargo test --workspace

test-rust-release:
    cargo test --workspace --release

test-python:
    uv run pytest

test-unit:
    cargo test --workspace --lib
    uv run pytest python-packages/*/tests/unit/

test-verification:
    cargo test --workspace --test '*'
    uv run pytest verification/tests/

test-verif VERIF:
    cargo test --workspace -- {{VERIF}}
    uv run pytest verification/tests/ -k "{{VERIF}}"

test-req REQ:
    cargo test --workspace -- {{REQ}}
    uv run pytest verification/tests/ -k "{{REQ}}"

test-proptest:
    cargo test --workspace --features proptest -- proptest
    uv run pytest --hypothesis-show-statistics

test-coverage:
    cargo tarpaulin --workspace --out Html --output-dir target/coverage

# ─── Lint ───

lint: lint-rust lint-python

lint-rust:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings

lint-rust-fix:
    cargo fmt --all
    cargo clippy --workspace --all-targets --fix --allow-dirty

lint-python:
    uv run ruff check .
    uv run ruff format --check .

lint-python-fix:
    uv run ruff check --fix .
    uv run ruff format .

typecheck:
    uv run pyright

# ─── Validation ───

validate-requirements:
    uv run python scripts/validate_requirements.py

validate-contracts:
    uv run python scripts/validate_contracts.py

check-dependencies:
    uv run python devtools/dependency-boundary-checker/main.py

# ─── Replay ───

replay-verify:
    cargo run --bin core-engine -- replay --trace data/traces/golden --verify

# ─── Documentation ───

generate-repo-map:
    uv run python scripts/generate_repo_map.py

serve-docs:
    echo "Serving docs at http://localhost:8080"

# ─── Database ───

db-migrate:
    cargo run --bin core-engine -- migrate

db-reset:
    rm -f var/journal/core-journal.sqlite
    rm -f var/spool/research-store-spool.sqlite
    cargo run --bin core-engine -- migrate

# ─── Clean ───

clean:
    cargo clean
    rm -rf var/ .pytest_cache/ target/ dist/ *.egg-info/
    find . -type d -name __pycache__ -exec rm -rf {} +
    find . -type d -name .ruff_cache -exec rm -rf {} +

# ─── Docker ───

docker-build:
    docker build -t binary-event-research-core -f deploy/containers/core-engine.Dockerfile .
    docker build -t binary-event-research-mock -f deploy/containers/mock-gateway.Dockerfile .

docker-run-core:
    docker run --rm -v $(pwd)/var:/app/var -v /run/binary-event-research:/run/binary-event-research binary-event-research-core

# ─── CI ───

ci: lint test validate-requirements validate-contracts check-dependencies replay-verify
    @echo "==> All CI checks passed."
