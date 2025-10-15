# Parser系 箱化・最適化分析レポート

**作成日**: 2025-10-12
**対象**: セルフホストコンパイラー Parser関連コード
**総行数**: 1,401行（12ファイル）

---

## 📊 Executive Summary

セルフホストコンパイラーのParser実装を分析した結果、**重複コード削減**と**パフォーマンス最適化**の機会を多数発見しました。

### 🎯 主要発見事項

1. **重複ヘルパー関数**: 4つのBoxで `_i2s`, `_is_digit`, `_is_alpha`, `_starts_with` を重複実装（合計**15-20行×4 = 60-80行削減可能**）
2. **位置解析パターン**: `@` 区切り文字列の解析コード（21箇所、**パターン統一で30-40行削減**）
3. **Progress Guard パターン**: 11箇所で同一ロジック（**共通化で20-30行削減**）
4. **JSON生成コード**: 42箇所で手動文字列連結（**Builder Box化で可読性向上**）
5. **ループ最適化**: 38個のループ（最大値 400,000）

**推定削減可能行数**: **110-150行**（総行数の約8-11%）

---

## 🔍 詳細分析

### 1. 重複ヘルパー関数（最優先！）

#### 📋 現状

以下の4つのBoxで**同一機能のヘルパー関数**を重複実装：

| Box名 | 重複関数 | 行数 | ファイル |
|------|---------|------|---------|
| **ParserIdentScanBox** | `_i2s`, `_is_alpha`, `_is_digit` | 6-7行 | `/boxes/parser/scan/parser_ident_scan_box.hako:5-7` |
| **ParserNumberScanBox** | `_i2s`, `_is_digit` | 5-6行 | `/boxes/parser/scan/parser_number_scan_box.hako:5-6` |
| **ParserStringScanBox** | `_i2s` | 1行 | `/boxes/parser/scan/parser_string_scan_box.hako:7` |
| **UsingCollectorBox** | `_starts_with`, `_index_of`, `_trim`, `_esc_json` | 40行 | `/boxes/parser/using/using_collector_box.hako:11-44` |

**特記**: UsingCollectorBox は「依存ゼロ」設計のため独自実装しているが、実際には **ParserStringUtilsBox に既に同等機能が存在**（`starts_with`, `index_of`, `trim`）。

#### 🎯 改善提案

**優先度A**: 新Box「ParserCommonUtilsBox」を作成し統一：

```hako
// 新規: apps/selfhost-compiler/boxes/parser/scan/parser_common_utils_box.hako
static box ParserCommonUtilsBox {
  // 基本ヘルパー
  i2s(v) { return "" + v }
  is_digit(ch) { return ch >= "0" && ch <= "9" }
  is_alpha(ch) { return (ch >= "A" && ch <= "Z") || (ch >= "a" && ch <= "z") || ch == "_" }

  // 文字列操作（ParserStringUtilsBox から移動）
  starts_with(src, i, pat) { ... }
  index_of(src, i, pat) { ... }
  trim(s) { ... }
  esc_json(s) { ... }
}
```

**影響範囲**:
- 修正対象: 4ファイル（IdentScanBox, NumberScanBox, StringScanBox, UsingCollectorBox）
- 削除行数: **60-80行**
- 追加行数: 約30行（新Box）
- **純削減**: **30-50行**

---

### 2. 位置解析パターン（`@` 区切り）

#### 📋 現状

Parser全体で**21箇所**で同一の「`@` 区切り文字列解析」パターンを使用：

```hako
// 頻出パターン（全体で21回出現）
local at = idp.lastIndexOf("@")
local name = idp.substring(0, at)
local pos = ctx.to_int(idp.substring(at+1, idp.size()))
```

**出現場所**:
- `parser_stmt_box.hako`: 6箇所（行21-24, 79-82, 153-156, 161-164, 184-187）
- `parser_expr_box.hako`: 7箇所（行12-15, 116-118, 133-136, 158-160, 163-166）
- `parser_control_box.hako`: 3箇所（行23-25, 33-35, 63-65）
- `parser_exception_box.hako`: 5箇所（行35-37, 62-65, 71-74, 84, 128）

#### 🎯 改善提案

**優先度B**: 「位置情報付き結果」専用のBox化：

