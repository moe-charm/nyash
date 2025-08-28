# 🎉 Codex Hook 成功！

ついにCodexとhook-serverの連携が成功したにゃ！

## 📝 正しい使い方

### 1. hook-serverを起動（ポート8770）
```bash
HOOK_SERVER_PORT=8770 node tools/codex-tmux-driver/hook-server.js
```

### 2. Codexを起動（クリーン版）
```bash
./tools/codex-tmux-driver/start-codex-simple.sh
```

### 3. メッセージを送る
```bash
node tools/codex-tmux-driver/send-greeting-clean.js
```

## 🐛 トラブルシューティング

画面がぐちゃぐちゃになったら：
- Codexを再起動して`start-codex-simple.sh`を使う（デバッグ出力なし）
- または環境変数で制御：`export CODEX_HOOK_BANNER=false`

## 🎯 次のステップ

- Claude-Codexブリッジの実装
- 自動応答システムの構築
- フィルタリング機能の追加

やったにゃー！🐱🎉