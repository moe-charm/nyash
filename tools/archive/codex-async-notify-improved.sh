#!/bin/bash
# codex-async-notify-improved.sh - tmux send-keys の信頼性向上版

# 使い方を表示
if [ $# -eq 0 ]; then
    echo "Usage: $0 <task description> [tmux_session]"
    echo "Examples:"
    echo "  $0 'Refactor MIR builder to 13 instructions'"
    echo "  $0 'Write paper introduction' gemini-session"
    echo "  $0 'Review code quality' chatgpt"
    echo ""
    echo "Default tmux session: codex (override with CODEX_DEFAULT_SESSION env or 2nd arg)"
    exit 1
fi

# 引数解析
TASK="$1"
# デフォルトは env `CODEX_DEFAULT_SESSION`、なければ "codex"
TARGET_SESSION="${2:-${CODEX_DEFAULT_SESSION:-codex}}"

# 設定
WORK_DIR="$HOME/.codex-async-work"
LOG_DIR="$WORK_DIR/logs"
WORK_ID=$(date +%s%N)
LOG_FILE="$LOG_DIR/codex-${WORK_ID}.log"

# 作業ディレクトリ準備
mkdir -p "$LOG_DIR"

# tmux send-keys with delay
send_keys_safe() {
    local session="$1"
    local text="$2"
    
    # Send text without Enter first
    tmux send-keys -t "$session" "$text"
    
    # Small delay before Enter
    sleep 0.05
    
    # Send Enter
    tmux send-keys -t "$session" Enter
    
    # Small delay after Enter  
    sleep 0.05
}

# 非同期実行関数
run_codex_async() {
    {
        # Detach: silence this background subshell's stdout/stderr while still tee-ing to log
        if [ "${CODEX_ASYNC_DETACH:-0}" = "1" ]; then
            exec >/dev/null 2>&1
        fi
        echo "=====================================" | tee "$LOG_FILE"
        echo "🚀 Codex Task Started" | tee -a "$LOG_FILE"
        echo "Work ID: $WORK_ID" | tee -a "$LOG_FILE"
        echo "Task: $TASK" | tee -a "$LOG_FILE"
        echo "Start: $(date)" | tee -a "$LOG_FILE"
        echo "=====================================" | tee -a "$LOG_FILE"
        echo "" | tee -a "$LOG_FILE"
        
        # Codex実行
        START_TIME=$(date +%s)
        codex exec "$TASK" 2>&1 | tee -a "$LOG_FILE"
        EXIT_CODE=${PIPESTATUS[0]}
        END_TIME=$(date +%s)
        DURATION=$((END_TIME - START_TIME))
        
        echo "" | tee -a "$LOG_FILE"
        echo "=====================================" | tee -a "$LOG_FILE"
        echo "✅ Codex Task Completed" | tee -a "$LOG_FILE"
        echo "Exit Code: $EXIT_CODE" | tee -a "$LOG_FILE"
        echo "Duration: ${DURATION}s" | tee -a "$LOG_FILE"
        echo "End: $(date)" | tee -a "$LOG_FILE"
        echo "=====================================" | tee -a "$LOG_FILE"
        
        # 最後の15行を取得（もう少し多めに）
        LAST_OUTPUT=$(tail -15 "$LOG_FILE" | head -10)
        
        # ターゲットセッションに通知
        if tmux has-session -t "$TARGET_SESSION" 2>/dev/null; then
            # 通知メッセージを送信
            send_keys_safe "$TARGET_SESSION" ""
            send_keys_safe "$TARGET_SESSION" "# 🤖 Codex作業完了通知 [$(date +%H:%M:%S)]"
            send_keys_safe "$TARGET_SESSION" "# Work ID: $WORK_ID"
            send_keys_safe "$TARGET_SESSION" "# Task: $TASK"
            send_keys_safe "$TARGET_SESSION" "# Status: $([ $EXIT_CODE -eq 0 ] && echo '✅ Success' || echo '❌ Failed')"
            send_keys_safe "$TARGET_SESSION" "# Duration: ${DURATION}秒"
            send_keys_safe "$TARGET_SESSION" "# Log: $LOG_FILE"
            send_keys_safe "$TARGET_SESSION" "# === 最後の出力 ==="
            
            # 最後の出力を送信
            echo "$LAST_OUTPUT" | while IFS= read -r line; do
                # 空行をスキップ
                [ -z "$line" ] && continue
                send_keys_safe "$TARGET_SESSION" "# > $line"
            done
            
            send_keys_safe "$TARGET_SESSION" "# =================="
            send_keys_safe "$TARGET_SESSION" ""
        else
            echo "⚠️  Target tmux session '$TARGET_SESSION' not found"
            echo "   Notification was not sent, but work completed."
            echo "   Available sessions:"
            tmux list-sessions 2>/dev/null || echo "   No tmux sessions running"
        fi
    } &
}

# バックグラウンドで実行
run_codex_async
ASYNC_PID=$!

# 実行開始メッセージ
echo ""
echo "✅ Codex started asynchronously!"
echo "   PID: $ASYNC_PID"
echo "   Work ID: $WORK_ID"
echo "   Log file: $LOG_FILE"
echo ""
echo "📝 Monitor progress:"
echo "   tail -f $LOG_FILE"
echo ""
echo "🔍 Check status:"
echo "   ps -p $ASYNC_PID"
echo ""
echo "Codex is now working in the background..."
