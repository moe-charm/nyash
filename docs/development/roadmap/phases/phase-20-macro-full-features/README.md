# Phase 20: マクロシステムフル機能実装（セルフホスティング後）

Date: 2025-10-06
Status: **PLANNED** - セルフホスティング完了後に開始
Prerequisite: Phase 15.7完了（セルフホスティング実現）

## 🎯 **目的**

Phase 16でRust実装したマクロシステムのフル機能を、Hakoruneでセルフホスト実装に書き直す。

## 📋 **背景**

### **現状（2025-10-06）**
- ✅ Phase 16でRust実装完了（2025-09-19）
- ❌ **バグ発見**: フル機能にバグあり
- ✅ **暫定対応**: ChatGPTに`.hako`で単純置き換えマクロ実装中
  - 場所: `apps/macros/selfhost_min/`, `apps/macros/examples/`
  - 方式: JSON文字列操作のみ（`MacroBoxSpec.expand(json) -> json`）
- 🔜 **本格対応**: セルフホスティング後にフル機能を.hakoで再実装

### **Phase 16実装済み機能（Rust版、バグあり）**
場所: `src/macro/`

1. **AST Pattern Matching** (`pattern.rs`, 253行)
   - TemplatePattern（$placeholder変数）
   - OrPattern（複数テンプレート選択）
   - Variadic pattern（$...args）

2. **Quote/Unquote システム** (`pattern.rs:9-114`)
   - AstBuilder::quote()（文字列→AST）
   - AstBuilder::unquote()（テンプレート展開）
   - Variadic splicing

3. **Match式構文** (`src/parser/expr/match_expr.rs`, 392行)
   - リテラルパターン + OR
   - 型パターン（`TypeName(bind) => ...`）
   - ガード式（`pattern if condition => ...`）

4. **@derive マクロ** (`engine.rs:68-96`)
   - @derive(Equals)（自動equals()生成）
   - @derive(ToString)（自動toString()生成）
   - Public fieldsのみ使用

5. **@test ランナー** (`mod.rs:61-401`)
   - test_* 関数自動検出
   - JSON引数注入
   - 自動ハーネス生成

6. **マクロエンジン** (`engine.rs`, 195行)
   - 多段展開（最大32パス）
   - 循環検出
   - パフォーマンストレース

## 🚀 **Phase 20 実装計画**

### **優先度1: @derive マクロ（最も便利）**

#### **実装目標**
```hakorune
// ユーザーコード
@derive(Equals, ToString)
box Person {
    name: StringBox
    age: IntegerBox
}

// 自動生成される:
// equals(other) { return me.name == other.name && me.age == other.age }
// toString() { return "Person(" + me.name + "," + me.age + ")" }
```

#### **実装場所案**
```
apps/macros/derive/
├── derive_equals.hako       # Equals実装
├── derive_tostring.hako     # ToString実装
├── derive_clone.hako        # Clone実装（将来）
├── derive_debug.hako        # Debug実装（将来）
└── README.md                # 使い方・設計書
```

#### **実装契約**
```hakorune
// apps/macros/derive/derive_equals.hako
static box DeriveEqualsMacro {
    expand(json, ctx) {
        // 1. JSON AST解析
        // 2. BoxDeclaration検出
        // 3. public_fieldsを取得
        // 4. equals()メソッドをJSON ASTで生成
        // 5. methodsに追加
        return modified_json
    }
}
```

### **優先度2: @test ランナー**

セルフホスティングコードのテスト自動化。

#### **実装場所案**
```
apps/macros/test/
├── test_runner.hako         # テストランナー本体
├── test_collector.hako      # test_*関数収集
└── README.md
```

### **優先度3: AST Pattern Matching**

複雑なマクロのための基盤機能。

#### **実装場所案**
```
apps/macros/pattern/
├── pattern_matcher.hako     # パターンマッチエンジン
├── ast_walker.hako          # AST走査ヘルパー
└── README.md
```

### **優先度4: Quote/Unquote**

テンプレートベースのコード生成。

### **優先度5: Match式型パターン**

