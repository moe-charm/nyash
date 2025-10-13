#!/bin/bash
# selfhost_mir_min_vm.sh — Ny製の最小MIR(JSON v0) 実行器スモーク（const→ret）
# tags: selfhost

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi
if [ "${SMOKES_SELFHOST_M2M3_ENABLE:-0}" != "1" ]; then test_skip "selfhost M2/M3 gated (set SMOKES_SELFHOST_M2M3_ENABLE=1)"; exit 0; fi

# Dev-time guards
export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1
export NYASH_BUILDER_REWRITE_INSTANCE=1

TEST_DIR="/tmp/selfhost_mir_min_vm_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# ルートの modules 解決を利用するため、ここでは nyash.toml は最小限
cat > nyash.toml << EOF
[using]
paths = ["$NYASH_ROOT/apps", "$NYASH_ROOT/lib", "."]
EOF

# ドライバ（MirVmMin を経由して const→ret の値を出力）
cat > driver.nyash << 'EOF'
static box MirVmMin {
  index_of_from(hay, needle, pos) {
    if pos < 0 { pos = 0 }
    local n = hay.length()
    if pos >= n { return -1 }
    local m = needle.length()
    if m <= 0 { return pos }
    local i = pos
    local limit = n - m
    loop (i <= limit) {
      local seg = hay.substring(i, i + m)
      if seg == needle { return i }
      i = i + 1
    }
    return -1
  }
  read_digits(json, pos) {
    local out = ""
    local i = pos
    loop (true) {
      local s = json.substring(i, i+1)
      if s == "" { break }
      if s == "0" || s == "1" || s == "2" || s == "3" || s == "4" || s == "5" || s == "6" || s == "7" || s == "8" || s == "9" {
        out = out + s
        i = i + 1
      } else { break }
    }
    return out
  }
  _str_to_int(s) {
    local i = 0
    local n = s.length()
    local acc = 0
    loop (i < n) {
      local ch = s.substring(i, i+1)
      if ch == "0" { acc = acc * 10 + 0  i = i + 1  continue }
      if ch == "1" { acc = acc * 10 + 1  i = i + 1  continue }
      if ch == "2" { acc = acc * 10 + 2  i = i + 1  continue }
      if ch == "3" { acc = acc * 10 + 3  i = i + 1  continue }
      if ch == "4" { acc = acc * 10 + 4  i = i + 1  continue }
      if ch == "5" { acc = acc * 10 + 5  i = i + 1  continue }
      if ch == "6" { acc = acc * 10 + 6  i = i + 1  continue }
      if ch == "7" { acc = acc * 10 + 7  i = i + 1  continue }
      if ch == "8" { acc = acc * 10 + 8  i = i + 1  continue }
      if ch == "9" { acc = acc * 10 + 9  i = i + 1  continue }
      break
    }
    return acc
  }
  _int_to_str(n) {
    if n == 0 { return "0" }
    local v = n
    local out = ""
    local digits = "0123456789"
    loop (v > 0) {
      local d = v % 10
      local ch = digits.substring(d, d+1)
      out = ch + out
      v = v / 10
    }
    return out
  }
  _extract_first_const_i64(json) {
    if json == null { return 0 }
    local p = json.indexOf("\"op\":\"const\"")
    if p < 0 { return 0 }
    local key = "\"value\":{\"type\":\"i64\",\"value\":"
    local q = me.index_of_from(json, key, p)
    if q < 0 { return 0 }
    q = q + key.length()
    local digits = me.read_digits(json, q)
    if digits == "" { return 0 }
    return me._str_to_int(digits)
  }
  run(mir_json_text) {
    local v = me._extract_first_const_i64(mir_json_text)
    print(me._int_to_str(v))
    return 0
  }
}

static box Main {
  main() {
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":42}},{\"op\":\"ret\",\"value\":1}]}]}]}"
    return MirVmMin.run(j)
  }
}
EOF

output=$(run_nyash_vm driver.nyash --dev)
output=$(echo "$output" | tail -n 1 | tr -d '\r' | xargs)

expected="42"
if [ "$output" = "$expected" ]; then
  log_success "selfhost_mir_min_vm prints $expected"
  cd /
  rm -rf "$TEST_DIR"
  exit 0
else
  log_error "selfhost_mir_min_vm expected $expected, got: $output"
  cd /
  rm -rf "$TEST_DIR"
  exit 1
fi
