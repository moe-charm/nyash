#!/bin/bash
# 安全にラッパーをテストするスクリプト

echo "🧪 Codex Wrapper Safe Test"
echo ""

# 1. バイナリ確認
echo "1️⃣ Checking Codex binary..."
REAL_CODEX=/home/tomoaki/.volta/bin/codex
if [ -f "$REAL_CODEX" ]; then
    echo "✅ Found: $REAL_CODEX"
    $REAL_CODEX --version
else
    echo "❌ Not found: $REAL_CODEX"
    exit 1
fi

echo ""
echo "2️⃣ Testing wrapper (hook disabled)..."
export CODEX_REAL_BIN=$REAL_CODEX
export CODEX_HOOK_ENABLE=false
cd $(dirname "$0")
node codex-hook-wrapper.js --version

echo ""
echo "3️⃣ Testing wrapper (hook enabled, port 8770)..."
export CODEX_HOOK_ENABLE=true
export CODEX_HOOK_SERVER=ws://localhost:8770
export CODEX_LOG_FILE=/tmp/codex-test.log
echo "Will try to connect to $CODEX_HOOK_SERVER"
node codex-hook-wrapper.js --version

echo ""
echo "✅ Wrapper test complete!"
echo ""
echo "To use with real Codex:"
echo "  export CODEX_REAL_BIN=$REAL_CODEX"
echo "  export CODEX_HOOK_SERVER=ws://localhost:8770"
echo "  node codex-hook-wrapper.js exec --ask-for-approval never"