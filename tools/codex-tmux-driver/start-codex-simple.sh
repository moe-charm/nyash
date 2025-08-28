#!/bin/bash
# 最もシンプルな起動方法

# 環境変数設定
export CODEX_REAL_BIN=/home/tomoaki/.volta/bin/codex
export CODEX_HOOK_SERVER=ws://localhost:8770
export CODEX_HOOK_BANNER=false

# ラッパー起動（2>/dev/nullを削除）
exec node /mnt/c/git/nyash-project/nyash/tools/codex-tmux-driver/codex-hook-wrapper.js "$@"