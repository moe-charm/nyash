# 🌉 Codex-Claude Auto Bridge

## 🎯 機能

CodexとClaudeの間で応答を自動的に橋渡しするシステムです。

### できること
- ✅ Codexの出力を自動検知
- ✅ 出力完了を判定（Working状態の終了を検知）
- ✅ 応答内容を抽出してファイルに保存
- ✅ Claudeが読める形式で出力
- ✅ tmux経由でCodexにメッセージ送信

## 📦 構成

1. **codex-output-watcher.js** - Codexの画面を監視
2. **codex-claude-auto-bridge.js** - 自動橋渡しシステム
3. **tmux-codex-controller.js** - tmux制御

## 🚀 使い方

### 1. Codexをtmuxで起動
```bash
./tmux-launch-only.sh
```

### 2. 自動ブリッジを起動
```bash
node codex-claude-auto-bridge.js
```

### 3. 最初のメッセージを送る
```bash
node codex-claude-auto-bridge.js "Nyashプロジェクトについて教えて"
```

### 4. Codexの応答を確認
```bash
cat codex-response.txt
```

### 5. 応答を読んで次のメッセージを送る
```bash
tmux send-keys -t codex-safe "次の質問" Enter
```

## 🔄 自動化フロー

```
Claude → メッセージ作成
  ↓
tmux send-keys → Codexに送信
  ↓
Codex → 処理中（Working...）
  ↓
codex-output-watcher → 完了検知
  ↓
codex-response.txt → 応答保存
  ↓
Claude → ファイルを読んで返答
```

## 💡 高度な使い方

### 監視だけする
```javascript
const watcher = new CodexOutputWatcher();
watcher.on('response', (response) => {
  console.log('Got response:', response);
});
watcher.start();
```

### プログラムから制御
```javascript
const bridge = new CodexClaudeAutoBridge();
await bridge.start();
await bridge.sendToCodex("質問");
// codex-response.txt に応答が保存される
```

## ⚠️ 注意事項

- Codexが勝手に動作しないよう監視が必要
- tmuxセッションは使用後に必ず終了する
- 応答ファイルは上書きされるので注意

## 🐛 トラブルシューティング

**Q: 応答が検出されない**
A: Working状態が終わるまで待ってください

**Q: 文字化けする**
A: ANSIエスケープシーケンスが含まれている可能性があります

**Q: tmuxエラー**
A: セッション名が正しいか確認してください