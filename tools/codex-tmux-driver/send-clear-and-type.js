const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:8770');

ws.on('open', () => {
  console.log('✨ Clearing and sending new message...');
  
  // まずCtrl+Uで現在の行をクリア
  ws.send(JSON.stringify({
    type: 'inject-input',
    data: '\x15'  // Ctrl+U (行クリア)
  }));
  
  // 少し待ってから新しいメッセージ
  setTimeout(() => {
    ws.send(JSON.stringify({
      type: 'inject-input',
      data: 'やっほー！Nyashから挨拶にゃ🐱'
    }));
  }, 100);
  
  setTimeout(() => {
    ws.close();
    console.log('✅ Clear and type complete!');
  }, 1000);
});

ws.on('error', (err) => {
  console.error('Error:', err);
});