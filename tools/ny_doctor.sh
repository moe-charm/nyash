#!/usr/bin/env bash
set -euo pipefail

ROOT="$(pwd)"
echo "[doctor] CWD: $ROOT"

find_root() {
  local d="$PWD"
  while [ "$d" != "/" ]; do
    if [ -f "$d/hako.toml" ] || [ -f "$d/nyash.toml" ]; then echo "$d"; return 0; fi
    d="$(dirname "$d")"
  done
  return 1
}

HR="${HAKO_ROOT:-}"; NR="${NYASH_ROOT:-}";
CONF_ROOT="$(find_root || true)"

echo "[doctor] HAKO_ROOT=${HR:-<unset>} NYASH_ROOT=${NR:-<unset>}"
echo "[doctor] detected config root: ${CONF_ROOT:-<not found>}"

if [ -n "$CONF_ROOT" ]; then
  if [ -f "$CONF_ROOT/hako.toml" ]; then echo "[doctor] using: $CONF_ROOT/hako.toml"; fi
  if [ -f "$CONF_ROOT/nyash.toml" ]; then echo "[doctor] compat: $CONF_ROOT/nyash.toml"; fi
else
  echo "[hint] set HAKO_ROOT to your project root or run from repo root"; fi

USING=${HAKO_USING:-${NYASH_USING:-<unset>}}
STRAT=${HAKO_USING_STRATEGY:-${NYASH_USING_STRATEGY:-<unset>}}
ALLOW=${HAKO_ALLOW_USING_FILE:-${NYASH_ALLOW_USING_FILE:-<unset>}}
PROF=${HAKO_USING_PROFILE:-${NYASH_USING_PROFILE:-<unset>}}

echo "[using] HAKO_USING=${USING} STRATEGY=${STRAT} ALLOW_FILE=${ALLOW} PROFILE=${PROF}"

if [ "${USING:-0}" = "0" ]; then
  echo "[advice] using is disabled. For dev: source tools/dev_env.sh using";
fi
if [ "${STRAT:-resolver}" != "prelude" ]; then
  echo "[advice] STRATEGY is not 'prelude'. Dev recommends HAKO_USING_STRATEGY=prelude";
fi
if [ "${ALLOW:-0}" = "0" ]; then
  echo "[advice] file path using disallowed. For dev enable HAKO_ALLOW_USING_FILE=1 or register modules in hako.toml";
fi

echo "[doctor] done"

