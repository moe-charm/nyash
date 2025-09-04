#!/bin/bash
# MIR13移行専用ヘルパー（ChatGPT5方式を応用）

set -euo pipefail

# === MIR13特化設定 ===
LEGACY_PATTERNS=(
    "ArrayGet" "ArraySet"
    "RefNew" "RefGet" "RefSet"
    "TypeCheck" "Cast"
    "PluginInvoke"
    "Copy" "Debug" "Print"
    "Nop" "Throw" "Catch"
    "Safepoint"
)

# 統合先
UNIFICATION_MAP="
ArrayGet:BoxCall
ArraySet:BoxCall
RefNew:BoxCall
RefGet:BoxCall
RefSet:BoxCall
TypeCheck:TypeOp
Cast:TypeOp
PluginInvoke:BoxCall
"

# === Phase 1: レガシー命令の使用箇所を検出 ===
echo "🔍 Detecting legacy instruction usage..."
mkdir -p mir13-migration/{detections,patches,results}

for pattern in "${LEGACY_PATTERNS[@]}"; do
    echo "Searching for: $pattern"
    rg -l "MirInstruction::$pattern" src/ > "mir13-migration/detections/$pattern.txt" || true
    
    count=$(wc -l < "mir13-migration/detections/$pattern.txt" 2>/dev/null || echo 0)
    if [[ $count -gt 0 ]]; then
        echo "  Found in $count files"
        
        # 各ファイルに対してCodexタスク生成
        while IFS= read -r file; do
            target_instruction=$(echo "$UNIFICATION_MAP" | grep "^$pattern:" | cut -d: -f2 || echo "appropriate")
            
            task="In $file, replace all uses of MirInstruction::$pattern with MirInstruction::$target_instruction. 
Ensure semantic equivalence is maintained. For array operations, use BoxCall with appropriate method names.
For type operations, use TypeOp with appropriate parameters."
            
            echo "  Creating task for: $file"
            echo "$task" > "mir13-migration/patches/$pattern-$(basename "$file").task"
        done < "mir13-migration/detections/$pattern.txt"
    fi
done

# === Phase 2: 並列実行スクリプト生成 ===
cat > mir13-migration/execute_migration.sh << 'MIGRATION_SCRIPT'
#!/bin/bash
# Execute MIR13 migration tasks

JOBS=${JOBS:-3}
LOG_DIR="logs-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$LOG_DIR"

echo "🚀 Executing MIR13 migration tasks..."

# 各タスクファイルを処理
for task_file in patches/*.task; do
    [[ -e "$task_file" ]] || continue
    
    task_content=$(cat "$task_file")
    task_name=$(basename "$task_file" .task)
    
    echo "Processing: $task_name"
    
    # Codex実行（ここは環境に応じて調整）
    ../tools/codex-async-notify.sh "$task_content" > "$LOG_DIR/$task_name.log" 2>&1 &
    
    # API制限対策で少し待つ
    sleep 3
done

echo "✅ All tasks submitted!"
echo "📋 Monitor progress in tmux or check logs in: $LOG_DIR"

# 結果集計スクリプト
cat > verify_migration.sh << 'VERIFY'
#!/bin/bash
echo "🔍 Verifying MIR13 migration..."

# レガシー命令が残っていないか確認
legacy_found=0
for pattern in ArrayGet ArraySet RefNew RefGet RefSet TypeCheck Cast PluginInvoke; do
    if rg -q "MirInstruction::$pattern" ../src/; then
        echo "❌ Still found: $pattern"
        rg "MirInstruction::$pattern" ../src/ | head -3
        ((legacy_found++))
    fi
done

if [[ $legacy_found -eq 0 ]]; then
    echo "✅ No legacy instructions found!"
    echo "🎉 MIR13 migration complete!"
else
    echo "⚠️  Found $legacy_found legacy instruction types remaining"
fi

# 新しい統一命令の使用統計
echo ""
echo "📊 Unified instruction usage:"
echo -n "  BoxCall: "
rg "MirInstruction::BoxCall" ../src/ | wc -l
echo -n "  TypeOp: "
rg "MirInstruction::TypeOp" ../src/ | wc -l
VERIFY

chmod +x verify_migration.sh
MIGRATION_SCRIPT

chmod +x mir13-migration/execute_migration.sh

# === サマリー表示 ===
echo ""
echo "📊 MIR13 Migration Summary:"
echo "=========================="

total_files=0
for pattern in "${LEGACY_PATTERNS[@]}"; do
    file="mir13-migration/detections/$pattern.txt"
    if [[ -s "$file" ]]; then
        count=$(wc -l < "$file")
        echo "  $pattern: $count files"
        ((total_files += count))
    fi
done

echo "=========================="
echo "  Total: $total_files file occurrences"
echo ""
echo "🚀 Next steps:"
echo "  1. cd mir13-migration"
echo "  2. Review task files in patches/"
echo "  3. Run: ./execute_migration.sh"
echo "  4. After completion, run: ./verify_migration.sh"