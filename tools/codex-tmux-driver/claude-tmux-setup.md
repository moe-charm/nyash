# 🚀 Claude Code × tmux セットアップ完全ガイド

## 📋 概要
Claude Codeもtmuxで動かすことで、Codexとの完璧な双方向通信を実現します！

## 🎯 手順（1から）

### 1️⃣ 現在のClaude Codeを終了
```bash
# 現在のセッションを保存して終了
exit
```

### 2️⃣ tmuxセッションでClaude Codeを起動
```bash
# 新しいtmuxセッションを作成（名前: claude-8771）
tmux new-session -d -s claude-8771

# Claude Codeをtmuxセッションで起動
tmux send-keys -t claude-8771 "cd /mnt/c/git/nyash-project/nyash" Enter
tmux send-keys -t claude-8771 "claude" Enter

# セッションにアタッチして作業
tmux attach -t claude-8771
```

### 3️⃣ hook-serverを起動（別ターミナル）
```bash
# 新しいターミナルを開いて
cd /mnt/c/git/nyash-project/nyash/tools/codex-tmux-driver
HOOK_SERVER_PORT=8770 node hook-server.js
```

### 4️⃣ Codexをtmuxで起動（さらに別ターミナル）
```bash
# 既存のスクリプトを使用
cd /mnt/c/git/nyash-project/nyash/tools/codex-tmux-driver
./tmux-launch-only.sh
```

### 5️⃣ 双方向ブリッジを起動（さらに別ターミナル）
```bash
cd /mnt/c/git/nyash-project/nyash/tools/codex-tmux-driver
node claude-codex-unified-bridge.js
```

## 🔄 完成図

```
┌─────────────────┐
│ Terminal 1      │
│ tmux: claude    │ ←──┐
└─────────────────┘    │
                       │
┌─────────────────┐    │    ┌──────────────┐
│ Terminal 2      │    ├────┤ hook-server  │
│ hook-server     │    │    │ port: 8770   │
└─────────────────┘    │    └──────────────┘
                       │
┌─────────────────┐    │
│ Terminal 3      │    │
│ tmux: codex     │ ←──┘
└─────────────────┘

双方向自動通信！
```

## 💡 使い方

### Claude → Codex（従来通り）
```javascript
// Claude Code内で実行
const WebSocket = require('ws');
const ws = new WebSocket('ws://localhost:8770');
ws.send(JSON.stringify({
  source: 'claude',
  type: 'inject-input',
  data: 'Hello Codex!'
}));
```

### Codex → Claude（新機能！）
Codexが自動的にClaudeに返信します（unified-bridgeが処理）

## 🎮 tmux基本操作

```bash
# セッション一覧
tmux ls

# セッションにアタッチ
tmux attach -t claude-8771

# デタッチ（セッションから抜ける）
Ctrl+B, D

# セッション削除
tmux kill-session -t claude-8771

# 画面分割（横）
Ctrl+B, "

# 画面分割（縦）
Ctrl+B, %

# ペーン間移動
Ctrl+B, 矢印キー
```

## ⚠️ 注意事項

1. **tmuxセッション名の重複**
   - claude-8771, codex-safe は固定名なので重複注意

2. **ポート番号**
   - 8770: hook-server（固定）
   - 変更する場合は全ての設定を統一

3. **終了時の手順**
   1. ブリッジを停止（Ctrl+C）
   2. hook-serverを停止（Ctrl+C）
   3. tmuxセッションを終了

## 🚨 トラブルシューティング

**Q: セッションが既に存在する**
```bash
tmux kill-session -t claude-8771
tmux kill-session -t codex-safe
```

**Q: hook-serverに接続できない**
```bash
# プロセスを確認
ps aux | grep "node.*hook-server"
# 強制終了
pkill -f "node.*hook-server"
```

**Q: メッセージが届かない**
- hook-serverのログを確認
- WebSocketの接続状態を確認
- tmuxセッション名が正しいか確認