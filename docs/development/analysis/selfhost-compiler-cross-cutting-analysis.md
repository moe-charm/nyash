# セルフホストコンパイラー 横断的重複コード分析レポート

**分析日**: 2025-10-12
**分析対象**: apps/selfhost-compiler/ 全体 (71ファイル、5,733行)
**目的**: 共通ユーティリティ・重複コード特定、箱化・最適化機会の発見

---

## 📊 エグゼクティブサマリー

### 核心発見
- **重複コード: 高度に分散** - 文字列操作関数が20+ファイルに散在
- **Everything is Box準拠度: 65%** - 一部は箱化済みだが、統合の余地あり
- **最大の機会**: StringUtilsBox統合で **推定300-400行削減可能**

### 優先度付き推奨事項
1. **超高優先度**: StringUtilsBox統一 (影響: 30+ファイル)
2. **高優先度**: JSON操作ヘルパー統合 (影響: 15+ファイル)
3. **中優先度**: MapHelpersBox拡張 (型安全アクセサ標準化)
4. **低優先度**: DebugBox共通化 (現在3箇所のみ使用)

---

## 📈 統計データ

### ファイル構成
```
総ファイル数: 71 *.hako
総行数: 5,733行
ディレクトリ構造:
  - pipeline_v2/: 36ファイル (main compiler pipeline)
  - boxes/parser/: 20+ファイル (parser boxes)
  - common/: 5ファイル (共通ユーティリティ)
  - builder/: SSA構築系
```

### コード密度
- **substring()**: 258回 (30ファイル)
- **.size()**: 216回 (37ファイル)
- **indexOf/lastIndexOf**: 72回 (15ファイル)
- **trim/skip_ws**: 103回 (10ファイル)
- **escape_string/quote**: 17回 (5ファイル)
- **新規Box生成**: 33回 (new ArrayBox/MapBox)
- **loop()**: 114回
- **if文**: 1007回
- **local変数**: 1224回
- **null チェック**: 38ファイル

---

## 🔍 横断的重複パターン詳細分析

### 1. 文字列操作の重複 (最重要!)

#### パターンA: 文字列検索・部分文字列
**重複箇所**: 30+ファイル

**具体例**:
```hakorune
// boxes/json_program_box.hako (531行)
index_of(s, start, pat) {
  local n = s.size()
  local m = pat.size()
  if m == 0 { return start }
  local i = start
  loop(i + m <= n) {
    local j = 0
    local matched = 1
    loop(j < m) {
      if s.substring(i + j, i + j + 1) != pat.substring(j, j + 1) {
        matched = 0
        break
      }
      j = j + 1
    }
    if matched == 1 { return i }
    i = i + 1
  }
  return -1
}

// boxes/parser/scan/parser_string_utils_box.hako (83行)
index_of(src, i, pat) {
  local n = src.size()
  local m = pat.size()
  if m == 0 { return i }
  local j = i
  loop(j + m <= n) {
    if me.starts_with(src, j, pat) { return j }
    j = j + 1
  }
  return -1
}

// boxes/mir_emitter_box.hako (230行)
_index_of_from(hay, needle, pos) {
  if pos < 0 { pos = 0 }
  local n = hay.size()
  if pos >= n { return -1 }
  local m = needle.size()
  if m <= 0 { return pos }
  local i = pos
  local limit = n - m
  loop (i <= limit) {
    local seg = hay.substring(i, i + m)
    if seg == needle { return i }
    i = i + 1
  }
  return -1
}
```

**影響**: 22ファイルに同様のロジックが存在 (index_of関連)

#### パターンB: trim/skip_ws (空白スキップ)
**重複箇所**: 10ファイル

