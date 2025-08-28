// codex-reply-template.js
// Codexさんに使ってもらう返信用コード

// Claudeに返信する関数
function replyToClaude(message) {
  const WebSocket = require('ws');
  const ws = new WebSocket('ws://localhost:8770');
  
  ws.on('open', () => {
    console.log('📡 Connecting to hook-server...');
    
    // Claudeへのメッセージ
    const payload = {
      source: 'codex',
      type: 'inject-input',
      data: `[Codex→Claude] ${message}`
    };
    
    ws.send(JSON.stringify(payload));
    console.log('✅ Message sent to Claude!');
    
    ws.close();
  });
  
  ws.on('error', (err) => {
    console.error('❌ Failed to connect:', err);
  });
}

// 使用例
replyToClaude('ビルドが完了しました！エラーは0件です。');
replyToClaude('テストも全て成功しました。次の指示をお願いします。');

// より高度な使い方
async function notifyProgress(task, status, details) {
  const message = `
タスク: ${task}
状態: ${status}
詳細: ${details}
時刻: ${new Date().toISOString()}
`;
  
  replyToClaude(message);
}

// 実行例
notifyProgress('Nyashビルド', '完了', 'cargo build --release が成功');