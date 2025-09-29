#!/bin/bash
# lang_match_digit_vm.sh — Verify match on string digits maps to integers (sum=45)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1

TMP_DIR="/tmp/lang_match_digit_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
static box Main {
  digit_to_int(ch) {
    return match ch {
      "0" => 0, "1" => 1, "2" => 2, "3" => 3,
      "4" => 4, "5" => 5, "6" => 6, "7" => 7,
      "8" => 8, "9" => 9,
      _ => -1
    }
  }
  d2c(n) {
    return match n {
      0 => "0", 1 => "1", 2 => "2", 3 => "3",
      4 => "4", 5 => "5", 6 => "6", 7 => "7",
      8 => "8", 9 => "9",
      _ => "0"
    }
  }
  int_to_str(n) {
    if n == 0 { return "0" }
    local s = ""
    local x = n
    loop(x > 0) {
      local d = x % 10
      s = me.d2c(d) + s
      x = x / 10
    }
    return s
  }
  main() {
    local s = "0123456789"
    local i = 0
    local n = s.length()
    local sum = 0
    loop(i < n) {
      local ch = s.substring(i, i+1)
      sum = sum + me.digit_to_int(ch)
      i = i + 1
    }
    print(me.int_to_str(sum))
    return sum
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="45"
compare_outputs "$expected" "$out" "lang_match_digit_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
