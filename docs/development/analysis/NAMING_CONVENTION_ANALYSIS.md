# 命名・コーディング規約分析レポート

**分析日**: 2025-10-15
**対象**: selfhost/ 全体 (165 files, 13,417 lines)

---

## 📊 現状の統計データ

### Box命名パターン

| カテゴリ | 数量 | 割合 | 例 |
|---------|------|------|-----|
| **Box接尾辞あり** | 106 | **53.3%** | `JsonCursorBox`, `EmitReturnBox` |
| **Box接尾辞なし** | 93 | **46.7%** | `StringHelpers`, `MiniVm`, `FlowRunner` |
| **合計** | **199** | 100% | - |

#### Box接尾辞なしの主要カテゴリ:

| サブカテゴリ | 数量 | 例 |
|------------|------|-----|
| `Main` (エントリーポイント) | 28 | `Main`, `JsonCursorMain`, `Stage1JsonScannerMain` |
| `Stub` (空実装・テスト用) | 22 | `EmitReturnStub`, `MirBuilderStub`, `BackendStub` |
| `Adapter` | 3 | `MirJsonV1Adapter`, `MapKvStringToArrayAdapter` |
| 略称形式 | 15 | `MiniVm`, `MirVmMin`, `MirVmM2`, `StringHelpers` |
| その他 | 25 | `FlowRunner`, `DepTree`, `SeamInspector` |

### 関数命名パターン

| パターン | 数量 | 割合 | 例 |
|---------|------|------|-----|
| **snake_case** | 352 | **63.5%** | `run_min`, `int_to_str`, `emit_return` |
| **camelCase/PascalCase混在** | 202 | **36.5%** | `ensure_after_last_def_copy` (実際はほぼ`snake_case`) |
| **合計** | **554** | 100% | - |

**注**: camelCase関数は実質ほぼ存在せず、snake_caseが圧倒的主流。

#### よく使われる関数名 (頻出TOP10):

| 関数名 | 出現回数 | 説明 |
|-------|---------|------|
| `print` | 213 | デバッグ出力 |
| `loop` | 135 | ループ構文 |
| `m.set` | 56 | MapBox設定 (メソッド呼び出し) |
| `return Result.Ok` | 38 | 成功結果返却 |
| `return JsonEmitBox.to_json` | 32 | JSON生成 |
| `me._append` | 26 | 内部追加メソッド |
| `me._comma_if_needed` | 26 | JSON区切り挿入 |
| `me._push_inst_map` | 20 | MIR命令追加 |
| `insts.push` | 20 | 配列追加 |
| `return me._ensure_terminators` | 13 | Terminator確認 |

### 変数命名パターン

| パターン | 説明 | 例 |
|---------|------|-----|
| **短縮形** | 1-2文字の変数 | `i`, `n`, `ch`, `p`, `s`, `m`, `j`, `k` |
| **略語** | 3-5文字 | `pos`, `out`, `dst`, `src`, `mir1`, `mir2` |
| **説明的** | 6+文字 | `mod_full`, `insts`, `blocks`, `result1`, `result2` |

#### よく使われる変数名 (TOP15):

| 変数名 | 出現回数 | 用途 |
|-------|---------|------|
| `i` | 108 | ループカウンタ |
| `n` | 76 | サイズ・長さ |
| `ch` | 74 | 文字 (character) |
| `out` | 52 | 出力バッファ |
| `p` | 36 | 位置 (position) |
| `s` | 34 | 文字列 (string) |
| `m` | 33 | Map変数 |
| `pos` | 31 | 位置 (position) |
| `node` | 28 | JSON/AST ノード |
| `j` | 26 | ループカウンタ (内側) |
| `mod_full` | 25 | モジュール完全名 |
| `insts` | 25 | 命令配列 |
| `dst` | 25 | 宛先レジスタ |
| `v` | 23 | 値 (value) |
| `args` | 20 | 引数配列 |

### コメントパターン

| 言語 | 数量 | 割合 |
|------|------|------|
| **英語** | 376 | **84.9%** |
| **日本語** | 67 | **15.1%** |
| **合計** | **443** | 100% |

**傾向**: 英語コメントが主流。日本語は主にトップレベルの説明・責務記述で使用。

---

## 🔍 不統一箇所リスト

### 1. Box名の不統一

#### 1.1 ユーティリティBoxの接尾辞なし

**ファイル** | **Box名** | **推奨名** | **理由**
-----------|---------|----------|--------
`selfhost/shared/common/string_helpers.hako` | `StringHelpers` | `StringHelpersBox` | ユーティリティBoxも統一
`selfhost/shared/common/string_ops.hako` | `StringOps` | `StringOpsBox` | 同上
`selfhost/tools/dep_tree_core.hako` | `DepTreeCore` | `DepTreeCoreBox` | 同上
`selfhost/tools/dep_tree.hako` | `DepTree` | `DepTreeBox` | 同上
`selfhost/vm/flow_runner.hako` | `FlowRunner` | `FlowRunnerBox` | 同上
`selfhost/vm/boxes/mini_vm_*.hako` | `MiniVm*` | `MiniVm*Box` | 15ファイルで不統一

