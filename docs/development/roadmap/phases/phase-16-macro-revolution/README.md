# Phase 16: Macro Revolution - 世界最強マクロシステムの構築

Date: 2025-09-19  
Status: **ACTIVE** - AST Pattern Matching実装中  
Target: 2025年12月完了  

## 🎯 **革命の概要**

**Everything is Box** 設計の究極進化形として、世界初の **Box-Based Macro System** を実装。  
Lisp、Rust、C++、Nim、Juliaを超越する次世代マクロ言語を目指す。

## 🔥 **なぜ革命的か**

### **従来のマクロシステム**
```rust
// Rust: 型ごとに別実装が必要
#[derive(Debug)] struct A {}     // struct用実装
#[derive(Debug)] enum B {}       // enum用実装  
#[derive(Debug)] union C {}      // union用実装
```

### **Nyash: Box-Based Macro**
```nyash
// すべてがBoxなので、1つの実装で全対応！
@derive(Debug) box A {}          // 同じ実装
@derive(Debug) box B {}          // 同じ実装
@derive(Debug) box C {}          // 同じ実装
```

**複雑度**: `O(型の種類 × マクロの種類)` → `O(マクロの種類)`

## 🏗️ **アーキテクチャ設計**

### **Phase 1: AST Pattern Matching基盤**
```nyash
// 安全なASTパターンマッチング
match ast_node {
    BoxDeclaration { name, fields, methods, .. } => {
        // 型安全な変換処理
    }
    FunctionDeclaration { name: @fname, params: [first, ...rest] } => {
        // 束縛とワイルドカード対応
    }
}
```

### **Phase 2: Quote/Unquote システム**
```nyash
// 安全なコード生成
let template = quote! {
    $(method_name)(other) {
        return $(field_comparison_logic)
    }
}

// 型安全な展開
let generated = unquote! {
    template with {
        method_name: "equals",
        field_comparison_logic: generate_field_comparisons(box_fields)
    }
}
```

### **Phase 3: HIRパッチ式マクロエンジン**
```nyash
// MIR命令は増やさない！HIRレベルで変換
box MacroEngineBox {
    expand_derive(input_box: BoxAst) -> Vec<MethodAst> {
        // HIRレベルでパッチ適用
        // 既存MIR14命令セットで実行
    }
}
```

## 🎯 **実装する実マクロ**

### **@derive マクロファミリー**
```nyash
@derive(Equals, ToString, Clone, Debug)
box Person {
    name: StringBox
    age: IntegerBox
    address: AddressBox  // ネストしたBoxも自動対応
}

// 自動生成される:
// - equals(other) メソッド
// - toString() メソッド  
// - clone() メソッド
// - debug() メソッド
```

### **@test マクロ + ランナー**
```nyash
@test
test_person_creation() {
    local person = new Person("Alice", 25, new AddressBox("Tokyo"))
    assert_equals(person.name, "Alice")
    assert_equals(person.age, 25)
}

@test  
test_person_equals() {
    local p1 = new Person("Bob", 30, new AddressBox("Osaka"))
    local p2 = new Person("Bob", 30, new AddressBox("Osaka"))
    assert_equals(p1.equals(p2), true)
}
```

```bash
# テスト実行
nyash --run-tests my_program.nyash
# [TEST] test_person_creation ... OK
# [TEST] test_person_equals ... OK
# Tests: 2 passed, 0 failed
```

## 🛡️ **ガードレール（安全性保証）**

### **1. Hygiene（名前衝突回避）**
```nyash
macro generate_counter() {
    local temp = gensym("counter_temp")  // 自動でユニーク名生成
    quote! {
        local $(temp) = 0
        $(temp).increment()
    }
}
```

### **2. 循環検出・再帰制限**
```nyash
// マクロ展開時に自動チェック
macro recursive_macro(depth) {
    if macro_depth() > 100 {
        compile_error!("Macro recursion limit exceeded")
    }
}
```

### **3. 決定性・副作用なし**
```nyash
// ✅ 決定的なマクロ（推奨）
macro pure_derive(box_name, trait_name) {
    // 同じ入力なら常に同じ出力
    return generate_method(box_name, trait_name)
}

// ❌ 副作用のあるマクロ（禁止）
macro bad_macro() {
    println!("This is forbidden!")  // コンパイル時IO禁止
}
```

## 🎨 **開発者体験**

