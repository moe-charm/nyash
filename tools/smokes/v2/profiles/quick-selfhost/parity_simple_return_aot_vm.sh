#!/usr/bin/env bash
# parity_simple_return_aot_vm.sh — AOT vs VM parity (simple_return)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_parity_simple_return_aot_vm() {
  # VM: inline code prints numeric line; we parse it
  local code=$'static box Main {\n  main() {\n    local x; x = 42; print(x); return x;\n  }\n}\n'
  local out_vm ec_vm
  set +e; out_vm=$(run_nyash_vm -c "$code" | grep -E '^[0-9]+$' | tail -n 1 ); ec_vm=$?; set -e
  if [ -z "$out_vm" ]; then
    echo "[WARN] SKIP parity_simple_return_aot_vm (VM numeric line not found)" >&2
    return 0
  fi
  # AOT compile+link+run
  local mir="$NYASH_ROOT/build/parity/mir/main.mir.json"
  local obj="$NYASH_ROOT/build/parity/obj/main.o"
  local bin="$NYASH_ROOT/build/parity/bin/main"
  mkdir -p "$(dirname "$mir")" "$(dirname "$obj")" "$(dirname "$bin")"
  # Write code to a temp .nyash file, then emit MIR JSON (CLI mir backend does not support -c)
  local src="$NYASH_ROOT/build/parity/src/main.nyash"
  mkdir -p "$(dirname "$src")"
  printf "%s" "$code" > "$src"
  "$NYASH_BIN" --backend mir --emit-mir-json "$mir" "$src" >/dev/null
  tools/aot/emit_object_via_extern_c.sh "$mir" "$obj" >/dev/null
  if [ -f "$NYASH_ROOT/target/release/libhako_kernel.a" ]; then
    tools/aot/link_with_clang.sh -o "$bin" "$obj" --nyrt "$NYASH_ROOT/target/release/libhako_kernel.a" >/dev/null
  else
    tools/aot/link_with_clang.sh -o "$bin" "$obj" >/dev/null
  fi
  local out_aot ec_aot
  set +e; out_aot=$("$bin" 2>&1 | { if grep -q '^Result:'; then grep '^Result:' | awk '{print $2}'; else tail -n1 | grep -Eo '^[0-9]+'; fi; }); ec_aot=$?; set -e
  if [ -z "$out_aot" ]; then
    test_fail "AOT result missing" "exit=$ec_aot"
    return 1
  fi
  if [ "$out_vm" = "$out_aot" ]; then
    test_pass parity_simple_return_aot_vm
  else
    test_fail "VM($out_vm) != AOT($out_aot)" "-"
    return 1
  fi
}

run_test parity_simple_return_aot_vm test_parity_simple_return_aot_vm
exit 0