**具体例**:
```hakorune
// boxes/json_program_box.hako
skip_ws(s, idx) {
  local i = idx
  local n = s.size()
  loop(i < n) {
    local ch = s.substring(i, i + 1)
    if ch == " " || ch == "\n" || ch == "\r" || ch == "\t" {
      i = i + 1
    } else {
      break
    }
  }
  return i
}

trim(s) {
  if s == null { return "" }
  local n = s.size()
  local i = 0
  loop(i < n && (s.substring(i, i + 1) == " " || ...)) {
    i = i + 1
  }
  local j = n
  loop(j > i && (s.substring(j - 1, j) == " " || ...)) {
    j = j - 1
  }
  return s.substring(i, j)
}

// boxes/parser/scan/parser_string_utils_box.hako
trim(s) {
  local i = 0
  local n = s.size()
  loop(i < n && (s.substring(i,i+1) == " " || s.substring(i,i+1) == "\t")) {
    i = i + 1
  }
  local j = n
  loop(j > i && (s.substring(j-1,j) == " " || ...)) {
    j = j - 1
  }
  return s.substring(i, j)
}
```

**影響**: 10ファイルに同様のtrim/skip_ws実装

#### パターンC: 文字列エスケープ/クォート
**重複箇所**: 5ファイル

**具体例**:
```hakorune
// boxes/json_program_box.hako
escape_string(s) {
  if s == null { return "" }
  local out = ""
  local i = 0
  local n = s.size()
  loop(i < n) {
    local ch = s.substring(i, i + 1)
    if ch == "\\" { out = out + "\\\\" }
    else if ch == "\"" { out = out + "\\\"" }
    else if ch == "\n" { out = out + "\\n" }
    // ... (pattern repeats 5 times)
    i = i + 1
  }
  return out
}

quote(s) {
  if s == null { s = "" }
  return "\"" + me.escape_string(s) + "\""
}

// apps/selfhost/common/string_helpers.hako
json_quote(s) {
  if s == null { return "\"\"" }
  local out = ""
  // ... (identical escape logic)
  return "\"" + out + "\""
}
```

**影響**: 5ファイルに同様のエスケープロジック

#### パターンD: 数値変換 (to_int/i2s)
**重複箇所**: 22ファイル

**具体例**:
```hakorune
// boxes/parser/scan/parser_string_utils_box.hako
to_int(s) {
  local n = s.size()
  if n == 0 { return 0 }
  local i = 0
  local acc = 0
  loop(i < n) {
    local d = s.substring(i, i+1)
    local dv = 0
    if d == "1" { dv = 1 }
    else if d == "2" { dv = 2 }
    // ... nested if hell (10+ levels)
    acc = acc * 10 + dv
    i = i + 1
  }
  return acc
}

i2s(v) { return "" + v }

// apps/selfhost/common/string_helpers.hako
to_i64(x) {
  local s = "" + x
  local i = 0
  local neg = 0
  if s.substring(0,1) == "-" { neg = 1  i = 1 }
  local n = s.size()
  if i >= n { return 0 }
  local acc = 0
  loop (i < n) {
    local ch = call("String.substring/2", s, i, i+1)
    if ch < "0" || ch > "9" { break }
    local ds = "0123456789"
    local dpos = ds.indexOf(ch)
    if dpos < 0 { break }
    acc = acc * 10 + dpos
    i = i + 1
  }
  if neg == 1 { return 0 - acc }
  return acc
}
```

**影響**: 22ファイルに数値変換ロジック散在

---

### 2. JSON操作の重複

#### パターンE: JSON読み取り・パース
**重複箇所**: JsonProgramBox (531行) に巨大な実装

**特徴**:
- `read_string()`, `read_object()`, `read_array()`, `read_literal()`
- `extract_value()`, `extract_string_value()`
- `split_top_level()` (JSON配列のトップレベル分割)

**問題点**:
- JsonProgramBox単独で531行 (全体の9.3%)
- 他のファイルからも類似ロジックが散見される
- JSON操作が高度に特殊化されている (boxes内に閉じている)

#### パターンF: JSON minify/stringify
**重複箇所**: 4ファイル

```hakorune
// pipeline_v2/json_minify_box.hako (38行)
minify(text) {
  // whitespace stripping inside JSON
  local in_str = 0
  loop(i < n) {
    local ch = s.substring(i, i+1)
    if in_str == 1 {
      if ch == "\\" { /* escape handling */ }
      else if ch == "\"" { in_str = 0 }
    } else {
      if ch == "\"" { in_str = 1 }
      else if ch == " " || ch == "\n" || ... { /* skip */ }
    }
    i = i + 1
  }
}

// JSON.stringify() usage: 7回 (4ファイル)
```

---

