# CallableBox提案 vs 既存Phase 1/2案の総合評価

**作成日**: 2025-10-10
**調査者**: Task Teacher (統合評価)
**目的**: 3つの設計案を比較し、最適な実装戦略を決定する

---

## エグゼクティブサマリー

### 推奨判定: **案C (Phase 2: Handler Box) を推奨**

**理由**:
- ✅ **今すぐ実行可能** (1週間以内で完成)
- ✅ **技術的負債を大幅削減** (if文25個→0個)
- ✅ **Hakoruneの既存機能のみ使用** (新規実装不要)
- ✅ **段階的移行が可能** (Phase 1→Phase 2→将来的にCallableBox)
- ✅ **Rust VM設計と整合** (Box-based dispatch)

**ChatGPT提案 (CallableBox) の評価**: 技術的には正しいが、実装コストが高すぎる (4-6人日)。Phase 20+で検討すべき将来機能。

---

## 1. 3つの設計案の詳細比較

### 案A: ChatGPT提案 (CallableBox + ref構文)

```hakorune
// 提案されたコード
local handlers = new MapBox()
handlers.set("double", ref Math.double/1)
local cb = handlers.get("double")
local result = cb.call([10])  // → 20
```

**実現に必要な新機能**:
1. `ref FunctionName/Arity` 構文 (関数参照取得)
2. CallableBox型 (関数をラップするBox)
3. `.call()` メソッド (CallableBox実行)
4. MIR拡張 (FunctionRef, CallableBoxCall命令)

**実装規模**:
- パーサー改修: 2人日 (ref構文追加、AST拡張)
- MIR改修: 1.5人日 (FunctionRef命令、CallableBox型)
- VM改修: 1人日 (CallableBox実行、.call()実装)
- LLVM対応: 0.5人日 (関数ポインタ実装)
- テスト: 1人日 (統合テスト、エッジケース)
- **合計**: 6人日

---

### 案B: Phase 1 (Registry + 文字列ID)

```hakorune
static box MethodRegistry {
    registry: MapBox  // key=method_name, value=handler_id

    dispatch(method, value) {
        local handler_id = me.registry.get(method)
        if handler_id == "handler_double" {
            return value * 2
        } else if handler_id == "handler_triple" {
            return value * 3
        } else if handler_id == "handler_square" {
            return value * value
        }
        // ... 25個のif-else文
        else {
            return null  // Unknown method
        }
    }

    register(method, handler_id) {
        me.registry.set(method, handler_id)
    }
}
```

**実装規模**:
- Registry box作成: 0.5人日
- dispatch実装: 1人日 (25個のif-else)
- テスト: 0.5人日
- **合計**: 2人日

**問題点**:
- ❌ **ハードコーディング**: 25個のif文をベタ書き
- ❌ **メンテナンス性**: メソッド追加のたびにdispatch修正
- ❌ **拡張性**: 新Boxタイプ追加が困難

---

### 案C: Phase 2 (Handler Box)

```hakorune
// ハンドラーインターフェース (既存Hakoruneで実現可能)
static box HandlerInterface {
    // すべてのハンドラーはinvoke()を実装
}

// 個別ハンドラー実装
box DoubleHandler {
    invoke(args) {
        return args.get(0) * 2
    }
}

box TripleHandler {
    invoke(args) {
        return args.get(0) * 3
    }
}

box SquareHandler {
    invoke(args) {
        local val = args.get(0)
        return val * val
    }
}

// Registry (ハンドラーBoxを格納)
static box MethodRegistry {
    registry: MapBox  // key=method_name, value=HandlerBox instance

    dispatch(method, value) {
        local handler = me.registry.get(method)
        if handler == null {
            return null  // Unknown method
        }

        local args = new ArrayBox()
        args.push(value)
        return handler.invoke(args)  // 動的ディスパッチ (0個のif文!)
    }

    register(method, handler) {
        me.registry.set(method, handler)
    }
}

// 使用例
static box Main {
    main() {
        MethodRegistry.register("double", new DoubleHandler())
        MethodRegistry.register("triple", new TripleHandler())
        MethodRegistry.register("square", new SquareHandler())

        local result1 = MethodRegistry.dispatch("double", 10)  // → 20
        local result2 = MethodRegistry.dispatch("triple", 5)   // → 15
        local result3 = MethodRegistry.dispatch("square", 4)   // → 16
    }
}
```

