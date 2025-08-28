const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:8770');

ws.on('open', () => {
  console.log('✨ Sending message with newline to Codex...');
  
  // 改行を含むメッセージ
  ws.send(JSON.stringify({
    type: 'inject-input',
    data: '\nこんにちは！Nyashプロジェクトだにゃ🐱\nJIT開発の進捗はどう？'
  }));
  
  setTimeout(() => {
    ws.close();
    console.log('✅ Message sent with newlines!');
  }, 1000);
});

ws.on('error', (err) => {
  console.error('Error:', err);
});