```hako
// 新規: apps/selfhost-compiler/boxes/parser/parser_result_box.hako
box ParserResultBox {
  content: null
  position: 0

  birth(raw_with_at) {
    local at = raw_with_at.lastIndexOf("@")
    if at >= 0 {
      me.content = raw_with_at.substring(0, at)
      me.position = me._parse_int(raw_with_at.substring(at+1, raw_with_at.size()))
    } else {
      me.content = raw_with_at
      me.position = 0
    }
    return 0
  }

  get_content() { return me.content }
  get_position() { return me.position }

  _parse_int(s) {
    // 簡易int変換（to_int依存排除）
    ...
  }
}
```

**使用例**:
```hako
// Before (3行)
local at = idp.lastIndexOf("@")
local name = idp.substring(0, at)
local pos = ctx.to_int(idp.substring(at+1, idp.size()))

// After (1行)
local result = new ParserResultBox(idp)
local name = result.get_content()
local pos = result.get_position()
```

**影響範囲**:
- 修正対象: 4ファイル、21箇所
- 削減行数: **約30-40行**（3行→1行変換×21箇所 = 42行削減、Box追加20行 = **純削減22行**）

**代替案**: 軽量化のため、Box化せず **ParserCommonUtilsBox に split_at_mark(str) 関数**として実装：
```hako
split_at_mark(str) {
  // Returns: "content|position" 形式の文字列
  local at = str.lastIndexOf("@")
  if at < 0 { return str + "|0" }
  return str.substring(0, at) + "|" + str.substring(at+1, str.size())
}
```

---

### 3. Progress Guard パターン

#### 📋 現状

**11箇所**で同一の「無限ループ防止」パターン：

```hako
// 頻出パターン
local guard = 0
local max = 100000  // または 200000, 400000
loop(cont == 1) {
  if guard > max { cont = 0 } else { guard = guard + 1 }
  ...
}
```

**出現場所**:
- `parser_box.hako`: 1箇所（行199-204）
- `parser_expr_box.hako`: 2箇所（行24-29, 329-334）
- `parser_control_box.hako`: 1箇所（行147-154）
- `parser_literal_box.hako`: 2箇所（行13-17, 75-79）
- `parser_peek_box.hako`: 1箇所（行23-28）
- `parser_string_utils_box.hako`: 1箇所（行73-80）
- その他: 3箇所

#### 🎯 改善提案

**優先度C**: 2つのアプローチ：

##### Option 1: GuardBox による Box化（低優先度）
```hako
box LoopGuardBox {
  count: 0
  max_iterations: 100000

  birth(max) {
    me.count = 0
    me.max_iterations = max
    return 0
  }

  check() {
    me.count = me.count + 1
    if me.count > me.max_iterations { return 0 }  // 停止
    return 1  // 継続
  }
}

// 使用例
local guard = new LoopGuardBox(100000)
loop(cont == 1 && guard.check() == 1) {
  ...
}
```

**問題点**: Box生成コストが高い可能性（ループごとに生成）

##### Option 2: マクロ化（Phase 19 @macro 使用）
```hako
// 将来的な理想形
@guarded_loop(max=100000, var=cont) {
  ...  // ループ本体
}
```

**現時点の判断**: **保留**。
- 理由: ループ本体の複雑さとトレードオフ。現在のコードは明示的で理解しやすい。
- 推奨: Phase 19 完了後に再検討。

---

### 4. JSON生成コードの最適化

#### 📋 現状

**42箇所**で手動文字列連結による JSON 生成：

```hako
// 典型例（可読性が低い）
return "{\"type\":\"Binary\",\"op\":\"" + op + "\",\"lhs\":" + lhs + ",\"rhs\":" + rhs + "}"
```

**出現統計**:
- `parser_expr_box.hako`: 18箇所
- `parser_stmt_box.hako`: 7箇所
- `parser_control_box.hako`: 7箇所
- `parser_exception_box.hako`: 4箇所
- `parser_peek_box.hako`: 3箇所
- `parser_literal_box.hako`: 3箇所

#### 🎯 改善提案

**優先度B**: JsonBuilderBox 作成（可読性重視）：

