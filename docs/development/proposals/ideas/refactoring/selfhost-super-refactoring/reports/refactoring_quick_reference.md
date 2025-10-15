# 超リファクタリング計画 - クイックリファレンス

## 🎯 一言で言うと
**14組の重複ファイルを統一し、921行の巨大ファイルを分割し、箱化率100%＋モジュール構造化を4日間で達成する計画**

---

## 📊 現状 → 目標

| 指標 | 現状 | 目標 |
|-----|------|------|
| .nyashファイル数 | 14 | **0** |
| 重複ファイル | 14組 | **0組** |
| 箱化率 | 82% | **100%** |
| 最大ファイル行数 | 921 | **<300** |
| モジュール構造 | 曖昧 | **明確（5ディレクトリ）** |
| INTERFACES.md網羅率 | 30% | **100%** |

---

## ⏱️ 工数サマリー

| Phase | 内容 | 時間 | 累積 |
|-------|------|------|------|
| **Phase 0** | 事前準備 | 1-2h | 2h |
| **Phase 1** | 緊急修正（重複統一） | 3-4h | 6h |
| **Phase 2** | 箱化推進（分割・構造化） | 5-6h | 12h |
| **Phase 3** | インターフェース統一 | 3-4h | 16h |
| **Phase 4** | 最適化・クリーンアップ | 4-5h | 21h |
| **合計** | | **16-21h** | **21h** |

**推奨スケジュール**: 4日間（1日5-6時間）

---

## 📋 Phase別チェックリスト（超要約版）

### Phase 0: 事前準備（2時間）
```bash
# やること
□ スモークテスト実行（ベースライン）
□ 依存関係マップ作成
□ 重複ファイル差分確認
□ ブランチ作成: refactor/selfhost-super-cleanup

# 成果物
- /tmp/baseline_test_results.txt
- /tmp/dependency_map.md
- /tmp/duplicate_analysis.txt
```

### Phase 1: 緊急修正（4時間）
```bash
# やること
□ 14組の.nyash→.hako統一
  1. interfaces.nyash → interfaces.hako
  2. parser/lexer.nyash → parser/lexer.hako
  3. parser/parser.nyash → parser/parser.hako
  4. parser/ast.nyash → parser/ast.hako
  5-14. （残り10ファイル）
□ parser_box分割計画策定
□ スモークテスト＆コミット

# 成果
✅ .nyashファイル 0個
✅ 箱化率 100%
```

### Phase 2: 箱化推進（6時間）
```bash
# やること
□ parser_box.hako分割（921→3箱）
  - lexer_box.hako (~300行)
  - parser_core_box.hako (~400行)
  - ast_builder_box.hako (~250行)
□ json_program_box.hako精査（520→2箱）
□ local.hako統合（547行統一）
□ pipeline_v2/構造整理（5ディレクトリ）
  - core/ extractors/ emitters/ flows/ utils/ ssa/
□ スモークテスト＆コミット

# 成果
✅ 全ファイル<300行
✅ モジュール構造化完了
```

### Phase 3: インターフェース統一（4時間）
```bash
# やること
□ INTERFACES.md v2.0作成
  - 全箱のインターフェース定義
  - 依存関係マトリックス
  - 契約（Contracts）強化
□ 箱間インターフェース検証
  - verify_interfaces.sh作成
□ エラーハンドリング統一（Fail-Fast）
□ スモークテスト＆コミット

# 成果
✅ INTERFACES.md完全同期
✅ 検証ツール完成
```

### Phase 4: 最適化・クリーンアップ（5時間）
```bash
# やること
□ デッドコード削除
  - find_dead_code.sh作成＆実行
  - 未使用メソッド・箱削除
□ パフォーマンス改善
  - MapBox最適化
  - 不要コピー削減
□ ドキュメント完備
  - README×3更新
  - 各箱にコメント追加
□ 最終スモークテスト＆コミット

# 成果
✅ デッドコード 0行
✅ ドキュメント完備
✅ 全KPI達成
```

---

## 🚨 重要コマンド集

### テスト実行
```bash
# 全スモークテスト（Phase完了毎）
tools/smokes/v2/run.sh --profile quick --filter "selfhost_*"

# 統合テスト（Phase 4最終）
tools/smokes/v2/run.sh --profile quick
tools/smokes/v2/run.sh --profile integration
```

### ファイル統合（Phase 1）
```bash
# 重複ファイル差分確認
diff -u file.nyash file.hako

# .hakoに統合後、.nyash削除
git rm file.nyash

# 個別テスト
tools/smokes/v2/run.sh --profile quick --filter "関連テスト"
```

