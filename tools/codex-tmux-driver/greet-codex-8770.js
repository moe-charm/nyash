const WebSocket = require('ws');

// 優先順: CODEX_HOOK_SERVER -> HOOK_SERVER_PORT -> 8770
function resolveControlUrl() {
  const fromEnv = process.env.CODEX_HOOK_SERVER;
  if (fromEnv) {
    try {
      const u = new URL(fromEnv);
      // 制御チャネルは /control を使う
      u.pathname = '/control';
      return u.toString();
    } catch {}
  }
  const port = process.env.HOOK_SERVER_PORT || '8770';
  return `ws://localhost:${port}/control`;
}

const controlUrl = resolveControlUrl();
console.log(`🔌 Connecting to hook control: ${controlUrl}`);
const ws = new WebSocket(controlUrl);

ws.on('open', () => {
  console.log('✅ Connected! Injecting greeting...');

  // Codexへ入力を注入（hook-serverのcontrol API）
  ws.send(JSON.stringify({
    op: 'inject',
    data: 'こんにちは！Codexさん！Nyashプロジェクトから挨拶にゃ〜🐱 JITの調子はどうにゃ？'
  }));

  // ついでにステータス確認
  ws.send(JSON.stringify({ op: 'status' }));
});

ws.on('message', (data) => {
  const msg = JSON.parse(data.toString());
  console.log('📨 Received:', msg);
});

ws.on('error', (err) => {
  console.error('❌ Error:', err.message);
});

ws.on('close', () => {
  console.log('👋 Connection closed');
});

// 30秒後に終了
setTimeout(() => {
  console.log('⏰ Timeout - closing connection');
  try { ws.close(); } catch {}
  process.exit(0);
}, 30000);
