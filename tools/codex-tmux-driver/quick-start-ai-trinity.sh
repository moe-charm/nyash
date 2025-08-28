#!/bin/bash
# AI Trinity クイックスタート - Claude×2 + Codex を一発起動

echo "🤖 AI Trinity (Claude×2 + Codex) 起動スクリプト"
echo "================================================"

# Hook serverが起動しているか確認
if ! lsof -i:8770 >/dev/null 2>&1; then
    echo "⚠️  Hook serverが起動していません！"
    echo "💡 別ターミナルで以下を実行してください:"
    echo "   HOOK_SERVER_PORT=8770 node hook-server.js"
    echo ""
    echo -n "Hook serverを起動してから続行しますか？ (y/n): "
    read answer
    if [ "$answer" != "y" ]; then
        echo "中止しました"
        exit 1
    fi
fi

# 既存セッションのクリーンアップ
echo ""
echo "🧹 既存セッションをクリーンアップ中..."
for session in claude1-8770 claude2-8770 codex-8770; do
    if tmux has-session -t "$session" 2>/dev/null; then
        tmux kill-session -t "$session"
        echo "  - $session を終了しました"
    fi
done

# Claude Code 1を起動
echo ""
echo "🚀 Claude Code #1 を起動中..."
./start-ai-tmux.sh claude1-8770 /home/tomoaki/.volta/bin/codex
sleep 2

# Claude Code 2を起動
echo "🚀 Claude Code #2 を起動中..."
./start-ai-tmux.sh claude2-8770 /home/tomoaki/.volta/bin/codex
sleep 2

# Codexを起動（実際のパスは環境に応じて変更必要）
echo "🚀 Codex を起動中..."
if [ -z "$REAL_CODEX_PATH" ]; then
    echo "⚠️  REAL_CODEX_PATH が設定されていません"
    echo "💡 export REAL_CODEX_PATH=/path/to/real/codex"
    echo "   スキップします..."
else
    ./start-ai-tmux.sh codex-8770 "$REAL_CODEX_PATH" --ask-for-approval never --sandbox danger-full-access
fi

# 状態表示
echo ""
echo "==============================================="
echo "📊 最終状態:"
./manage-ai-sessions.sh status

echo ""
echo "🎯 次のステップ:"
echo "  1. 各セッションに接続: tmux attach -t <session-name>"
echo "  2. AI間でメッセージ送信テスト"
echo "  3. WebSocket経由での通信テスト"
echo ""
echo "📝 便利なコマンド:"
echo "  ./manage-ai-sessions.sh send claude1-8770 'Hello!'"
echo "  ./manage-ai-sessions.sh broadcast 'Hello everyone!'"
echo "  ./manage-ai-sessions.sh attach claude1-8770"