**実装規模**:
- HandlerInterface定義: 0.1人日 (コメント・ドキュメント)
- Handler boxes作成: 0.5人日 (25個のHandler × 10-20行/個)
- Registry実装: 0.3人日 (MapBox使用、if文1個のみ)
- テスト: 0.6人日 (25メソッド × 複数ケース)
- **合計**: 1.5人日

**利点**:
- ✅ **完全動的ディスパッチ**: if文0個 (nullチェック1個のみ)
- ✅ **拡張容易**: 新ハンドラー追加 = 新Box作成のみ
- ✅ **既存機能のみ**: MapBox + Box + 既存MIR命令
- ✅ **Rust VM統一**: MethodRouterBoxと同じパターン

---

## 2. 比較表 (6つの評価軸)

| 評価軸 | 案A: CallableBox | 案B: Phase 1 | 案C: Phase 2 |
|--------|-----------------|-------------|-------------|
| **実現可能性** | 中 (新機能必須) | 高 (既存機能のみ) | **高 (既存機能のみ)** |
| **実装規模** | 6人日 | 2人日 | **1.5人日** |
| **技術的リスク** | 高 (パーサー・MIR拡張) | 低 (標準実装) | **低 (標準実装)** |
| **if文削減** | 0個 (完全削除) | 25個 (ベタ書き) | **0個 (完全削除)** |
| **保守性** | 高 | 低 (ハードコーディング) | **高 (Box追加のみ)** |
| **拡張性** | 高 | 低 (dispatch修正必須) | **高 (Box追加のみ)** |
| **Rust VM統一** | 中 (新パターン) | 低 (文字列ベース) | **高 (既存パターン)** |
| **パフォーマンス** | 高 (最適化可能) | 中 (if文ループ) | **高 (Box dispatch)** |
| **総合評価** | 7/10 | 4/10 | **9/10** |

---

## 3. 各案の詳細評価

### 案A: CallableBox (ChatGPT提案)

#### メリット
- ✅ **最も洗練**: 関数が第一級オブジェクト (functional programming)
- ✅ **完全動的**: 実行時に関数を自由に組み合わせ
- ✅ **将来性**: ラムダ式・クロージャへの拡張パス
- ✅ **理論的に正しい**: 関数型言語の標準パターン

#### デメリット
- ❌ **実装コスト高**: 6人日 (Phase 2の4倍)
- ❌ **新機能必須**: ref構文、CallableBox型、MIR拡張
- ❌ **リスク高**: パーサー拡張は予期せぬバグの温床
- ❌ **Hakoruneのフィロソフィー**: "Everything is Box" ではなく "Functions are Values"

#### 実装詳細

**1. ref構文 (パーサー拡張)**
```rust
// src/parser/expressions.rs に追加
pub fn parse_ref_expression(&mut self) -> Result<ASTNode, ParseError> {
    self.expect_token(TokenType::REF)?;  // 新トークン
    let func_name = self.expect_identifier()?;
    self.expect_token(TokenType::SLASH)?;
    let arity = self.expect_integer()?;
    Ok(ASTNode::FunctionRef { name: func_name, arity })
}
```

**2. CallableBox型 (MIR拡張)**
```rust
// src/mir/instruction.rs に追加
pub enum MirInstruction {
    // 既存命令...
    FunctionRef {
        func: String,
        arity: usize,
        dst: ValueId,
    },
    CallableBoxCall {
        callable: ValueId,
        args: Vec<ValueId>,
        dst: ValueId,
    },
}
```

**3. VM実装 (Rust)**
```rust
// src/backend/mir_interpreter/handlers/callable.rs (新規)
pub struct CallableBox {
    func_name: String,
    arity: usize,
}

impl CallableBox {
    pub fn call(&self, interp: &mut MirInterpreter, args: &[VMValue]) -> Result<VMValue, VMError> {
        // 関数名から実際の関数を解決して実行
        interp.call_global_function(&self.func_name, args)
    }
}
```

#### 実装優先度
- **Phase 20+で検討** (現在は不要)
- 理由:
  - Hakorune VMが完成し、実際に「関数参照が頻繁に必要」となった場合に検討
  - 現時点では Phase 2 (Handler Box) で十分

