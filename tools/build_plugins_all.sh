#!/usr/bin/env bash
set -euo pipefail

# Build all plugins in cdylib/staticlib forms and copy artifacts next to Cargo.toml

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT_DIR"

PROFILE=${PROFILE:-release}
JOBS=${JOBS:-24}

echo "[plugins] building all (profile=$PROFILE, jobs=$JOBS)"

# Build all plugins in one go for maximum efficiency
echo "[plugins] building workspace..."
cargo build --workspace --$PROFILE -j $JOBS >/dev/null

# Copy artifacts to plugin directories
for dir in plugins/*; do
  [[ -d "$dir" && -f "$dir/Cargo.toml" ]] || continue
  pkg=$(grep -m1 '^name\s*=\s*"' "$dir/Cargo.toml" | sed -E 's/.*"(.*)".*/\1/')
  # Determine lib name (prefer [lib].name, else package name with '-' -> '_')
  libname=$(awk '/^\[lib\]/{flag=1;next}/^\[/{flag=0}flag && /name\s*=/{print; exit}' "$dir/Cargo.toml" | sed -E 's/.*"(.*)".*/\1/')
  if [[ -z "${libname}" ]]; then
    libname=${pkg//-/_}
  fi
  echo "[plugins] -> $pkg (libname=$libname)"
  # Copy artifacts
  outdir="target/$PROFILE"
  # cdylib (.so/.dylib/.dll)
  for ext in so dylib dll; do
    f="${outdir}/lib${libname}.${ext}"
    if [[ -f "$f" ]]; then
      cp -f "$f" "$dir/" && echo "  copied $(basename "$f")"
    fi
  done
  # staticlib (.a)
  fa="${outdir}/lib${libname}.a"
  if [[ -f "$fa" ]]; then
    cp -f "$fa" "$dir/" && echo "  copied $(basename "$fa")"
  fi
done

echo "[plugins] done"

