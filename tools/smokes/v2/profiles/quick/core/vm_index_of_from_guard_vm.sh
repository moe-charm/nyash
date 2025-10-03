#!/bin/bash
# vm_index_of_from_guard_vm.sh — Repro: index_of_from scan termination

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2

# Disabled by default (diagnostic repro). Enable with SMOKES_ENABLE_VM_REPRO=1
if [ "${SMOKES_ENABLE_VM_REPRO:-0}" != "1" ]; then
  test_skip "vm_index_of_from_guard_vm (diagnostic repro)" \
    "Enable with SMOKES_ENABLE_VM_REPRO=1 to run"
  exit 0
fi

TMP_DIR="/tmp/vm_index_of_from_guard_vm_$$"
mkdir -p "$TMP_DIR"
cat > "$TMP_DIR/driver.nyash" << 'EOF'
static box Scan {
  index_of_from(hay, needle, pos) {
    if pos < 0 { pos = 0 }
    local n = hay.length()
    if pos >= n { return -1 }
    local m = needle.length()
    if m <= 0 { return pos }
    local i = pos
    local limit = n - m
    local steps = 0
    loop (i <= limit) {
      steps = steps + 1
      if steps > 10000 { return -9999 }
      if hay.substring(i, i+m) == needle { return i }
      i = i + 1
    }
    return -1
  }
}

static box Main {
  main() {
    local s = "0123456789abcdef"
    print("A=" + (""+Scan.index_of_from(s, "012", 0)))
    print("B=" + (""+Scan.index_of_from(s, "def", 13)))
    print("C=" + (""+Scan.index_of_from(s, "xyz", 0)))
    return 0
  }
}
EOF

raw_output=$(run_nyash_vm "$TMP_DIR/driver.nyash")
gotA=$(echo "$raw_output" | awk -F'=' '/^A=/{print $2; exit}')
gotB=$(echo "$raw_output" | awk -F'=' '/^B=/{print $2; exit}')
gotC=$(echo "$raw_output" | awk -F'=' '/^C=/{print $2; exit}')

# Diagnostics: print raw lines for analysis
echo "$raw_output" | sed -n '1,120p' >&2

# Optional strict assertion when SMOKES_ASSERT=1
if [ "${SMOKES_ASSERT:-0}" = "1" ]; then
  if [ "$gotA" != "0" ] || [ "$gotB" != "13" ] || [ "$gotC" != "-1" ]; then
    log_error "vm_index_of_from_guard_vm assertion failed (enable-only)"
    exit 1
  fi
fi

log_success "vm_index_of_from_guard_vm ran (diagnostic)"
rm -rf "$TMP_DIR"
exit 0
