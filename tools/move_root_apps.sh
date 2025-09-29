#!/usr/bin/env bash
set -euo pipefail

# Move root-level app/app_* binaries into artifacts/apps and (by default) create symlinks back.
# Usage:
#   tools/move_root_apps.sh move    # move to artifacts/apps + create/update symlinks
#   tools/move_root_apps.sh unlink  # remove root-level symlinks pointing to artifacts/apps
#   tools/move_root_apps.sh restore # move files back from artifacts/apps to root and remove symlinks
#
# Env:
#   APP_SYMLINKS=0 to disable symlink creation on move (default 1)

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
DEST_DIR="${2:-$ROOT_DIR/artifacts/apps}"
CMD="${1:-move}"
SYMLINKS=${APP_SYMLINKS:-1}

mkdir -p "$DEST_DIR"

move_files() {
  local moved=0
  cd "$ROOT_DIR"
  for f in app app_*; do
    [[ -e "$f" ]] || continue
    # Move only regular files (skip existing symlinks/dirs)
    if [[ -f "$f" ]]; then
      mv -f "$f" "$DEST_DIR/$f"
      echo "[move] $f -> $DEST_DIR/$f"
      moved=$((moved+1))
    fi
  done
  if [[ $SYMLINKS -eq 1 ]]; then
    for t in "$DEST_DIR"/app*; do
      [[ -e "$t" ]] || continue
      local base
      base=$(basename "$t")
      ln -sfn "$(realpath --relative-to="$ROOT_DIR" "$t")" "$ROOT_DIR/$base" || true
      echo "[link] $ROOT_DIR/$base -> $t"
    done
  fi
  echo "[summary] moved=$moved symlinks=$SYMLINKS dest=$DEST_DIR"
}

unlink_links() {
  local removed=0
  cd "$ROOT_DIR"
  for f in app app_*; do
    if [[ -L "$f" ]]; then
      local target
      target=$(readlink "$f" || true)
      if echo "$target" | grep -q "artifacts/apps"; then
        rm -f "$f"
        echo "[unlink] $f (-> $target)"
        removed=$((removed+1))
      fi
    fi
  done
  echo "[summary] unlinked=$removed"
}

restore_files() {
  local restored=0
  cd "$ROOT_DIR"
  for t in "$DEST_DIR"/app*; do
    [[ -e "$t" ]] || continue
    local base
    base=$(basename "$t")
    # Remove symlink if present
    if [[ -L "$base" ]]; then rm -f "$base" || true; fi
    # Move back if not already a regular file
    if [[ ! -f "$base" ]]; then
      mv -f "$t" "$ROOT_DIR/$base"
      echo "[restore] $t -> $ROOT_DIR/$base"
      restored=$((restored+1))
    fi
  done
  echo "[summary] restored=$restored from=$DEST_DIR"
}

case "$CMD" in
  move) move_files ;;
  unlink) unlink_links ;;
  restore) restore_files ;;
  *) echo "usage: $0 {move|unlink|restore} [dest_dir]" >&2; exit 2 ;;
esac