### ディレクトリ構造化（Phase 2）
```bash
# pipeline_v2/構造化
mkdir -p pipeline_v2/{core,extractors,emitters,flows,utils,ssa}

# ファイル移動
git mv pipeline_v2/execution_pipeline_box.hako pipeline_v2/core/
# ... (以下同様)

# using文更新（各.hakoファイル内）
```

### コミット（Phase毎）
```bash
git add -A
git commit -m "refactor(selfhost): Phase X完了 - タイトル

- ✅ 成果1
- ✅ 成果2
- ✅ 全スモークテストPASS

Phase X 完了。Phase Y へ。
"
git push
```

---

## 📈 進捗確認方法

### KPI確認コマンド
```bash
# .nyashファイル数
find apps/selfhost-compiler -name "*.nyash" | wc -l
# 目標: 0

# 重複ファイル数
for f in $(find apps/selfhost-compiler -name "*.nyash"); do
  base="${f%.nyash}"
  [ -f "${base}.hako" ] && echo "DUPLICATE: $base"
done | wc -l
# 目標: 0

# 最大ファイル行数
find apps/selfhost-compiler -name "*.hako" -exec wc -l {} + | sort -rn | head -1
# 目標: <300行

# 箱化率（.hakoファイル割合）
hako_count=$(find apps/selfhost-compiler -name "*.hako" | wc -l)
nyash_count=$(find apps/selfhost-compiler -name "*.nyash" | wc -l)
total=$((hako_count + nyash_count))
echo "箱化率: $((hako_count * 100 / total))%"
# 目標: 100%
```

---

## 🛡️ リスク軽減策（要約）

### リスク1: 既存機能破壊
**軽減策**: Phase毎の全スモークテスト実行

### リスク2: 重複ファイル統合ミス
**軽減策**: diff確認＆個別テスト

### リスク3: 巨大ファイル分割複雑化
**軽減策**: 責務分析→インターフェース設計→実装の順守

### リスク4: 工数超過
**軽減策**: 80/20ルール適用（優先度低いタスクはスキップ）

### リスク5: ドキュメント更新漏れ
**軽減策**: Phase 3でドキュメント集中対応

---

## 🎯 成功の判定基準

### 必須条件（Must Have）
- ✅ .nyashファイル 0個
- ✅ 重複ファイル 0組
- ✅ 全スモークテストPASS
- ✅ INTERFACES.md完全同期

### 推奨条件（Should Have）
- ✅ 最大ファイル行数 <300行
- ✅ pipeline_v2/構造化（5ディレクトリ）
- ✅ ドキュメント完備

### 理想条件（Nice to Have）
- ✅ デッドコード 0行
- ✅ パフォーマンス改善 10%以上

---

## 📞 迷ったら

### Phase 0-1で迷ったら
- **Q**: 重複ファイルの差分が大きい
- **A**: 慎重にdiff確認→新しい方を採用→個別テスト

### Phase 2で迷ったら
- **Q**: parser_box分割の境界が不明確
- **A**: 責務分析を徹底→INTERFACES.md先行設計

### Phase 3で迷ったら
- **Q**: インターフェース定義が複雑
- **A**: 既存の動作を確認→最小限の契約から開始

### Phase 4で迷ったら
- **Q**: デッドコード判定が難しい
- **A**: 保守的に（疑わしいものは残す）→次回に延期

---

## 🚀 次の一手（Phase完了後）

### Phase 1完了後
→ **parser_box分割計画**を確認し、Phase 2へ

### Phase 2完了後
→ **モジュール構造**を確認し、Phase 3へ

### Phase 3完了後
→ **INTERFACES.md**を確認し、Phase 4へ

### Phase 4完了後
→ **全KPI達成確認**し、🎉完遂！

---

## 📚 関連ドキュメント

- **詳細計画**: /tmp/refactoring_master_plan.md
- **ガントチャート**: /tmp/refactoring_gantt_chart.md
- **CLAUDE.md**: 箱理論・開発方針
- **INTERFACES.md**: 現行インターフェース定義
- **pipeline_v2.md**: パイプライン設計

---

**🎯 このクイックリファレンスで迷わず実行できます！**
**箱理論4原則を守り、Fail-Fastで進めましょう！**

**開始コマンド**:
```bash
# Phase 0開始
tools/smokes/v2/run.sh --profile quick --filter "selfhost_*"
git checkout -b refactor/selfhost-super-cleanup
```

**🚀 Let's go! 超リファクタリング、開始！**