### **マクロ展開の可視化**
```bash
# マクロ展開結果を表示
nyash --expand my_program.nyash

# 詳細トレース
NYASH_MACRO_TRACE=1 nyash my_program.nyash
# [MACRO] @derive(Equals) -> generating equals() method for Person
# [MACRO] @test -> collecting test_person_creation()  
# [MACRO] Expansion complete: 2 macros processed, 0 errors
```

### **エラーメッセージの親切さ**
```nyash
@derive(UnknownTrait)
box Person {}

// エラー例:
// error: Unknown derive trait 'UnknownTrait'
//   --> person.nyash:1:9
//    |
//  1 | @derive(UnknownTrait)
//    |         ^^^^^^^^^^^^
//    |
//    = help: Available traits: Equals, ToString, Clone, Debug
//    = note: Did you mean 'ToString'?
```

## 📊 **他言語との比較優位性**

| 言語 | 型安全性 | 学習コスト | デバッグ性 | 実行性能 | 表現力 |
|------|----------|------------|------------|----------|--------|  
| **Nyash** | ✅ | 🟢 Low | 🟢 High | 🟢 High | 🟢 High |
| Rust | ✅ | 🔴 High | 🔴 Low | 🟢 High | 🟡 Medium |
| Lisp | ❌ | 🔴 High | 🔴 Low | 🟡 Medium | 🟢 High |
| C++ | ❌ | 🔴 Very High | 🔴 Very Low | 🟢 High | 🟢 High |
| Nim | 🟡 | 🟡 Medium | 🟡 Medium | 🟢 High | 🟢 High |

## 🚀 **実装スケジュール**

### **Week 1-2: AST Pattern Matching基盤**
- [x] AST定義の拡張（パターン用）
- [ ] パターンマッチング構文の実装
- [ ] 束縛・ワイルドカードサポート
- [ ] 基本テストケース

### **Week 3-4: Quote/Unquote システム**  
- [ ] Quote構文の実装
- [ ] Unquote展開エンジン
- [ ] スパン情報伝播
- [ ] エラー処理強化

### **Week 5-6: HIRパッチ式エンジン**
- [ ] マクロエンジンコア実装
- [ ] HIRレベル変換処理
- [ ] MIR14命令互換性確保
- [ ] パフォーマンス最適化

### **Week 7-8: 実マクロ実装**
- [ ] @derive(Equals, ToString)
- [ ] @test マクロ + ランナー
- [ ] CLI統合（--expand, --run-tests）
- [ ] ドキュメント・例示

### **Week 9-12: 安定化・拡張**
- [ ] ガードレール強化
- [ ] エラーメッセージ改善  
- [ ] 性能最適化・メモリ効率
- [ ] 実用アプリでの検証

## 🎯 **成功指標**

### **技術指標**
- [ ] @derive マクロで100行→5行の圧縮達成
- [ ] マクロ展開時間 < 100ms（中規模プロジェクト）
- [ ] 型エラー100%コンパイル時検出
- [ ] メモリ使用量増加 < 20%

### **開発者体験指標**  
- [ ] 学習コスト: 30分でマクロ作成可能
- [ ] デバッグ時間: --expand で即座に問題特定
- [ ] エラー理解率: 初学者でも90%理解

## 🏆 **世界制覇への道**

この実装完了により、Nyashは：

1. **Lisp超越**: 型安全性と構造化でパワーアップ
2. **Rust超越**: 学習コストとデバッグ性で圧勝  
3. **C++超越**: 安全性と開発効率で完全勝利
4. **Nim超越**: Box統一でさらに直感的
5. **Julia超越**: Python統合で科学計算も制覇

**世界最強マクロ言語**の地位を確立する！🌟

---

## 🔗 **関連Phase**

**⚠️ Phase 16実装完了後の状況（2025-10-06）**:
- ✅ Rust実装完了（2025-09-19）
- ❌ **バグ発見**: フル機能にバグあり
- ✅ **暫定対応**: `.hako`で単純置き換えマクロ実装中（`apps/macros/`）
- 🔜 **Phase 20**: セルフホスティング後にフル機能を.hakoで再実装

**Next Steps**:
- 現在: Phase 15.7完了待ち（セルフホスティング実現）
- その後: [Phase 20: マクロフル機能Hakorune実装](../phase-20-macro-full-features/README.md)

---

**Next**: [実装詳細](./IMPLEMENTATION.md) | [技術仕様](./TECHNICAL_SPEC.md)