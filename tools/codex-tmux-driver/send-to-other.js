#!/usr/bin/env node
// send-to-other.js - 他のCodexセッションにメッセージ送信

const WebSocket = require('ws');

const message = process.argv[2];
const to = process.argv[3];
const from = process.argv[4] || 'unknown';

if (!message || !to) {
  console.log('使い方: node send-to-other.js "メッセージ" 送信先 [送信元名]');
  console.log('例: node send-to-other.js "こんにちは" codex2 codex1');
  process.exit(1);
}

const ws = new WebSocket('ws://localhost:8770');
ws.on('open', () => {
  console.log(`📤 ${from} → ${to}: "${message}"`);
  ws.send(JSON.stringify({
    type: 'inject-input',
    data: message,
    source: from,
    target: to
  }));
  setTimeout(() => {
    console.log('✅ Message sent!');
    ws.close();
    process.exit(0);
  }, 500);
});

ws.on('error', (e) => {
  console.error('❌ Error:', e.message);
  process.exit(1);
});