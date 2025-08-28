#!/bin/bash
# すべてのコンポーネントを起動

echo "🚀 双方向通信システム起動中..."

# 1. Hook Serverを起動（バックグラウンド）
echo "1️⃣ Hook Server起動中..."
HOOK_SERVER_PORT=8770 node tools/codex-tmux-driver/hook-server.js > /tmp/hook-server.log 2>&1 &
echo "   PID: $!"
sleep 2

# 2. Claude Code を起動
echo "2️⃣ Claude Code 起動中..."
./tools/codex-tmux-driver/start-ai-tmux.sh claude /home/tomoaki/.volta/bin/codex
sleep 2

# 3. 本物のCodex を起動
echo "3️⃣ 本物のCodex 起動中..."
# 本物のCodexのパスが必要（環境変数で設定）
REAL_CODEX=${REAL_CODEX_PATH:-/path/to/real/codex}
if [ ! -f "$REAL_CODEX" ]; then
    echo "⚠️  REAL_CODEX_PATH が設定されていません！"
    echo "   export REAL_CODEX_PATH=/path/to/real/codex"
    echo "   本物のCodexをスキップします..."
else
    ./tools/codex-tmux-driver/start-ai-tmux.sh codex "$REAL_CODEX" --ask-for-approval never --sandbox danger-full-access
fi
sleep 2

echo ""
echo "✅ すべて起動完了！"
echo ""
echo "📋 次のステップ："
echo "  - Codex→Claude送信: node tools/codex-tmux-driver/test-bidirectional-claude-codex.js"
echo "  - Claude→Codex送信: node tools/codex-tmux-driver/test-bidirectional-codex-claude.js"
echo ""
echo "  - Claude Codeに接続: tmux attach -t claude"
echo "  - 本物のCodexに接続: tmux attach -t codex"
echo ""
echo "  - すべて終了: pkill -f hook-server.js && tmux kill-session -t claude && tmux kill-session -t codex"