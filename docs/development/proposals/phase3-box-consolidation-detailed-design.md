# Phase 3: Box化統合詳細設計レポート

**作成日**: 2025-10-06
**対象範囲**: apps/selfhost (38 .hako files, 3,573 lines)
**分析基準**: Task先生提案の7 Box統合計画

---

## 📊 Box別実装計画

### 1. CompareOpsBox（既存Box、統合対象）

**目的**: 比較演算子マッピング・評価ロジックの完全統合

**現状分析**:
- **既存実装**: `apps/selfhost/vm/boxes/compare_ops.hako` (25行)
- **重複箇所**:
  - `mir_vm_min.hako`: 行228-233 (6行の完全重複)
  - `op_handlers.hako`: 行67-76 (10行の完全重複、`_map_cmp_symbol` + `_eval_cmp`)
- **検出パターン**: `if kind == "Eq"` 系の6連続if文

**統合対象ファイル**:
1. `apps/selfhost/vm/boxes/compare_ops.hako` (既存、保持)
2. `apps/selfhost/vm/boxes/mir_vm_min.hako` (16行削除可能)
3. `apps/selfhost/vm/boxes/op_handlers.hako` (10行削除可能)

**削減効果**: 319行 → 293行 (**26行削減、8.2%削減**)

**実装難易度**: **Low**
- CompareOpsBoxは既に存在
- mir_vm_min.hakoは228-233行、299-304行の重複コードをCompareOpsBox.eval()に置換するだけ
- op_handlers.hakoは既にCompareOpsBoxをusing済み（行5）

**依存関係**: なし（leaf Box）

**リスク**: **Low**
- CompareOpsBoxは既存＋テスト済み
- mir_vm_minの2箇所の重複は完全に同一ロジック

**実装ステップ**:
1. mir_vm_min.hako: 行228-233, 299-304の重複コードを `CompareOpsBox.eval(cmp, lv, rv)` に置換
2. op_handlers.hako: 行67-76の `_map_cmp_symbol`, `_eval_cmp` を削除（既にCompareOpsBoxを使用中）
3. スモークテスト実行（`tools/smokes/v2/run.sh --profile quick`）

---

### 2. NumericParserBox（新規Box）

**目的**: 数値パース処理（to_i64, int_to_str, is_numeric_str）の統一

**現状分析**:
- **既存実装**: `apps/selfhost/common/string_helpers.hako` (86行中、数値関連42行)
- **使用箇所**: 24ファイルで `to_i64` / `_str_to_int` を使用
- **重複パターン**:
  ```hako
  _str_to_int(s) { return StringHelpers.to_i64(s) }  // 10ファイルで重複
  ```

**統合対象メソッド**:
- `to_i64(x)`: 文字列→整数変換（"-"対応）
- `int_to_str(n)`: 整数→文字列変換
- `is_numeric_str(s)`: 数値文字列判定
- `read_digits(text, pos)`: 連続数字読み取り

**削減効果**: **間接的削減（ラッパー削除）**
- 現在: 10ファイル × 1行ラッパー = 10行
- 統合後: 0行（直接 NumericParserBox.to_i64() を使用）
- **推定削減**: 10-15行

**実装難易度**: **Low**
- StringHelpersから数値処理部分を分離するだけ
- 既存テスト: `apps/selfhost/test_string_helpers.hako` が存在

**依存関係**:
- StringHelpersBox（分離後は json_quote のみ残す）
- JsonFragBox（`_str_to_int` wrapper削除）

**リスク**: **Low**
- 既存ロジックの移動のみ
- StringHelpersは既に広く使用されており、テスト済み

**実装ステップ**:
1. `apps/selfhost/common/numeric_parser.hako` を新規作成
2. StringHelpers から `to_i64`, `int_to_str`, `is_numeric_str`, `read_digits` を移動
3. 全24ファイルの `using StringHelpers` を `using NumericParserBox` に置換（数値処理のみ）
4. JsonFragBox等のラッパー削除（`_str_to_int` → `NumericParserBox.to_i64`）
5. スモークテスト

---

### 3. JsonScannerBox（既存Box、拡張対象）

**目的**: JSON走査ロジック（seek_array_end, seek_obj_end, scan_string_end）の統一

