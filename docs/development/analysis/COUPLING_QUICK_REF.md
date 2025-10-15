# 依存関係・結合度 クイックリファレンス

**📊 全体評価: B+ (良好、改善余地あり)**

---

## 🚦 ステータス一覧

| 項目 | ステータス | 詳細 |
|------|----------|------|
| **循環依存** | ✅ **0件** | 完全にクリーン |
| **双方向依存** | ✅ **0件** | 優れた疎結合 |
| **平均依存度** | ✅ **3.02** | 適正範囲（2-5） |
| **レイヤー違反** | ⚠️ **12件** | 要修正 |
| **未使用import** | ⚠️ **86個** | 要削除 |
| **Hub modules** | ⚠️ **5個** | SPOF候補 |

---

## 🎯 Top 3 優先対応事項

### 1. 未使用import削除（0.5日、ROI: ⭐⭐⭐⭐⭐）
```bash
# 自動スクリプト実行
python3 /tmp/remove_unused_imports.py selfhost/
```
**対象**: 86個のimport（36ファイル）

---

### 2. レイヤー違反修正（1日、ROI: ⭐⭐⭐⭐⭐）
```bash
# Mini VM関連を適切な層に移動
mv selfhost/shared/common/mini_vm_*.hako selfhost/vm/mini/
mv selfhost/vm/boxes/result_box.hako selfhost/shared/common/
```
**効果**: 重大度2違反（3件）を解消

---

### 3. Hub Module Interface抽出（3-5日、ROI: ⭐⭐⭐⭐）

**対象**:
- `json_cursor` (22依存)
- `value_manager` (20依存)
- `json_field_extractor` (17依存)

**効果**: テスト容易性+30%、変更影響-50%

---

## 🔴 要注意モジュール

### Hub Modules（SPOF候補）
```
🔴 shared.json.json_cursor           (22 dependents)
🔴 hakorune-vm.value_manager         (20 dependents)
🔴 hakorune-vm.json_field_extractor  (17 dependents)
🔴 hakorune-vm.hakorune_vm_core      (16 dependents)
🟡 compiler.pipeline_v2.regex_flow   (15 dependents)
```

**リスク**: これらの変更が20+モジュールに波及

---

### 高依存モジュール（複雑度高）
```
🔴 compiler.pipeline_v2.pipeline           (25 dependencies)
🔴 hakorune-vm.instruction_dispatcher      (19 dependencies)
🔴 hakorune-vm.mircall_handler             (17 dependencies)
🔴 hakorune-vm.hakorune_vm_core            (17 dependencies)
🟡 vm.boxes.mir_vm_min                     (14 dependencies)
```

**推奨**: モジュール分割またはFacade化

---

## ⚠️ レイヤー違反（12件）

### 重大度2（Foundation → VM）
```
❌ shared.common.mini_vm_binop    → vm.json
❌ shared.common.mini_vm_compare  → vm.scan
❌ shared.common.mini_vm_scan     → shared.json
```

### 重大度1（Infrastructure → VM）
```
❌ shared.mir.mir_io_box → hakorune-vm.* (6 dependencies)
❌ vm.flow_runner → compiler.flow_entry
```

---

## 📈 結合度マトリックス

```
                      Efferent  Afferent  Total  Instability
hakorune_vm_core         17        16      33       0.52
pipeline                 25         1      26       0.96
json_cursor               3        22      25       0.12  ← 安定Hub
value_manager             2        20      22       0.09  ← 安定Hub
instruction_dispatcher   19         1      20       0.95  ← 不安定
```

**Legend**:
- **Efferent**: 出力結合（他への依存）
- **Afferent**: 入力結合（他からの依存）
- **Instability**: 不安定度 = Efferent / Total（0=安定、1=不安定）

---

## 🏗️ レイヤー構造

```
Layer 5: Tools/Tests     (  4 modules)
    ↓
Layer 4: Compiler        ( 32 modules)
    ↓
Layer 3: VM              ( 84 modules) ← 最大
    ↓
Layer 2: Infrastructure  ( 11 modules)
    ↓
Layer 1: Foundation      (  3 modules)
```

