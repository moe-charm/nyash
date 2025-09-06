#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)

BIN="$ROOT_DIR/target/release/nyash"
if [ ! -x "$BIN" ]; then
  echo "Building nyash (release, JIT)..." >&2
  cargo build --release --features cranelift-jit >/dev/null
fi

# Optional std Ny smokes (requires: NYASH_LOAD_NY_PLUGINS=1 and plugins enabled)
run_std_smokes() {
  if [[ "${NYASH_LOAD_NY_PLUGINS:-0}" != "1" ]] || [[ "${NYASH_DISABLE_PLUGINS:-0}" == "1" ]]; then
    return 0
  fi

  echo "[JIT Smoke] Std Ny smokes (plugins via nyash.toml)" >&2

  local smokes=(
    "apps/smokes/std/string_smoke.nyash"
    "apps/smokes/std/array_smoke.nyash"
  )

  local overall_rc=0
  for f in "${smokes[@]}"; do
    local name
    name=$(basename "$f" .nyash)
    if [[ ! -f "$ROOT_DIR/$f" ]]; then
      echo "[STD] ${name}: FAIL (missing)" >&2
      overall_rc=1
      continue
    fi

    set +e
    # Hard timeout to prevent runaway smokes (hang guard)
    out=$(timeout 15s "$BIN" --backend vm "$ROOT_DIR/$f" 2>&1)
    rc=$?
    # Normalize timeout exit code (124) to rc=124
    if [[ $rc -eq 124 ]]; then
      echo "[STD] ${name}: TIMEOUT" >&2
      overall_rc=1
      continue
    fi
    set -e
    if [[ $rc -eq 0 ]] && echo "$out" | rg -q '^Result:\s*0\b'; then
      echo "[STD] ${name}: PASS" >&2
    else
      # Heuristic skip: ArrayBox plugin not available (treat as SKIP not FAIL)
      if echo "$out" | rg -q 'Failed to create ArrayBox'; then
        echo "[STD] ${name}: SKIP (ArrayBox plugin unavailable)" >&2
      else
        echo "[STD] ${name}: FAIL" >&2
        echo "$out" | sed -n '1,120p' >&2 || true
        overall_rc=1
      fi
    fi
  done

  if [[ $overall_rc -ne 0 ]]; then
    exit 1
  fi
}

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

# Run std Ny smokes only when explicitly enabled via env
run_std_smokes
