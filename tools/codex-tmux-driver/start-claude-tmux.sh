#!/bin/bash
# Claude専用のtmux起動スクリプト

SESSION_NAME="${1:-claude}"
# 第2引数がなければデフォルトのClaudeバイナリを使用
if [ $# -ge 2 ]; then
    CLAUDE_BINARY="$2"
    shift 2
    ADDITIONAL_ARGS="$@"
else
    CLAUDE_BINARY="/home/tomoaki/.volta/tools/image/node/22.16.0/bin/claude"
    shift 1
    ADDITIONAL_ARGS=""
fi

# Hook serverのポート
HOOK_PORT=${HOOK_SERVER_PORT:-8770}

echo "🚀 Claude起動設定:"
echo "  セッション名: $SESSION_NAME"
echo "  Claudeバイナリ: $CLAUDE_BINARY"
echo "  追加引数: $ADDITIONAL_ARGS"
echo "  Hook server: ws://localhost:$HOOK_PORT"
echo ""

# 既存セッションがあれば削除
if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
    echo "⚠️  既存セッション '$SESSION_NAME' を削除します..."
    tmux kill-session -t "$SESSION_NAME"
fi

# ラッパースクリプトのパス
WRAPPER_PATH="$(cd "$(dirname "$0")" && pwd)/claude-hook-wrapper.js"

# tmuxセッションを作成
echo "📦 新しいtmuxセッションを作成中..."
tmux new-session -d -s "$SESSION_NAME" \
  "export CLAUDE_REAL_BIN='$CLAUDE_BINARY'; \
   export CLAUDE_HOOK_SERVER='ws://localhost:$HOOK_PORT'; \
   export CLAUDE_HOOK_ENABLE=true; \
   echo '🔌 Connecting to hook-server at port $HOOK_PORT...'; \
   node '$WRAPPER_PATH' $ADDITIONAL_ARGS"

# 成功メッセージ
echo "✅ Claude起動完了！"
echo ""
echo "📋 便利なコマンド:"
echo "  接続: tmux attach -t $SESSION_NAME"
echo "  メッセージ送信: tmux send-keys -t $SESSION_NAME 'your message' Enter"
echo "  セッション確認: tmux ls"
echo "  終了: tmux kill-session -t $SESSION_NAME"