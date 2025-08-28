const WebSocket = require('ws');

console.log('🔌 Testing hook server debug...');
const ws = new WebSocket('ws://localhost:8770');

ws.on('open', () => {
  console.log('✅ Connected as hook client!');
  
  // デバッグ: いろいろな形式を試す
  
  // 1. hook-serverが期待する形式？
  ws.send(JSON.stringify({
    type: 'hook-event',
    event: 'test-inject',
    data: 'テストメッセージ1'
  }));
  
  // 2. 直接inject-input
  setTimeout(() => {
    ws.send(JSON.stringify({
      type: 'inject-input',
      data: 'テストメッセージ2'
    }));
  }, 1000);
  
  // 3. シンプルなメッセージ
  setTimeout(() => {
    ws.send(JSON.stringify({
      message: 'テストメッセージ3'
    }));
  }, 2000);
});

ws.on('message', (data) => {
  console.log('📨 Received from server:', data.toString());
});

ws.on('error', (err) => {
  console.error('❌ Error:', err.message);
});

// 5秒後に終了
setTimeout(() => {
  console.log('👋 Closing...');
  ws.close();
  process.exit(0);
}, 5000);