#!/bin/bash
# 複数Codexインスタンスを簡単に起動するスクリプト
# 使い方: ./start-instance.sh A 8769
#        ./start-instance.sh B 8770 --foreground

INSTANCE_NAME="${1:-A}"
PORT="${2:-8769}"
FOREGROUND=false

# オプション解析
if [[ "$3" == "--foreground" ]] || [[ "$3" == "-f" ]]; then
    FOREGROUND=true
fi

# カラー定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 Starting Codex Instance ${INSTANCE_NAME} on port ${PORT}${NC}"

# hook-serverの起動
if [ "$FOREGROUND" = true ]; then
    echo -e "${YELLOW}Starting hook-server in foreground...${NC}"
    echo "Commands:"
    echo "  export CODEX_HOOK_SERVER=ws://localhost:${PORT}"
    echo "  export CODEX_LOG_FILE=/tmp/codex-${INSTANCE_NAME}.log"
    echo "  codex exec"
    echo ""
    
    HOOK_SERVER_PORT=$PORT HOOK_SERVER_AUTO_EXIT=false \
        node tools/codex-tmux-driver/hook-server.js
else
    # バックグラウンドで起動
    echo -e "${YELLOW}Starting hook-server in background...${NC}"
    
    HOOK_SERVER_PORT=$PORT HOOK_SERVER_AUTO_EXIT=true \
        nohup node tools/codex-tmux-driver/hook-server.js \
        > /tmp/hook-${INSTANCE_NAME}.log 2>&1 &
    
    HOOK_PID=$!
    echo "Hook server PID: $HOOK_PID"
    
    # 起動確認
    sleep 1
    if kill -0 $HOOK_PID 2>/dev/null; then
        echo -e "${GREEN}✅ Hook server started successfully${NC}"
    else
        echo -e "${RED}❌ Hook server failed to start${NC}"
        echo "Check log: /tmp/hook-${INSTANCE_NAME}.log"
        exit 1
    fi
    
    # Codex起動コマンドの表示
    echo ""
    echo "Now run these commands in another terminal:"
    echo -e "${GREEN}export CODEX_HOOK_SERVER=ws://localhost:${PORT}${NC}"
    echo -e "${GREEN}export CODEX_LOG_FILE=/tmp/codex-${INSTANCE_NAME}.log${NC}"
    echo -e "${GREEN}codex exec --ask-for-approval never${NC}"
    echo ""
    echo "To monitor:"
    echo "  tail -f /tmp/hook-${INSTANCE_NAME}.log"
    echo "  tail -f /tmp/codex-${INSTANCE_NAME}.log"
fi