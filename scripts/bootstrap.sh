#!/bin/bash
# Bootstrap script for setting up the development environment.
# Requires: Linux, git, rustup, uv, docker/podman

set -euo pipefail

echo "==> Bootstrap: Binary Event Forecasting Research Environment"

# Check prerequisites
command -v rustup >/dev/null 2>&1 || { echo "ERROR: rustup not found. Install from https://rustup.rs"; exit 1; }
command -v uv >/dev/null 2>&1 || { echo "ERROR: uv not found. Install from https://github.com/astral-sh/uv"; exit 1; }
command -v just >/dev/null 2>&1 || { echo "WARN: just not found. Install with 'cargo install just' or your package manager."; }

# Install Rust toolchain
echo "==> Installing Rust toolchain..."
rustup show

# Install Python dependencies
echo "==> Installing Python dependencies..."
uv sync

# Create var directories
echo "==> Creating runtime directories..."
mkdir -p var/journal var/spool var/artifacts/sha256 data/traces data/fixtures data/local

# Initialize SQLite journal
echo "==> Initializing SQLite journal..."
sqlite3 var/journal/core-journal.sqlite < storage/sqlite/migrations/001_initial_journal.sql 2>/dev/null || true

echo "==> Bootstrap complete."
echo "Run 'just build' to compile, 'just test' to run tests."
