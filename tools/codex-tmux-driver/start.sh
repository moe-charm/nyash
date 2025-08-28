#!/bin/bash
# Codex tmux Driver 起動スクリプト

# デフォルト設定
SESSION_NAME="${CODEX_SESSION:-codex-session}"
PORT="${CODEX_PORT:-8766}"
LOG_DIR="${CODEX_LOG_DIR:-/tmp}"
LOG_FILE="$LOG_DIR/codex-$(date +%Y%m%d-%H%M%S).log"

# Node.jsがインストールされているか確認
if ! command -v node &> /dev/null; then
    echo "Error: Node.js is not installed"
    exit 1
fi

# tmuxがインストールされているか確認
if ! command -v tmux &> /dev/null; then
    echo "Error: tmux is not installed"
    exit 1
fi

# npm install実行（初回のみ）
if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    npm install
fi

# 起動
echo "=== Starting Codex tmux Driver ==="
echo "Session: $SESSION_NAME"
echo "Port: $PORT"
echo "Log: $LOG_FILE"
echo ""

node codex-tmux-driver.js \
    --session="$SESSION_NAME" \
    --port="$PORT" \
    --log="$LOG_FILE" \
    "$@"