**現状分析**:
- **既存実装**:
  - `apps/selfhost/common/json/core/json_scan.hako` (73行、seek_obj_end/seek_array_end/find_key_dual)
  - `apps/selfhost/common/json/core/string_scan.hako` (scan_string_end)
  - `apps/selfhost/common/json/json_cursor.hako` (20行、facade)
- **使用箇所**: 12ファイル（grep結果）
- **重複パターン**:
  - mir_vm_min.hako: 行47-48に独自の `_seek_array_end` 実装（20行）

**統合対象ファイル**:
1. `apps/selfhost/common/json/core/json_scan.hako` (保持)
2. `apps/selfhost/common/json/core/string_scan.hako` (保持)
3. `apps/selfhost/common/json/json_cursor.hako` (保持、facade)
4. `apps/selfhost/vm/boxes/mir_vm_min.hako` (行47-49の独自実装を削除)

**削減効果**: 319行 → 299行 (**20行削除、6.3%削減**)

**実装難易度**: **Low**
- mir_vm_min.hakoの `_seek_array_end` をJsonCursorBox.seek_array_end()に置換するだけ
- 既存のJsonCursorBoxが完全に同一機能を提供

**依存関係**:
- JsonCursorBox（既存）
- StringScanBox（既存）

**リスク**: **Low**
- 既存のJsonCursorBoxは広くテスト済み

**実装ステップ**:
1. mir_vm_min.hako の冒頭に `using JsonCursorBox` を追加
2. 行47-49の `_seek_array_end` / `_block_insts_end` を削除
3. 使用箇所（行78等）を `JsonCursorBox.seek_array_end(mjson, insts_start)` に置換
4. スモークテスト

---

### 4. JsonPatternBox（新規Box）

**目的**: JSON key-value検索パターン（get_int, get_str, _find_kv_*）の共通化

**現状分析**:
- **既存実装**:
  - `JsonFragBox.get_int()` / `get_str()` (json_frag.hako)
  - `OpHandlersBox._find_kv_int()` / `_find_kv_str()` (op_handlers.hako)
- **使用箇所**: 7ファイルで61回使用（grep結果）
- **機能重複**:
  ```hako
  // JsonFragBox (json_frag.hako:15-23)
  get_int(seg, key) {
    local pat1 = "\"" + key + "\":"
    local p = me.index_of_from(seg, pat1, 0)
    if p >= 0 {
      local v = me.read_digits(seg, p + pat1.length())
      if v != "" { return me._str_to_int(v) }
    }
    return null
  }

  // OpHandlersBox (op_handlers.hako:47-51)
  _find_kv_int(seg, key) {
    local pat = "\"" + key + "\":"
    return me._find_int_in(seg, pat)
  }
  ```

**統合方針**: **既存のJsonFragBoxを拡張せず、統合は見送り**

**理由**:
1. **設計哲学の違い**:
   - JsonFragBox: null返却（null-safe設計）
   - OpHandlersBox: エラー時print（fail-fast設計）
2. **スコープの違い**:
   - JsonFragBox: 汎用JSON utilities
   - OpHandlersBox: VM固有のエラーハンドリング
3. **統合コスト > 効果**:
   - 削減可能: 10-15行程度
   - 必要な変更: エラーハンドリング方針の統一（高リスク）

**削減効果**: **統合見送り（0行削減）**

**実装難易度**: **High**（設計方針の統一が必要）

**リスク**: **High**（エラーハンドリングの変更はVM動作に影響）

**推奨**: **Phase 4以降で再検討**（エラーハンドリング統一後）

---

### 5. StringBuilderBox（新規Box、優先度Low）

**目的**: 文字列連結最適化（`out = out + ch` パターン）

**現状分析**:
- **使用箇所**: 22ファイルで文字列連結パターン（grep: `out = out +`）
- **典型例**:
  ```hako
  // string_helpers.hako:12-18
  local out = ""
  loop (v > 0) {
    local d = v % 10
    local ch = digits.substring(d, d+1)
    out = ch + out  // ← N回連結（O(N²)）
    v = v / 10
  }
  ```

**統合方針**: **Phase 3では見送り（Phase 4以降）**