---

### 案B: Phase 1 (Registry + 文字列ID)

#### メリット
- ✅ **実装簡単**: 2人日で完成
- ✅ **既存機能のみ**: MapBox + if-else
- ✅ **理解容易**: 初学者でも読める

#### デメリット
- ❌ **技術的負債**: 25個のif文ベタ書き
- ❌ **保守性最悪**: メソッド追加 = dispatch関数修正
- ❌ **拡張性なし**: 新Boxタイプ追加が困難
- ❌ **テストコスト**: 25個のif文 × 各テスト = 膨大

#### 実装詳細

**MethodRegistry実装**
```hakorune
static box MethodRegistry {
    registry: MapBox

    dispatch(method, value) {
        local handler_id = me.registry.get(method)

        // ❌ ハードコーディング地獄
        if handler_id == "handler_double" {
            return value * 2
        } else if handler_id == "handler_triple" {
            return value * 3
        } else if handler_id == "handler_square" {
            return value * value
        } else if handler_id == "handler_upper" {
            return value.toUpper()
        } else if handler_id == "handler_lower" {
            return value.toLower()
        }
        // ... 20個以上のif-else続く
        else {
            return null
        }
    }
}
```

**問題点の具体例**:
- 新メソッド追加 → 5箇所修正 (register, dispatch, test, doc, smoke)
- if文の順序依存 (パフォーマンス影響)
- テスト漏れのリスク (25個のif)

#### 実装優先度
- **不採用** (技術的負債が大きすぎる)

---

### 案C: Phase 2 (Handler Box) ⭐推奨

#### メリット
- ✅ **最短実装**: 1.5人日で完成
- ✅ **完全動的**: if文0個 (nullチェック1個のみ)
- ✅ **既存機能のみ**: MapBox + Box (MIR拡張不要)
- ✅ **拡張容易**: 新ハンドラー = 新Box作成のみ
- ✅ **Rust VM統一**: MethodRouterBoxと同パターン
- ✅ **テスト容易**: ハンドラー単体テスト可能

#### デメリット
- ⚠️ **若干の冗長性**: Handler box × 25個作成
- ⚠️ **命名規則**: invoke()という名前を統一する必要

#### 実装詳細

**1. ハンドラーインターフェース (コメントベース)**
```hakorune
// apps/lib/patterns/handler_interface.hako
// すべてのHandlerはこのパターンを実装
//
// interface Handler {
//   invoke(args: ArrayBox): any
// }
//
// 注: Hakoruneには明示的interfaceがないため、
//     コメントでパターンを定義
```

**2. 具体的なハンドラー実装例**
```hakorune
// apps/selfhost/hakorune-vm/handlers/double_handler.hako
box DoubleHandler {
    invoke(args) {
        local value = args.get(0)
        return value * 2
    }
}

// apps/selfhost/hakorune-vm/handlers/upper_handler.hako
box UpperHandler {
    invoke(args) {
        local str = args.get(0)
        return str.toUpper()
    }
}

// apps/selfhost/hakorune-vm/handlers/substring_handler.hako
box SubstringHandler {
    invoke(args) {
        local str = args.get(0)
        local start = args.get(1)
        local end = args.get(2)
        return str.substring(start, end)
    }
}
```

**3. Registry実装**
```hakorune
// apps/selfhost/hakorune-vm/method_registry.hako
using "apps/selfhost/hakorune-vm/handlers/double_handler.hako" as DoubleHandler
using "apps/selfhost/hakorune-vm/handlers/upper_handler.hako" as UpperHandler
// ... 他のハンドラー

static box MethodRegistry {
    handlers: MapBox

    birth() {
        me.handlers = new MapBox()

        // ハンドラー登録 (起動時1回のみ)
        me.handlers.set("double", new DoubleHandler())
        me.handlers.set("triple", new TripleHandler())
        me.handlers.set("square", new SquareHandler())
        me.handlers.set("upper", new UpperHandler())
        me.handlers.set("lower", new LowerHandler())
        // ... 25個のハンドラー
    }

    dispatch(method, args) {
        local handler = me.handlers.get(method)

        // ✅ if文1個のみ! (nullチェック)
        if handler == null {
            print("[ERROR] Unknown method: " + method)
            return null
        }

        // 完全動的ディスパッチ
        return handler.invoke(args)
    }
}
```

