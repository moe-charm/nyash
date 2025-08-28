#!/bin/bash
# クリーンな環境でCodexを起動

echo "🧹 Starting Codex with clean environment..."

# すべてのフック関連出力を無効化
export CODEX_REAL_BIN=/home/tomoaki/.volta/bin/codex
export CODEX_HOOK_SERVER=ws://localhost:8770
export CODEX_USE_SCRIPT_PTY=false  # scriptコマンドを無効化
export CODEX_HOOK_BANNER=false     # バナー出力を無効化
export CODEX_LOG_FILE=/dev/null    # ログ出力を無効化

# stdinを正しく接続
exec node /mnt/c/git/nyash-project/nyash/tools/codex-tmux-driver/codex-hook-wrapper.js