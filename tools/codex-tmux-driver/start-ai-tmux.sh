#!/bin/bash
# 汎用AI起動スクリプト - Claude Code/Codexをtmux経由で起動
# 使い方: ./start-ai-tmux.sh <session-name> <ai-binary-path> [additional-args...]

# 引数チェック
if [ $# -lt 2 ]; then
    echo "❌ 使い方: $0 <session-name> <ai-binary-path> [additional-args...]"
    echo ""
    echo "例:"
    echo "  # Claude Code 1番目"
    echo "  $0 claude1-8770 /home/tomoaki/.volta/bin/codex"
    echo ""
    echo "  # Claude Code 2番目" 
    echo "  $0 claude2-8770 /home/tomoaki/.volta/bin/codex"
    echo ""
    echo "  # 本物のCodex（制限解除の引数付き）"
    echo "  $0 codex-real-8770 /path/to/real/codex --ask-for-approval never --sandbox danger-full-access"
    echo ""
    exit 1
fi

SESSION_NAME="$1"
AI_BINARY="$2"
shift 2
ADDITIONAL_ARGS="$@"

# Hook serverのポート（環境変数でカスタマイズ可能）
HOOK_PORT=${HOOK_SERVER_PORT:-8770}

echo "🚀 起動設定:"
echo "  セッション名: $SESSION_NAME"
echo "  AIバイナリ: $AI_BINARY"
echo "  追加引数: $ADDITIONAL_ARGS"
echo "  Hook server: ws://localhost:$HOOK_PORT"
echo ""

# 既存セッションがあれば削除
if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
    echo "⚠️  既存セッション '$SESSION_NAME' を削除します..."
    tmux kill-session -t "$SESSION_NAME"
fi

# ラッパースクリプトのパス（カレントディレクトリ基準）
WRAPPER_PATH="$(cd "$(dirname "$0")" && pwd)/codex-hook-wrapper.js"

# tmuxセッションを作成
echo "📦 新しいtmuxセッションを作成中..."
tmux new-session -d -s "$SESSION_NAME" \
  "export CODEX_REAL_BIN='$AI_BINARY'; \
   export CODEX_HOOK_SERVER='ws://localhost:$HOOK_PORT'; \
   export CODEX_HOOK_BANNER=false; \
   export CODEX_HOOK_ECHO_INJECT=true; \
   export CODEX_HOOK_ENABLE=true; \
   echo '🔌 Connecting to hook-server at port $HOOK_PORT...'; \
   node '$WRAPPER_PATH' $ADDITIONAL_ARGS"

# 成功メッセージ
echo "✅ AI起動完了！"
echo ""
echo "📋 便利なコマンド:"
echo "  接続: tmux attach -t $SESSION_NAME"
echo "  メッセージ送信: tmux send-keys -t $SESSION_NAME 'your message' Enter"
echo "  セッション確認: tmux ls"
echo "  終了: tmux kill-session -t $SESSION_NAME"
echo ""

# 複数AI同時起動の例を表示
if [ "$SESSION_NAME" == "claude1-8770" ]; then
    echo "💡 2つ目のClaude Codeを起動するには:"
    echo "  $0 claude2-8770 $AI_BINARY"
fi
