# Phase 23: Type System & Rune (型システムとルーン)

**ステータス**: 提案段階（Phase 15完了後に検討）
**優先度**: Phase 20-25期間中に実装
**前提条件**: Phase 15セルフホスティング完了

---

## 🎯 概要

Hakoruneの型システムを拡張し、**Rune-First設計**を導入する。

**Rune（ルーン）**: 箱に刻まれる能力の印。不変の契約として複数組み合わせ可能。

### 目的

1. **共通API保証**: すべてのBoxで共通の操作を提供
2. **安全ダウンキャスト**: 型判別と安全な型変換
3. **Capability表現**: 決定性・シリアライズ可能性の明示
4. **型パスへの橋渡し**: Phase 20-25の型推論・型検査との連携

---

## 📋 主要ドキュメント

### [rune-design.md](./rune-design.md)
Rune-First設計の完全仕様。他言語の事例研究、実装案、Phase 15との関係を詳述。

---

## 🚀 実装計画（3段階）

### Phase A: プレリュード関数（Phase 15完了直後）
- **実装コスト**: 50-100行、1日
- **内容**: `box_type_id()`, `try_as_string()`, `is_deterministic()`等の関数
- **ファイル**: `core/prelude/box_protocol.hako`

### Phase B: Rune構文（Phase 20-25）
- **実装コスト**: 200-300行、1週間
- **内容**: `@rune ValueLike { ... }`, `implements ValueLike for StringBox`
- **脱糖**: プレリュード関数を呼ぶ
- **カプセル化制御**: Visibility/Ownership runes で公開制御（詳細は後述）

### Phase C: @deriveマクロ（Phase 25+）
- **実装コスト**: 500-700行、2週間
- **内容**: `@derive(ValueLike, Debug, Hash)`
- **前提**: マクロシステム完成

---

## 📊 他言語の参考事例

| 言語 | 機能 | Hakoruneへの応用 |
|------|------|----------------|
| Rust | `Any` trait | 型判別＋ダウンキャスト |
| Swift | Protocol | 複数Rune組み合わせ |
| Elixir | Protocol | 型と実装の分離 |
| Haskell | Type Class | Runeを型制約として表現 |

---

## 🔒 Encapsulation Control with Rune

**重要**: Runeによるカプセル化制御は**2層構造**で実現します。

### 🎯 役割分担: hako_module.toml vs Rune

#### **Layer 1: File-Level Boundaries（hako_module.toml）**
- **粒度**: ファイル単位の粗い境界
- **用途**: モジュール全体の公開/非公開制御
- **検査**: コンパイル時の using 文チェック

```toml
# hako_module.toml
[module]
name = "selfhost.compiler"

[exports]
main = "compiler.hako"              # ✅ Public API
pipeline = "pipeline_v2/pipeline.hako"

[private]
patterns = ["pipeline_v2/internal/**", "pipeline_v2/*_scan*.hako"]  # ❌ Path-based using forbidden
```

#### **Layer 2: Box-Level Control（Rune）**
- **粒度**: Box/メソッド単位の細かい制御
- **用途**: 同じファイル内での公開範囲制御
- **検査**: コンパイル時の call/boxcall チェック

```hako
// pipeline_v2/pipeline.hako
@implements Public
static box PipelineV2 {
  main(args) { ... }  // ✅ 外部から呼び出し可能
}

@implements Internal
static box Stage1Scanner {
  find_body(json) { ... }  // ❌ 同じモジュール内のみ呼び出し可能
}

@implements Experimental
static box BetaFeature {
  process() { ... }  // ⚠️ 外部から呼び出し可能だが警告付き
}
```

### 📋 Visibility Runes（公開制御）

#### Phase B 実装候補（基本3種類）

```hako
// 1. Public — 公開API（外部モジュールから自由に使える）
@rune Public { }
@implements Public
static box CompilerAPI { ... }

// 2. Internal — 内部実装（同じモジュール内のみ）
@rune Internal { }
@implements Internal
static box PipelineInternal { ... }

// 3. Experimental — 実験的機能（外部から使えるが警告付き）
@rune Experimental { }
@implements Experimental
static box BetaOptimizer { ... }
```

#### Phase C 拡張候補（Rust風細粒度制御）

```hako
// 4. ModuleOnly — 同じモジュール内のみ
@ownership(module_only)
static box ModuleHelper { ... }

// 5. CrateOnly — 同じクレート（プロジェクト）内のみ
@ownership(crate_only)
static box ProjectInternal { ... }
```

