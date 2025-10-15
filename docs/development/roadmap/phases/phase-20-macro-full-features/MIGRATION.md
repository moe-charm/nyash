# Phase 20: Rust → Hakorune マクロ移行ガイド

Date: 2025-10-06
Status: **REFERENCE** - Phase 20実装時に参照

## 🎯 **移行の目的**

Phase 16で実装したRust版マクロシステムの主要機能を、
Hakoruneでセルフホスト実装に書き直す。

## 📋 **移行対象機能一覧**

### **優先度 HIGH: 即座に便利**
| 機能 | Rust実装場所 | 移行先（案） | 難易度 |
|------|-------------|-------------|--------|
| @derive(Equals) | `src/macro/engine.rs:151-174` | `apps/macros/derive/derive_equals.hako` | ⭐⭐ |
| @derive(ToString) | `src/macro/engine.rs:176-194` | `apps/macros/derive/derive_tostring.hako` | ⭐⭐ |
| @test ランナー | `src/macro/mod.rs:61-401` | `apps/macros/test/test_runner.hako` | ⭐⭐⭐ |

### **優先度 MEDIUM: 複雑なマクロ用基盤**
| 機能 | Rust実装場所 | 移行先（案） | 難易度 |
|------|-------------|-------------|--------|
| TemplatePattern | `src/macro/pattern.rs:116-238` | `apps/macros/pattern/template_pattern.hako` | ⭐⭐⭐⭐ |
| Quote/Unquote | `src/macro/pattern.rs:9-114` | `apps/macros/quote/quote_system.hako` | ⭐⭐⭐⭐ |
| MacroEngine | `src/macro/engine.rs:11-101` | `apps/macros/engine/macro_engine.hako` | ⭐⭐⭐ |

### **優先度 LOW: 構文サポート（Parser側）**
| 機能 | Rust実装場所 | 移行判断 |
|------|-------------|----------|
| Match式構文 | `src/parser/expr/match_expr.rs` | **移行不要**（Rust Parserのまま維持） |
| PatternAst | `src/ast.rs` | **移行不要**（AST定義はRust） |

## 🔄 **移行戦略: 3つのアプローチ**

### **Strategy 1: JSON AST直接操作（推奨）**

**使用場面**: @derive, @test
**難易度**: ⭐⭐

```hakorune
// apps/macros/derive/derive_equals.hako
static box DeriveEqualsMacro {
    expand(json_ast, ctx) {
        // 1. JSON文字列をパース
        local ast, box_decl, fields, new_method
        ast = JSON.parse(json_ast)

        // 2. BoxDeclaration検出
        if ast.kind != "Program" {
            return json_ast
        }

        // 3. public_fieldsを取得
        fields = box_decl.public_fields

        // 4. equals()メソッドJSON生成
        new_method = me.build_equals_method(fields)

        // 5. methodsに追加
        box_decl.methods.set("equals", new_method)

        // 6. JSON文字列化
        return JSON.stringify(ast)
    }

    build_equals_method(fields) {
        // JSON AST構築（詳細は後述）
        return method_json
    }
}
```

**利点**:
- ✅ 現在の暫定実装（JSON文字列操作）の延長
- ✅ Hakoruneの機能だけで完結
- ✅ バグが少ない

**欠点**:
- ❌ JSON操作が冗長
- ❌ 複雑なマクロは困難

### **Strategy 2: AstBuilderBox実装（中間層）**

**使用場面**: Pattern Matching, Quote/Unquote
**難易度**: ⭐⭐⭐⭐

```hakorune
// apps/macros/ast/ast_builder.hako
box AstBuilderBox {
    // JSON ASTをHakoruneオブジェクトにラップ
    root: MapBox

    birth(json_str) {
        from MapBox.birth()
        me.root = JSON.parse(json_str)
    }

    // ヘルパーメソッド
    find_box_decl(name) {
        // AST走査してBoxDeclaration検出
    }

    add_method(box_name, method_name, body_ast) {
        // メソッド追加
    }

    to_json() {
        return JSON.stringify(me.root)
    }
}
```

