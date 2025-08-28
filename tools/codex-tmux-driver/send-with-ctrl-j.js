const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:8770');

ws.on('open', () => {
  console.log('✨ Sending message with Ctrl+J for newline...');
  
  // Ctrl+J (改行) を含むメッセージ
  ws.send(JSON.stringify({
    type: 'inject-input',
    data: 'Nyashです！\x0AJITの進捗どう？\x0A箱作戦は最高にゃ🐱'  // \x0A = Ctrl+J (LF)
  }));
  
  setTimeout(() => {
    ws.close();
    console.log('✅ Message with Ctrl+J sent!');
  }, 1000);
});

ws.on('error', (err) => {
  console.error('Error:', err);
});