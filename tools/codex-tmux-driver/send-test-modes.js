const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:8770');

ws.on('open', () => {
  console.log('✨ Testing different line endings...');
  
  // いろんな改行を試す
  const messages = [
    { data: 'Test1: Normal', desc: 'Normal' },
    { data: 'Test2:\nWith LF', desc: 'With \\n (LF)' },
    { data: 'Test3:\rWith CR', desc: 'With \\r (CR)' },
    { data: 'Test4:\r\nWith CRLF', desc: 'With \\r\\n (CRLF)' },
    { data: 'Test5:\x0AWith Ctrl+J', desc: 'With Ctrl+J' }
  ];
  
  let index = 0;
  const interval = setInterval(() => {
    if (index >= messages.length) {
      clearInterval(interval);
      ws.close();
      console.log('✅ All tests sent!');
      return;
    }
    
    const msg = messages[index];
    console.log(`  Sending: ${msg.desc}`);
    ws.send(JSON.stringify({
      type: 'inject-input',
      data: msg.data
    }));
    index++;
  }, 2000);
});

ws.on('error', (err) => {
  console.error('Error:', err);
});