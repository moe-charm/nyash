const WebSocket = require('ws');

const url = process.env.CODEX_HOOK_SERVER || 'ws://localhost:8770';
const message = 'Hello Claude! 双方向通信テスト成功！';

console.log(`🔌 Connecting to ${url} ...`);
const ws = new WebSocket(url);

ws.on('open', () => {
  console.log('✅ Connected. Sending greeting...');
  const payload = {
    source: 'codex',
    type: 'inject-input',
    data: message,
  };
  ws.send(JSON.stringify(payload));
  console.log('📤 Sent:', payload);
  ws.close();
});

ws.on('error', (err) => {
  console.error('❌ WebSocket error:', err.message);
  process.exitCode = 1;
});

ws.on('close', () => {
  console.log('👋 Connection closed.');
});