### 3. MapBox/ArrayBox操作の重複

#### パターンG: 型付きMapアクセサ
**統一済み**: MapHelpersBox (48行) に集約

**良い設計例**:
```hakorune
// pipeline_v2/map_helpers_box.hako
static box MapHelpersBox {
  get_str(m, key) { /* safe string extraction */ }
  get_i64(m, key) { /* safe int extraction */ }
  expect_str(m, key) { /* required field */ }
  opt_str(m, key, def) { /* optional with default */ }
  opt_i64(m, key, def) { /* optional int with default */ }
}
```

**問題点**:
- 使用箇所: わずか10ファイル程度
- 他の23ファイルでは生の `.get()` / `.set()` を直接使用
- 型安全性が不均一

---

### 4. デバッグ・ロギングの重複

#### パターンH: ConsoleBox生成の散在
**重複箇所**: ConsoleBox生成が1ファイルに3回

```hakorune
// boxes/debug_box.hako (39行)
box DebugBox {
  enabled
  birth() { me.enabled = 0 }
  log(msg) {
    if me.enabled {
      local c = new ConsoleBox()  // ← 毎回生成!
      c.println("[DEBUG] " + msg)
    }
  }
  info(msg) {
    if me.enabled {
      local c = new ConsoleBox()  // ← 毎回生成!
      c.println("[INFO] " + msg)
    }
  }
  error(msg) {
    if me.enabled {
      local c = new ConsoleBox()  // ← 毎回生成!
      c.println("[ERROR] " + msg)
    }
  }
}
```

**問題点**:
- ConsoleBox を毎回生成 (3箇所)
- DebugBox の使用箇所はわずか3箇所のみ (採用率低い)
- 他のファイルでは直接 print() を使用

---

### 5. MIR/emit系の重複

#### パターンI: MIR命令生成ヘルパー
**部分的に統一済み**:

```hakorune
// common/mir_emit_box.hako (14行) - 基本命令のみ
make_const(dst, val)
make_compare(kind, lhs, rhs, dst)
make_copy(dst, src)
make_branch(cond, then_id, else_id)
make_jump(target)
make_ret(val)

// common/call_emit_box.hako (38行) - call系
make_call(name, arg_ids, dst)
make_boxcall(method, recv_id, arg_ids, dst)
make_mir_call_global(name, arg_ids, dst)
make_mir_call_extern(name, arg_ids, dst)
make_mir_call_method(method, recv_id, arg_ids, dst)
make_mir_call_constructor(box_type, arg_ids, dst)

// common/newbox_emit_box.hako (35行) - newbox系
make_new(box_type, arg_ids, dst)
with_args_array(node, arg_ids)
with_args_text(node, args_text)

// common/header_emit_box.hako (23行) - ヘッダ系
make_block(id, insts)
make_function_main(blocks)
make_module_with_functions(fns)
```

**評価**:
- ✅ 適切に箱化されている
- ✅ 責務が明確に分離
- ⚠️ 統一的なインポート戦略がない (個別import)

---

### 6. using/namespace解決の重複

#### パターンJ: 名前解決ロジック
**統一済み**: UsingResolverBox (249行)、NamespaceBox

**良い設計**:
```hakorune
// pipeline_v2/using_resolver_box.hako
box UsingResolverBox {
  birth() { /* initialize maps */ }
  load_usings_json(json) { /* parse using declarations */ }
  load_modules_json(json) { /* load module metadata */ }
  resolve_namespace_alias(alias) { /* alias → namespace */ }
  upgrade_aliases() { /* module-first resolution */ }
}

// pipeline_v2/namespace_box.hako
static box NamespaceBox {
  normalize_global_name(raw, resolver)
  normalize_class_name(raw, resolver)
}
```

**評価**:
- ✅ 適切に箱化
- ✅ パイプラインで一貫して使用 (pipeline.hako で25+箇所)

---

## 🎯 箱化機会の定量分析

### 推奨: 統一StringUtilsBox