**利点**:
- ✅ API設計でRust版に近い書き方が可能
- ✅ 複雑なマクロ実装が楽

**欠点**:
- ❌ AstBuilderBox実装コストが大きい
- ❌ JSON↔オブジェクト変換オーバーヘッド

### **Strategy 3: Rust Hybrid（非推奨）**

Rust側にヘルパー関数を残し、Hakoruneから呼ぶ。

**使用場面**: なし（セルフホスティングの意義を損なう）
**難易度**: ⭐

❌ **推奨しない理由**:
- Phase 20の目的は「Rust依存を減らす」こと
- ハイブリッドは複雑性が増すだけ

## 📝 **移行実装例: @derive(Equals)**

### **Rust版（参考）**
```rust
// src/macro/engine.rs:151-174
fn build_equals_method(_box_name: &str, fields: &Vec<String>) -> ASTNode {
    let cond = if fields.is_empty() {
        ASTNode::Literal { value: LiteralValue::Bool(true), span: Span::unknown() }
    } else {
        let mut it = fields.iter();
        let first = it.next().unwrap();
        let mut expr = bin_eq(me_field(first), var_field("__ny_other", first));
        for f in it {
            expr = bin_and(expr, bin_eq(me_field(f), var_field("__ny_other", f)));
        }
        expr
    };
    ASTNode::FunctionDeclaration {
        name: "equals".to_string(),
        params: vec!["__ny_other".to_string()],
        body: vec![ASTNode::Return { value: Some(Box::new(cond)), span: Span::unknown() }],
        is_static: false,
        is_override: false,
        span: Span::unknown(),
    }
}
```

### **Hakorune版（Strategy 1: JSON直接操作）**
```hakorune
// apps/macros/derive/derive_equals.hako
static box DeriveEqualsMacro {
    expand(json_str, ctx) {
        local ast, stmts, i, modified
        ast = JSON.parse(json_str)

        if ast.kind != "Program" {
            return json_str
        }

        stmts = ast.statements
        modified = false
        i = 0

        loop(i < stmts.length()) {
            local stmt
            stmt = stmts.get(i)

            if stmt.kind == "BoxDeclaration" {
                // public_fieldsチェック
                if stmt.public_fields && stmt.public_fields.length() > 0 {
                    // equals()メソッドが存在しないか確認
                    if !stmt.methods || !stmt.methods.has("equals") {
                        // equals()生成
                        local equals_method
                        equals_method = me.build_equals_json(stmt.name, stmt.public_fields)

                        // methodsに追加
                        if !stmt.methods {
                            stmt.methods = json({})
                        }
                        stmt.methods.set("equals", equals_method)
                        modified = true
                    }
                }
            }

            i = i + 1
        }

        if modified {
            return JSON.stringify(ast)
        } else {
            return json_str
        }
    }

    build_equals_json(box_name, fields) {
        // JSON AST for: equals(__ny_other) { return me.f1 == __ny_other.f1 && ... }
        local body, cond, i

        if fields.length() == 0 {
            cond = json({
                "kind": "Literal",
                "value": true,
                "span": json({})
            })
        } else {
            i = 0
            cond = null

            loop(i < fields.length()) {
                local field_name, eq_expr
                field_name = fields.get(i)

                // me.field == __ny_other.field
                eq_expr = json({
                    "kind": "BinaryOp",
                    "operator": "Equal",
                    "left": json({
                        "kind": "FieldAccess",
                        "object": json({ "kind": "Me", "span": json({}) }),
                        "field": field_name,
                        "span": json({})
                    }),
                    "right": json({
                        "kind": "FieldAccess",
                        "object": json({ "kind": "Variable", "name": "__ny_other", "span": json({}) }),
                        "field": field_name,
                        "span": json({})
                    }),
                    "span": json({})
                })

                if !cond {
                    cond = eq_expr
                } else {
                    // cond && eq_expr
                    cond = json({
                        "kind": "BinaryOp",
                        "operator": "And",
                        "left": cond,
                        "right": eq_expr,
                        "span": json({})
                    })
                }

                i = i + 1
            }
        }

        // FunctionDeclaration
        return json({
            "kind": "FunctionDeclaration",
            "name": "equals",
            "params": arr(["__ny_other"]),
            "body": arr([
                json({
                    "kind": "Return",
                    "value": cond,
                    "span": json({})
                })
            ]),
            "is_static": false,
            "is_override": false,
            "span": json({})
        })
    }
}
```

