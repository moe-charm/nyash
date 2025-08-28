#!/bin/bash
# 複数Codexインスタンスの一括管理
# 使い方: ./manage-instances.sh start
#        ./manage-instances.sh status
#        ./manage-instances.sh stop

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PIDFILE="/tmp/codex-instances.pid"

# インスタンス定義
declare -A INSTANCES=(
    ["A"]="8769"
    ["B"]="8770"
    ["C"]="8771"
)

function start_instances() {
    echo "🚀 Starting all Codex instances..."
    
    for name in "${!INSTANCES[@]}"; do
        port="${INSTANCES[$name]}"
        echo ""
        echo "Starting instance $name on port $port..."
        
        # hook-server起動
        HOOK_SERVER_PORT=$port HOOK_SERVER_AUTO_EXIT=true \
            nohup node "$SCRIPT_DIR/hook-server.js" \
            > "/tmp/hook-$name.log" 2>&1 &
        
        pid=$!
        echo "$name:$port:$pid" >> "$PIDFILE"
        echo "  Hook server PID: $pid"
        
        # 環境変数の出力
        echo "  For instance $name, use:"
        echo "    export CODEX_HOOK_SERVER=ws://localhost:$port"
        echo "    export CODEX_LOG_FILE=/tmp/codex-$name.log"
        echo "    codex exec"
    done
    
    echo ""
    echo "✅ All instances started!"
}

function status_instances() {
    echo "📊 Codex instances status:"
    echo ""
    
    if [ ! -f "$PIDFILE" ]; then
        echo "No instances found."
        return
    fi
    
    while IFS=: read -r name port pid; do
        if kill -0 "$pid" 2>/dev/null; then
            echo "✅ Instance $name (port $port): Running [PID: $pid]"
            
            # 接続数の確認
            connections=$(lsof -i :$port 2>/dev/null | grep ESTABLISHED | wc -l)
            echo "   Connections: $connections"
        else
            echo "❌ Instance $name (port $port): Stopped"
        fi
    done < "$PIDFILE"
}

function stop_instances() {
    echo "🛑 Stopping all Codex instances..."
    
    if [ ! -f "$PIDFILE" ]; then
        echo "No instances to stop."
        return
    fi
    
    while IFS=: read -r name port pid; do
        if kill -0 "$pid" 2>/dev/null; then
            echo "Stopping instance $name [PID: $pid]..."
            kill "$pid"
        fi
    done < "$PIDFILE"
    
    rm -f "$PIDFILE"
    echo "✅ All instances stopped!"
}

function logs_instances() {
    echo "📜 Showing recent logs..."
    echo ""
    
    for name in "${!INSTANCES[@]}"; do
        echo "=== Instance $name ==="
        echo "Hook log (/tmp/hook-$name.log):"
        tail -5 "/tmp/hook-$name.log" 2>/dev/null || echo "  (no log)"
        echo ""
        echo "Codex log (/tmp/codex-$name.log):"
        tail -5 "/tmp/codex-$name.log" 2>/dev/null || echo "  (no log)"
        echo ""
    done
}

# コマンド処理
case "$1" in
    start)
        start_instances
        ;;
    stop)
        stop_instances
        ;;
    status)
        status_instances
        ;;
    logs)
        logs_instances
        ;;
    restart)
        stop_instances
        sleep 2
        start_instances
        ;;
    *)
        echo "Usage: $0 {start|stop|status|logs|restart}"
        echo ""
        echo "Configured instances:"
        for name in "${!INSTANCES[@]}"; do
            echo "  $name: port ${INSTANCES[$name]}"
        done
        exit 1
        ;;
esac