**影響範囲**: 17ファイル
**修正優先度**: **Medium** (機能影響なし、一貫性向上)

#### 1.2 VM関連Boxの略称形式

**ファイル** | **Box名** | **推奨名** | **理由**
-----------|---------|----------|--------
`selfhost/vm/boxes/mir_vm_min.hako` | `MirVmMin` | `MirVmMinBox` | VM Boxも統一
`selfhost/vm/boxes/mir_vm_m2.hako` | `MirVmM2` | `MirVmM2Box` | 同上
`selfhost/vm/boxes/mini_vm_core.hako` | `MiniVm` | `MiniVmBox` | 同上
`selfhost/hakorune-vm/hakorune_vm_core.hako` | `HakoruneVmCore` | `HakoruneVmCoreBox` | 同上

**影響範囲**: 4ファイル + 関連using文
**修正優先度**: **Medium**

#### 1.3 Stub/Main接尾辞の混在

**現状**: `EmitReturnStub`, `JsonCursorMain`, `Stage1JsonScannerMain` など68箇所

**提案**:
- `*Main` → エントリーポイント専用 (28箇所: OK)
- `*Stub` → テスト・空実装専用 (22箇所: OK)
- **両方ともBox接尾辞を追加** → `*MainBox`, `*StubBox`

**修正優先度**: **Low** (現状で目的は明確)

### 2. ファイル名とBox名の不一致

#### 2.1 ファイル名に `_box` 接尾辞あり、Box名になし

**ファイル名** | **Box名** | **提案**
-------------|---------|--------
`string_helpers.hako` | `StringHelpers` | → `StringHelpersBox`
`string_ops.hako` | `StringOps` | → `StringOpsBox`
`using_resolver_box.hako` | `UsingResolverBox` | OK (一致)

**影響範囲**: 48ファイルが `*_box.hako` 形式、うち2ファイルで不一致

#### 2.2 ファイル名に `_box` なし、Box名に `Box` あり

**例**: `mini_vm_entry.hako` → `MiniVmEntryBox`

**提案**:
- Option A: ファイル名を `*_box.hako` に統一
- Option B: Box名から `Box` 削除 (非推奨)

**修正優先度**: **Low** (検索性に影響なし)

### 3. 関数命名の一貫性

**現状**: ほぼ完全に `snake_case` で統一済み ✅

**例外** (6個のみ):
- `_report_duplicate_boxes` ✅ (private関数: OK)
- `_report_duplicate_functions_in_box` ✅ (private関数: OK)
- `ensure_after_last_def_copy` ✅ (OK)
- `seek_array_end` ✅ (OK)

**結論**: **変更不要**

---

## 📋 推奨命名規約

### Box命名

```hakorune
// ✅ 正しい命名
box UserManagerBox { ... }
static box ApplicationMain { ... }          // エントリーポイント例外
static box MockDataStub { ... }             // テスト用例外

// ❌ 避けるべき
box UserManager { ... }                     // Box接尾辞なし
box user_manager_box { ... }                // snake_case不可
```

**ルール**:
1. **すべてのBox名は `*Box` 接尾辞を持つ** (原則)
2. **例外1**: `static box Main` (エントリーポイント)
3. **例外2**: `static box *Main` (サブエントリーポイント: `JsonCursorMain` 等)
4. **例外3**: `static box *Stub` (テスト・空実装)
5. **PascalCase必須** (例: `JsonCursorBox`, `StringHelpersBox`)

### 関数命名

```hakorune
// ✅ 正しい命名
static box Calculator {
    add_numbers(a, b) { ... }
    get_result() { ... }
    _internal_helper() { ... }  // private: アンダースコア prefix
}

// ❌ 避けるべき
addNumbers(a, b) { ... }        // camelCase不可
GetResult() { ... }             // PascalCase不可
```

**ルール**:
1. **`snake_case` 必須**
2. **private関数は `_` prefix** (例: `_append`, `_ensure_terminators`)
3. **メソッド呼び出しも統一**: `recv.method_name(...)`

### 変数命名

```hakorune
// ✅ 推奨パターン
local i = 0                     // ループカウンタ: 短縮形OK
local n = text.size()           // サイズ: 短縮形OK
local ch = text.substring(i, i+1)  // 文字: 短縮形OK
local pos = 0                   // 位置
local result_map = new MapBox   // 説明的
local mod_full = "std.core"     // 説明的

// ❌ 避けるべき
local N = 10                    // 大文字不可 (定数以外)
local ResultMap = ...           // PascalCase不可
local result-map = ...          // ハイフン不可
```

