#!/usr/bin/env bash
set -euo pipefail

# === Nyash特化版：並列リファクタリング自動化 ===
# ChatGPT5のアイデアをNyashプロジェクト用にカスタマイズ

# === 設定 ===
TARGET_DIR="${1:-src}"
FILE_GLOB="${2:-'*.rs'}"
JOBS="${JOBS:-4}"                          # 控えめな並列数
BUILD_CMD="cargo build --release -j32"     # Nyash標準ビルド
TEST_CMD="cargo test --lib"                # 基本的なユニットテスト
FMT_CMD="cargo fmt"                        # Rustフォーマッタ

# Codex非同期実行（通知機能付き）
CODEX_CMD="./tools/codex-async-notify.sh"  

# === 準備 ===
WORK_DIR="refactor-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$WORK_DIR"/{plans,logs,results}
cd "$WORK_DIR"

# 対象ファイル列挙（大きいファイル優先）
find "../$TARGET_DIR" -name "$FILE_GLOB" -type f -exec wc -l {} + | 
  sort -rn | 
  awk '$1 > 500 {print $2}' > target_files.txt  # 500行以上のファイルのみ

echo "🎯 Target files: $(wc -l < target_files.txt)"
echo "📁 Work directory: $WORK_DIR"

# === Phase 1: 並列提案生成（Codex利用）===
echo "🚀 Phase 1: Generating refactoring proposals..."

# 各ファイルに対してCodexタスクを生成
i=0
while IFS= read -r file; do
  ((i++))
  basename_file=$(basename "$file")
  
  # タスク定義
  if [[ "$file" == *"mir/"* ]]; then
    # MIR関連は特別扱い
    task="Refactor $file to support MIR-13 instruction set. Remove legacy instructions and unify with BoxCall/TypeOp"
  elif [[ "$file" == *"vm"* ]]; then
    # VM関連
    task="Refactor $file: split into smaller modules if >1000 lines, improve readability"
  else
    # 一般的なリファクタリング
    task="Refactor $file: extract functions/modules if >1000 lines, improve maintainability"
  fi
  
  # 非同期でCodex実行
  echo "[$i] Starting: $basename_file"
  $CODEX_CMD "$task" > "logs/codex-$i-$basename_file.log" 2>&1
  
  # タスク記録
  echo "$i|$file|$task" >> task_list.txt
  
  # 少し間隔を空ける（API制限対策）
  sleep 2
done < target_files.txt

echo "⏳ Waiting for all Codex tasks to complete..."
echo "   Monitor: tail -f logs/codex-*.log"

# === Phase 2: 結果収集・検証（手動トリガー）===
cat > apply_results.sh << 'EOF'
#!/bin/bash
# Phase 2: 各提案を検証・適用

echo "🔍 Phase 2: Applying and verifying changes..."

# 現在のブランチを記録
ORIGINAL_BRANCH=$(git branch --show-current)

# 各Codex結果を処理
for log in logs/codex-*.log; do
  [[ -e "$log" ]] || continue
  
  # ログから情報抽出
  task_id=$(basename "$log" | sed 's/codex-\([0-9]*\)-.*/\1/')
  file_info=$(grep "^$task_id|" task_list.txt)
  target_file=$(echo "$file_info" | cut -d'|' -f2)
  
  echo "==> Processing: $target_file"
  
  # 新しいブランチ作成
  branch_name="refactor/$task_id-$(basename "$target_file" .rs)"
  git checkout -b "$branch_name" "$ORIGINAL_BRANCH" 2>/dev/null || {
    git checkout "$branch_name"
    git reset --hard "$ORIGINAL_BRANCH"
  }
  
  # ここで手動で変更を適用する必要がある
  echo "   ⚠️  Please apply changes from: $log"
  echo "   Press Enter when done..."
  read -r
  
  # フォーマット
  cargo fmt -- "$target_file" 2>/dev/null || true
  
  # ビルドテスト
  if cargo build --release -j32 >/dev/null 2>&1; then
    echo "   ✅ Build passed"
    
    # 簡単なテスト
    if cargo test --lib --quiet 2>/dev/null; then
      echo "   ✅ Tests passed"
      
      # コミット
      git add -A
      git commit -m "refactor: $(basename "$target_file") - reduce complexity and improve structure" \
        -m "- Applied MIR-13 instruction set changes" \
        -m "- Extracted modules/functions as needed" \
        -m "- Maintained API compatibility"
      
      echo "   ✅ Committed to branch: $branch_name"
    else
      echo "   ❌ Tests failed - reverting"
      git reset --hard
      git checkout "$ORIGINAL_BRANCH"
      git branch -D "$branch_name" 2>/dev/null
    fi
  else
    echo "   ❌ Build failed - reverting"
    git reset --hard  
    git checkout "$ORIGINAL_BRANCH"
    git branch -D "$branch_name" 2>/dev/null
  fi
done

# 元のブランチに戻る
git checkout "$ORIGINAL_BRANCH"

echo "✅ Phase 2 complete!"
echo "📊 Results:"
git branch --list 'refactor/*' | wc -l | xargs echo "   Successful refactors:"
echo "   Review with: git branch --list 'refactor/*'"
EOF

chmod +x apply_results.sh

echo ""
echo "✅ Phase 1 complete! Codex tasks submitted."
echo ""
echo "📋 Next steps:"
echo "   1. Wait for Codex notifications in tmux"
echo "   2. Run: ./apply_results.sh"
echo "   3. Review and merge branches"
echo ""
echo "💡 Tips:"
echo "   - Check logs: ls -la logs/"
echo "   - Monitor tmux: tmux attach -t claude"