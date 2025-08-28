const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:8770');

ws.on('open', () => {
  console.log('✨ Testing special key sequences...');
  
  // いろんな特殊キーを試す
  const keys = [
    { data: 'Test with Enter\x0D', desc: 'CR (\\x0D) - Classic Enter' },
    { data: 'Test with Return\x0D\x0A', desc: 'CRLF (\\x0D\\x0A)' },
    { data: 'Test with special\x1B[13~', desc: 'ESC sequence for Enter' },
    { data: 'Test with raw\x1B\x0D', desc: 'ESC + CR' },
    { data: 'Test submit', desc: 'Just text (for Ctrl+J manual test)' }
  ];
  
  let index = 0;
  const interval = setInterval(() => {
    if (index >= keys.length) {
      clearInterval(interval);
      ws.close();
      console.log('✅ All key tests sent!');
      console.log('💡 Try pressing Ctrl+J manually after the last message!');
      return;
    }
    
    const key = keys[index];
    console.log(`  Sending: ${key.desc}`);
    ws.send(JSON.stringify({
      type: 'inject-input',
      data: key.data
    }));
    index++;
  }, 2000);
});

ws.on('error', (err) => {
  console.error('Error:', err);
});