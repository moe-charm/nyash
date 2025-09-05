#!/bin/bash
# Simple Codex to Claude notification via tmux

# 既定セッション名: codex（必要なら環境変数 CLAUDE_SESSION で上書き可）
CLAUDE_SESSION="${CLAUDE_SESSION:-codex}"
LOG_FILE="$HOME/.codex-work.log"

# Codex実行を記録
echo "[$(date)] Starting: codex $*" >> "$LOG_FILE"

# Codexを実行
codex "$@"
EXIT_CODE=$?

# 結果を記録
echo "[$(date)] Completed with code: $EXIT_CODE" >> "$LOG_FILE"

# Claudeに通知（tmuxセッションがあれば）
if tmux has-session -t "$CLAUDE_SESSION" 2>/dev/null; then
    MESSAGE="🤖 Codex作業完了！ Exit code: $EXIT_CODE"
    tmux send-keys -t "$CLAUDE_SESSION" "# $MESSAGE" Enter
    echo "✅ Notification sent to Claude"
else
    echo "⚠️  Claude session not found"
fi

exit $EXIT_CODE

