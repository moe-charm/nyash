#!/bin/bash
# codex-async-notify.sh - Codexを非同期実行してtmuxセッションに通知

# 使い方を表示
if [ $# -eq 0 ]; then
    echo "Usage: $0 <task description> [tmux_session]"
    echo "Examples:"
    echo "  $0 'Refactor MIR builder to 13 instructions'"
    echo "  $0 'Write paper introduction' gemini-session"
    echo "  $0 'Review code quality' chatgpt"
    echo ""
    echo "Default tmux session: claude"
    exit 1
fi

# 引数解析
TASK="$1"
TARGET_SESSION="${2:-claude}"  # デフォルトは "claude"
# 通知用ウィンドウ名（既定: codex-notify）。存在しなければ作成する
NOTIFY_WINDOW_NAME="${CODEX_NOTIFY_WINDOW:-codex-notify}"

# 設定
WORK_DIR="$HOME/.codex-async-work"
LOG_DIR="$WORK_DIR/logs"
WORK_ID=$(date +%s%N)
LOG_FILE="$LOG_DIR/codex-${WORK_ID}.log"

# 作業ディレクトリ準備
mkdir -p "$LOG_DIR"

# === オプショナル並列制御 ===
# - CODEX_MAX_CONCURRENT: 許容最大同時実行数（0または未設定で無制限）
# - CODEX_CONCURRENCY_MODE: "block"(既定) or "drop"（上限超過時に起動を諦める）
# - CODEX_DEDUP: 1 で同一 TASK が実行中なら重複起動を避ける
MAX_CONCURRENT=${CODEX_MAX_CONCURRENT:-0}
CONC_MODE=${CODEX_CONCURRENCY_MODE:-block}
DEDUP=${CODEX_DEDUP:-0}
# 検出パターン（上書き可）。例: export CODEX_PROC_PATTERN='codex .* exec'
CODEX_PROC_PATTERN=${CODEX_PROC_PATTERN:-'codex .* exec'}
FLOCK_WAIT=${CODEX_FLOCK_WAIT:-5}
LOG_RETENTION_DAYS=${CODEX_LOG_RETENTION_DAYS:-0}
LOG_MAX_BYTES=${CODEX_LOG_MAX_BYTES:-0}

list_running_codex() {
  # 実ジョブのみを検出: "codex ... exec ..." を含むコマンドライン
  if command -v pgrep >/dev/null 2>&1; then
    # pgrep -af は PID と引数を出力
    pgrep -af -- "$CODEX_PROC_PATTERN" || true
  else
    # フォールバック: ps+grep（grep 自身は除外）
    ps -eo pid=,args= | grep -E -- "$CODEX_PROC_PATTERN" | grep -v grep || true
  fi
}

count_running_codex() {
  local n
  n=$(list_running_codex | wc -l | tr -d ' ')
  echo "${n:-0}"
}

is_same_task_running() {
  # ざっくり: 引数列に TASK 文字列（1行化）を含む行があれば重複とみなす
  local oneline
  oneline=$(echo "$TASK" | tr '\n' ' ' | sed 's/  */ /g')
  list_running_codex | grep -F -- "$oneline" >/dev/null 2>&1
}

maybe_wait_for_slot() {
  [ "${MAX_CONCURRENT}" -gt 0 ] || return 0

  # 起動判定〜起動直前までをクリティカルセクションにする
  mkdir -p "$WORK_DIR"
  exec 9>"$WORK_DIR/concurrency.lock"
  if ! flock -w "$FLOCK_WAIT" 9; then
    echo "⚠️  concurrency.lock の取得に失敗（${FLOCK_WAIT}s）。緩やかに続行。" >&2
  else
    export CODEX_LOCK_HELD=1
  fi

  if [ "$DEDUP" = "1" ] && is_same_task_running; then
    echo "⚠️  同一タスクが実行中のため起動をスキップ: $TASK"
    exit 0
  fi

  local current
  current=$(count_running_codex)
  if [ "$current" -lt "$MAX_CONCURRENT" ]; then
    return 0
  fi

  case "$CONC_MODE" in
    drop)
      echo "⚠️  上限(${MAX_CONCURRENT})到達のため起動をスキップ。現在: $current"
      exit 3
      ;;
    block|*)
      echo "⏳ スロット待機: 現在 $current / 上限 $MAX_CONCURRENT"
      while :; do
        sleep 1
        current=$(count_running_codex)
        if [ "$current" -lt "$MAX_CONCURRENT" ]; then
          echo "✅ 空きスロット確保: 現在 $current / 上限 $MAX_CONCURRENT"
          break
        fi
      done
      ;;
  esac
}

# 上限が定義されていれば起動前に調整
maybe_wait_for_slot

