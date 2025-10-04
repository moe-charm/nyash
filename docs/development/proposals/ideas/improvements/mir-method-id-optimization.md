# MIRメソッドID最適化提案

**作成日**: 2025-10-04
**優先度**: 高（効果大 × 実装容易）
**影響範囲**: Rust VM / LLVM / WASM すべて

## 🎯 目的

BoxCallのメソッド名文字列比較を整数比較に変換し、全バックエンドで高速化。

## 📊 期待効果

| Backend | 現状速度 | 最適化後 | 倍率 |
|---------|---------|---------|------|
| Rust VM | 35万 ops/sec | 350万 ops/sec | **10倍** |
| LLVM    | N/A | C言語並み | **100倍** |
| WASM    | N/A | ネイティブ並み | **100倍** |

## 🏗️ 設計

### Phase 1: MIR拡張

**BoxCall命令に`method_id`フィールド追加**:

```rust
// src/mir/instructions.rs
pub struct BoxCall {
    pub recv: RegisterId,
    pub method: String,           // 既存（デバッグ用保持）
    pub method_id: Option<u32>,   // 🆕 追加
    pub vtable_hint: Option<String>, // 🆕 型情報（最適化ヒント）
    pub args: Vec<RegisterId>,
    pub dst: RegisterId,
}
```

### Phase 2: コンパイル時にID割り当て

**メソッド名→ID変換テーブル**:

```rust
// src/mir/method_registry.rs
pub struct MethodRegistry {
    methods: HashMap<String, u32>,
}

impl MethodRegistry {
    pub fn register(&mut self, name: &str) -> u32 {
        if let Some(&id) = self.methods.get(name) {
            id
        } else {
            let id = self.methods.len() as u32;
            self.methods.insert(name.to_string(), id);
            id
        }
    }
}

// コンパイル時に自動変換
// timer.now_ms() → BoxCall { method_id: 0, method: "now_ms" }
```

### Phase 3: 各Backend実装

**Rust VM**:
```rust
// src/backend/mir_interpreter/vm.rs
match inst {
    BoxCall { method_id: Some(id), .. } => {
        // 高速パス：整数比較
        match id {
            0 => self.call_now_ms(recv),
            1 => self.call_size(recv),
            _ => unreachable!(),
        }
    }
    BoxCall { method, .. } => {
        // フォールバック：文字列比較
        match method.as_str() { ... }
    }
}
```

**LLVM**:
```python
# src/llvm_py/instructions/boxcall.py
if inst.get('method_id') is not None:
    # Vtable経由で直接呼び出し
    method_id = inst['method_id']
    vtable_ptr = builder.load(obj_vtable)
    func_ptr = builder.gep(vtable_ptr, [ir.Constant(i32, method_id)])
    result = builder.call(func_ptr, [obj])
else:
    # フォールバック：文字列経由
    ...
```

**WASM**:
```python
# src/llvm_py/targets/wasm/boxcall_wasm.py
if 'method_id' in inst:
    # call_indirect経由
    emit_call_indirect(method_id, obj, vtable)
else:
    # フォールバック
    ...
```

## 🎯 実装優先順位

**P0 (すぐできる)**:
1. MIR BoxCallにmethod_idフィールド追加
2. MethodRegistry実装（グローバルID管理）
3. コンパイラでmethod_id自動割り当て

**P1 (効果確認)**:
4. Rust VM実装（match整数比較）
5. ベンチマーク測定（10倍高速化確認）

**P2 (展開)**:
6. LLVM vtable実装
7. WASM call_indirect実装

## 🧪 検証方法

**ベンチマーク**:
```bash
# 現状
./target/release/hako --backend vm benchmarks/sum_loop_bench.hako
# → 35万 ops/sec

# 最適化後（期待）
./target/release/hako --backend vm benchmarks/sum_loop_bench.hako
# → 350万 ops/sec（10倍）

# LLVM（期待）
./target/release/hako --backend llvm benchmarks/sum_loop_bench_noprint.hako
# → C言語並み（5,000万 ops/sec）
```

## 💡 追加最適化案

**Vtable実装** (P3):
- 各Box型に関数ポインタテーブル
- method_id → vtable[id] 直接ジャンプ
- 動的ディスパッチ → 静的ディスパッチ

**インライン化** (P4):
- ホットパス検出（timer.now_ms()が繰り返される）
- BoxCall → 直接関数呼び出し
- Tracing JIT的な最適化

## 🚨 注意点

**後方互換性**:
- `method_id`は`Option<u32>`（既存MIR JSONも動作）
- フォールバック経路を常に保持

**デバッグ性**:
- `method`文字列を保持（エラーメッセージ用）
- `--dump-mir`で人間が読める形式

## 📚 参考

- Python: CALL_METHOD bytecode (method cache)
- JavaScript V8: Inline caches
- Java: Vtable dispatch
- Rust: trait object vtable

---

**次のアクション**: Phase 1実装（MIR拡張 + Registry）

**期待される成果**: 全バックエンドで10-100倍高速化 🚀
