# Macro System Improvements (Phase 20+)

**Status**: Parking Lot (post-freeze)
**Created**: 2025-10-16
**Priority**: P2 (after Phase 15.77 freeze)

## Context

Hakoruneマクロシステムの現状分析と改善策。Phase 15.77凍結後に実施する。

## Current Strengths (つよつよポイント)

1. **セルフホスト性** ⭐⭐⭐⭐⭐ (Lisp級)
   - マクロ自体をHakorune言語で書ける
   - 63日セルフホスト達成の証明

2. **シンプルさ** ⭐⭐⭐⭐☆ (業界トップクラス)
   - 特別な構文不要、ただの箱
   - 「Everything is Box」哲学の体現

3. **安全性** ⭐⭐⭐⭐☆ (業界唯一)
   - サンドボックス実行
   - Capability制御
   - タイムアウト機構

## Current Weaknesses (よわよわポイント)

1. **型安全性** ⭐☆☆☆☆ (C級)
   - JSON文字列操作 = 型チェックなし
   - 実行時まで検出不可

2. **パフォーマンス** ⭐⭐☆☆☆ (最下位)
   - Rustの10-100倍遅い
   - 子プロセス起動オーバーヘッド

3. **デバッグ性** ⭐⭐☆☆☆ (C級)
   - ツーリング未成熟
   - AST可視化なし

## Improvement Roadmap

### Phase 1: クイックウィン（1-2週間）

#### 1. マクロ展開後検証
```rust
// src/macro/engine.rs
fn expand_with_validation(&mut self, node: &ASTNode) -> Result<ASTNode, String> {
    let expanded = self.expand_node(node);

    // 展開後のASTをパーサーで再検証
    let json = ast_to_json(&expanded);
    match json_to_ast(&json) {
        Some(ast) => Ok(ast),
        None => Err(format!(
            "Macro expansion produced invalid AST. \
             This is a macro bug, not your code."
        ))
    }
}
```

**効果**: 型安全性 20%↑

#### 2. MacroErrorBox実装
```hakorune
box MacroErrorBox {
    macro_name: StringBox
    error_message: StringBox
    input_ast: StringBox
    output_ast: StringBox

    format() {
        return "Macro Error in '" + me.macro_name + "'\n" +
               "  Error: " + me.error_message + "\n" +
               "  Hint: This is a macro bug."
    }
}
```

**効果**: エラーメッセージ 100%↑

#### 3. MacroTraceBox実装
```hakorune
static box MacroTrace {
    start(macro_name) {
        print("[macro] " + macro_name + " START")
    }

    step(description, ast_snippet) {
        print("[macro]   " + description)
    }
}
```

**効果**: デバッグ性 50%↑

### Phase 2: 中期改善（1-2ヶ月）

#### 4. ASTSchemaBox実装
```hakorune
static box ASTSchema {
    parse(json) {
        local schema = new ASTSchemaBox()
        schema.validate(json)
        return schema
    }

    build_program(statements) {
        local p = new ProgramNodeBox()
        p.statements = statements
        return p
    }
}
```

**効果**: 型安全性 80%↑

#### 5. MacroTestBox実装
```hakorune
static box MacroTest {
    ast_from_code(code) {
        return parse_to_json(code)
    }

    assert_ast_eq(expected, actual, message) {
        if expected != actual {
            print("FAIL: " + message)
            return false
        }
        print("PASS: " + message)
        return true
    }
}
```

**効果**: テスタビリティ 100%↑

#### 6. マクロプロセスプール
```rust
pub struct MacroPool {
    workers: Vec<MacroWorker>,
}

impl MacroPool {
    pub fn expand(&mut self, ast: &ASTNode) -> ASTNode {
        // ラウンドロビンで空いているワーカーに割り当て
        let worker = &mut self.workers[self.next_worker_idx];

        // パイプ経由でJSON送信/受信
        worker.expand_via_pipe(ast)
    }
}
```

**効果**: パフォーマンス 10倍↑

### Phase 3: 長期改善（3-6ヶ月）

#### 7. LSP実装
```rust
pub fn handle_hover(params: HoverParams) -> Option<Hover> {
    let node = get_ast_node_at_pos(&uri, pos)?;

    if is_macro_expandable(&node) {
        let expanded = expand_macro(&node);
        return Some(hover_with_expansion(expanded));
    }
}
```

**効果**: IDE統合 100%↑

#### 8. インメモリAST
```rust
pub fn expand_fast(&mut self, ast: &ASTNode) -> ASTNode {
    // JSON変換なし、直接AST操作
    match self.macro_behavior {
        MacroBehavior::Identity => ast.clone(),
        MacroBehavior::LoopNormalize => {
            loop_normalize_transform(ast)
        }
    }
}
```

**効果**: パフォーマンス 100倍↑

## Score Prediction

| カテゴリ | 現在 | Phase 1 | Phase 2 | Phase 3 |
|---------|-----|---------|---------|---------|
| 型安全性 | 1/5 | 2/5 | 4/5 | 4/5 |
| エラーメッセージ | 1/5 | 5/5 | 5/5 | 5/5 |
| パフォーマンス | 2/5 | 2/5 | 4/5 | 5/5 |
| デバッグ性 | 2/5 | 4/5 | 5/5 | 5/5 |
| テスタビリティ | 1/5 | 1/5 | 5/5 | 5/5 |
| IDE統合 | 0/5 | 0/5 | 0/5 | 4/5 |
| **総合** | **27/40** | **34/40** | **43/40** | **48/40** |
| | **(67.5%)** | **(85%)** | **(107.5%)** | **(120%)** |

## Implementation Priority

1. ✅ **Phase 1** - After Phase 15.77 freeze
2. ⚠️ **Phase 2** - After Mini-VM completion
3. ⚠️ **Phase 3** - Long-term investment

## References

- Language comparison analysis (2025-10-16 session)
- Macro system architecture: `docs/guides/macro-box.md`
- AST JSON spec: `docs/reference/ir/ast-json-v0.md`

## Related Issues

- Phase 19: @enum/@match implementation (blocked on VM equals() bug)
- Phase 20: VariantBox Core