# 非同期実行関数
run_codex_async() {
    {
        # Dedupロック: 子プロセス側で握る（同一TASKの多重起動回避）
        if [ "${CODEX_DEDUP_FILE:-}" != "" ]; then
            exec 8>"${CODEX_DEDUP_FILE}"
            if ! flock -n 8; then
                echo "⚠️  Duplicate task detected (dedup lock busy). Skipping: $TASK" | tee -a "$LOG_FILE"
                exit 0
            fi
            trap 'rm -f "${CODEX_DEDUP_FILE}" >/dev/null 2>&1 || true' EXIT
        fi
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
        
        # 末尾表示行数（環境変数で上書き可）/ ミニマル通知モード
        TAIL_N=${CODEX_NOTIFY_TAIL:-20}
        MINIMAL=${CODEX_NOTIFY_MINIMAL:-1}

        # 通知内容を一時ファイルに組み立て（空行も保持）
        TASK_ONELINE=$(echo "$TASK" | tr '\n' ' ' | sed 's/  */ /g')
        NOTIFY_FILE="$WORK_DIR/notify-${WORK_ID}.tmp"
        if [ "$MINIMAL" = "1" ]; then
            {
                echo ""
                echo "# 🤖 Codex作業完了通知 [$(date +%H:%M:%S)]"
                echo "# Work ID: $WORK_ID"
                echo "# Status: $([ $EXIT_CODE -eq 0 ] && echo '✅ Success' || echo '❌ Failed')"
                echo "# Log: $LOG_FILE"
                echo ""
            } > "$NOTIFY_FILE"
        else
            {
                echo ""
                echo "# 🤖 Codex作業完了通知 [$(date +%H:%M:%S)]"
                echo "# Work ID: $WORK_ID"
                echo "# Task: $TASK_ONELINE"
                echo "# Status: $([ $EXIT_CODE -eq 0 ] && echo '✅ Success' || echo '❌ Failed')"
                echo "# Duration: ${DURATION}秒"
                echo "# Log: $LOG_FILE"
                echo "# === 最後の出力 (tail -n ${TAIL_N}) ==="
                tail -n "$TAIL_N" "$LOG_FILE" | sed -e 's/^/# > /'
                echo "# =================="
                echo ""
            } > "$NOTIFY_FILE"
        fi

        # 共通のシンプル通知（4行）を指定の tmux セッションのアクティブペインへ送る（Enter= C-m）
        if tmux has-session -t "$TARGET_SESSION" 2>/dev/null; then
            STATUS_MARK=$([ $EXIT_CODE -eq 0 ] && echo '✅ Success' || echo '❌ Failed')
            CHAT_FILE="$WORK_DIR/chat-${WORK_ID}.tmp"
            TASK_ONELINE=$(echo "$TASK" | tr '\n' ' ' | sed 's/  */ /g')
            {
              echo "# 🤖 Codex: Done [$(date +%H:%M:%S)]"
              echo "# Work ID: $WORK_ID"
              echo "# Status: $STATUS_MARK"
              echo "# Log: $LOG_FILE"
              echo "# Task: $TASK_ONELINE — まだタスクがあれば次のタスクお願いします"
              echo ""
            } > "$CHAT_FILE"
            # アクティブペインを取得
            TARGET_PANE=$(tmux list-panes -t "$TARGET_SESSION" -F '#{pane_id} #{pane_active}' 2>/dev/null | awk '$2=="1"{print $1; exit}')
            [ -z "$TARGET_PANE" ] && TARGET_PANE="$TARGET_SESSION"
            BUF_NAME="codex-chat-$WORK_ID"
            tmux load-buffer -b "$BUF_NAME" "$CHAT_FILE" 2>/dev/null || true
            tmux paste-buffer -b "$BUF_NAME" -t "$TARGET_PANE" 2>/dev/null || true
            tmux delete-buffer -b "$BUF_NAME" 2>/dev/null || true
            # Small delay to ensure paste completes before sending Enter
            sleep 0.2
            tmux send-keys -t "$TARGET_PANE" C-m 2>/dev/null || true
            sleep 0.05
            tmux send-keys -t "$TARGET_PANE" C-m 2>/dev/null || true
            rm -f "$NOTIFY_FILE" "$CHAT_FILE" 2>/dev/null || true
        else
            echo "⚠️  Target tmux session '$TARGET_SESSION' not found"
            echo "   Notification was not sent, but work completed."
            echo "   Available sessions:"
            tmux list-sessions 2>/dev/null || echo "   No tmux sessions running"
        fi
        # 古いログの自動クリーン（任意）
        if [ "$LOG_RETENTION_DAYS" -gt 0 ] 2>/dev/null; then
            find "$LOG_DIR" -type f -name 'codex-*.log' -mtime +"$LOG_RETENTION_DAYS" -delete 2>/dev/null || true
        fi
        if [ "$LOG_MAX_BYTES" -gt 0 ] 2>/dev/null; then
            # 容量超過時に古い順で間引く
            CUR=$(du -sb "$LOG_DIR" 2>/dev/null | awk '{print $1}' || echo 0)
            while [ "${CUR:-0}" -gt "$LOG_MAX_BYTES" ]; do
                oldest=$(ls -1t "$LOG_DIR"/codex-*.log 2>/dev/null | tail -n 1)
                [ -n "$oldest" ] || break
                rm -f "$oldest" 2>/dev/null || true
                CUR=$(du -sb "$LOG_DIR" 2>/dev/null | awk '{print $1}' || echo 0)
            done
        fi
    } &
}

# Dedupファイルパス（子へ受け渡し）
if [ "$DEDUP" = "1" ]; then
  # TASK 正規化→SHA1
  TASK_ONELINE=$(echo "$TASK" | tr '\n' ' ' | sed 's/  */ /g')
  TASK_SHA=$(printf "%s" "$TASK_ONELINE" | sha1sum | awk '{print $1}')
  export CODEX_DEDUP_FILE="$WORK_DIR/dedup-${TASK_SHA}.lock"
fi

# バックグラウンドで実行（必要ならロック解放は起動直後に）
run_codex_async
ASYNC_PID=$!

# すぐにロックが残っていれば解放（他の起動を待たせない）
if [ "${CODEX_LOCK_HELD:-0}" = "1" ]; then
  flock -u 9 2>/dev/null || true
fi

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