**4. BoxCallHandlerBox統合**
```hakorune
// apps/selfhost/hakorune-vm/boxcall_handler.hako
using "apps/selfhost/hakorune-vm/method_registry.hako" as MethodRegistry

static box BoxCallHandlerBox {
    handle(inst_json, regs) {
        local method_name = JsonFieldExtractor.extract_string(inst_json, "method")
        local args_array = me._extract_args(inst_json, regs)

        // ✅ 25個のif-else削除! Registry経由で動的ディスパッチ
        local result_val = MethodRegistry.dispatch(method_name, args_array)

        if dst_reg != null {
            ValueManagerBox.set(regs, dst_reg, result_val)
        }

        return Result.Ok(0)
    }
}
```

#### Rust VMとの設計統一

**Rust VM (MethodRouterBox)**:
```rust
// src/runtime/method_router_box/mod.rs (既存コード)
pub fn route(
    interp: &mut MirInterpreter,
    receiver: &VMValue,
    method: &str,
    args: &[VMValue],
) -> Result<VMValue, VMError> {
    // 型ベースディスパッチ
    match receiver {
        VMValue::String(s) => {
            // StringBox methods
            match method {
                "toUpper" => Ok(VMValue::String(s.to_uppercase())),
                "toLower" => Ok(VMValue::String(s.to_lowercase())),
                // ...
            }
        }
        VMValue::BoxRef(bx) => {
            // Box methods
            match bx.type_name() {
                "ArrayBox" => { /* ... */ }
                "MapBox" => { /* ... */ }
                _ => Err(VMError::InvalidInstruction(...))
            }
        }
    }
}
```

**Hakorune VM (MethodRegistry) - 同じパターン**:
```hakorune
static box MethodRegistry {
    dispatch(method, args) {
        local handler = me.handlers.get(method)  // 型ベースディスパッチの代わりにMapBox
        if handler == null { return null }
        return handler.invoke(args)  // 動的ディスパッチ
    }
}
```

**統一性のポイント**:
- Rust VM: 型チェック (match type_name) → メソッド実行
- Hakorune VM: MapBox検索 (get method) → ハンドラー実行
- **同じ構造**: Registry → Dispatcher → Handler

#### 実装優先度
- **最優先** (今週実装すべき)

---

## 4. 段階的移行の可能性

### Phase 1 → Phase 2 → CallableBox の移行パス

#### Step 1: Phase 1実装 (2人日)
```hakorune
static box MethodRegistry {
    dispatch(method, value) {
        if method == "double" { return value * 2 }
        else if method == "triple" { return value * 3 }
        // ... 25個のif
    }
}
```

#### Step 2: Phase 2移行 (1.5人日)
```hakorune
// Phase 1のdispatchを段階的にHandlerに置き換え
static box MethodRegistry {
    handlers: MapBox

    dispatch(method, args) {
        // Step 2a: 既存のif-elseを残しつつ、一部をHandlerに移行
        local handler = me.handlers.get(method)
        if handler != null {
            return handler.invoke(args)
        }

        // Step 2b: 残りのif-else (段階的に削減)
        if method == "double" { return args.get(0) * 2 }
        else if method == "triple" { return args.get(0) * 3 }
        // ...
    }
}
```

#### Step 3: CallableBox移行 (Phase 20+)
```hakorune
// 将来的にCallableBoxが実装されたら:
static box MethodRegistry {
    handlers: MapBox

    dispatch(method, args) {
        local callable = me.handlers.get(method)
        if callable == null { return null }

        // CallableBox.call() を使用
        return callable.call(args)
    }
}
```

**移行の利点**:
- ✅ **段階的**: Phase 1 → Phase 2 → CallableBox
- ✅ **後方互換**: 各ステップで動作確認可能
- ✅ **リスク分散**: 一度に大きな変更をしない

### 相互運用性

**Handler Box と CallableBox の混在**:
```hakorune
// 将来的に両方が共存可能
static box MethodRegistry {
    handlers: MapBox

    dispatch(method, args) {
        local item = me.handlers.get(method)
        if item == null { return null }

        // 型チェック (将来的に実装)
        if item.type() == "CallableBox" {
            return item.call(args)
        } else {
            // Handler Boxとして扱う
            return item.invoke(args)
        }
    }
}
```

