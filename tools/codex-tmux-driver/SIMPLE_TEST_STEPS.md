# 🚀 超シンプルテスト手順

## 1️⃣ hook-server起動（まだなら）
```bash
cd /mnt/c/git/nyash-project/nyash/tools/codex-tmux-driver
HOOK_SERVER_PORT=8770 node hook-server.js
```

## 2️⃣ Claude Codeから送信テスト

### 方法A: ワンライナー（一番簡単）
```javascript
require('ws').connect('ws://localhost:8770').on('open', function() { this.send(JSON.stringify({ source: 'claude-test', type: 'inject-input', data: 'テスト成功！' })); this.close(); });
```

### 方法B: 分かりやすい版
```javascript
const ws = require('ws');
const client = new ws('ws://localhost:8770');
client.on('open', () => {
  client.send(JSON.stringify({
    source: 'claude',
    type: 'inject-input',
    data: 'Hello! WebSocketテスト成功！'
  }));
  client.close();
});
```

## 3️⃣ 確認方法
hook-serverのターミナルに以下が表示されれば成功：
```
[inject-input] Hello! WebSocketテスト成功！
🔄 Relaying inject-input from hook client
```

## 🎯 成功したら
同じ方法でCodexからも送信できます！
sourceを'codex'に変えるだけ！