**対象機能**:
```hakorune
static box StringUtilsBox {
  // 文字列検索 (22ファイル → 1箇所)
  index_of(s, start, pat)
  last_index_of(s, pat)
  starts_with(s, i, pat)

  // 空白処理 (10ファイル → 1箇所)
  trim(s)
  skip_ws(s, idx)

  // 文字判定 (parser系で分散)
  is_digit(ch)
  is_alpha(ch)
  is_space(ch)

  // 数値変換 (22ファイル → 1箇所)
  to_i64(x)         // マイナス対応
  read_digits(text, pos)
  int_to_str(n)
  i2s(v)

  // JSON文字列操作 (5ファイル → 1箇所)
  escape_string(s)
  unescape_string(s)
  json_quote(s)
}
```

**削減見積もり**:
- **重複コード**: 30ファイル × 平均10-15行 = **300-450行削減**
- **純削減**: 統一実装80行 → **実質220-370行削減**

**影響ファイル**:
- JsonProgramBox (531行 → 450行程度、15%削減)
- ParserStringUtilsBox (83行 → 20行程度、75%削減)
- MirEmitterBox (230行 → 200行程度、13%削減)
- 他20+ファイル

---

### 推奨: JsonUtilsBox (高度JSON操作)

**対象機能**:
```hakorune
static box JsonUtilsBox {
  // JSON読み取り (JsonProgramBox から抽出)
  read_string(json, idx)
  read_object(json, idx)
  read_array(json, idx)
  read_literal(json, idx)
  skip_string(json, idx)

  // JSON値抽出
  extract_value(json, key)
  extract_string_value(json, key, default)

  // JSON配列操作
  split_top_level(array_json)

  // JSON整形
  minify(text)
}
```

**削減見積もり**:
- **JsonProgramBox内部**: 200行程度を移動
- **外部化による可読性向上**: JsonProgramBox 531行 → 330行 (38%削減)
- **再利用可能性**: 他のJSON操作箇所でも利用可能

---

### 推奨: MapHelpersBox拡張 (型安全アクセサ標準化)

**現状**: 10ファイルのみ使用、他23ファイルは生アクセス

**拡張提案**:
```hakorune
static box MapHelpersBox {
  // 既存 (良好)
  get_str(m, key)
  get_i64(m, key)
  expect_str(m, key)
  opt_str(m, key, def)
  opt_i64(m, key, def)

  // 追加提案
  get_array(m, key)          // ArrayBox取得
  expect_array(m, key)       // required array
  get_map(m, key)            // nested MapBox
  has_key(m, key)            // null-safe existence check
  set_if_not_null(m, key, v) // conditional set
}
```

**削減見積もり**:
- **生アクセス置き換え**: 23ファイル × 5-10箇所 = 115-230箇所
- **エラーハンドリング統一**: null安全性の一貫性向上
- **純削減**: 50-80行程度 (冗長なnullチェック削減)

---

### オプション: DebugBox改善

**現状問題**:
- ConsoleBox を毎回生成 (非効率)
- 使用箇所わずか3箇所 (採用率低い)

**改善案1**: ConsoleBox再利用
```hakorune
box DebugBox {
  enabled
  console   // ← ConsoleBox をフィールド化

  birth() {
    me.enabled = 0
    me.console = new ConsoleBox()  // 1回のみ生成
  }

  log(msg) {
    if me.enabled {
      me.console.println("[DEBUG] " + msg)
    }
  }
}
```

**改善案2**: static box化 (よりHakorune的)
```hakorune
static box DebugBox {
  enabled: IntegerBox

  init() {
    me.enabled = 0
  }

  log(msg) {
    if me.enabled {
      print("[DEBUG] " + msg)  // 直接print (ConsoleBox不要)
    }
  }
}
```

**削減見積もり**: 5-10行程度 (小規模改善)

---

## 📊 Everything is Box 準拠度スコア

### スコア: **65/100** (良好だが改善余地あり)

#### 採点基準:
| カテゴリ | 現状 | 理想 | スコア |
|---------|------|------|--------|
| **ユーティリティ箱化** | 部分的 | 完全統一 | 60/100 |
| **emit系の箱化** | 良好 | 完璧 | 85/100 |
| **データ構造の箱化** | 良好 | 完璧 | 80/100 |
| **デバッグ系の箱化** | 低採用率 | 標準化 | 30/100 |
| **パイプライン統合** | 優秀 | 完璧 | 90/100 |

