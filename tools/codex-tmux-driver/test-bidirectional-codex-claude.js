#!/usr/bin/env node
// ClaudeからCodexへ双方向通信テスト

const WebSocket = require('ws');

// Hook serverに接続
const ws = new WebSocket('ws://localhost:8770');

ws.on('open', () => {
  console.log('✅ WebSocket接続成功！');
  
  // ClaudeからCodexへメッセージ送信
  const message = {
    source: 'claude',
    type: 'inject-input',
    data: '🌟 Claudeから双方向通信テスト成功！逆方向も動作しています！'
  };
  
  console.log('📤 送信メッセージ:', message);
  ws.send(JSON.stringify(message));
  
  // 送信後すぐに接続を閉じる
  setTimeout(() => {
    ws.close();
    console.log('🔌 接続を閉じました');
  }, 100);
});

ws.on('error', (err) => {
  console.error('❌ エラー:', err.message);
});

ws.on('close', () => {
  console.log('👋 WebSocket接続終了');
});