#!/bin/bash
# vm_string_substring_concat_vm.sh — Repro: substring + concat rendering bug

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2

# Disabled by default (diagnostic repro). Enable with SMOKES_ENABLE_VM_REPRO=1
if [ "${SMOKES_ENABLE_VM_REPRO:-0}" != "1" ]; then
  test_skip "vm_string_substring_concat_vm (diagnostic repro)" \
    "Enable with SMOKES_ENABLE_VM_REPRO=1 to run"
  exit 0
fi

TMP_DIR="/tmp/vm_string_substring_concat_vm_$$"
mkdir -p "$TMP_DIR"
cat > "$TMP_DIR/driver.nyash" << 'EOF'
static box Main {
  main() {
    // JSON-like segment
    local inst_seg = "{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":7}},{\"op\":\"ret\",\"value\":3}"
    print("LEN=" + (""+inst_seg.length()))
    local s = 0
    local e = 55
    local piece = inst_seg.substring(s, e)
    // A) direct print of piece
    print("A=" + piece)
    // B) print head via substring-of-substring
    local head = piece.substring(0, piece.length() < 16 ? piece.length() : 16)
    print("B=" + head)
    // C) show piece length
    print("C.len=" + (""+piece.length()))
    return 0
  }
}
EOF

raw_output=$(run_nyash_vm "$TMP_DIR/driver.nyash")
if [ "${SMOKES_DEV_LOG:-0}" = "1" ]; then
  echo "----- [DEV LOG] full output begin -----" >&2
  echo "$raw_output" >&2
  echo "----- [DEV LOG] full output end -----" >&2
fi

# Diagnostics: print raw lines for analysis
echo "$raw_output" | sed -n '1,120p' >&2

# Optional strict assertion when SMOKES_ASSERT=1
if [ "${SMOKES_ASSERT:-0}" = "1" ]; then
  got_len=$(echo "$raw_output" | awk -F'=' '/^LEN=/{print $2; exit}')
  got_A=$(echo "$raw_output" | awk -F'=' '/^A=/{print $2; exit}')
  got_B=$(echo "$raw_output" | awk -F'=' '/^B=/{print $2; exit}')
  got_C=$(echo "$raw_output" | awk -F'=' '/^C.len=/{print $2; exit}')
  if [ -z "$got_len" ] || [ -z "$got_C" ] || [ -z "$got_A" ] || [ -z "$got_B" ]; then
    log_error "vm_string_substring_concat_vm assertion failed (enable-only)"
    exit 1
  fi
fi

log_success "vm_string_substring_concat_vm ran (diagnostic)"
rm -rf "$TMP_DIR"
exit 0
