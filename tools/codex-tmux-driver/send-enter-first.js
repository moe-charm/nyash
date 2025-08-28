const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:8770');

ws.on('open', async () => {
  console.log('✨ Sending Enter key first, then message...');
  
  // まずEnterキーを送信して現在の入力を実行
  ws.send(JSON.stringify({
    type: 'inject-input',
    data: ''  // 空文字列 + Enterで現在の行を実行
  }));
  
  // 少し待ってから新しいメッセージ
  setTimeout(() => {
    ws.send(JSON.stringify({
      type: 'inject-input',
      data: 'Nyashプロジェクトから挨拶だにゃ！JIT開発頑張ってるにゃ？🐱'
    }));
    
    setTimeout(() => {
      ws.close();
      console.log('✅ Complete!');
    }, 500);
  }, 1000);
});

ws.on('error', (err) => {
  console.error('Error:', err);
});