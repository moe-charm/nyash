#!/bin/bash
# クリーンな表示のためのCodex起動スクリプト

# 環境変数設定
export CODEX_REAL_BIN=/home/tomoaki/.volta/bin/codex
export CODEX_HOOK_SERVER=ws://localhost:8770
export CODEX_HOOK_BANNER=false

# エコー機能を有効化（入力を画面に表示）
export CODEX_HOOK_ECHO_INJECT=true

# デバッグログをファイルにリダイレクト
exec node /mnt/c/git/nyash-project/nyash/tools/codex-tmux-driver/codex-hook-wrapper.js "$@" 2>/tmp/codex-debug.log