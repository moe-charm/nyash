const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:8770');

ws.on('open', () => {
  console.log('✨ Sending greeting to Codex...');
  
  ws.send(JSON.stringify({
    type: 'inject-input',
    data: 'こんにちは！Nyashプロジェクトから挨拶だにゃ🐱 JITの開発はどう？'
  }));
  
  setTimeout(() => {
    ws.close();
    console.log('✅ Message sent!');
  }, 1000);
});

ws.on('error', (err) => {
  console.error('Error:', err);
});