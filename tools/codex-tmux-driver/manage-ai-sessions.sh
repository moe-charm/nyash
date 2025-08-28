#!/bin/bash
# 複数AI セッション管理スクリプト

# デフォルト設定
CLAUDE_BIN=${CLAUDE_BIN:-"/home/tomoaki/.volta/bin/codex"}
CODEX_BIN=${REAL_CODEX_BIN:-"/path/to/real/codex"}
HOOK_PORT=${HOOK_SERVER_PORT:-8770}

# カラー設定
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

function show_usage() {
    echo "🤖 AI セッション管理ツール"
    echo ""
    echo "使い方: $0 <command> [options]"
    echo ""
    echo "コマンド:"
    echo "  start-all     - Claude1, Claude2, Codexを全て起動"
    echo "  start-claude  - Claude Code 2つを起動"
    echo "  start-codex   - 本物のCodexを起動"
    echo "  status        - 全セッション状態を表示"
    echo "  send <session> <message> - 特定セッションにメッセージ送信"
    echo "  broadcast <message>      - 全セッションにメッセージ送信"
    echo "  kill-all      - 全セッション終了"
    echo "  attach <session>         - セッションに接続"
    echo ""
    echo "セッション名:"
    echo "  claude1-8770  - Claude Code インスタンス1"
    echo "  claude2-8770  - Claude Code インスタンス2"
    echo "  codex-8770    - 本物のCodex"
}

function start_claude_sessions() {
    echo -e "${BLUE}🚀 Claude Code セッションを起動中...${NC}"
    ./start-ai-tmux.sh claude1-8770 "$CLAUDE_BIN"
    sleep 1
    ./start-ai-tmux.sh claude2-8770 "$CLAUDE_BIN"
    echo -e "${GREEN}✅ Claude Code 2つ起動完了！${NC}"
}

function start_codex_session() {
    echo -e "${BLUE}🚀 本物のCodexを起動中...${NC}"
    # Codexには制限解除のための引数が必要
    ./start-ai-tmux.sh codex-8770 "$CODEX_BIN" --ask-for-approval never --sandbox danger-full-access
    echo -e "${GREEN}✅ Codex起動完了！${NC}"
}

function show_status() {
    echo -e "${BLUE}📊 AIセッション状態:${NC}"
    echo ""
    
    for session in claude1-8770 claude2-8770 codex-8770; do
        if tmux has-session -t "$session" 2>/dev/null; then
            echo -e "  ${GREEN}✅${NC} $session - 稼働中"
        else
            echo -e "  ${RED}❌${NC} $session - 停止"
        fi
    done
    
    echo ""
    echo -e "${YELLOW}Hook Server状態:${NC}"
    if lsof -i:$HOOK_PORT >/dev/null 2>&1; then
        echo -e "  ${GREEN}✅${NC} Hook server (port $HOOK_PORT) - 稼働中"
    else
        echo -e "  ${RED}❌${NC} Hook server (port $HOOK_PORT) - 停止"
        echo -e "  ${YELLOW}💡${NC} 起動するには: HOOK_SERVER_PORT=$HOOK_PORT node hook-server.js"
    fi
}

function send_to_session() {
    local session="$1"
    local message="$2"
    
    if tmux has-session -t "$session" 2>/dev/null; then
        tmux send-keys -t "$session" "$message" Enter
        echo -e "${GREEN}✅${NC} メッセージを $session に送信しました"
    else
        echo -e "${RED}❌${NC} セッション $session は存在しません"
    fi
}

function broadcast_message() {
    local message="$1"
    echo -e "${BLUE}📢 全セッションにブロードキャスト中...${NC}"
    
    for session in claude1-8770 claude2-8770 codex-8770; do
        send_to_session "$session" "$message"
    done
}

function kill_all_sessions() {
    echo -e "${RED}🛑 全セッションを終了中...${NC}"
    
    for session in claude1-8770 claude2-8770 codex-8770; do
        if tmux has-session -t "$session" 2>/dev/null; then
            tmux kill-session -t "$session"
            echo -e "  ${YELLOW}⚠️${NC}  $session を終了しました"
        fi
    done
    
    echo -e "${GREEN}✅ 完了${NC}"
}

# メインコマンド処理
case "$1" in
    start-all)
        start_claude_sessions
        start_codex_session
        show_status
        ;;
    start-claude)
        start_claude_sessions
        show_status
        ;;
    start-codex)
        start_codex_session
        show_status
        ;;
    status)
        show_status
        ;;
    send)
        if [ $# -lt 3 ]; then
            echo -e "${RED}❌ 使い方: $0 send <session> <message>${NC}"
            exit 1
        fi
        send_to_session "$2" "$3"
        ;;
    broadcast)
        if [ $# -lt 2 ]; then
            echo -e "${RED}❌ 使い方: $0 broadcast <message>${NC}"
            exit 1
        fi
        broadcast_message "$2"
        ;;
    kill-all)
        kill_all_sessions
        ;;
    attach)
        if [ $# -lt 2 ]; then
            echo -e "${RED}❌ 使い方: $0 attach <session>${NC}"
            exit 1
        fi
        tmux attach -t "$2"
        ;;
    *)
        show_usage
        ;;
esac