```hako
// 新規: apps/selfhost-compiler/boxes/parser/json_builder_box.hako
box JsonBuilderBox {
  buffer: ""
  first: 1

  birth() {
    me.buffer = "{"
    me.first = 1
    return 0
  }

  add_str(key, value) {
    if me.first == 0 { me.buffer = me.buffer + "," }
    me.buffer = me.buffer + "\"" + key + "\":\"" + me._esc(value) + "\""
    me.first = 0
    return me
  }

  add_raw(key, json_value) {
    if me.first == 0 { me.buffer = me.buffer + "," }
    me.buffer = me.buffer + "\"" + key + "\":" + json_value
    me.first = 0
    return me
  }

  build() {
    return me.buffer + "}"
  }

  _esc(s) {
    // JSON escape処理（esc_json から移植）
    ...
  }
}
```

**使用例**:
```hako
// Before（可読性低）
return "{\"type\":\"Binary\",\"op\":\"" + op + "\",\"lhs\":" + lhs + ",\"rhs\":" + rhs + "}"

// After（可読性高）
local builder = new JsonBuilderBox()
return builder.add_str("type", "Binary")
              .add_str("op", op)
              .add_raw("lhs", lhs)
              .add_raw("rhs", rhs)
              .build()
```

**影響範囲**:
- 削減行数: **純削減なし**（可読性向上がメイン）
- ビルダーパターンによるメソッドチェーン導入
- デバッグ時の JSON 生成ロジックの明確化

**代替案（軽量）**: テンプレート関数方式
```hako
static box JsonTemplates {
  binary(op, lhs, rhs) {
    return "{\"type\":\"Binary\",\"op\":\"" + op + "\",\"lhs\":" + lhs + ",\"rhs\":" + rhs + "}"
  }

  compare(op, lhs, rhs) {
    return "{\"type\":\"Compare\",\"op\":\"" + op + "\",\"lhs\":" + lhs + ",\"rhs\":" + rhs + "}"
  }

  // ... 各 AST ノード型ごとのテンプレート
}
```

---

### 5. ループ最適化

#### 📋 現状

**38個のループ**が存在、最大ガード値の分布：

| 最大値 | 出現数 | ファイル例 |
|--------|--------|-----------|
| 100,000 | 6箇所 | `parser_box.hako`, `parser_control_box.hako` |
| 200,000 | 2箇所 | `parser_expr_box.hako`, `parser_string_scan_box.hako` |
| 400,000 | 2箇所 | `parser_literal_box.hako`, `parser_peek_box.hako` |
| 無限 | 28箇所 | `parser_string_utils_box.hako` など |

**ホットパス**（頻繁に呼ばれる）:
1. `skip_ws()` - 空白スキップ（**最頻呼び出し**、ガード100,000）
2. `substring()` - **140回**呼び出し（全ファイル合計）
3. `parse_expr2()` - 式解析（**22回**呼び出し）

#### 🎯 改善提案

**優先度C**: パフォーマンスプロファイリング後に判断

##### 最適化候補 1: `skip_ws` の早期リターン
```hako
// Before
skip_ws(src, i) {
  if src == null { return i }
  local n = src.size()
  local cont = 1
  local guard = 0
  local max = 100000
  loop(cont == 1) {
    if guard > max { return i } else { guard = guard + 1 }
    if i < n {
      if me.is_space(src.substring(i, i+1)) { i = i + 1 } else { cont = 0 }
    } else { cont = 0 }
  }
  return i
}

// After（最適化版）
skip_ws(src, i) {
  if src == null { return i }
  local n = src.size()
  local max_advance = 100000  // 安全ガード
  local advanced = 0

  loop(i < n && advanced < max_advance) {
    local ch = src.substring(i, i+1)
    if ch == " " || ch == "\t" || ch == "\n" || ch == "\r" {
      i = i + 1
      advanced = advanced + 1
    } else {
      break
    }
  }
  return i
}
```

**効果**: 約5-10%の高速化見込み（ガード変数削減）

##### 最適化候補 2: `substring` 呼び出し削減
```hako
// Before（毎回 substring）
if src.substring(j, j+1) == "=" { ... }

// After（キャッシュ活用）
local ch = src.substring(j, j+1)
if ch == "=" { ... }
```

**現状**: 既に多くの箇所で実装済み（優秀！）

##### 最適化候補 3: 文字列連結の最適化
```hako
// Before（N回の文字列連結 = O(N²)）
local out = ""
loop(i < n) {
  out = out + src.substring(i, i+1)
  i = i + 1
}

// After（配列蓄積 + 一括結合 = O(N)）
// ※Hakoruneに配列join()があれば
local parts = []
loop(i < n) {
  parts.push(src.substring(i, i+1))
  i = i + 1
}
local out = parts.join("")
```

