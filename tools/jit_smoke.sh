#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)

BIN="$ROOT_DIR/target/release/nyash"
if [ ! -x "$BIN" ]; then
  echo "Building nyash (release, JIT)..." >&2
  cargo build --release --features cranelift-jit >/dev/null
fi

echo "[JIT Smoke] Core VM/JIT (plugins disabled)" >&2
NYASH_DISABLE_PLUGINS=1 NYASH_CLI_VERBOSE=1 "$ROOT_DIR/tools/smoke_vm_jit.sh" >/tmp/nyash-jit-core.out
grep -q '^✅ smoke done' /tmp/nyash-jit-core.out || { echo "FAIL: core VM/JIT smoke" >&2; cat /tmp/nyash-jit-core.out; exit 1; }
echo "PASS: core VM/JIT smoke" >&2

echo "[JIT Smoke] Examples (string_p0, array_p0, map_p0)" >&2
set -o pipefail
NYASH_DISABLE_PLUGINS=1 "$BIN" --backend vm "$ROOT_DIR/apps/examples/string_p0.nyash" > /tmp/nyash-ex-str.out
NYASH_DISABLE_PLUGINS=1 "$BIN" --backend vm "$ROOT_DIR/apps/examples/array_p0.nyash" > /tmp/nyash-ex-arr.out
NYASH_DISABLE_PLUGINS=1 "$BIN" --backend vm "$ROOT_DIR/apps/examples/map_p0.nyash" > /tmp/nyash-ex-map.out
if rg -q '^Result:\s*0\b' /tmp/nyash-ex-str.out && rg -q '^Result:\s*0\b' /tmp/nyash-ex-arr.out && rg -q '^Result:\s*0\b' /tmp/nyash-ex-map.out; then
  echo "PASS: examples" >&2
else
  echo "FAIL: examples" >&2; { echo '--- string_p0 ---'; cat /tmp/nyash-ex-str.out; echo '--- array_p0 ---'; cat /tmp/nyash-ex-arr.out; echo '--- map_p0 ---'; cat /tmp/nyash-ex-map.out; } >&2; exit 1
fi

echo "All PASS" >&2

# Optional: ensure ny_plugins load does not break core path
echo "[JIT Smoke] Plugins opt-in load (sanity)" >&2
NYASH_LOAD_NY_PLUGINS=1 "$BIN" --backend vm "$ROOT_DIR/apps/examples/string_p0.nyash" > /tmp/nyash-ex-plugins.out || true
if rg -q '^Result:\s*0\b' /tmp/nyash-ex-plugins.out; then
  echo "PASS: plugins load (sanity)" >&2
else
  echo "WARN: plugins load path did not complete cleanly; continuing (optional)" >&2
  sed -n '1,120p' /tmp/nyash-ex-plugins.out >&2 || true
fi
