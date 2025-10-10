# Closure実装計画（Phase 2予定）

**作成日**: 2025-01-10
**状態**: Phase 1完了後に実装予定

---

## 🎯 **実装範囲の明確化**

### **重要な発見**
- **NewClosure命令** = クロージャ生成（FunctionBox作成）
- **Callee::Closure** = クロージャ呼び出し指定子（Call命令で使用）
- **2つは別フェーズ**: 生成→呼び出しの流れ

### **Task Teacher調査結果**

#### 誤った想定（修正済み）
- ❌ "closure_id"フィールドは存在しない
- ✅ 実際のフィールド: `params`, `captures`, `me_capture`

#### 正しい構造
```rust
// NewClosure命令
NewClosure {
    dst: ValueId,
    params: Vec<String>,
    body: Vec<ASTNode>,
    captures: Vec<(String, ValueId)>,
    me: Option<ValueId>,
}

// Callee::Closure
Callee::Closure {
    params: Vec<String>,
    captures: Vec<(String, ValueId)>,
    me_capture: Option<ValueId>,
}
```

---

## 📊 **3つの実装オプション**

### Option A: Phase 1完了宣言（採用✅）
- **実装**: なし（Closureスキップ）
- **時間**: 0時間
- **理由**: 基本命令16種完了、MirCall 5/7完了、Selfhost Compiler Phase 1に十分

### Option B: NewClosure生成のみ
- **実装**: NewClosureHandlerBox作成
- **時間**: 3-4時間
- **内容**:
  - params/captures/me_capture抽出
  - ClosureBox生成→レジスタ格納
  - テスト: Closure生成確認のみ

### Option C: Closure完全実装
- **実装**: NewClosure + Callee::Value（生成+呼び出し）
- **時間**: 10-12時間
- **内容**:
  - NewClosureHandlerBox（3-4時間）
  - ValueCallHandlerBox（6-8時間）
  - Captures展開・スコープ注入
  - 再帰的VM呼び出し

---

## 🔧 **実装タスク詳細（Option B/C用）**

### Phase 2-1: NewClosureHandlerBox実装（3-4時間）

**ファイル**: `apps/selfhost/hakorune-vm/newclosure_handler.hako` (60-80行)

**実装内容**:
```hako
static box NewClosureHandlerBox {
  handle(inst_json, regs) {
    // 1. params抽出
    local params = JsonFieldExtractor.extract_array(inst_json, "params")

    // 2. captures抽出
    local captures = JsonFieldExtractor.extract_array(inst_json, "captures")

    // 3. me_capture抽出
    local me_capture = JsonFieldExtractor.extract_int(inst_json, "me")

    // 4. ClosureBox生成
    local closure_box = new MapBox()
    closure_box.set("params", params)
    closure_box.set("captures", captures)
    closure_box.set("me_capture", me_capture)

    // 5. dst抽出＋格納
    local dst = JsonFieldExtractor.extract_int(inst_json, "dst")
    ValueManagerBox.set(regs, dst, closure_box)

    return Result.Ok(0)
  }
}
```

**テストケース**:
```hako
// Test 1: Simple closure生成（0 captures）
local mir1 = r#"{"functions":[{"name":"test","blocks":[{"id":0,"instructions":[
  {"op":"newclosure","dst":1,"params":["x"],"body":[],"captures":[],"me":null},
  {"op":"ret","value":1}
],"terminator":{"op":"ret","value":1}}]}]}"#

// Test 2: Closure with captures
local mir2 = r#"{"functions":[{"name":"test","blocks":[{"id":0,"instructions":[
  {"op":"const","dst":2,"value":{"type":"i64","value":42}},
  {"op":"newclosure","dst":3,"params":["x"],"body":[],"captures":[["y",2]],"me":null},
  {"op":"ret","value":3}
],"terminator":{"op":"ret","value":3}}]}]}"#
```

---

### Phase 2-2: ValueCallHandlerBox実装（6-8時間）

**ファイル**: `apps/selfhost/hakorune-vm/value_call_handler.hako` (80-100行)

**実装内容**:
```hako
static box ValueCallHandlerBox {
  handle(mir_call_json, dst_reg, regs) {
    // 1. ValueId抽出
    local value_id = CalleeParserBox.extract_value_id(mir_call_json)

    // 2. ClosureBox取得
    local closure_box = ValueManagerBox.get(regs, value_id)

    // 3. Captures展開
    local captures = closure_box.get("captures")
    // TODO: captures をレジスタに展開

    // 4. 引数設定
    local args_array = ArgsExtractorBox.extract_and_load(mir_call_json, regs)
    // TODO: params にマッピング

    // 5. Closure実行（再帰的VM呼び出し？）
    // TODO: FunctionBox.body を実行

    return Result.Ok(0)
  }
}
```

---

## 📋 **JSON Emitter拡張（必要）**

**問題**: NewClosure命令がJSON出力されない

**修正箇所**: `src/runner/mir_json_emit.rs:492`
```rust
// Before:
_ => { /* skip non-essential ops for initial harness */ }

// After:
I::NewClosure { dst, params, body, captures, me } => {
    let captures_json: Vec<_> = captures.iter()
        .map(|(name, vid)| json!([name, vid.as_u32()]))
        .collect();

    insts.push(json!({
        "op": "newclosure",
        "dst": dst.as_u32(),
        "params": params,
        "body": serialize_ast_nodes(body), // ← AST→JSON変換必要
        "captures": captures_json,
        "me": me.map(|v| v.as_u32())
    }));
    emitted_defs.insert(dst.as_u32());
}
```

---

## 🎯 **実装優先順位**

### Phase 2開始時:
1. **JSON Emitter拡張**: NewClosure命令のJSON出力（1時間）
2. **NewClosureHandlerBox**: Closure生成ハンドラー（3-4時間）
3. **ValueCallHandlerBox**: Closure呼び出しハンドラー（6-8時間）

### 総見積もり: 10-13時間

---

## 📚 **参考資料**

- **Rust VM定義**: `src/mir/instruction.rs:80-88`
- **Lambda→MIR変換**: `src/mir/builder/exprs_lambda.rs:166-172`
- **Callee定義**: `src/mir/definitions/call_unified.rs:48-62`
- **既存実装**: ConstructorCallHandlerBox (90行、3時間で実装済み)

---

## 🎓 **重要な学び**

1. **NewClosure vs Callee::Closure**: 生成と呼び出しは別の段階
2. **JSON Emitter未実装**: NewClosure命令はJSON化されていない
3. **body: Vec<ASTNode>**: MIR中にAST構造が残る特殊ケース
4. **2レイヤーVM連携**: Hakorune VM → Rust VMのClosureBox呼び出し

---

**次回実装時**: このドキュメントからスタート
