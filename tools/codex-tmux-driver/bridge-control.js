// bridge-control.js
// Codex-Claudeブリッジの制御用CLI

const WebSocket = require('ws');
const readline = require('readline');

const ws = new WebSocket('ws://localhost:8768');
const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  prompt: 'Bridge> '
});

// 接続時の処理
ws.on('open', () => {
  console.log('🌉 Connected to Codex-Claude Bridge');
  console.log('Commands:');
  console.log('  status    - Show bridge status');
  console.log('  queue     - Show pending items');
  console.log('  approve N - Approve queue item N');
  console.log('  toggle    - Enable/disable bridge');
  console.log('  exit      - Quit');
  console.log('');
  
  // 初期ステータス取得
  ws.send(JSON.stringify({ op: 'status' }));
  
  rl.prompt();
});

ws.on('message', (data) => {
  const msg = JSON.parse(data);
  
  switch (msg.type) {
    case 'status':
      console.log('\n📊 Bridge Status:');
      console.log(`  Active: ${msg.state.active ? '✅' : '❌'}`);
      console.log(`  Bridges: ${msg.state.bridgeCount}`);
      console.log(`  Queue: ${msg.state.queue.length} items`);
      console.log('\n📈 Statistics:');
      console.log(`  Total: ${msg.stats.total}`);
      console.log(`  Forwarded: ${msg.stats.forwarded}`);
      console.log(`  Blocked: ${msg.stats.blocked}`);
      console.log(`  Queued: ${msg.stats.queued}`);
      break;
      
    case 'queue':
      console.log('\n📋 Pending Queue:');
      if (msg.items.length === 0) {
        console.log('  (empty)');
      } else {
        msg.items.forEach((item, idx) => {
          console.log(`  [${idx}] ${item.reason.reason}`);
          console.log(`       "${item.message.data.substring(0, 50)}..."`);
          console.log(`       ${new Date(item.timestamp).toLocaleTimeString()}`);
        });
      }
      break;
      
    case 'toggled':
      console.log(`\n🔄 Bridge is now ${msg.active ? 'ACTIVE' : 'INACTIVE'}`);
      break;
      
    default:
      console.log(`\n[${msg.type}]`, msg);
  }
  
  rl.prompt();
});

ws.on('error', (err) => {
  console.error('❌ Connection error:', err.message);
  process.exit(1);
});

ws.on('close', () => {
  console.log('\n🔌 Disconnected');
  process.exit(0);
});

// コマンド処理
rl.on('line', (line) => {
  const parts = line.trim().split(' ');
  const cmd = parts[0];
  
  switch (cmd) {
    case 'status':
      ws.send(JSON.stringify({ op: 'status' }));
      break;
      
    case 'queue':
      ws.send(JSON.stringify({ op: 'queue' }));
      break;
      
    case 'approve':
      const id = parseInt(parts[1]);
      if (!isNaN(id)) {
        ws.send(JSON.stringify({ op: 'approve', id }));
        console.log(`✅ Approving item ${id}...`);
      } else {
        console.log('❌ Usage: approve <number>');
      }
      break;
      
    case 'toggle':
      ws.send(JSON.stringify({ op: 'toggle' }));
      break;
      
    case 'exit':
    case 'quit':
      ws.close();
      rl.close();
      return;
      
    default:
      if (cmd) {
        console.log(`❓ Unknown command: ${cmd}`);
      }
  }
  
  rl.prompt();
});