#### 採点詳細:

**✅ 箱化が優秀な領域 (80-90点)**:
1. **MIR emit系**: MirEmitBox, CallEmitBox, NewBoxEmitBox など明確に分離
2. **パイプライン系**: UsingResolverBox, NamespaceBox など責務明確
3. **パイプライン構成**: flow PipelineV2 による統一的な制御フロー

**⚠️ 箱化が不十分な領域 (30-60点)**:
1. **文字列操作**: 30+ファイルに散在 (StringHelpers.hakoは存在するが統合不十分)
2. **JSON操作**: JsonProgramBox に巨大実装 (531行)、他に波及していない
3. **MapBox操作**: MapHelpersBox はあるが採用率低い (10/33ファイル)
4. **デバッグ**: DebugBox は存在するが使用箇所わずか3箇所

**❌ 箱化されていない領域**:
1. **ループガード**: 複数ファイルで独自実装 (guard/max_iterations)
2. **エラーハンドリング**: null チェックが散在 (統一パターンなし)

---

## 🚀 箱化・最適化ロードマップ

### Phase 1: 基盤ユーティリティ統一 (推定20-30時間)

#### Week 1: StringUtilsBox統合
**目標**: 30+ファイルの文字列操作を1箇所に集約

**作業内容**:
1. 既存 `apps/selfhost/common/string_helpers.hako` を拡張
2. 以下の機能を統合:
   - `index_of`, `last_index_of`, `starts_with`
   - `trim`, `skip_ws`
   - `is_digit`, `is_alpha`, `is_space`
   - `to_i64`, `read_digits`, `int_to_str`, `i2s`
   - `escape_string`, `unescape_string`, `json_quote`

3. 影響ファイル更新 (30ファイル):
   - JsonProgramBox (531行)
   - ParserStringUtilsBox (83行)
   - MirEmitterBox (230行)
   - 他27ファイル

**期待成果**:
- **純削減**: 220-370行
- **可読性向上**: 重複ロジック削除
- **保守性向上**: 単一ソース原則確立

**リスク**:
- 影響範囲広い (30ファイル) → 段階的移行が必要
- テスト不足のリスク → スモークテスト必須

---

#### Week 2: JsonUtilsBox抽出
**目標**: JsonProgramBox (531行) を責務分離

**作業内容**:
1. 新規 `apps/selfhost-compiler/common/json_utils_box.hako` 作成
2. JsonProgramBox から以下を移動:
   - `read_string`, `read_object`, `read_array`, `read_literal`
   - `extract_value`, `extract_string_value`
   - `split_top_level`
   - JsonMinifyBox の minify() も統合

3. JsonProgramBox をリファクタ:
   - 531行 → 330行程度 (38%削減)
   - 純粋な JSON v0 正規化に集中

**期待成果**:
- **純削減**: 50-80行 (重複排除)
- **可読性向上**: JsonProgramBox が読みやすくなる
- **再利用性向上**: JsonUtilsBox が他でも使える

**リスク**:
- JsonProgramBox は最大ファイル (531行) → 慎重な抽出が必要
- 依存関係複雑 → 段階的テストが必須

---

### Phase 2: 型安全アクセサ標準化 (推定10-15時間)

#### Week 3: MapHelpersBox拡張と採用促進
**目標**: 生MapBoxアクセスを型安全アクセサに置き換え

**作業内容**:
1. MapHelpersBox に機能追加:
   - `get_array(m, key)`
   - `expect_array(m, key)`
   - `get_map(m, key)`
   - `has_key(m, key)`
   - `set_if_not_null(m, key, v)`

2. 23ファイルの生アクセスを置き換え:
   - `m.get(key)` → `MapHelpersBox.get_str(m, key)`
   - `if m == null || m.get(key) == null` → `MapHelpersBox.opt_str(m, key, "")`

**期待成果**:
- **純削減**: 50-80行 (冗長nullチェック削減)
- **型安全性向上**: null安全保証
- **エラーハンドリング統一**: 一貫性向上

**リスク**:
- 影響範囲広い (23ファイル) → 段階的移行
- パフォーマンス懸念 (追加関数呼び出し) → ベンチマーク必要

---

