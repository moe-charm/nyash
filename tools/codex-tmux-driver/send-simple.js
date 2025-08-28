const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:8770');

ws.on('open', () => {
  console.log('Connected! Sending simple text...');
  
  // シンプルなテキストを送信
  ws.send(JSON.stringify({
    type: 'inject-input',
    data: 'hello'
  }));
  
  setTimeout(() => {
    ws.close();
  }, 1000);
});

ws.on('error', (err) => {
  console.error('Error:', err);
});