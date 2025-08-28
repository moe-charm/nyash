#!/bin/bash
# Codexをアップデートチェックなしで起動

echo "🚀 Starting Codex without update check..."

# アップデートチェックをスキップ
export CODEX_SKIP_UPDATE_CHECK=1
export CODEX_HOOK_SERVER=ws://localhost:8770
export CODEX_LOG_FILE=/tmp/codex-8770.log

# 直接オリジナルのCodexを起動（hook-wrapperをバイパス）
if [ -f "$HOME/.local/bin/codex.original" ]; then
    echo "Using codex.original..."
    $HOME/.local/bin/codex.original exec --ask-for-approval never
else
    echo "❌ codex.original not found!"
    echo "Trying regular codex..."
    /usr/local/bin/codex exec --ask-for-approval never
fi