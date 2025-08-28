// claude-to-claude-test.js
// Claude Code同士の双方向通信テスト

const WebSocket = require('ws');

// テスト1: 送信側として動作
function testAsSender(message = 'Hello from Claude A!') {
  console.log('📤 送信テスト開始...');
  
  const ws = new WebSocket('ws://localhost:8770');
  
  ws.on('open', () => {
    const payload = {
      source: 'claude-a',
      type: 'inject-input',
      data: `[Claude A → Claude B] ${message}`
    };
    
    ws.send(JSON.stringify(payload));
    console.log('✅ メッセージ送信成功:', message);
    
    ws.close();
  });
  
  ws.on('error', (err) => {
    console.error('❌ エラー:', err.message);
  });
}

// テスト2: 受信側として動作
function testAsReceiver() {
  console.log('📥 受信待機開始...');
  
  const ws = new WebSocket('ws://localhost:8770');
  
  ws.on('open', () => {
    console.log('✅ hook-serverに接続しました');
    
    // 自分を受信者として登録
    ws.send(JSON.stringify({
      source: 'claude-b',
      type: 'register',
      data: 'receiver'
    }));
  });
  
  ws.on('message', (data) => {
    const msg = JSON.parse(data.toString());
    console.log('📨 受信:', msg);
    
    // Claude Aからのメッセージの場合、返信
    if (msg.source === 'claude-a') {
      console.log('💬 返信を送信...');
      
      ws.send(JSON.stringify({
        source: 'claude-b',
        type: 'inject-input',
        data: '[Claude B → Claude A] メッセージ受信しました！'
      }));
    }
  });
  
  ws.on('error', (err) => {
    console.error('❌ エラー:', err.message);
  });
}

// コマンドライン引数で動作モードを選択
const mode = process.argv[2] || 'send';

if (mode === 'send') {
  const message = process.argv.slice(3).join(' ') || 'テストメッセージ';
  testAsSender(message);
} else if (mode === 'receive') {
  testAsReceiver();
} else {
  console.log(`
使い方:
  node claude-to-claude-test.js send [メッセージ]    # 送信モード
  node claude-to-claude-test.js receive              # 受信モード
  `);
}