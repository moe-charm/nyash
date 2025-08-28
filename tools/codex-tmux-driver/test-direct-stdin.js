const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:8770');

ws.on('open', () => {
  console.log('✨ Testing direct stdin write...');
  
  // 改行だけを送る
  ws.send(JSON.stringify({
    type: 'inject-input',
    data: ''  // 空文字列 + 改行
  }));
  
  setTimeout(() => {
    // テキストと改行を別々に送る
    ws.send(JSON.stringify({
      type: 'inject-input',
      data: 'Test message without newline'
    }));
    
    setTimeout(() => {
      // 改行だけを送る
      ws.send(JSON.stringify({
        type: 'inject-input',
        data: ''  // これでEnterキーの効果
      }));
      
      setTimeout(() => {
        ws.close();
        console.log('✅ Test complete!');
      }, 500);
    }, 1000);
  }, 1000);
});

ws.on('error', (err) => {
  console.error('Error:', err);
});