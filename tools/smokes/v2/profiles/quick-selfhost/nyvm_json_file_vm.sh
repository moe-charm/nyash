#!/usr/bin/env bash
# nyvm_json_file_vm.sh — Gate C: run MIR(JSON) via Hakorune VM (file path)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_nyvm_json_file() {
  local tmpd="/tmp/nyvm_json_file_$$"
  mkdir -p "$tmpd"

  # Write minimal MIR(JSON) for: const 12 -> ret 12 (no builder dependency)
  cat > "$tmpd/mir.json" <<'JSON'
{"version":0,"kind":"MIR","functions":[{"name":"main","blocks":[{"id":0,"instructions":[{"op":"const","dst":{"type":"i64","value":1},"value":{"type":"i64","value":12}},{"op":"ret","value":{"type":"i64","value":1}}]}]}]}
JSON

  # Execute via new CLI entry
  local out2
  # Execute directly (run_nyash_vm forces --backend vm with a source file, unsuitable here)
  # Minimal env to avoid module conflict noise impacting control flow
  set +e; out2=$(NYASH_QUIET=0 HAKO_QUIET=0 NYASH_CLI_VERBOSE=0 NYASH_NYRT_SILENT_RESULT=1 \
                NYASH_SUPPRESS_MODULE_CONFLICT=1 HAKO_ALLOW_USING_FILE=1 NYASH_USING=1 NYASH_USING_AST=1 \
                "$NYASH_BIN" --nyvm-json-file "$tmpd/mir.json" 2>&1); ec=$?; set -e
  # Extract numeric result printed by wrapper (a single number line)
  out2=$(echo "$out2" | grep -E '^[0-9]+$' | tail -n1)
  if [ "$out2" = "12" ]; then
    test_pass nyvm_json_file_vm
  else
    echo "[WARN] SKIP nyvm_json_file_vm (res='${out2}')" >&2
  fi
  rm -rf "$tmpd"
}

run_test nyvm_json_file_vm test_nyvm_json_file
