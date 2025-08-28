# Codex tmux Driver

tmux経由でCodexを管理し、イベントをWebSocketで配信するツールです。
Codexからの頻繁な返答を整理・フィルタリングして、ChatGPT5さんとの協調作業を効率化します。

## 🎯 機能

- tmuxセッション内でCodexを実行・管理
- Codexの出力をリアルタイムでWebSocket配信
- パターン認識によるイベント分類（response/thinking/error/complete）
- フィルタリング機能（CodexFilterBox）で重要な情報のみ抽出
- 画面キャプチャ・履歴管理

## 📦 インストール

```bash
cd tools/codex-tmux-driver
npm install
```

## 🚀 使い方

### 1. ドライバ起動

```bash
# 基本起動
node codex-tmux-driver.js

# オプション指定
node codex-tmux-driver.js --session=my-codex --port=8767 --log=/tmp/codex.log
```

### 2. テストクライアント

```bash
# 別ターミナルで
node test-client.js
```

### 3. WebSocket API

```javascript
// 接続
const ws = new WebSocket('ws://localhost:8766');

// Codexに入力送信
ws.send(JSON.stringify({
  op: 'send',
  data: 'Nyashの箱作戦について教えて'
}));

// 画面キャプチャ
ws.send(JSON.stringify({ op: 'capture' }));

// ステータス確認
ws.send(JSON.stringify({ op: 'status' }));

// 履歴取得
ws.send(JSON.stringify({ op: 'history', count: 20 }));

// イベントフィルタ
ws.send(JSON.stringify({ op: 'filter', event: 'response' }));
```

## 🎁 CodexFilterBox

Codexの出力を分類・フィルタリングする箱です。

```javascript
const CodexFilterBox = require('./codex-filter-box');
const filter = new CodexFilterBox();

// フィルタ実行
const result = filter.filter('Codex: バグ発見！重大な問題があります');
// → { category: 'urgent', priority: 'high', forward: true, ... }

// カスタムルール追加
filter.addRule('nyash-specific', {
  patterns: ['箱作戦', 'Everything is Box'],
  action: 'forward-to-chatgpt5',
  priority: 'medium',
  forward: true
});
```

### フィルタカテゴリ

- **urgent**: 緊急対応が必要（バグ、セキュリティ）
- **implementation**: 実装完了通知
- **proposal**: 提案・相談（キューに保存）
- **thinking**: 思考中（ログのみ）
- **ignore**: 無視可能な雑談

## 🔧 設定

### 環境変数
```bash
export CODEX_SESSION=my-codex
export CODEX_PORT=8767
export CODEX_LOG_DIR=/var/log/codex
export CODEX_HOOK_ENTER=crlf  # Enter送信方式: lf|cr|crlf (デフォルト: crlf)
export HOOK_SERVER_PORT=8769  # hook-serverのポート
export HOOK_SERVER_AUTO_EXIT=true   # 最後のhook切断で自動終了
export HOOK_IDLE_EXIT_MS=2000       # 自動終了までの猶予(ms)
```

### tmuxセッションのカスタマイズ
```javascript
// codex-tmux-driver.js の CODEX_CMD を変更
const CODEX_CMD = argv.cmd || 'codex exec --mode=assistant';
```

## 📊 統計情報

```javascript
// フィルタ統計
const stats = filter.getStats();
console.log(stats);
// → { total: 100, filtered: { urgent: 5, ... }, forwarded: 15, queued: 10 }
```

## 🎯 活用例

### ChatGPT5との連携

```javascript
// Codexの重要な出力のみChatGPT5に転送
ws.on('message', (data) => {
  const msg = JSON.parse(data);
  if (msg.type === 'codex-event') {
    const filtered = filter.filter(msg.data);
    
    if (filtered.forward) {
      // ChatGPT5のAPIに転送
      forwardToChatGPT5(filtered);
    }
  }
});
```

### 定期レビュー

```javascript
// 1時間ごとにキューを確認
setInterval(() => {
  const queue = filter.getQueue();
  if (queue.length > 0) {
    console.log('📋 Review queue:', queue);
    // 必要なものだけChatGPT5に相談
  }
}, 3600000);
```

## 🐛 トラブルシューティング

### tmuxセッションが作成できない
```bash
# 既存セッションを確認
tmux ls

# 既存セッションを削除
tmux kill-session -t codex-session
```