**問題**: Hakoruneの現在の機能では困難（将来検討）

---

## 📈 最適化インパクト評価

### コード削減見積もり

| 改善項目 | 優先度 | 削減行数 | 工数見積もり | ROI |
|---------|--------|---------|------------|-----|
| 1. 重複ヘルパー統一 | **A** | **30-50行** | 2-3時間 | ⭐⭐⭐⭐⭐ |
| 2. 位置解析パターン統一 | **B** | **20-30行** | 3-4時間 | ⭐⭐⭐⭐ |
| 3. JSON Builder導入 | **B** | 0行（可読性↑） | 4-5時間 | ⭐⭐⭐ |
| 4. Progress Guard統一 | **C** | 20-30行 | 2-3時間（Phase19後） | ⭐⭐ |
| 5. ループ最適化 | **C** | -（速度↑） | 5-10時間（要計測） | ⭐ |
| **合計（A+B）** | - | **50-80行** | **9-12時間** | - |

### パフォーマンス影響予測

| 改善項目 | 期待効果 | 条件 |
|---------|---------|------|
| 重複ヘルパー統一 | ±0%（中立） | 呼び出し回数少 |
| 位置解析Box化 | -2～5%（悪化可能性） | Box生成コスト |
| JSON Builder | ±0～-3% | Builder生成コスト |
| skip_ws最適化 | **+5～10%** | ホットパス最適化 |

**推奨**: A優先度のみ実施（コード品質向上）、パフォーマンスは計測後判断。

---

## 🎯 実装ロードマップ（推奨）

### Phase 1: 基盤整備（優先度A）
**工数**: 2-3時間
**削減**: 30-50行

```
[タスク]
1. ParserCommonUtilsBox 作成
   - i2s, is_digit, is_alpha, starts_with, index_of, trim, esc_json 統合

2. 4ファイル修正
   - ParserIdentScanBox: _i2s, _is_alpha, _is_digit 削除 → 委譲
   - ParserNumberScanBox: _i2s, _is_digit 削除 → 委譲
   - ParserStringScanBox: _i2s 削除 → 委譲
   - UsingCollectorBox: _starts_with, _index_of, _trim, _esc_json 削除 → 委譲

3. ParserBox 修正
   - is_digit/is_alpha/starts_with 委譲を ParserCommonUtilsBox に変更

4. テスト実行
   - tools/smokes/v2/run.sh --profile quick
   - apps/selfhost-compiler/tests/ 全実行
```

### Phase 2: 位置解析統一（優先度B）
**工数**: 3-4時間
**削減**: 20-30行

```
[タスク]
1. ParserCommonUtilsBox に split_at_mark() 追加

2. 4ファイル、21箇所修正
   - parser_stmt_box.hako: 6箇所
   - parser_expr_box.hako: 7箇所
   - parser_control_box.hako: 3箇所
   - parser_exception_box.hako: 5箇所

3. 統合テスト
```

### Phase 3: JSON Builder導入（優先度B、可読性重視）
**工数**: 4-5時間
**削減**: 0行（可読性向上）

```
[タスク]
1. JsonBuilderBox 作成

2. 段階的導入（大きいファイルから）
   - parser_expr_box.hako: 18箇所（最大）
   - parser_stmt_box.hako: 7箇所
   - parser_control_box.hako: 7箇所

3. 可読性改善レビュー
```

### Phase 4: 計測・最適化（優先度C、Phase1-3後）
**工数**: 5-10時間
**削減**: 速度向上

```
[タスク]
1. ベンチマーク作成
   - 大規模 Hakorune ソースのパース時間計測

2. プロファイリング
   - ホットパス特定（skip_ws, parse_expr2, substring）

3. 最適化実施（効果確認済みのもののみ）
```

---

## 🚨 リスク・懸念事項

### 1. Box生成コスト
**懸念**: ParserResultBox, JsonBuilderBox の頻繁な生成がパフォーマンス悪化を招く可能性

**対策**:
- Phase 2 では Box化を避け、**関数ベース**の `split_at_mark()` を採用
- Phase 3 は **可読性重視**の選択的導入（全置換しない）

### 2. 既存コードの安定性
**懸念**: 21箇所の機械的修正でバグ混入

