#!/bin/bash
source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_body(){
  ensure_hako_toml
  local tmp
  tmp=$(mktemp)
  cat > "$tmp" << 'SRC'
using "selfhost/shared/mir/mir_io_box.hako" as MirIo

static box Main {
  main() {
    local j = "{\n  \"kind\":\"MIR\",\n  \"schema_version\":\"1.0\",\n  \"functions\":[{\n    \"id\":0,\n    \"blocks\":[{\n      \"id\":0,\n      \"instructions\":[{\"op\":\"ret\",\"value\":null}],\n      \"terminator\":{\"op\":\"ret\",\"value\":null}\n    }]\n  }]\n}"
    local r1 = MirIo.functions(j)
    if r1.is_Err() { print("err1") return -1 }
    local m1 = r1.as_Ok()
    local fjson = m1.get("content")
    if fjson.indexOf("\"blocks\"") < 0 { print("err2") return -2 }
    local r2 = MirIo.blocks(fjson)
    if r2.is_Err() { print("err3") return -3 }
    local m2 = r2.as_Ok()
    local bcontent = m2.get("content")
    if bcontent == null || bcontent == "" { print("err4") return -4 }
    print("ok")
    return 0
  }
}
SRC
  NYASH_DISABLE_PLUGINS=0 HAKO_PLUGIN_POLICY=auto HAKO_MIRIO_PROVIDER=yyjson "$NYASH_BIN" --backend vm "$tmp" 2>&1 | filter_noise > "$tmp.out"
  out=$(grep -m1 -E 'ok|err[0-9]+' "$tmp.out" || true)
  compare_outputs "ok" "${out}" "mirio_provider_yyjson_vm"
}

run_test "mirio_provider_yyjson_vm" test_body || exit 1
print_summary
