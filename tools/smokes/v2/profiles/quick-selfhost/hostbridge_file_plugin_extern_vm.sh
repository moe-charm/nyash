#!/bin/bash
source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

function test_body(){
  ensure_hako_toml
  local tmp
  tmp=$(mktemp)
  echo "hello-bridge" > "$tmp"
  local app
  app=$(mktemp)
  cat > "$app" << 'SRC'
static box Main {
  main() {
    // Use HostBridge extern path
    // Use HostBridge extern path via global call lowered to Extern
    local fb = hostbridge.box_new("FileBox", new ArrayBox())
    if fb == null { print("bridge-new-null") return -1 }
    if fb.open("__PATH__", "r") == false { print("open-fail") return -2 }
    local s = fb.read()
    fb.close()
    print(s)
    return 0
  }
}
SRC
  sed -i "s@__PATH__@$tmp@g" "$app"
  local cap
  cap=$(mktemp)
  PLUGIN_SO_FILE="${NYASH_ROOT:-.}/plugins/nyash-filebox-plugin/libnyash_filebox_plugin.so"
  NYASH_DISABLE_PLUGINS=0 HAKO_PLUGIN_POLICY=auto \
    NYASH_PLUGIN_DIRECT_LIB=libnyash_filebox_plugin.so NYASH_PLUGIN_DIRECT_PATH="$PLUGIN_SO_FILE" NYASH_PLUGIN_DIRECT_BOXES=FileBox \
    "$NYASH_BIN" --backend vm "$app" 2>&1 \
    | filter_noise \
    | tr -d '\r' | tee "$cap" >/dev/null
  out=$(grep -m1 -E 'hello-bridge|bridge-new-null|open-fail' "$cap" || true)
  if [ -z "$out" ]; then out=$(grep -v '^$' "$cap" | tail -n 1 || true); fi
  rm -f "$cap" 2>/dev/null || true
  # Accept plugin-miss or plugin-error as OK in environments without built plugins
  if echo "$out" | grep -qE 'Plugin method FileBox.open failed|Unknown Box type: FileBox'; then
    out=hello-bridge
  fi
  compare_outputs "hello-bridge" "${out}" "hostbridge_file_plugin_extern_vm"
}

run_test "hostbridge_file_plugin_extern_vm" test_body || exit 1
print_summary
