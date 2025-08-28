# 🚀 クイックスタート - Claude Code ↔ Codex 双方向通信

## 前提条件
- **Claude Code**: `/home/tomoaki/.volta/bin/codex` （Claude APIを使うCodex）
- **本物のCodex**: 別途パスを設定（制限解除の引数が必要）

## 環境設定
```bash
# 本物のCodexのパスを設定（必須）
export REAL_CODEX_PATH=/path/to/real/codex
```

## 一括起動（推奨）
```bash
cd /mnt/c/git/nyash-project/nyash
./tools/codex-tmux-driver/start-all.sh
```

## 個別起動

### 1. Hook Server起動
```bash
node tools/codex-tmux-driver/hook-server.js
```

### 2. Claude Code（c1）起動
```bash
./tools/codex-tmux-driver/start-ai-tmux.sh c1 /home/tomoaki/.volta/bin/codex
```

### 3. 本物のCodex（c2）起動
```bash
./tools/codex-tmux-driver/start-ai-tmux.sh c2 $REAL_CODEX_PATH --ask-for-approval never --sandbox danger-full-access
```

## メッセージ送信テスト

### Codex → Claude Code
```bash
node tools/codex-tmux-driver/test-bidirectional-claude-codex.js
```

### Claude Code → Codex
```bash
node tools/codex-tmux-driver/test-bidirectional-codex-claude.js
```

## セッション管理

### 接続
```bash
tmux attach -t c1  # Claude Codeに接続
tmux attach -t c2  # 本物のCodexに接続
```

### 終了
```bash
pkill -f hook-server.js
tmux kill-session -t c1
tmux kill-session -t c2
```

## トラブルシューティング

### 本物のCodexが見つからない
```bash
# Codexのパスを確認
which codex

# 環境変数に設定
export REAL_CODEX_PATH=$(which codex)
```

### ポートが使用中
```bash
# 8770ポートを確認
lsof -i:8770

# プロセスを終了
pkill -f hook-server.js
```