---

## 5. 推奨判定 (4つの基準)

### 基準1: 今すぐ実行可能 (1週間以内)

| 案 | 実行可能性 | 理由 |
|----|-----------|------|
| 案A: CallableBox | ❌ 不可能 | パーサー・MIR拡張に4-6人日必要 |
| 案B: Phase 1 | ✅ 可能 | 2人日で実装可能 |
| **案C: Phase 2** | **✅ 可能** | **1.5人日で実装可能** |

**推奨**: **案C (Phase 2)**

---

### 基準2: 技術的負債削減が最優先

| 案 | ハードコーディング削減度 | 保守性 | 拡張性 |
|----|----------------------|-------|-------|
| 案A: CallableBox | ✅ 完全削除 (0個) | 最高 | 最高 |
| 案B: Phase 1 | ❌ 削減なし (25個) | 最悪 | 最悪 |
| **案C: Phase 2** | **✅ 完全削除 (0個)** | **最高** | **最高** |

**推奨**: **案C (Phase 2)** (CallableBoxと同等の効果、1/4の実装コスト)

---

### 基準3: 長期的な保守性重視

| 案 | 拡張性 | Rust VM統一度 | 将来性 |
|----|-------|-------------|-------|
| 案A: CallableBox | 最高 (関数型) | 中 (新パターン) | 最高 (ラムダへの道) |
| 案B: Phase 1 | 最悪 (修正地獄) | 低 (文字列ベース) | 最悪 (負債拡大) |
| **案C: Phase 2** | **最高 (Box追加)** | **最高 (既存パターン)** | **高 (CallableBoxへ移行可能)** |

**推奨**: **案C (Phase 2)** (現時点で最適、将来的にCallableBoxへ移行可能)

---

### 基準4: バランス重視 (実装コストと効果)

| 案 | 実装コスト | 効果 | コストパフォーマンス |
|----|----------|------|------------------|
| 案A: CallableBox | 6人日 | if削減100% | 16.7% / 人日 |
| 案B: Phase 1 | 2人日 | if削減0% | 0% / 人日 |
| **案C: Phase 2** | **1.5人日** | **if削減100%** | **66.7% / 人日** |

**推奨**: **案C (Phase 2)** (最高のコストパフォーマンス)

---

## 6. ChatGPT提案の評価

### 正しい部分
- ✅ **関数を第一級オブジェクトにする**: 理論的に正しい
- ✅ **動的ディスパッチの必要性**: 問題認識が正確
- ✅ **ハードコーディング削減**: 目的が明確

### 問題がある部分
- ❌ **実装コストの過小評価**: 「簡単に実装できる」という誤認識
- ❌ **既存機能の見落とし**: Handler Boxで同じことが実現可能
- ❌ **優先度判断の誤り**: 今すぐ必要な機能ではない

### 実現可能性
**Phase 20+で可能** (今すぐは不要)

**理由**:
1. Handler Box (Phase 2) で同じ効果が得られる
2. パーサー・MIR拡張のリスクが高い
3. Hakoruneの哲学 ("Everything is Box") と一致しない

### 実装優先度
**低優先** (Phase 20+で検討)

**判断基準**:
- 現在の課題: BoxCallHandlerBoxの25個のif文削除
- 解決策: Handler Box (1.5人日)
- CallableBox: 将来的な拡張機能 (現在は不要)

---

## 7. 総合推奨

### 短期 (今週)
**推奨案**: **案C (Phase 2: Handler Box)**

**実装計画**:
1. Day 1 (0.6人日): ハンドラーBox × 25個作成
2. Day 2 (0.3人日): MethodRegistry実装
3. Day 3 (0.6人日): BoxCallHandlerBox統合 + テスト
4. **合計**: 1.5人日

**成果物**:
- ✅ if文25個 → 0個
- ✅ 拡張容易 (新Box追加のみ)
- ✅ Rust VM統一 (MethodRouterBoxパターン)
- ✅ テスト容易 (Handler単体テスト)

---

### 中期 (1ヶ月)
**継続**: **案C (Phase 2)** を使い続ける

