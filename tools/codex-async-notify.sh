#!/bin/bash
# codex-async-notify.sh - Codexを非同期実行してClaudeに通知

# 設定
CLAUDE_SESSION="claude"  # Claudeのtmuxセッション名
WORK_DIR="$HOME/.codex-async-work"
LOG_DIR="$WORK_DIR/logs"

# 使い方を表示
if [ $# -eq 0 ]; then
    echo "Usage: $0 <task description>"
    echo "Example: $0 'Refactor MIR builder to 13 instructions'"
    exit 1
fi

TASK="$1"
WORK_ID=$(date +%s%N)
LOG_FILE="$LOG_DIR/codex-${WORK_ID}.log"

# 作業ディレクトリ準備
mkdir -p "$LOG_DIR"

# 非同期実行関数
run_codex_async() {
    {
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
        
        # Claudeに通知
        if tmux has-session -t "$CLAUDE_SESSION" 2>/dev/null; then
            # 通知メッセージを送信
            tmux send-keys -t "$CLAUDE_SESSION" "" Enter
            tmux send-keys -t "$CLAUDE_SESSION" "# 🤖 Codex作業完了通知 [$(date +%H:%M:%S)]" Enter
            tmux send-keys -t "$CLAUDE_SESSION" "# Work ID: $WORK_ID" Enter
            tmux send-keys -t "$CLAUDE_SESSION" "# Task: $TASK" Enter
            tmux send-keys -t "$CLAUDE_SESSION" "# Status: $([ $EXIT_CODE -eq 0 ] && echo '✅ Success' || echo '❌ Failed')" Enter
            tmux send-keys -t "$CLAUDE_SESSION" "# Duration: ${DURATION}秒" Enter
            tmux send-keys -t "$CLAUDE_SESSION" "# Log: $LOG_FILE" Enter
            tmux send-keys -t "$CLAUDE_SESSION" "# === 最後の出力 ===" Enter
            
            # 最後の出力を送信
            echo "$LAST_OUTPUT" | while IFS= read -r line; do
                # 空行をスキップ
                [ -z "$line" ] && continue
                tmux send-keys -t "$CLAUDE_SESSION" "# > $line" Enter
            done
            
            tmux send-keys -t "$CLAUDE_SESSION" "# ==================" Enter
            tmux send-keys -t "$CLAUDE_SESSION" "" Enter
        else
            echo "⚠️  Claude tmux session '$CLAUDE_SESSION' not found"
            echo "   Notification was not sent, but work completed."
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