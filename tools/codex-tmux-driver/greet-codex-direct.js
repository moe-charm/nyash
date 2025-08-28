const WebSocket = require('ws');

console.log('🔌 Connecting to hook server on port 8770...');
const ws = new WebSocket('ws://localhost:8770');

ws.on('open', () => {
  console.log('✅ Connected! Sending inject command...');
  
  // hook-wrapperが期待する形式でメッセージを送信
  ws.send(JSON.stringify({
    type: 'inject-input',
    data: 'こんにちは！ポート8770のCodexさん！Nyashプロジェクトから挨拶にゃ〜🐱'
  }));
  
  console.log('📤 Message sent!');
});

ws.on('message', (data) => {
  console.log('📨 Received:', data.toString());
});

ws.on('error', (err) => {
  console.error('❌ Error:', err.message);
});

ws.on('close', () => {
  console.log('👋 Connection closed');
});

// 10秒後に終了
setTimeout(() => {
  console.log('⏰ Closing...');
  ws.close();
  process.exit(0);
}, 10000);