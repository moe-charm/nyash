#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." &>/dev/null && pwd)
BIN="$ROOT_DIR/target/release/nyash"
APP="$ROOT_DIR/test_filebox_simple.nyash"

echo "[plugin-v2-smoke] building nyash (release)…" >&2
cargo build --release -j 2 >/dev/null

echo "[plugin-v2-smoke] building FileBox plugin (release)…" >&2
pushd "$ROOT_DIR/plugins/nyash-filebox-plugin" >/dev/null
cargo build --release -j 2 >/dev/null
popd >/dev/null

echo "[plugin-v2-smoke] building PathBox plugin (release)…" >&2
pushd "$ROOT_DIR/plugins/nyash-path-plugin" >/dev/null
cargo build --release -j 2 >/dev/null
popd >/dev/null

echo "[plugin-v2-smoke] building Math/Time plugin (release)…" >&2
pushd "$ROOT_DIR/plugins/nyash-math-plugin" >/dev/null
cargo build --release -j 2 >/dev/null
popd >/dev/null

echo "[plugin-v2-smoke] building Regex plugin (release)…" >&2
pushd "$ROOT_DIR/plugins/nyash-regex-plugin" >/dev/null
cargo build --release -j 2 >/dev/null
popd >/dev/null

echo "[plugin-v2-smoke] building Net plugin (release)…" >&2
pushd "$ROOT_DIR/plugins/nyash-net-plugin" >/dev/null
cargo build --release -j 2 >/dev/null
popd >/dev/null

if [[ ! -x "$BIN" ]]; then
  echo "error: nyash binary not found at $BIN" >&2
  exit 1
fi

export NYASH_CLI_VERBOSE=${NYASH_CLI_VERBOSE:-1}
export NYASH_DEBUG_PLUGIN=${NYASH_DEBUG_PLUGIN:-1}
export NYASH_VM_USE_PY=0
unset NYASH_DISABLE_PLUGINS

echo "[plugin-v2-smoke] running: $BIN --backend vm $APP" >&2
set +e
timeout -s KILL 25s "$BIN" --backend vm "$APP" > /tmp/nyash-plugin-v2-smoke.out 2>&1
code=$?
set -e
echo "--- plugin v2 smoke output ---"
tail -n 80 /tmp/nyash-plugin-v2-smoke.out || true
echo "-------------------------------"

if [[ $code -ne 0 ]]; then
  echo "plugin-v2-smoke: nyash exited with code $code" >&2
  exit $code
fi

# Basic assertions: plugin loader ran and configured; program completed
if ! grep -q "\[PluginLoaderV2\] load_plugin:" /tmp/nyash-plugin-v2-smoke.out; then
  echo "plugin-v2-smoke: plugin loader did not log load_plugin (NYASH_DEBUG_PLUGIN=1 required)" >&2
  exit 1
fi

# Confirm both FileBox and PathBox libraries were attempted
grep -q "libnyash_filebox_plugin.so" /tmp/nyash-plugin-v2-smoke.out || {
  echo "plugin-v2-smoke: missing FileBox load log" >&2; exit 1; }
grep -q "libnyash_path_plugin.so" /tmp/nyash-plugin-v2-smoke.out || {
  echo "plugin-v2-smoke: missing PathBox load log" >&2; exit 1; }
grep -q "libnyash_math_plugin.so" /tmp/nyash-plugin-v2-smoke.out || {
  echo "plugin-v2-smoke: missing Math/Time load log" >&2; exit 1; }
grep -q "libnyash_regex_plugin.so" /tmp/nyash-plugin-v2-smoke.out || {
  echo "plugin-v2-smoke: missing Regex load log" >&2; exit 1; }
grep -q "libnyash_net_plugin.so" /tmp/nyash-plugin-v2-smoke.out || {
  echo "plugin-v2-smoke: missing Net load log" >&2; exit 1; }

echo "plugin-v2-smoke: OK" >&2

# Optional functional sample (regex + response only)
FUNC_APP="$ROOT_DIR/apps/tests/plugin_v2_functional.nyash"
if [[ -f "$FUNC_APP" ]]; then
  echo "[plugin-v2-smoke] functional: $BIN --backend vm $FUNC_APP" >&2
  set +e
  timeout -s KILL 25s "$BIN" --backend vm "$FUNC_APP" > /tmp/nyash-plugin-v2-func.out 2>&1
  code=$?
  set -e
  tail -n 50 /tmp/nyash-plugin-v2-func.out || true
  if [[ $code -eq 0 ]]; then
    # We don't assert printed values strictly (PyVM path may differ), only that it ran to completion.
    echo "plugin-v2-functional: OK" >&2
  else
    echo "plugin-v2-functional: nyash exited with code $code" >&2
    exit $code
  fi
fi

# Optional functional sample (net round-trip)
NET_APP="$ROOT_DIR/apps/tests/net_roundtrip.nyash"
if [[ -f "$NET_APP" ]]; then
  echo "[plugin-v2-smoke] functional (net): $BIN --backend vm $NET_APP" >&2
  set +e
  timeout -s KILL 25s "$BIN" --backend vm "$NET_APP" > /tmp/nyash-plugin-v2-net.out 2>&1
  code=$?
  set -e
  tail -n 60 /tmp/nyash-plugin-v2-net.out || true
  if [[ $code -eq 0 ]]; then
    echo "plugin-v2-net-functional: OK" >&2
  else
    echo "plugin-v2-net-functional: nyash exited with code $code" >&2
    exit $code
  fi
fi
exit 0
