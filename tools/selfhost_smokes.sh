#!/usr/bin/env bash
set -euo pipefail

PROFILE=${1:-quick}
ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)

case "$PROFILE" in
  quick|integration|full) : ;; 
  *) echo "usage: $0 {quick|integration|full}" >&2; exit 2 ;;
esac

SMOKES="$ROOT_DIR/smokes/v2/run.sh"
if [[ ! -x "$SMOKES" ]]; then
  echo "error: smokes runner not found: $SMOKES" >&2
  exit 2
fi

exec "$SMOKES" --profile "$PROFILE" --filter "selfhost_*"

