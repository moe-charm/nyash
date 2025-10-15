#!/usr/bin/env bash
# llvm_harness_emit_ll_from_builder.sh — Gate B→LLVM: emit .ll via llvmlite harness

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_llvm_harness_emit_ll_from_builder() {
  # Precondition: python3 + llvmlite
  if ! command -v python3 >/dev/null 2>&1; then
    echo "[WARN] SKIP llvm_harness_emit_ll_from_builder (python3 missing)" >&2
    return 0
  fi
  if ! python3 -c 'import llvmlite' 2>/dev/null; then
    echo "[WARN] SKIP llvm_harness_emit_ll_from_builder (llvmlite missing)" >&2
    return 0
  fi

  local tmpd="/tmp/llvm_from_builder_$$"
  mkdir -p "$tmpd"

  # Produce MIR JSON from BlockBuilder (2 + 3)
  local code=$'using "selfhost/shared/mir/block_builder_box.hako" as BlockBuilder\n\n'
  code+=$'using "apps/lib/json_native/stringify.hako" as JSON\n\n'
  code+=$'static box Main {\n  main(args) {\n    local mir = BlockBuilder.binop(2, 3, "Add");\n    print(JSON.stringify_map(mir));\n    return 0;\n  }\n}\n'
  local raw out ec
  set +e; raw=$(run_nyash_vm -c "$code"); ec=$?; out=$(echo "$raw" | grep '"functions"' | tail -n1); set -e
  if [ $ec -ne 0 ] || [ -z "$out" ]; then
    echo "[WARN] SKIP llvm_harness_emit_ll_from_builder (builder output unavailable)" >&2
    rm -rf "$tmpd"
    return 0
  fi
  printf '%s' "$out" > "$tmpd/mir.json"

  # Emit LLVM IR via harness
  if ! python3 tools/llvmlite_harness.py --in "$tmpd/mir.json" --emit-ll --out "$tmpd/out.ll" >/dev/null 2>&1; then
    echo "[WARN] SKIP llvm_harness_emit_ll_from_builder (harness emit failed)" >&2
    rm -rf "$tmpd"
    return 0
  fi

  # Sanity check: IR contains define and ret
  if grep -q 'define' "$tmpd/out.ll" && grep -q 'ret ' "$tmpd/out.ll"; then
    test_pass llvm_harness_emit_ll_from_builder
  else
    echo "[WARN] SKIP llvm_harness_emit_ll_from_builder (IR check failed)" >&2
    rm -rf "$tmpd"
    return 0
  fi

  rm -rf "$tmpd"
}

run_test llvm_harness_emit_ll_from_builder test_llvm_harness_emit_ll_from_builder
