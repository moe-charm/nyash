#!/bin/bash
# selfhost_using_modules_alias_json_vm.sh — [modules] alias E2E (Json → lib.json_native.stringify)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi

TEST_main() {
  # Using alias: Json; modules maps to lib.json_native.stringify
  local us_raw='[{"name":"Json"}]'
  local mods_raw='{ "lib.json_native.stringify": "apps/lib/json_native/stringify.hako" }'

  local us=$(printf '%s' "$us_raw" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')
  local mods=$(printf '%s' "$mods_raw" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')

  local program='
using selfhost.compiler.pipeline_v2.using_resolver as UsingResolverBox
using selfhost.compiler.pipeline_v2.namespace as NamespaceBox

static box Main {
  main() {
    local r = new UsingResolverBox()
    r.load_usings_json(__US__)
    r.load_modules_json(__MODS__)
    // Provide explicit alias→namespace mapping for this case
    r.add_ns("Json", "lib.json_native.stringify")
    local name = NamespaceBox.normalize_global_name("Json.stringify", r)
    print(name)
    return 0
  }
}
'
  program=${program/__US__/$us}
  program=${program/__MODS__/$mods}

  local out
  out=$(run_nyash_vm -c "$program" 2>&1 | filter_noise | tail -n 1 | tr -d '\n' | xargs)
  [ "$out" = "lib.json_native.stringify.stringify" ] || { echo "$out"; return 1; }
  return 0
}

run_test "selfhost_using_modules_alias_json_vm" TEST_main