### 🔐 Ownership Runes（所有権制御）

**Phase C以降で検討**: Rust風の所有権概念をRuneで表現

```hako
// 1. Owned — 所有権あり（Public API）
@ownership(owned)
static box PublicAPI {
  process(input) { ... }  // inputをfull controlできる
}

// 2. Borrowed — 借用（Internal実装）
@ownership(borrowed)
static box InternalHelper {
  assist(data) { ... }  // dataは読み取り専用
}

// 3. Shared — 共有（Read-Only）
@ownership(shared)
static box SharedConfig {
  get_value(key) { ... }  // 状態変更不可
}
```

### 🌟 2層構造の具体例

```hako
// File: apps/selfhost-compiler/pipeline_v2/pipeline.hako
// hako_module.toml: [exports] pipeline = "pipeline_v2/pipeline.hako"
// → ファイル全体が公開対象（Layer 1: Pass）

using selfhost.compiler.pipeline_v2.emit_compare as EmitCompare  // ❌ Layer 1: Fail ([private]に含まれる)
using selfhost.compiler.pipeline as Pipeline                     // ✅ Layer 1: Pass

// Layer 2: Box-Level Control
@implements Public
static box PipelineV2 {
  lower_stage1(ast_json) {
    local scanner = Stage1Scanner.new()  // ✅ Layer 2: Pass（同じファイル内）
    return scanner.find_body(ast_json)
  }
}

@implements Internal
static box Stage1Scanner {
  find_body(json) { ... }
}

// 外部ファイルから:
using selfhost.compiler.pipeline as Pipeline
Pipeline.PipelineV2.lower_stage1(ast)  // ✅ Layer 1 & 2: Pass
Pipeline.Stage1Scanner.find_body(ast)  // ❌ Layer 2: Fail (@implements Internal)
```

### 🔍 エラーメッセージ例

```bash
# Layer 1 violation (hako_module.toml [private])
Error: Cannot use private module path
  --> apps/selfhost/test.hako:3:1
  |
3 | using "apps/selfhost-compiler/pipeline_v2/emit_compare_box.hako" as EmitCompare
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  | This file is marked [private] in hako_module.toml
  | Use module-based using instead: using selfhost.compiler.pipeline

# Layer 2 violation (@implements Internal)
Error: Cannot call Internal box from external module
  --> apps/selfhost/test.hako:10:5
  |
10 |     Stage1Scanner.find_body(json)
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   | Stage1Scanner is marked @implements Internal
   | Can only be called within the same module (selfhost.compiler)
```

### ⚙️ 実装ロードマップ

| Phase | 機能 | Layer 1 | Layer 2 | コスト |
|-------|------|---------|---------|--------|
| **15.7** | [private] パターン検査 | ✅ | - | 1日 |
| **23-B** | @implements Public/Internal/Experimental | ✅ | ✅ | 1週間 |
| **23-C** | @ownership(owned/borrowed/shared) | ✅ | ✅ | 2週間 |

### 💡 設計原則

1. **2層で相補**: hako_module.toml は粗い境界、Rune は細かい制御
2. **両方必要**: どちらか一方では不十分
3. **Fail-Fast**: 違反は即座にコンパイルエラー
4. **段階実装**: Phase 15.7 → 23-B → 23-C と段階的に拡充

---

## ⚠️ Phase 15との関係

**重要**: Phase 15中はRust VMを複雑にしない！

- ❌ Rust VMにRune実装
- ❌ Rust VMにtrait追加
- ✅ .hakoコードで実装
- ✅ Phase 15完了後に開始

---

## 🎊 Hakorune哲学との整合性

- ✅ **Everything is Box**: Boxの能力（Rune）として表現
- ✅ **Plugin-First**: プラグインもRune実装可能
- ✅ **Deterministic**: `is_deterministic()`で明示
- ✅ **De-sugaring**: MIR14に脱糖
- ✅ **Fail-Fast**: Option/Resultで安全失敗

---

## 🔗 関連Phase

- **Phase 15**: セルフホスティング（前提条件）
- **Phase 20**: Python統合（Capability連携）
- **Phase 21**: 最適化（型情報活用）
- **Phase 24-25**: 型推論・型検査（Rune制約）

---

**作成日**: 2025-10-03
**レビュー**: ChatGPT o1 + Claude Sonnet 4.5