### Phase 3: オプション改善 (推定5-10時間)

#### 任意: DebugBox改善
**目標**: ConsoleBox再生成の排除

**作業内容**:
1. DebugBox を static box 化
2. ConsoleBox生成を1回のみに削減
3. 採用促進 (現在3箇所 → 10+箇所)

**期待成果**:
- **純削減**: 5-10行
- **パフォーマンス向上**: 微小 (ConsoleBox生成回数削減)
- **採用率向上**: デバッグの標準化

**リスク**: 低 (影響範囲小さい)

---

## 📋 統合優先度マトリックス

| 項目 | 影響度 | 削減行数 | 実装工数 | 優先度 |
|-----|--------|---------|---------|--------|
| **StringUtilsBox統合** | 🔴 超高 (30ファイル) | 220-370行 | 20-30h | **P0** |
| **JsonUtilsBox抽出** | 🟠 高 (5ファイル) | 50-80行 | 10-15h | **P1** |
| **MapHelpersBox拡張** | 🟡 中 (23ファイル) | 50-80行 | 10-15h | **P2** |
| **DebugBox改善** | 🟢 低 (3ファイル) | 5-10行 | 5-10h | **P3** |

---

## 🎓 学び: Everything is Box の成功事例

### 成功例1: emit系の箱化
**ファイル**:
- `common/mir_emit_box.hako` (14行)
- `common/call_emit_box.hako` (38行)
- `common/newbox_emit_box.hako` (35行)
- `common/header_emit_box.hako` (23行)

**成功要因**:
- ✅ **責務が明確**: 各Boxが1つのMIR命令カテゴリを担当
- ✅ **薄いファサード**: 複雑なロジックなし、純粋な構築ヘルパー
- ✅ **統一的なインターフェース**: make_XXX() の命名規則統一

**教訓**:
> 小さく、責務明確な Box は成功する。巨大化させない。

---

### 成功例2: UsingResolverBox
**ファイル**: `pipeline_v2/using_resolver_box.hako` (249行)

**成功要因**:
- ✅ **状態管理**: birth() でマップ初期化、状態を保持
- ✅ **段階的構築**: load_usings_json() → load_modules_json() → upgrade_aliases()
- ✅ **パイプラインで一貫使用**: pipeline.hako で25+箇所

**教訓**:
> 状態を持つ Box は、段階的な構築とパイプライン統合で威力発揮。

---

### 失敗例: DebugBox
**ファイル**: `boxes/debug_box.hako` (39行)

**失敗要因**:
- ❌ **採用率低い**: わずか3箇所しか使用されていない
- ❌ **非効率**: ConsoleBox を毎回生成
- ❌ **一貫性なし**: 他ファイルでは直接 print() 使用

**教訓**:
> Box を作っただけでは不十分。採用促進と標準化が必要。

---

## 🔬 技術的深掘り: 最大のボトルネック

### JsonProgramBox (531行) の分析

**責務**:
1. JSON v0 正規化 (Program/Stmt/Expr)
2. JSON読み取りユーティリティ (read_XXX系)
3. 文字列操作 (index_of, trim, escape, etc.)
4. メタデータ注入 (usings)

**問題点**:
- **単一ファイルに4つの責務** → 単一責任原則違反
- **531行 (全体の9.3%)** → 最大ファイル
- **再利用性低い**: JSON操作が内部に閉じている

**リファクタ案**:
```
JsonProgramBox (531行)
  ↓
JsonProgramBox (330行) - 正規化のみ
+ JsonUtilsBox (150行) - JSON操作
+ StringUtilsBox (統合済み) - 文字列操作
= 純削減: 50-80行 (重複排除)
```

---

## 📈 定量的効果予測

### 総合削減見積もり

| Phase | 対象 | 削減行数 | 工数 |
|-------|------|---------|------|
| Phase 1-Week1 | StringUtilsBox | 220-370行 | 20-30h |
| Phase 1-Week2 | JsonUtilsBox | 50-80行 | 10-15h |
| Phase 2 | MapHelpersBox | 50-80行 | 10-15h |
| Phase 3 | DebugBox | 5-10行 | 5-10h |
| **合計** | | **325-540行** | **45-70h** |