**ルール**: 上位 → 下位のみ依存可

---

## 🚀 4週間リファクタリングプラン

### Week 1: Quick Wins
- Day 1-2: 未使用import削除（86個）
- Day 3-4: レイヤー違反修正（4ファイル移動）
- Day 5: テスト・検証

### Week 2: Hub Refactoring
- Day 1-2: `json_cursor` interface抽出
- Day 3-4: `value_manager` interface抽出
- Day 5: `json_field_extractor` interface抽出

### Week 3-4: Complex Module Split
- `pipeline` 分割（4モジュール）
- `instruction_dispatcher` Facade化
- `hakorune_vm_core` 分割（3モジュール）

**期待効果**:
- 結合度: -30%
- テスト容易性: +30%
- 保守工数: -20%

---

## 🔧 よく使うコマンド

### 依存関係分析
```bash
# 全体分析
python3 /tmp/analyze_dependencies.py

# 詳細分析
python3 /tmp/detailed_coupling_analysis.py

# グラフ生成
python3 /tmp/generate_simplified_dot.py
```

### グラフ可視化
```bash
# レイヤー間依存グラフ
dot -Tpng /tmp/selfhost_deps_layers.dot -o layers.png

# 主要モジュール依存グラフ
dot -Tpng /tmp/selfhost_deps_simplified.dot -o modules.png
```

### 未使用import検出
```bash
# アクションプラン生成（未使用import一覧含む）
python3 /tmp/generate_action_plan.py | head -150
```

---

## 📚 詳細ドキュメント

| ドキュメント | 用途 | 読了時間 |
|-------------|------|---------|
| [COUPLING_ANALYSIS_SUMMARY.md](./COUPLING_ANALYSIS_SUMMARY.md) | エグゼクティブサマリー | 5-10分 |
| [DEPENDENCY_COUPLING_ANALYSIS.md](./DEPENDENCY_COUPLING_ANALYSIS.md) | 完全レポート | 30-45分 |
| [DEPENDENCY_GRAPH_TEXT.md](./DEPENDENCY_GRAPH_TEXT.md) | グラフ可視化 | 10-15分 |
| [TASK_6_DELIVERABLES.md](./TASK_6_DELIVERABLES.md) | 成果物一覧 | 5分 |

---

## 💡 開発時の注意点

### 新しいモジュール追加時
1. ✅ 依存方向を確認（上位→下位のみ）
2. ✅ 依存数を5以下に抑える
3. ✅ Hub modulesへの依存を最小化
4. ✅ 循環依存チェック

### モジュール変更時
1. ⚠️ Hub modules変更は影響範囲を確認
2. ⚠️ 高依存モジュール変更は慎重に
3. ⚠️ 未使用importは即削除
4. ⚠️ レイヤー違反を導入しない

### レビュー時チェック項目
- [ ] 新規依存は妥当か？
- [ ] レイヤー違反はないか？
- [ ] 循環依存を導入していないか？
- [ ] Hub modulesへの依存は必要最小限か？
- [ ] 未使用importはないか？

---

## 🎯 成功指標（KPI）

| 指標 | 現状 | 目標（4週間後） | 測定方法 |
|------|------|----------------|---------|
| **平均依存度** | 3.02 | ≤ 2.5 | スクリプト実行 |
| **レイヤー違反** | 12件 | 0件 | スクリプト実行 |
| **未使用import** | 86個 | 0個 | スクリプト実行 |
| **Hub modules** | 5個 | 3個 | Interface抽出 |
| **循環依存** | 0件 | 0件 | 維持 |

---

## 🔗 クイックリンク

- [メインレポート](./DEPENDENCY_COUPLING_ANALYSIS.md)
- [サマリー](./COUPLING_ANALYSIS_SUMMARY.md)
- [グラフ可視化](./DEPENDENCY_GRAPH_TEXT.md)
- [成果物一覧](./TASK_6_DELIVERABLES.md)

---

**更新日**: 2025-10-15
**次回更新推奨**: 2週間後（Phase 1完了時）