**理由**:
1. **パフォーマンス問題は未顕在化**:
   - 現在の文字列は短い（数値変換: 10桁程度）
   - O(N²)でも実用上問題なし
2. **ArrayBox + join() パターンで代替可能**（既存機能で対応可能）
3. **ベンチマーク未実施**（最適化の効果が不明）

**削減効果**: **0行（最適化のみ、行数削減なし）**

**実装難易度**: **Medium**
- ArrayBox-based builder実装は容易
- 全22ファイルのリファクタリングが必要

**リスク**: **Medium**
- 既存の文字列連結ロジックを大幅に変更
- パフォーマンス改善は測定が必要（効果が不明）

**推奨**: **Phase 4（パフォーマンス最適化）で再検討**

---

### 6. ErrorHandlerBox（新規Box、Phase 4以降）

**目的**: エラーハンドリング統一（print("[ERROR] ...") パターン）

**現状分析**:
- **使用箇所**: 10ファイルで25回 `[ERROR]` パターン
- **現在の実装**:
  ```hako
  // 3つのパターンが混在
  print("[ERROR] Missing key: " + key)                    // 即座にprint
  if v == null { print("[ERROR] ...") return }           // fail-fast
  if msg.indexOf("[ERROR]") >= 0 { print(msg) }          // フィルタリング
  ```

**統合方針**: **Phase 3では見送り（Phase 4以降）**

**理由**:
1. **ResultBox が既に存在**:
   - `apps/selfhost/vm/boxes/result_box.hako` (34行)
   - Result.ok(v) / Result.err(msg) パターン実装済み
2. **段階的移行が必要**:
   - 現在: print-based error（即座に表示）
   - 将来: Result-based error（エラー伝播）
   - 中間状態での混在は複雑化
3. **アーキテクチャ変更**:
   - 全関数シグネチャの変更（→ Result返却）
   - 呼び出し側の変更（is_ok() チェック）

**削減効果**: **0行（設計変更のみ）**

**実装難易度**: **High**（全ファイルのエラーハンドリング変更）

**リスク**: **High**（VM動作の根本的変更）

**推奨**: **Phase 4（Result型統一）で計画的に実施**

---

### 7. ResultBox（既存Box、Phase 4以降で拡張）

**目的**: Result型パターンの全面導入

**現状分析**:
- **既存実装**: `apps/selfhost/vm/boxes/result_box.hako` (34行)
  ```hako
  box ResultBox {
    _val: Box
    _err: StringBox
    _ok: IntegerBox

    is_ok() { return me._ok }
    value() { return me._val }
    error() { return me._err }
    unwrap_or(def) { ... }
  }

  static box Result {
    ok(v) { ... }
    err(msg) { ... }
  }
  ```
- **使用箇所**: 現在1ファイル（phi_decode_box.hako）のみ

**統合方針**: **Phase 3では見送り（Phase 4以降で全面展開）**

**理由**:
1. **大規模リファクタリング**:
   - 全関数を Result<T> 返却に変更
   - 呼び出し側を is_ok() チェックパターンに変更
2. **段階的導入が必要**:
   - Phase 3: 現状維持
   - Phase 4: critical path（parser/compiler）から導入
   - Phase 5: VM全体に展開

**削減効果**: **0行（設計改善のみ）**

**実装難易度**: **High**

**リスク**: **High**

**推奨**: **Phase 4で計画的に実施**

---

## 🎯 実装優先順位（費用対効果順）

| 優先度 | Box名 | 削減行数 | 難易度 | リスク | 推定工数 |
|--------|-------|----------|--------|--------|----------|
| **1** | CompareOpsBox統合 | 26行 | Low | Low | 1時間 |
| **2** | JsonScannerBox統合 | 20行 | Low | Low | 1時間 |
| **3** | NumericParserBox新規 | 10-15行 | Low | Low | 2時間 |
| 4 | JsonPatternBox新規 | 10-15行 | High | High | **見送り** |
| 5 | StringBuilderBox | 0行（最適化） | Medium | Medium | **Phase 4** |
| 6 | ErrorHandlerBox | 0行（設計） | High | High | **Phase 4** |
| 7 | ResultBox拡張 | 0行（設計） | High | High | **Phase 4** |

