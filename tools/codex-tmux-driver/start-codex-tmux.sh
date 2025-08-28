#!/bin/bash
# tmux経由でCodexを起動（本物の端末環境）

SESSION_NAME="codex-8770"

# 既存セッションがあれば削除
tmux kill-session -t $SESSION_NAME 2>/dev/null

# 環境変数を設定してtmuxセッションを作成
tmux new-session -d -s $SESSION_NAME \
  "export CODEX_REAL_BIN=/home/tomoaki/.volta/bin/codex; \
   export CODEX_HOOK_SERVER=ws://localhost:8770; \
   export CODEX_HOOK_BANNER=false; \
   export CODEX_HOOK_ECHO_INJECT=true; \
   export CODEX_USE_SCRIPT_PTY=true; \
   node /mnt/c/git/nyash-project/nyash/tools/codex-tmux-driver/codex-hook-wrapper.js"

echo "✅ Codex started in tmux session: $SESSION_NAME"
echo ""
echo "To attach: tmux attach -t $SESSION_NAME"
echo "To send keys: tmux send-keys -t $SESSION_NAME 'your text' Enter"
