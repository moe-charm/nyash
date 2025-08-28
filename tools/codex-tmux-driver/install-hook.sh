#!/bin/bash
# Codexフックをインストールするスクリプト

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
WRAPPER_SCRIPT="$SCRIPT_DIR/codex-hook-wrapper.js"
HOOK_DIR="$HOME/.local/bin"
HOOK_TARGET="$HOOK_DIR/codex"

# .local/binディレクトリを作成
mkdir -p "$HOOK_DIR"

# 既存のcodexバックアップ
if [ -f "$HOOK_TARGET" ] && [ ! -L "$HOOK_TARGET" ]; then
    echo "Backing up existing codex to codex.original"
    mv "$HOOK_TARGET" "$HOOK_TARGET.original"
fi

# ラッパースクリプトをシンボリックリンクで配置（node_modules解決のため）
echo "Installing codex hook wrapper (symlink)..."
ln -sf "$WRAPPER_SCRIPT" "$HOOK_TARGET"
chmod +x "$HOOK_TARGET"

# PATHの設定確認
if [[ ":$PATH:" != *":$HOOK_DIR:"* ]]; then
    echo ""
    echo "⚠️  Please add $HOOK_DIR to your PATH:"
    echo "    export PATH=\"$HOOK_DIR:\$PATH\""
    echo ""
    echo "Add this to your ~/.bashrc or ~/.zshrc"
fi

# 環境変数の説明
echo ""
echo "✅ Codex hook installed!"
echo ""
echo "Configuration (environment variables):"
echo "  CODEX_HOOK_ENABLE=true    # Enable/disable hook (default: true)"
echo "  CODEX_HOOK_SERVER=ws://localhost:8769  # WebSocket server"
echo "  CODEX_LOG_FILE=/tmp/codex-hook.log     # Log file location"
echo "  CODEX_HOOK_ENTER=crlf                  # Enter mode: lf|cr|crlf (default: crlf)"
echo ""
echo "To test:"
echo "  # Install dependencies if not yet"
echo "  (cd $SCRIPT_DIR && npm install)"
echo "  codex --version"
echo "  tail -f /tmp/codex-hook.log  # Watch logs"
echo ""
echo "To uninstall:"
echo "  rm $HOOK_TARGET"
if [ -f "$HOOK_TARGET.original" ]; then
    echo "  mv $HOOK_TARGET.original $HOOK_TARGET"
fi