**Phase 3 推奨スコープ**: 優先度1-3のみ（**合計削減: 50-60行、約1.6%削減**）

---

## 📋 推奨実装順序

### **Week 1: 即座に実装可能な統合**（優先度1-2）

#### Day 1: CompareOpsBox統合（26行削減）
```bash
# 実装ステップ
1. apps/selfhost/vm/boxes/mir_vm_min.hako:
   - 行228-233, 299-304 を CompareOpsBox.eval(cmp, lv, rv) に置換

2. apps/selfhost/vm/boxes/op_handlers.hako:
   - 行67-76 の _map_cmp_symbol, _eval_cmp を削除
   - 行67→CompareOpsBox.map_symbol(), 行68→CompareOpsBox.eval() に置換

3. スモークテスト:
   tools/smokes/v2/run.sh --profile quick
```

**期待結果**:
- mir_vm_min.hako: 319行 → 293行
- op_handlers.hako: 143行 → 133行
- **合計削減**: 26行

#### Day 2: JsonScannerBox統合（20行削減）
```bash
# 実装ステップ
1. apps/selfhost/vm/boxes/mir_vm_min.hako:
   - 冒頭に using "selfhost/shared/json/json_cursor.hako" as JsonCursorBox 追加
   - 行47-49 の _seek_array_end, _block_insts_end 削除
   - 行78等の使用箇所を JsonCursorBox.seek_array_end() に置換

2. スモークテスト:
   tools/smokes/v2/run.sh --profile quick
```

**期待結果**:
- mir_vm_min.hako: 293行 → 273行
- **合計削減**: 20行

---

### **Week 2: NumericParserBox新規作成**（優先度3）

#### Day 3-4: NumericParserBox実装（10-15行削減）
```bash
# 実装ステップ
1. apps/selfhost/common/numeric_parser.hako 新規作成:
   static box NumericParserBox {
     to_i64(x) { ... }        // StringHelpers から移動
     int_to_str(n) { ... }
     is_numeric_str(s) { ... }
     read_digits(text, pos) { ... }
   }

2. apps/selfhost/common/string_helpers.hako:
   - 数値処理メソッド削除（42行削除）
   - json_quote() のみ残す（44行に縮小）

3. 全24ファイルの using 修正:
   - StringHelpers → NumericParserBox（数値処理のみ）
   - 例: JsonFragBox, OpHandlersBox, mir_vm_min 等

4. ラッパー削除:
   - JsonFragBox._str_to_int() → NumericParserBox.to_i64()
   - OpHandlersBox._str_to_int() → NumericParserBox.to_i64()

5. スモークテスト:
   tools/smokes/v2/run.sh --profile integration
```

**期待結果**:
- string_helpers.hako: 86行 → 44行（-42行）
- numeric_parser.hako: 0行 → 50行（+50行）
- ラッパー削除: 10ファイル × 1行 = -10行
- **正味削減**: 10-15行

---

## 💡 即座に実装可能なBox（Quick Wins）

### ✅ CompareOpsBox統合（最優先！）

**即座に実装可能な理由**:
1. ✅ CompareOpsBoxは既に存在・テスト済み
2. ✅ 重複コードが完全に同一（行単位で一致）
3. ✅ リスクゼロ（既存Boxへの委譲のみ）
4. ✅ 26行削除（最大の削減効果）

**実装時間**: 1時間以内

**コマンド例**:
```bash
# mir_vm_min.hako の重複を削除
# 行228-233, 299-304 を以下に置換:
local cv = CompareOpsBox.eval(cmp, lv, rv)
```

---

### ✅ JsonScannerBox統合（2番目に優先）

**即座に実装可能な理由**:
1. ✅ JsonCursorBox/JsonScanBoxは既に存在・広く使用中
2. ✅ mir_vm_minの独自実装は完全に冗長
3. ✅ リスク極小（既存のfacade使用）
4. ✅ 20行削除

**実装時間**: 1時間以内

**コマンド例**:
```bash
# mir_vm_min.hako の _seek_array_end を削除
# using JsonCursorBox を追加
# _block_insts_end(mjson, start) → JsonCursorBox.seek_array_end(mjson, start)
```

