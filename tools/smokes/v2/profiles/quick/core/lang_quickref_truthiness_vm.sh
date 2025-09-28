#!/bin/bash
# lang_quickref_truthiness_vm.sh — Truthiness representative checks (planned)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

# Enabled: truthiness quickref (0→false, 1→true, empty string→false, non-empty→true)

TMP_DIR="/tmp/lang_quickref_truthiness_vm_$$"
mkdir -p "$TMP_DIR"
cat > "$TMP_DIR/code.nyash" << 'EOF'
static box Main {
  main(){
    if (0) { print("T0") } else { print("F0") }
    if (1) { print("T1") } else { print("F1") }
    local s
    s = ""
    if (s) { print("Ts") } else { print("Fs") }
    s = "x"
    if (s) { print("Tx") } else { print("Fx") }
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/code.nyash" --dev | tail -n 4 | tr -d '\r')
expected=$'F0\nT1\nFs\nTx'
compare_outputs "$expected" "$out" "lang_quickref_truthiness_vm" || { rm -rf "$TMP_DIR"; exit 1; }
rm -rf "$TMP_DIR"; exit 0