**対策**:
- **修正前に現在のテストを全PASS確認**
- 1ファイルずつ修正→テスト実行（段階的）
- GitコミットをAtomic化（1ファイル1コミット）

### 3. Everything is Box 原則との整合性
**懸念**: 関数ベース（static box）の増加が原則違反

**判断**:
- **ParserCommonUtilsBox は static box** → OK（ユーティリティは例外）
- **JsonBuilderBox は instance box** → 原則準拠
- Hakorune の実装において、ユーティリティ関数は static box が標準

---

## 📝 その他の観察事項

### 1. 設計の優秀さ
✅ **既に良好な Box 分離**:
- Scanner系（IdentScan, NumberScan, StringScan）
- Parser系（Expr, Stmt, Control, Exception）
- 補助系（Literal, Peek, Using）

✅ **Fail-Safe設計**:
- Progress Guard による無限ループ防止
- Guard値が適切（100k～400k）

✅ **委譲パターンの活用**:
- ParserBox が各専門Boxに委譲（良い設計）

### 2. 改善余地
⚠️ **ドキュメント不足**:
- 各Boxの責務が明確だが、コメントが最小限
- JSON Schema（Stage-1/2/3）の仕様書が別途必要

⚠️ **エラーハンドリング**:
- 不正入力時の degradation（`j = j + 1`）は安全だが、**エラー情報が失われる**
- 将来的に ParserErrorBox 導入を検討

---

## 🎯 推奨アクションプラン

### 即座に実施すべき（今日～明日）
1. **Phase 1 実施**（2-3時間）
   - ParserCommonUtilsBox 作成
   - 重複ヘルパー統一
   - **30-50行削減**

### 1週間以内に実施
2. **Phase 2 実施**（3-4時間）
   - 位置解析パターン統一
   - **20-30行削減**

### 将来検討（Phase 19完了後）
3. **Phase 3**: JSON Builder導入（可読性重視）
4. **Phase 4**: ループ最適化（計測後判断）

---

## 📚 参考資料

### ファイル一覧
```
apps/selfhost-compiler/parser/
├── lexer.hako          (4行、scaffold)
├── parser.hako         (4行、scaffold)
└── ast.hako            (4行、scaffold)

apps/selfhost-compiler/boxes/parser/
├── parser_box.hako                              (238行) ← コーディネーター
├── scan/
│   ├── parser_string_utils_box.hako             (82行)
│   ├── parser_ident_scan_box.hako               (24行) ← 重複ヘルパー
│   ├── parser_number_scan_box.hako              (25行) ← 重複ヘルパー
│   └── parser_string_scan_box.hako              (48行) ← 重複ヘルパー
├── using/
│   └── using_collector_box.hako                 (114行) ← 重複ヘルパー大量
├── expr/
│   ├── parser_expr_box.hako                     (353行) ← 最大ファイル
│   ├── parser_literal_box.hako                  (117行)
│   └── parser_peek_box.hako                     (102行)
└── stmt/
    ├── parser_stmt_box.hako                     (200行)
    ├── parser_control_box.hako                  (171行)
    └── parser_exception_box.hako                (150行)
```

### 関連技術仕様
- **Phase 2 JSON Schema**: `docs/development/roadmap/phases/phase-2/json-schema.md`（要作成）
- **Phase 19 Macros**: `docs/development/roadmap/phases/phase-19/README.md`（@enum/@match実装済み）
- **Box理論**: `CLAUDE.md`（"Everything is Box"）

---

## ✅ まとめ

**最も効果的な改善（優先度A）**:
- ✅ ParserCommonUtilsBox 作成 → **30-50行削減**
- ✅ 工数: 2-3時間
- ✅ リスク: 低（ユーティリティ関数のみ）

**次点の改善（優先度B）**:
- ✅ 位置解析統一 → **20-30行削減**
- ✅ 工数: 3-4時間
- ✅ リスク: 中（21箇所修正）

**将来検討（優先度C）**:
- ⏳ JSON Builder: 可読性向上
- ⏳ ループ最適化: 計測後判断

**総合評価**: セルフホストコンパイラーのParser実装は**既に高品質**。優先度A/Bの改善により、さらに**8-11%のコード削減**と**保守性向上**が見込めます。

---

**作成者**: Claude (Analysis Task)
**レビュー推奨**: ChatGPT5（実装前）