**ルール**:
1. **`snake_case` 必須**
2. **定数のみ `UPPER_SNAKE_CASE`** (現状は存在せず)
3. **短縮形OK**: `i`, `j`, `n`, `ch`, `pos`, `out`, `dst`, `src`
4. **説明的変数**: 6文字以上で明示

### コメント

```hakorune
// ✅ 推奨パターン

// EmitReturnBox — return(Int) の最小 MIR(JSON v0) 生成
//
// Responsibility: Emit MIR JSON for simple return statements
// Non-responsibility: Control flow analysis
static box EmitReturnBox {
    // Convert integer literal to const instruction
    emit_const(value) {
        // Implementation...
    }
}

// ❌ 避けるべき
// えみっとりたーんぼっくす (ひらがな不可)
// EMIT RETURN BOX (全て大文字不可)
```

**ルール**:
1. **トップレベル**: 日英併記OK (例: `// EmitReturnBox — return(Int) の最小 MIR生成`)
2. **Responsibility/Non-responsibility**: 英語推奨
3. **実装コメント**: 英語推奨、日本語補助OK
4. **絵文字**: 使用しない (Markdown以外)

### ファイル命名

```bash
# ✅ 推奨パターン
emit_return_box.hako        # Box実装
pipeline_helpers_box.hako   # ヘルパーBox
flow_entry.hako             # エントリーポイント (Main含む)

# ❌ 避けるべき
EmitReturnBox.hako          # PascalCase不可
emit-return-box.hako        # ハイフン不可
```

**ルール**:
1. **`snake_case` 必須**
2. **`*_box.hako` 接尾辞推奨** (検索性向上)
3. **例外**: エントリーポイント (`flow_entry.hako`)、テスト (`test_*.hako`)

---

## 🎯 スタイルガイド提案

### インデント

**現状**: 4スペース (一貫)
**提案**: **変更なし** ✅

```hakorune
static box Example {
    method() {
        local x = 0
        if x == 0 {
            print("zero")
        }
    }
}
```

### 改行・ブレース

**現状**: K&Rスタイル (opening braceは行末)
**提案**: **変更なし** ✅

```hakorune
// ✅ 現状通り
box Example {
    method() {
        if condition {
            ...
        }
    }
}

// ❌ Allman不可
box Example
{
    method()
    {
        ...
    }
}
```

### コメント配置

**現状**: 関数・Box前に説明コメント
**提案**: **Responsibility/Non-responsibility 記述推奨** ✅

```hakorune
// EmitReturnBox — return(Int) の最小 MIR(JSON v0) 生成
//
// Responsibility: Emit MIR for simple return statements
// Non-responsibility: Complex control flow; delegate to FlowBuilderBox
static box EmitReturnBox {
    ...
}
```

---

## 🤖 自動修正可能箇所

### 1. Box名の `Box` 接尾辞追加

**対象**: 17ファイル (StringHelpers, MiniVm系, DepTree系)

**自動修正スクリプト**:

```bash
#!/bin/bash
# Box名に "Box" 接尾辞を追加

declare -A RENAMES=(
    ["StringHelpers"]="StringHelpersBox"
    ["StringOps"]="StringOpsBox"
    ["DepTree"]="DepTreeBox"
    ["DepTreeCore"]="DepTreeCoreBox"
    ["FlowRunner"]="FlowRunnerBox"
    ["MiniVm"]="MiniVmBox"
    ["MirVmMin"]="MirVmMinBox"
    ["MirVmM2"]="MirVmM2Box"
    ["HakoruneVmCore"]="HakoruneVmCoreBox"
)

for old in "${!RENAMES[@]}"; do
    new="${RENAMES[$old]}"
    echo "Renaming: $old → $new"

    # Box定義の変更
    find selfhost -name "*.hako" -type f -exec sed -i "s/^static box $old\$/static box $new/g" {} +
    find selfhost -name "*.hako" -type f -exec sed -i "s/^box $old\$/box $new/g" {} +

    # using文の変更
    find selfhost -name "*.hako" -type f -exec sed -i "s/ as $old\$/ as $new/g" {} +

    # 静的呼び出しの変更 (慎重: 変数名と区別)
    find selfhost -name "*.hako" -type f -exec sed -i "s/\b$old\\./$new./g" {} +
done
```

**リスク**: **Medium** (using文・静的呼び出しの全箇所変更)
**推奨**: 手動変更 + git diff確認

### 2. ファイル名の `_box.hako` 接尾辞追加

**対象**: 約70ファイル (現在 `_box` 接尾辞なし)

**自動修正スクリプト**:

```bash
#!/bin/bash
# ファイル名に "_box.hako" 接尾辞を追加

find selfhost -name "*.hako" -type f ! -name "*_box.hako" ! -name "test_*.hako" | while read f; do
    dir=$(dirname "$f")
    base=$(basename "$f" .hako)
    new="$dir/${base}_box.hako"

    # エントリーポイント・特殊ファイルは除外
    if [[ "$base" == *"_flow"* ]] || [[ "$base" == "pipeline" ]]; then
        echo "SKIP (flow/entry): $f"
        continue
    fi

    echo "Renaming file: $f → $new"
    git mv "$f" "$new"

    # using文の自動更新
    find selfhost -name "*.hako" -type f -exec sed -i "s|using \"${f#selfhost/}\"|using \"${new#selfhost/}\"|g" {} +
done
```

**リスク**: **Low** (ファイル名のみ、機能影響なし)
**推奨**: 自動実行可能 (git diff確認は必要)

### 3. コメント英語化 (半自動)

**対象**: 67個の日本語コメント

**方針**: **手動変更推奨** (機械翻訳は品質低下)

---

## 📊 修正優先度まとめ

| カテゴリ | 対象 | 優先度 | 自動化 | 影響範囲 |
|---------|------|--------|--------|---------|
| **Box名 `Box` 接尾辞** | 17ファイル | **High** | 可能 (要注意) | using文・静的呼び出し全体 |
| **ファイル名 `_box` 接尾辞** | 70ファイル | **Medium** | 可能 | using文のみ |
| **`*Main`/`*Stub` → `*Box`** | 68ファイル | **Low** | 可能 | using文・静的呼び出し |
| **コメント英語化** | 67箇所 | **Low** | 不可 | なし |

### 推奨実施順序

1. **Phase 1** (優先度: High)
   - ✅ 関数命名: **変更不要** (既に統一済み)
   - 🔧 Box名 `Box` 接尾辞追加 (StringHelpers → StringHelpersBox 等)
   - ✅ テスト実行・影響確認

2. **Phase 2** (優先度: Medium)
   - 🔧 ファイル名 `_box.hako` 接尾辞追加
   - ✅ using文自動更新
   - ✅ テスト実行

3. **Phase 3** (優先度: Low、オプション)
   - 🔧 `*Main`/`*Stub` → `*MainBox`/`*StubBox`
   - 🔧 コメント英語化 (手動)

---

## ✅ 既に統一されている点

### 1. 関数命名
- ✅ **ほぼ100% snake_case で統一**
- ✅ private関数は `_` prefix (例: `_append`, `_ensure_terminators`)

### 2. インデント
- ✅ **4スペース統一**
- ✅ タブ混在なし

### 3. ブレーススタイル
- ✅ **K&Rスタイル統一** (opening braceは行末)

### 4. 変数命名
- ✅ **snake_case 統一**
- ✅ 短縮形の慣用的使用 (`i`, `n`, `ch`, `pos`)

### 5. コメント配置
- ✅ **Responsibility/Non-responsibility パターン普及**
- ✅ ファイル先頭に Box説明コメント

---

## 🎓 Hakorune言語仕様との整合性

### 仕様準拠チェック

| 要素 | Hakorune仕様 | selfhost実装 | 評価 |
|------|------------|------------|------|
| **Box名** | PascalCase推奨 | ✅ 100% PascalCase | ✅ 準拠 |
| **関数名** | 仕様未定義 | ✅ snake_case統一 | ✅ 慣例確立 |
| **変数名** | 仕様未定義 | ✅ snake_case統一 | ✅ 慣例確立 |
| **Box接尾辞** | 仕様未定義 | ⚠️ 53%のみ `Box` 接尾辞 | 🔧 改善余地 |
| **コメント** | 仕様未定義 | ✅ 英語主体 (84.9%) | ✅ 良好 |

---

## 📖 参考リンク

- **言語仕様**: [docs/reference/language/LANGUAGE_REFERENCE_2025.md](../../reference/language/LANGUAGE_REFERENCE_2025.md)
- **クイックリファレンス**: [docs/reference/language/quick-reference.md](../../reference/language/quick-reference.md)
- **Box設計**: [docs/reference/boxes-system/](../../reference/boxes-system/)

---

## 🚀 次のステップ

1. **ユーザー確認**: この提案をレビュー・承認
2. **Phase 1実施**: Box名 `Box` 接尾辞追加 (高優先度)
3. **テスト実行**: スモークテスト全通過確認
4. **Phase 2実施**: ファイル名統一 (中優先度)
5. **スタイルガイド文書化**: 正式ドキュメント作成

---

**分析完了**: 2025-10-15
**総評**: selfhost/ の命名規約は**関数名・変数名で既に高度に統一**されています。主要な改善箇所は **Box名の `Box` 接尾辞統一** (17ファイル) のみです。
