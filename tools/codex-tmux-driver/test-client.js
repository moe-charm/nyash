// test-client.js
// codex-tmux-driverのテスト用クライアント
// 使い方: node test-client.js

const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:8766');

// イベントハンドラー設定
ws.on('open', () => {
  console.log('[Connected] WebSocket接続成功');
  
  // ステータス確認
  ws.send(JSON.stringify({ op: 'status' }));
  
  // 履歴取得
  setTimeout(() => {
    ws.send(JSON.stringify({ op: 'history', count: 5 }));
  }, 1000);
});

ws.on('message', (data) => {
  const msg = JSON.parse(data);
  
  switch (msg.type) {
    case 'codex-event':
      console.log(`[Codex ${msg.event}] ${msg.data}`);
      
      // 応答があったら自動で画面キャプチャ
      if (msg.event === 'response' || msg.event === 'complete') {
        ws.send(JSON.stringify({ op: 'capture' }));
      }
      break;
      
    case 'status':
      console.log('[Status]', msg.data);
      break;
      
    case 'screen-capture':
      console.log('[Screen Capture]\n' + msg.data);
      break;
      
    case 'history':
      console.log('[History]');
      msg.data.forEach(event => {
        console.log(`  ${event.timestamp}: ${event.event || 'output'} - ${event.data}`);
      });
      break;
      
    case 'error':
      console.error('[Error]', msg.data);
      break;
      
    default:
      console.log(`[${msg.type}]`, msg.data);
  }
});

ws.on('error', (err) => {
  console.error('[WebSocket Error]', err);
});

ws.on('close', () => {
  console.log('[Disconnected] 接続終了');
});

// 標準入力から質問を受け付ける
const readline = require('readline');
const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  prompt: 'Codex> '
});

rl.prompt();

rl.on('line', (line) => {
  const cmd = line.trim();
  
  if (cmd === 'exit' || cmd === 'quit') {
    ws.close();
    rl.close();
    return;
  }
  
  if (cmd === 'capture') {
    ws.send(JSON.stringify({ op: 'capture' }));
  } else if (cmd === 'status') {
    ws.send(JSON.stringify({ op: 'status' }));
  } else if (cmd.startsWith('filter ')) {
    const event = cmd.split(' ')[1];
    ws.send(JSON.stringify({ op: 'filter', event }));
  } else if (cmd) {
    // 通常の入力はCodexに送信
    ws.send(JSON.stringify({ op: 'send', data: cmd }));
  }
  
  rl.prompt();
});

console.log('=== Codex tmux Driver Test Client ===');
console.log('Commands:');
console.log('  <text>     - Send text to Codex');
console.log('  capture    - Capture current screen');
console.log('  status     - Show status');
console.log('  filter <event> - Filter events (response/thinking/error/complete)');
console.log('  exit/quit  - Exit');
console.log('');