**拡張計画**:
- 新Boxタイプ追加時: 新Handlerを作成
- 新メソッド追加時: 既存Handlerに追加
- パフォーマンス最適化: Handler内部を最適化

---

### 長期 (Phase 20+)
**検討**: **案A (CallableBox)** への移行

**移行条件** (以下のいずれかが満たされた場合):
1. 関数参照が頻繁に必要になった
2. ラムダ式・クロージャの実装が決定
3. Hakoruneの哲学が変化 ("Functions are Values")

**移行方法**:
- Phase 2 (Handler Box) → CallableBox への段階的置き換え
- 互換レイヤー作成 (Handler.invoke → Callable.call)

---

## 8. ユーザーへの提案

### 今すぐ実行すべきアクション

#### Action 1: Phase 2 (Handler Box) 実装開始

**タスク**:
```bash
# 1. ハンドラーBox雛形作成 (テンプレート)
cat > apps/selfhost/hakorune-vm/handlers/TEMPLATE.hako << 'EOF'
// Handler template
box XxxHandler {
    invoke(args) {
        // Implementation here
        local value = args.get(0)
        return value  // Modify as needed
    }
}
EOF

# 2. 25個のHandlerを作成 (0.6人日)
# - double_handler.hako
# - triple_handler.hako
# - square_handler.hako
# - upper_handler.hako
# - lower_handler.hako
# ... (20個以上)

# 3. MethodRegistry実装 (0.3人日)
# - apps/selfhost/hakorune-vm/method_registry.hako

# 4. BoxCallHandlerBox統合 (0.6人日)
# - 既存の25個のif-elseを削除
# - MethodRegistry.dispatch()呼び出しに置き換え
```

**見積もり**: 1.5人日 (今週中に完成)

---

#### Action 2: CallableBox提案の記録

**タスク**:
```bash
# 将来の検討用にChatGPT提案を記録
cat > docs/development/proposals/ideas/callable-box-future.md << 'EOF'
# CallableBox提案 (Phase 20+検討)

## 概要
関数を第一級オブジェクトとして扱う機能。

## 実装規模
6人日 (パーサー2人日 + MIR1.5人日 + VM1人日 + テスト1.5人日)

## 優先度
Phase 20+ (現在は Handler Box で十分)

## 移行パス
Phase 2 (Handler Box) → CallableBox
EOF
```

---

#### Action 3: 実装完了後の確認

**テストチェックリスト**:
- [ ] 25個のHandlerが正しく動作
- [ ] MethodRegistry.dispatch()が正しく動作
- [ ] BoxCallHandlerBoxのif文が0個 (nullチェック除く)
- [ ] 既存のテストがすべてPASS
- [ ] 新規Handler追加が容易 (5分以内)

**成功基準**:
- ✅ if文25個 → 0個
- ✅ テスト100% PASS
- ✅ 実装時間1.5人日以内

---

## 9. まとめ

### 最終推奨: 案C (Phase 2: Handler Box)

**理由** (優先順):
1. **最短実装**: 1.5人日 (CallableBoxの1/4)
2. **完全削減**: if文25個 → 0個 (CallableBoxと同等)
3. **既存機能のみ**: MapBox + Box (リスク最小)
4. **Rust VM統一**: MethodRouterBoxと同パターン
5. **段階的移行**: 将来的にCallableBoxへ移行可能

**ChatGPT提案 (CallableBox) の位置づけ**:
- 技術的に正しい将来機能
- Phase 20+で検討
- 現時点では実装不要

**ユーザーが今すぐやるべきこと**:
- ✅ Phase 2 (Handler Box) 実装開始
- ✅ 1.5人日で完成を目指す
- ✅ CallableBoxは将来の検討事項として記録

---

**作成日**: 2025-10-10
**次回レビュー**: Phase 2実装完了後
**関連ドキュメント**:
- [BoxCallHandlerBox実装](/home/tomoaki/git/hakorune-selfhost/apps/selfhost/hakorune-vm/boxcall_handler.hako)
- [MethodRouterBox実装](/home/tomoaki/git/hakorune-selfhost/src/runtime/method_router_box/mod.rs)
- [MIR命令セット](/home/tomoaki/git/hakorune-selfhost/docs/reference/mir/INSTRUCTION_SET.md)
