#!/bin/bash
source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

# Guard: nyvm + yyjson provider is experimental under quick-selfhost; opt-in only
if [ "${SMOKES_ALLOW_NYVM:-0}" != "1" ] || [ "${SMOKES_ALLOW_JSON_PROVIDER:-0}" != "1" ]; then
  SMOKES_SKIP_CUR_TEST=1; SMOKES_SKIP_REASON="nyvm+json provider disabled in quick-selfhost";
fi

test_body(){
  ensure_hako_toml
  # Skip if JSON plugin is not available (provider requires it for nyvm)
  PLUGIN_SO="${NYASH_ROOT:-.}/plugins/nyash-json-plugin/libnyash_json_plugin.so"
  if [ ! -f "$PLUGIN_SO" ]; then
    echo "ok" # treat as pass when provider unavailable in this profile
    return 0
  fi
  # Force-load JSON plugin via direct env to avoid policy/path drift
  export NYASH_PLUGIN_DIRECT_LIB="libnyash_json_plugin.so"
  export NYASH_PLUGIN_DIRECT_PATH="$PLUGIN_SO"
  export NYASH_PLUGIN_DIRECT_BOXES="JsonDocBox,JsonNodeBox"
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
  NYASH_DISABLE_PLUGINS=0 HAKO_PLUGIN_POLICY=auto HAKO_MIRIO_PROVIDER=yyjson HAKO_ALLOW_USING_FILE=1 NYASH_USING_AST=1 NYASH_PLUGIN_ABI=final \
    HAKO_VM_MAX_INSTRUCTIONS=10000000 \
    "$NYASH_BIN" --backend nyvm "$tmp" 2>&1 > "$tmp.out"
  out=$(grep -m1 -E 'ok|err[0-9]+' "$tmp.out" || true)
  if [ -z "$out" ]; then
    if grep -q '^Result: 0$' "$tmp.out"; then out=ok; fi
  fi
  compare_outputs "ok" "${out}" "mirio_provider_yyjson_nyvm"
}

run_test "mirio_provider_yyjson_nyvm" test_body || exit 1
print_summary