型安全なパターンマッチング。

## 📊 **実装戦略**

### **Step 1: 最小限のderive実装（Phase 20.1）**
**期間**: 1-2週間
**成果物**: @derive(Equals) のみ動作

1. Rust版 `src/macro/engine.rs:build_equals_method()` を参考に.hako実装
2. JSON AST操作のみで実装（AstBuilderBox等を作成）
3. スモークテスト作成（`tools/smokes/v2/profiles/quick/macro/derive_equals_vm.sh`）

### **Step 2: derive拡張（Phase 20.2）**
**期間**: 1週間
**成果物**: @derive(ToString, Clone, Debug)

### **Step 3: @test ランナー（Phase 20.3）**
**期間**: 2-3週間
**成果物**: セルフホスティングコード全体のテスト自動化

### **Step 4: Pattern/Quote基盤（Phase 20.4-20.5）**
**期間**: 3-4週間
**成果物**: 複雑なマクロ実装が可能に

## 🎯 **成功条件**

### **Phase 20.1完了条件**
- [ ] @derive(Equals)が動作（最低3つのBoxでテスト）
- [ ] 既存Rust版deriveと同じ出力
- [ ] スモークテスト通過
- [ ] パフォーマンス: 展開時間 < 200ms

### **Phase 20完了条件**
- [ ] @derive(Equals, ToString, Clone, Debug)すべて動作
- [ ] @test ランナー動作（セルフホスティングコード100%カバー）
- [ ] Pattern Matching基盤動作
- [ ] Quote/Unquote動作
- [ ] Rust版マクロシステム削除可能（移行完了）

## 📝 **設計メモ**

### **JSON AST操作の課題**
Phase 16のRust実装は`nyash_rust::ASTNode`構造体を直接操作したが、
Phase 20では**JSON文字列としてのAST**を操作する必要がある。

**解決策候補**:
1. **JsonAstBox**: JSON AST操作ヘルパーBox
   - `parse(json) -> AstNode`（簡易AST表現）
   - `generate(ast) -> json`
   - `walk(ast, visitor)`（AST走査）

2. **文字列操作のみ**: 現在の暫定実装を拡張
   - 利点: シンプル
   - 欠点: 複雑なマクロは困難

→ **推奨**: JsonAstBoxを先に実装（Phase 20.0）

### **Rust版との互換性**
環境変数で切り替え可能にする:
```bash
# Rust版（バグあり、非推奨）
NYASH_MACRO_DERIVE_BACKEND=rust ./hako program.hako

# Hakorune版（デフォルト、Phase 20実装）
NYASH_MACRO_DERIVE_BACKEND=hako ./hako program.hako
```

## 🔗 **関連リソース**

### **Phase 16（Rust実装版）**
- [Phase 16 README](../phase-16-macro-revolution/README.md)
- [Phase 16 IMPLEMENTATION](../phase-16-macro-revolution/IMPLEMENTATION.md)

### **現在の暫定実装**
- `apps/macros/selfhost_min/README.md` - 単純置き換えマクロ
- `apps/macros/examples/upper_string_macro.nyash` - 実装例
- `src/macro/macro_box_ny.rs` - ユーザーマクロローダー

### **環境変数ドキュメント**
- `docs/guides/macro-system.md` - マクロシステム全体ガイド
- `docs/reference/environment-variables.md` - 環境変数リファレンス

## 📅 **スケジュール（暫定）**

- **Phase 20.0**: JsonAstBox基盤実装（1週間）
- **Phase 20.1**: @derive(Equals)実装（2週間）
- **Phase 20.2**: @derive拡張（1週間）
- **Phase 20.3**: @test ランナー（3週間）
- **Phase 20.4-20.5**: Pattern/Quote（4週間）

**合計**: 約11週間（2.5ヶ月）

**開始条件**: Phase 15.7完了（セルフホスティング実現）

---

**Next Steps**:
1. Phase 15.7完了を待つ
2. Phase 20.0開始: JsonAstBox設計・実装
3. Phase 20.1開始: @derive(Equals)最小実装
