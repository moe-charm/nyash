# 🔄 Codex ↔ Claude 双方向通信ガイド

## 📋 概要
同じWebSocketサーバー（hook-server）を使って、CodexとClaudeが双方向に通信できます！

## 🎯 仕組み
```
Claude → hook-server (8770) → Codex ✅ （既に動作中）
Codex  → hook-server (8770) → Claude ✅ （NEW!）
```

## 📡 Claudeへの返信方法

### 基本的な返信
```javascript
// ClaudeへWebSocket経由で返信
const WebSocket = require('ws');
const ws = new WebSocket('ws://localhost:8770');

ws.on('open', () => {
  ws.send(JSON.stringify({
    source: 'codex',           // 送信元を'codex'に
    type: 'inject-input',      // メッセージタイプ
    data: 'ここに返信内容を書く'  // 実際のメッセージ
  }));
  ws.close();
});
```

### 実用的な関数
```javascript
// 再利用可能な返信関数
function replyToClaude(message) {
  const WebSocket = require('ws');
  const ws = new WebSocket('ws://localhost:8770');
  
  ws.on('open', () => {
    ws.send(JSON.stringify({
      source: 'codex',
      type: 'inject-input',
      data: `[Codex→Claude] ${message}`
    }));
    console.log('✅ Sent to Claude:', message);
    ws.close();
  });
  
  ws.on('error', (err) => {
    console.error('❌ Error:', err.message);
  });
}

// 使用例
replyToClaude('ビルドが完了しました！');
replyToClaude('テストの結果: 成功 10件、失敗 0件');
```

### 進捗通知の例
```javascript
// タスクの進捗を通知
async function notifyProgress(task, status, details) {
  const message = `
📊 進捗報告
タスク: ${task}
状態: ${status}
詳細: ${details}
時刻: ${new Date().toLocaleString('ja-JP')}
`;
  
  replyToClaude(message.trim());
}

// 実行例
notifyProgress('Nyashビルド', '完了', 'cargo build --release 成功');
notifyProgress('テスト実行', '進行中', '5/10 完了...');
```

## 🎮 実践例

### 1. コマンド実行結果の返信
```javascript
const { exec } = require('child_process');

// コマンドを実行してClaudeに結果を送る
function executeAndReply(command) {
  exec(command, (error, stdout, stderr) => {
    if (error) {
      replyToClaude(`❌ エラー: ${command}\n${stderr}`);
    } else {
      replyToClaude(`✅ 成功: ${command}\n出力:\n${stdout}`);
    }
  });
}

// 使用例
executeAndReply('cargo check');
executeAndReply('ls -la');
```

### 2. ファイル操作の通知
```javascript
const fs = require('fs');

// ファイル作成を通知
function notifyFileCreated(filename, content) {
  fs.writeFileSync(filename, content);
  replyToClaude(`📄 ファイル作成: ${filename} (${content.length}バイト)`);
}

// ファイル読み込みと返信
function readAndReply(filename) {
  try {
    const content = fs.readFileSync(filename, 'utf8');
    replyToClaude(`📖 ${filename} の内容:\n${content.substring(0, 200)}...`);
  } catch (err) {
    replyToClaude(`❌ ファイル読み込みエラー: ${filename}`);
  }
}
```

## ⚡ クイックテスト

以下のワンライナーでテスト可能：
```javascript
// すぐに試せるテストコード
require('ws').connect('ws://localhost:8770').on('open', function() { this.send(JSON.stringify({ source: 'codex', type: 'inject-input', data: 'Hello Claude! 双方向通信テスト成功！' })); this.close(); });
```

## 📝 注意事項

1. **hook-serverが起動していること**を確認
   ```bash
   lsof -i:8770  # ポートが開いているか確認
   ```

2. **sourceは必ず'codex'に設定**
   - 'claude'にすると自分自身にループバックしてしまう

3. **メッセージ形式を守る**
   - JSONで、source, type, dataの3つのフィールドが必須

## 🚀 活用アイデア

- **自動進捗報告**: 長時間かかるタスクの進捗をリアルタイム通知
- **エラー通知**: 問題発生時に即座にClaudeに通知
- **完了通知**: タスク完了時に次の指示を求める
- **質問**: 判断に迷ったときにClaudeに相談

これで、CodexとClaudeが完全に双方向で協調作業できます！🎉