**削減率**: 5,733行 → 5,193-5,408行 = **5.7-9.4%削減**

**品質向上**:
- Everything is Box準拠度: 65% → **85-90%** (目標)
- 重複コード: 30+箇所 → **5箇所以下**
- 保守性: 単一ソース原則確立

---

## ✅ 即座に実行可能なクイックウィン

### クイックウィン1: StringHelpers統合 (既存を活用)
**現状**: `apps/selfhost/common/string_helpers.hako` は既に存在 (86行)

**アクション**:
1. このファイルに不足機能を追加:
   - `index_of`, `last_index_of`, `starts_with`
   - `trim`, `skip_ws`
   - `is_digit`, `is_alpha`, `is_space`

2. 影響が小さい5ファイルから段階的移行:
   - ParserStringUtilsBox (83行)
   - boxes/mir_emitter_box.hako (230行)
   - pipeline_v2/regex_flow.hako (103行)

**効果**: 50-80行削減、工数5-10時間

---

### クイックウィン2: MapHelpersBox採用促進
**現状**: MapHelpersBox は既に完成 (48行)、採用率30%

**アクション**:
1. 影響が小さい5ファイルから段階的移行:
   - pipeline_v2/emit_compare_box.hako
   - pipeline_v2/stage1_args_parser_box.hako
   - pipeline_v2/call_extract_box.hako

2. 生MapBoxアクセスを型安全アクセサに置き換え

**効果**: 10-20行削減、工数3-5時間

---

## 🚨 リスクと軽減策

### リスク1: 影響範囲が広い
**影響ファイル**: StringUtilsBox統合で30ファイル

**軽減策**:
- ✅ **段階的移行**: 5ファイルずつ移行、スモークテスト
- ✅ **フォールバック**: 旧実装を残して並行稼働
- ✅ **ローカル関数残す**: 移行前は using で統一、メソッドは残す

---

### リスク2: テスト不足
**現状**: スモークテストはあるが、ユニットテストなし

**軽減策**:
- ✅ **スモークテスト拡充**: 各Phase後に実行
- ✅ **回帰テスト**: 既存 tools/smokes/v2/run.sh --profile quick
- ✅ **ダンプ比較**: MIR出力を移行前後で比較

---

### リスク3: パフォーマンス劣化
**懸念**: 関数呼び出しオーバーヘッド (MapHelpersBox)

**軽減策**:
- ✅ **ベンチマーク**: Phase 2前にベンチマーク実施
- ✅ **インライン化**: 将来的にMIR最適化で対応
- ✅ **選択的適用**: ホットパス以外に限定

---

## 📚 参考資料

### 既存実装 (良い設計例)
1. **StringHelpers**: `apps/selfhost/common/string_helpers.hako` (86行)
2. **MapHelpersBox**: `apps/selfhost-compiler/pipeline_v2/map_helpers_box.hako` (48行)
3. **UsingResolverBox**: `apps/selfhost-compiler/pipeline_v2/using_resolver_box.hako` (249行)
4. **emit系Boxes**: `apps/selfhost-compiler/common/*.hako` (4ファイル)

### 関連ドキュメント
- [00_MASTER_ROADMAP.md](../roadmap/phases/00_MASTER_ROADMAP.md)
- [Phase 15 INDEX](../roadmap/phases/phase-15/INDEX.md)
- [Box理論](../../reference/language/LANGUAGE_REFERENCE_2025.md#box-system)

---

## 🎯 次のステップ

### 即座の推奨アクション
1. **✅ このレポートをレビュー**: ユーザー確認
2. **✅ Phase 1-Week1 着手判断**: StringUtilsBox統合 (最優先)
3. **✅ クイックウィン実施**: 5ファイルの段階的移行で効果検証

### 中長期計画
- **Phase 1完了後**: Everything is Box準拠度 65% → 75%
- **Phase 2完了後**: Everything is Box準拠度 75% → 85%
- **Phase 3完了後**: Everything is Box準拠度 85% → 90%

---

**生成日時**: 2025-10-12
**分析対象**: apps/selfhost-compiler/ (71ファイル、5,733行)
**分析ツール**: Claude Code横断的分析
**信頼度**: 高 (実ファイル読み取りベース)
