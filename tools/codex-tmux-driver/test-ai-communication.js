#!/usr/bin/env node
// AI同士の通信テストスクリプト

const WebSocket = require('ws');
const ws = new WebSocket('ws://localhost:8770');

ws.on('open', function() {
  console.log('🔌 WebSocketに接続しました');
  
  // Claude同士のテストメッセージ
  const message = {
    source: 'claude1',  // 送信元を明示
    type: 'inject-input',
    data: `[Claude1→Claude2] 🤖 AI同士の通信テストです！
    
このメッセージが見えたら、次のコマンドで返信してください：
node -e "console.log('受信確認: Claude1からのメッセージを受け取りました！');"

送信時刻: ${new Date().toLocaleString('ja-JP')}`
  };
  
  ws.send(JSON.stringify(message));
  console.log('✅ テストメッセージを送信しました');
  console.log('📨 内容:');
  console.log(message.data);
  
  ws.close();
});

ws.on('error', (err) => {
  console.error('❌ エラー:', err.message);
});