## 🧪 **テスト戦略**

### **1. ユニットテスト（Hakorune版）**
```hakorune
// apps/macros/derive/test_derive_equals.hako
static box TestDeriveEquals {
    test_simple_box() {
        local input, expected, actual

        input = json({
            "kind": "Program",
            "statements": arr([
                json({
                    "kind": "BoxDeclaration",
                    "name": "Person",
                    "public_fields": arr(["name", "age"]),
                    "methods": json({})
                })
            ])
        })

        actual = DeriveEqualsMacro.expand(JSON.stringify(input), "{}")

        // equals()メソッドが追加されていることを確認
        local result
        result = JSON.parse(actual)
        assert(result.statements.get(0).methods.has("equals"))

        return true
    }
}
```

### **2. 統合テスト（スモークテスト）**
```bash
# tools/smokes/v2/profiles/quick/macro/derive_equals_vm.sh
#!/usr/bin/env bash
# Smoke: @derive(Equals) basic functionality

NYASH_MACRO_ENABLE=1 \
NYASH_MACRO_PATHS=apps/macros/derive/derive_equals.hako \
./target/release/hako apps/tests/macro_derive_person.hako
```

### **3. パリティテスト（Rust版 vs Hakorune版）**
```bash
# 同じ入力に対して同じ出力を生成するか確認
diff <(NYASH_MACRO_DERIVE_BACKEND=rust ./hako test.hako --dump-ast) \
     <(NYASH_MACRO_DERIVE_BACKEND=hako ./hako test.hako --dump-ast)
```

## 📊 **進捗管理**

### **Phase 20.1: @derive(Equals)実装**
- [ ] DeriveEqualsMacro.build_equals_json() 実装
- [ ] ユニットテスト作成・通過
- [ ] スモークテスト作成・通過
- [ ] パリティテスト通過（Rust版と同一出力）
- [ ] ドキュメント作成

### **Phase 20.2: @derive拡張**
- [ ] DeriveToStringMacro 実装
- [ ] DeriveCloneMacro 実装
- [ ] DeriveDebugMacro 実装

### **Phase 20.3: @test ランナー**
- [ ] TestCollectorBox 実装
- [ ] TestRunnerBox 実装
- [ ] JSON引数注入対応

## 🚨 **リスク管理**

### **Risk 1: JSON操作の複雑性**
**対策**: AstBuilderBoxヘルパーを段階的に実装

### **Risk 2: パフォーマンス劣化**
**対策**: ベンチマーク必須（目標: Rust版の2倍以内）

### **Risk 3: バグ再発**
**対策**: Rust版の既知バグを事前に洗い出し、テストケース化

## 🔗 **参考資料**

- [Phase 16 README](../phase-16-macro-revolution/README.md) - Rust版設計書
- [Phase 16 IMPLEMENTATION](../phase-16-macro-revolution/IMPLEMENTATION.md) - Rust版実装ガイド
- `src/macro/engine.rs` - derive実装参考
- `src/macro/pattern.rs` - Pattern/Quote実装参考
- `src/macro/mod.rs` - @test実装参考

---

**Next**: Phase 15.7完了後、Phase 20.1開始
