#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"
exec python3 -m unittest discover -s src/llvm_py/tests -p 'test_*.py'

