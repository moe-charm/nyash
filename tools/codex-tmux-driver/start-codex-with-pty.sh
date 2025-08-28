#!/bin/bash
# PTY（擬似端末）を使ってCodexを起動

# 環境変数設定
export CODEX_REAL_BIN=/home/tomoaki/.volta/bin/codex
export CODEX_HOOK_SERVER=ws://localhost:8770
export CODEX_HOOK_BANNER=false

# PTYを強制的に有効化
export CODEX_USE_SCRIPT_PTY=true

# エコー機能を有効化
export CODEX_HOOK_ECHO_INJECT=true

# ラッパー起動
exec node /mnt/c/git/nyash-project/nyash/tools/codex-tmux-driver/codex-hook-wrapper.js "$@"