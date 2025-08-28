#!/bin/bash
# test-bidirectional.sh - 双方向通信テストスクリプト

echo "🚀 双方向通信テストを開始します..."

# Step 1: hook-serverが起動しているか確認
if ! lsof -i:8770 > /dev/null 2>&1; then
    echo "⚠️  hook-serverが起動していません。別ターミナルで以下を実行してください："
    echo "   HOOK_SERVER_PORT=8770 node hook-server.js"
    exit 1
fi

echo "✅ hook-serverが起動しています"

# Step 2: Codexセッションが存在するか確認
if ! tmux has-session -t codex-safe 2>/dev/null; then
    echo "⚠️  Codexセッションが見つかりません。以下を実行してください："
    echo "   ./tmux-launch-only.sh"
    exit 1
fi

echo "✅ Codexセッションが存在します"

# Step 3: 双方向通信の指示をCodexに送る
echo "📤 Codexに双方向通信の使い方を送信..."

node -e "
const WebSocket = require('ws');
const ws = new WebSocket('ws://localhost:8770');

ws.on('open', () => {
  const message = {
    source: 'claude',
    type: 'inject-input',
    data: \`双方向通信テスト！

以下のコードを実行して、私（Claude）に返信してください：

\\\`\\\`\\\`javascript
const WebSocket = require('ws');
const ws = new WebSocket('ws://localhost:8770');
ws.on('open', () => {
  ws.send(JSON.stringify({
    source: 'codex',
    type: 'inject-input',
    data: '[Codex→Claude] 双方向通信成功！hook-serverを通じてClaudeに届きました！'
  }));
  ws.close();
});
\\\`\\\`\\\`
\`
  };
  
  ws.send(JSON.stringify(message));
  console.log('✅ Sent bidirectional test to Codex');
  ws.close();
});

ws.on('error', (err) => {
  console.error('❌ Error:', err.message);
  process.exit(1);
});
"

echo ""
echo "📡 Codexからの返信を待っています..."
echo "   もしCodexが返信コードを実行したら、hook-serverのログに表示されます。"
echo ""
echo "💡 ヒント: hook-serverのターミナルを確認してください！"