### ログファイルが大きくなりすぎる
```bash
# ログローテーション設定
echo "0 * * * * truncate -s 0 /tmp/codex.log" | crontab -
```

## 🌉 Codex-Claude 自動ブリッジ

Codexが止まったときに自動的にClaudeに転送し、応答を返すシステムです。

### 起動方法

```bash
# 1. Codex tmuxドライバを起動
node codex-tmux-driver.js

# 2. 別ターミナルでブリッジを起動
node codex-claude-bridge.js

# 3. ブリッジ制御（別ターミナル）
node bridge-control.js
```

### 単独インスタンス運用（tmuxなし・自動終了）

```bash
# Aインスタンス用hook-server（バックグラウンド、自動終了有効）
HOOK_SERVER_PORT=8769 HOOK_SERVER_AUTO_EXIT=true \
  nohup node tools/codex-tmux-driver/hook-server.js >/tmp/hook-A.log 2>&1 &

# AインスタンスのCodex（同ターミナル）
export CODEX_HOOK_SERVER=ws://localhost:8769
export CODEX_LOG_FILE=/tmp/codex-A.log
codex exec --ask-for-approval never --sandbox danger-full-access
# ← Codex終了時にhook-serverも自動終了
```

### ブリッジの仕組み

```
Codex停止 → 検出 → フィルタ → Claude API → Codexに返信
```

### 安全機能

- **レート制限**: 1時間に最大50回
- **クールダウン**: 5秒間隔
- **フィルタリング**: 危険なコマンドをブロック
- **確認キュー**: 重要な操作は人間確認

### 制御コマンド

```
status    - ブリッジの状態確認
queue     - 保留中の項目表示
approve N - キューのN番目を承認
toggle    - ブリッジのON/OFF
```

### 設定（環境変数）

```bash
export CLAUDE_API_URL=http://localhost:8080/claude
export BRIDGE_MAX_PER_HOUR=30
export BRIDGE_COOLDOWN_MS=10000
```

## 🚀 NEW! 同一hook-server双方向通信

同じhook-serverを使って、CodexからClaudeへの返信も可能に！

### 仕組み

```
Claude → hook-server → Codex（既存）
Codex → hook-server → Claude（新機能！）

同じWebSocketで双方向通信が実現！
```

### Codexから返信する方法

1. **Codex側でWebSocketクライアントを作成**
```javascript
// Codex側のコード
const WebSocket = require('ws');
const ws = new WebSocket('ws://localhost:8770');

// Claudeへメッセージを送信
ws.send(JSON.stringify({
  source: 'codex',
  type: 'inject-input', 
  data: 'Claudeさん、処理が完了しました！結果は...'
}));
```

2. **hook-serverが自動的にリレー**
- sourceが'codex'のメッセージを検出
- 'claude'タイプのクライアントに転送
- Claudeの画面に表示される！

### 実装例：作業完了通知

```javascript
// Codex側：作業完了時に自動通知
function notifyClaude(message) {
  const ws = new WebSocket('ws://localhost:8770');
  ws.on('open', () => {
    ws.send(JSON.stringify({
      source: 'codex',
      type: 'inject-input',
      data: message
    }));
    ws.close();
  });
}

// 使用例
notifyClaude('ビルドが完了しました！エラー0件、警告2件です。');
```

### tmux-perfect-bridgeとの統合

```javascript
// 完全自動双方向ブリッジ（同一hook-server版）
class UnifiedBridge {
  constructor() {
    this.hookServer = 'ws://localhost:8770';
  }
  
  // Codexの出力を監視してClaudeへ転送
  async watchCodexOutput() {
    const output = await this.captureCodexPane();
    if (this.isComplete(output)) {
      this.sendToClaude(output);
    }
  }
  
  // hook-server経由で送信
  sendToClaude(message) {
    const ws = new WebSocket(this.hookServer);
    ws.send(JSON.stringify({
      source: 'codex',
      type: 'inject-input', 
      data: message
    }));
  }
}
```

## 📝 今後の拡張

- [x] Codex-Claudeブリッジ
- [x] 双方向通信（同一hook-server）
- [ ] 複数Codexセッション管理
- [ ] フィルタルールの永続化（JSON/YAML）
- [ ] Web UIダッシュボード
- [ ] プラグインシステム（カスタムフィルタ）
- [ ] メトリクス出力（Prometheus形式）

---

Codexさんの頻繁な返答も、箱作戦で整理すれば怖くない！🎁
そして今や、Codexからも返事ができるように！🔄