---

## 📈 Phase 3 削減効果まとめ

| 項目 | Before | After | 削減 |
|------|--------|-------|------|
| **mir_vm_min.hako** | 319行 | 273行 | **-46行** |
| **op_handlers.hako** | 143行 | 133行 | **-10行** |
| **string_helpers.hako** | 86行 | 44行 | **-42行** |
| **ラッパー削除** | 10行 | 0行 | **-10行** |
| **新規ファイル** | - | +50行 | +50行 |
| **正味削減** | - | - | **-58行** |

**全体効果**: 3,573行 → 3,515行（**1.6%削減**）

---

## 🚨 Phase 3で見送るBox（Phase 4以降）

### JsonPatternBox（見送り理由: 設計方針の違い）
- JsonFragBox（null-safe）vs OpHandlersBox（fail-fast）
- エラーハンドリング統一が先決
- 削減効果10行 vs 高リスク

### StringBuilderBox（見送り理由: 効果不明）
- パフォーマンス問題は未顕在化
- ベンチマーク未実施
- ArrayBox + join() で代替可能

### ErrorHandlerBox（見送り理由: 大規模変更）
- Result型統一が先決
- 全関数シグネチャ変更
- 段階的移行計画が必要

### ResultBox拡張（見送り理由: アーキテクチャ変更）
- Phase 4で計画的に実施
- critical path（parser/compiler）から段階導入

---

## 🎯 Phase 3 実装チェックリスト

### Week 1: 即座実装（Quick Wins）
- [ ] Day 1: CompareOpsBox統合（26行削減）
  - [ ] mir_vm_min.hako: 行228-233, 299-304 置換
  - [ ] op_handlers.hako: 行67-76 削除
  - [ ] スモークテスト実行
  - [ ] コミット: "refactor(vm): consolidate comparison operators into CompareOpsBox"

- [ ] Day 2: JsonScannerBox統合（20行削減）
  - [ ] mir_vm_min.hako: using JsonCursorBox 追加
  - [ ] mir_vm_min.hako: 行47-49 削除
  - [ ] スモークテスト実行
  - [ ] コミット: "refactor(vm): use JsonCursorBox.seek_array_end instead of local impl"

### Week 2: NumericParserBox新規
- [ ] Day 3: NumericParserBox作成
  - [ ] numeric_parser.hako 新規作成
  - [ ] StringHelpers から数値処理移動
  - [ ] 単体テスト作成

- [ ] Day 4: 全ファイル移行
  - [ ] 24ファイルの using 修正
  - [ ] ラッパー削除（10ファイル）
  - [ ] 統合スモークテスト実行
  - [ ] コミット: "refactor: extract NumericParserBox from StringHelpers"

### 完了基準
- [ ] 全スモークテスト通過（quick + integration）
- [ ] 削減行数: 50-60行達成
- [ ] 既存機能の完全な動作保証

---

## 📚 参考資料

### 分析対象ファイル
- `apps/selfhost/vm/boxes/mir_vm_min.hako` (319行)
- `apps/selfhost/common/json/mir_builder_min.hako` (397行)
- `apps/selfhost/common/string_helpers.hako` (86行)
- `apps/selfhost/common/json/utils/json_frag.hako` (69行)
- `apps/selfhost/vm/boxes/compare_ops.hako` (25行)
- `apps/selfhost/vm/boxes/op_handlers.hako` (143行)
- `apps/selfhost/vm/boxes/result_box.hako` (34行)

### 既存Box（活用対象）
- CompareOpsBox: apps/selfhost/vm/boxes/compare_ops.hako
- JsonScanBox: apps/selfhost/common/json/core/json_scan.hako
- JsonCursorBox: apps/selfhost/common/json/json_cursor.hako
- ResultBox: apps/selfhost/vm/boxes/result_box.hako

### スモークテスト
```bash
# Quick test (VM)
tools/smokes/v2/run.sh --profile quick

# Integration test (VM + LLVM)
tools/smokes/v2/run.sh --profile integration
```

---

**レポート作成**: Claude Code
**分析手法**: Grep/Read/Bash ツール活用による重複検出
**信頼性**: コード実測値に基づく（推測最小限）
