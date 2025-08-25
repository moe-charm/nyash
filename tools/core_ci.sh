#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

echo "[core-ci] Build (release)"
cargo build --release -j32

echo "[core-ci] Golden snapshots"
./tools/ci_check_golden.sh

echo "[core-ci] Done (core build + golden checks)"

