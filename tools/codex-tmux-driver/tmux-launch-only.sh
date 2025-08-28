#!/bin/bash
# tmuxでCodexを起動するだけ（自動実行なし！）

SESSION_NAME="codex-safe"

echo "🎯 Codexをtmuxで起動します（自動実行はしません）"
echo ""

# 既存セッションがあれば確認
if tmux has-session -t $SESSION_NAME 2>/dev/null; then
    echo "⚠️  既存のセッション '$SESSION_NAME' が存在します"
    echo -n "削除して新しく作成しますか？ (y/N): "
    read answer
    if [ "$answer" = "y" ] || [ "$answer" = "Y" ]; then
        tmux kill-session -t $SESSION_NAME
    else
        echo "中止しました"
        exit 0
    fi
fi

# tmuxセッションを作成（Codexを起動）
echo "📺 tmuxセッション '$SESSION_NAME' を作成しています..."
tmux new-session -d -s $SESSION_NAME /home/tomoaki/.volta/bin/codex

echo ""
echo "✅ 完了！"
echo ""
echo "📌 使い方:"
echo "  接続: tmux attach -t $SESSION_NAME"
echo "  切断: Ctrl+B, D"
echo "  終了: tmux kill-session -t $SESSION_NAME"
echo ""
echo "⚠️  注意: Codexは対話モードで起動しています"
echo "    自動的な操作は行いません"