#!/bin/bash
# 超リファクタリング計画 - 現状確認スクリプト
# 作成日: 2025-10-04

set -e

echo "🔍 Selfhost Compiler 現状分析"
echo "================================"
echo ""

# 1. .nyashファイル数
echo "📊 1. .nyashファイル数"
nyash_count=$(find /home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler -name "*.nyash" | wc -l)
echo "   .nyash: $nyash_count ファイル"
echo "   目標: 0 ファイル"
echo "   状態: $([ $nyash_count -eq 0 ] && echo '✅ 達成' || echo '❌ 未達成')"
echo ""

# 2. .hakoファイル数
echo "📊 2. .hakoファイル数"
hako_count=$(find /home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler -name "*.hako" | wc -l)
echo "   .hako: $hako_count ファイル"
echo ""

# 3. 箱化率
echo "📊 3. 箱化率"
total=$((hako_count + nyash_count))
if [ $total -gt 0 ]; then
    boxification_rate=$((hako_count * 100 / total))
    echo "   箱化率: $boxification_rate%"
    echo "   目標: 100%"
    echo "   状態: $([ $boxification_rate -eq 100 ] && echo '✅ 達成' || echo '❌ 未達成')"
else
    echo "   箱化率: 計算不可"
fi
echo ""

# 4. 重複ファイル数
echo "📊 4. 重複ファイル数"
duplicate_count=0
for f in $(find /home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler -name "*.nyash"); do
    base="${f%.nyash}"
    if [ -f "${base}.hako" ]; then
        duplicate_count=$((duplicate_count + 1))
        echo "   DUPLICATE: ${base#/home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler/}"
    fi
done
echo "   重複: $duplicate_count 組"
echo "   目標: 0 組"
echo "   状態: $([ $duplicate_count -eq 0 ] && echo '✅ 達成' || echo '❌ 未達成')"
echo ""

# 5. 最大ファイル行数
echo "📊 5. 最大ファイル行数"
max_file=$(find /home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler -name "*.hako" -o -name "*.nyash" | xargs wc -l 2>/dev/null | sort -rn | head -2 | head -1)
max_lines=$(echo "$max_file" | awk '{print $1}')
max_filename=$(echo "$max_file" | awk '{print $2}' | sed 's|/home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler/||')
echo "   最大: $max_lines 行 ($max_filename)"
echo "   目標: <300 行"
echo "   状態: $([ $max_lines -lt 300 ] && echo '✅ 達成' || echo '❌ 未達成')"
echo ""

# 6. 総行数
echo "📊 6. 総行数"
total_hako_lines=$(find /home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler -name "*.hako" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}')
total_nyash_lines=$(find /home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler -name "*.nyash" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}')
echo "   .hako: $total_hako_lines 行"
echo "   .nyash: $total_nyash_lines 行"
echo "   合計: $((total_hako_lines + total_nyash_lines)) 行"
echo ""

# 7. Top 5巨大ファイル
echo "📊 7. Top 5 巨大ファイル"
find /home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler -name "*.hako" -o -name "*.nyash" | xargs wc -l 2>/dev/null | sort -rn | head -6 | head -5 | while read line count file; do
    filename=$(echo "$file" | sed 's|/home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler/||')
    echo "   $line 行: $filename"
done
echo ""

# 8. pipeline_v2/構造
echo "📊 8. pipeline_v2/ ディレクトリ構造"
if [ -d "/home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler/pipeline_v2" ]; then
    echo "   現在のディレクトリ:"
    find /home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler/pipeline_v2 -type d | sed 's|/home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler/||' | sed 's|^|   - |'
    echo ""
    echo "   ファイル数:"
    box_files=$(find /home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler/pipeline_v2 -name "*_box.hako" | wc -l)
    flow_files=$(find /home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler/pipeline_v2 -name "*_flow.hako" | wc -l)
    other_files=$(find /home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler/pipeline_v2 -name "*.hako" -o -name "*.nyash" | wc -l)
    echo "   - *_box.hako: $box_files ファイル"
    echo "   - *_flow.hako: $flow_files ファイル"
    echo "   - その他: $((other_files - box_files - flow_files)) ファイル"
else
    echo "   ❌ pipeline_v2/ ディレクトリが存在しません"
fi
echo ""

# 9. INTERFACES.md確認
echo "📊 9. INTERFACES.md 確認"
interfaces_file="/home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler/INTERFACES.md"
if [ -f "$interfaces_file" ]; then
    lines=$(wc -l < "$interfaces_file")
    echo "   ✅ 存在: $lines 行"
else
    echo "   ❌ 存在しない"
fi
echo ""

# 10. サマリー
echo "================================"
echo "🎯 リファクタリング必要性評価"
echo "================================"
echo ""

issues=0
[ $nyash_count -gt 0 ] && issues=$((issues + 1))
[ $duplicate_count -gt 0 ] && issues=$((issues + 1))
[ $max_lines -ge 300 ] && issues=$((issues + 1))
[ $boxification_rate -lt 100 ] && issues=$((issues + 1))

if [ $issues -eq 0 ]; then
    echo "✅ すべてのKPI達成！リファクタリング完了状態"
elif [ $issues -le 2 ]; then
    echo "🟡 一部改善が必要（$issues 項目）"
else
    echo "🔴 リファクタリング強く推奨（$issues 項目未達成）"
fi

echo ""
echo "推奨アクション:"
[ $nyash_count -gt 0 ] && echo "  - Phase 1: .nyash→.hako統一（$nyash_count ファイル）"
[ $duplicate_count -gt 0 ] && echo "  - Phase 1: 重複ファイル解消（$duplicate_count 組）"
[ $max_lines -ge 300 ] && echo "  - Phase 2: 巨大ファイル分割（$max_filename: $max_lines 行）"
[ $boxification_rate -lt 100 ] && echo "  - Phase 2: 箱化推進（現在 $boxification_rate%）"

echo ""
echo "📚 詳細計画:"
echo "  - /tmp/refactoring_index.md（ドキュメント索引）"
echo "  - /tmp/refactoring_executive_summary.md（エグゼクティブサマリー）"
echo "  - /tmp/refactoring_quick_reference.md（クイックリファレンス）"
echo ""
