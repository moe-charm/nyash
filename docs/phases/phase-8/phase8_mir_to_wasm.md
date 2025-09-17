# Phase 8: MIR→WASM codegen (browser/wasmtime; sandboxed; Rust runtime free)

## Summary
- MIR から素の WebAssembly を生成し、ブラウザ/wasmtime（WASI）でサンドボックス実行する。
- Rust は「コンパイラ本体」のみ。実行は純WASM＋ホストimport（env.print 等）。
- Phase 6/7で実装済みのMIR命令（RefNew/RefGet/RefSet, FutureNew/Await等）をWASM命令に変換

## Technical Architecture
### WASM Module Structure
```wat
(module
  (memory (export "memory") 1)  ; 64KB initial
  (import "env" "print" (func $print (param i32)))
  
  ;; Heap management
  (global $heap_ptr (mut i32) (i32.const 1024))
  
  ;; Main entry point  
  (func (export "main") (result i32) 
    ;; Generated from MIR main function
  )
)
```

### Memory Layout
- `0x000-0x3FF`: Reserved/globals
- `0x400-0x7FF`: Stack space  
- `0x800+`: Heap (bump allocator)
- Box layout: `[type_id:i32][field_count:i32][field0:i32][field1:i32]...`

## Scope
- **ABI/Imports/Exports（最小）**
  - exports: `main() -> i32`, `memory`
  - imports: `env.print(i32)`（デバッグ用に整数のみ。将来文字列ABIを定義）
- **メモリ/ヒープ**
  - 線形メモリに簡易ヒープ（bump allocator → フリーリスト）
  - Box の固定レイアウト（フィールド→オフセット表; 型名→レイアウトは暫定固定）
- **命令カバレッジ（段階導入）**
  - **PoC1**: 算術/比較/分岐/loop/return/print
  - **PoC2**: RefNew/RefSet/RefGet（Phase 6 と整合）で `print(o.x)`
  - **PoC3**: Weak/Barrier の下地（WeakLoad は当面 Some 相当、Barrier は no-op）
  - **PoC4**: Future/Await の基本実装（スレッドなしの即座完了）
- **CLI 統合**
  - `nyash --backend wasm program.nyash` で生成・実行（wasmtime 呼び出し）
  - `--output program.wasm` でWASMファイル出力のみ

## Implementation Plan

### Phase 8.1: 基盤構築 (Foundation)
- [ ] **Task 1.1**: WASMバックエンドモジュール作成
  - `src/backend/wasm/mod.rs` - エントリポイント
  - `src/backend/wasm/codegen.rs` - MIR→WASM変換器
  - `src/backend/wasm/memory.rs` - メモリ管理
  - `src/backend/wasm/runtime.rs` - ランタイムヘルパー
  
- [ ] **Task 1.2**: WASM出力基盤
  - WAT形式での出力（人間可読、デバッグ用）
  - `wabt` crateでWAT→WASMバイナリ変換
  -基本的なmodule structure生成

### Phase 8.2: PoC1 - 基本演算 (Basic Operations)
- [ ] **Task 2.1**: MIR基本命令の変換実装
  - `MirInstruction::Const` → WASM `i32.const`
  - `MirInstruction::BinOp` → WASM算術命令 (`i32.add`, `i32.mul` etc.)
  - `MirInstruction::Compare` → WASM比較命令 (`i32.eq`, `i32.lt` etc.)
  
- [ ] **Task 2.2**: 制御フロー実装
  - `MirInstruction::Branch` → WASM `br_if`
  - `MirInstruction::Jump` → WASM `br`
  - `MirInstruction::Return` → WASM `return`
  
- [ ] **Task 2.3**: Print機能実装
  - `MirInstruction::Print` → `call $print`
  - env.print import の定義

**PoC1目標**: `42 + 8` のような基本計算がWASMで動作

### Phase 8.3: PoC2 - オブジェクト操作 (Object Operations)
- [ ] **Task 3.1**: メモリ管理実装
  - Bump allocator (`$heap_ptr` global)
  - `malloc(size) -> ptr` WASM function
  - Box layout定義 (`[type_id][field_count][fields...]`)

- [ ] **Task 3.2**: 参照操作実装
  - `MirInstruction::RefNew` → `call $malloc` + 初期化
  - `MirInstruction::RefGet` → memory load (`i32.load offset=...`)
  - `MirInstruction::RefSet` → memory store (`i32.store offset=...`)

**PoC2目標**: `o = new Obj(); o.x = 1; print(o.x)` 相当がWASMで動作

### Phase 8.4: PoC3 - 拡張機能下地 (Extension Foundation)
- [ ] **Task 4.1**: Weak参照ダミー実装
  - `MirInstruction::WeakNew` → 通常の参照として処理
  - `MirInstruction::WeakLoad` → 常にSome相当で成功
  
- [ ] **Task 4.2**: Barrier命令ダミー実装
  - `MirInstruction::BarrierRead/Write` → no-op

- [ ] **Task 4.3**: Future基本実装
  - `MirInstruction::FutureNew` → 即座に完了状態のFuture
  - `MirInstruction::Await` → 値をそのまま返す

### Phase 8.5: CLI統合 (CLI Integration)
- [ ] **Task 5.1**: CLI実装
  - `--backend wasm` オプション追加
  - `--output file.wasm` オプション追加
  - wasmtimeとの連携（`wasmtime run`）
  
- [ ] **Task 5.2**: エラーハンドリング
  - 未対応MIR命令の明確なエラーメッセージ
  - WASM生成失敗時の診断情報

## Acceptance Criteria

### PoC1 (Basic Operations)
- ✅ **WASM Generation**: 基本MIR命令がvalid WASMに変換される
- ✅ **Wasmtime Execution**: `wasmtime run output.wasm` で正常実行
- ✅ **Arithmetic**: `print(42 + 8)` → stdout: `50`
- ✅ **Control Flow**: if文、loop文が正しく動作

### PoC2 (Object Operations)  
- ✅ **Memory Allocation**: RefNew でヒープメモリが正しく割り当てられる
- ✅ **Field Access**: `o = new DataBox(); o.value = 1; print(o.value)` → stdout: `1`
- ✅ **Memory Layout**: Box構造がメモリ上で正しいレイアウトになる

### PoC3 (Extension Foundation)
- ✅ **Weak Reference**: WeakNew/WeakLoad命令がno-opとして動作
- ✅ **Memory Barriers**: BarrierRead/Write命令が含まれても実行できる
- ✅ **Future Operations**: FutureNew/Await が即座完了として動作

### CLI Integration
- ✅ **Command Line**: `nyash --backend wasm test.nyash` で実行可能
- ✅ **File Output**: `nyash --backend wasm --output test.wasm test.nyash` でファイル出力
- ✅ **Error Messages**: 未対応機能の明確なエラーメッセージ

## Test Strategy

### Unit Tests (Rust)
```rust
// tests/wasm_codegen_tests.rs
#[test]
fn test_basic_arithmetic_codegen() {
    let mir = /* 42 + 8 のMIR */;
    let wasm_bytes = WasmBackend::new().compile_module(mir).unwrap();
    let result = wasmtime_execute(&wasm_bytes);
    assert_eq!(result.stdout, "50\n");
}

#[test] 
fn test_ref_operations_codegen() {
    let mir = /* object field access のMIR */;
    let wasm_bytes = WasmBackend::new().compile_module(mir).unwrap();
    let result = wasmtime_execute(&wasm_bytes);
    assert_eq!(result.stdout, "1\n");
}
```

### Integration Tests
- `tests/wasm_poc1_arithmetic.nyash` → MIR → WASM → wasmtime実行
- `tests/wasm_poc2_objects.nyash` → RefNew/RefGet/RefSet使用 → WASM実行
- `tests/wasm_poc3_features.nyash` → Weak/Future命令含む → WASM実行

### Browser Testing
```html
<!-- tests/browser_test.html -->
<script type="module">
const importObject = {
  env: { 
    print: (value) => console.log(value) 
  }
};
const wasmModule = await WebAssembly.instantiateStreaming(
  fetch('./test.wasm'), importObject
);
const result = wasmModule.instance.exports.main();
console.log('Result:', result);
</script>
```

## Technical Dependencies

### Required Crates
- `wabt` - WAT ↔ WASM conversion
- `wasmtime` - Runtime execution (dev dependency)
- `wat` - WAT text format parsing (optional)

### WASM Tools
- `wasmtime` CLI - Local execution & testing
- `wasm-objdump` - Binary inspection (optional)
- `wasm-validate` - Validation (optional)

## Development Notes

### Memory Management Strategy
1. **Phase 8.3**: Simple bump allocator (no free)
2. **Future**: Free list allocator  
3. **Future**: Generational GC integration

### Type System Mapping
| Nyash Type | MIR Type | WASM Type | Memory Layout |
|-----------|----------|-----------|---------------|
| IntegerBox | Integer | i32 | 4 bytes |
| BoolBox | Bool | i32 | 4 bytes (0/1) |
| DataBox | Box("DataBox") | i32 | ptr to [type_id, field_count, fields...] |

### Debugging Support
- WAT output for human inspection
- Source map generation (future)
- WASM stack trace integration (future)

## Out of Scope (Phase 8)
- 本格的なGC（mark-sweep、generational等）
- Weak参照の実際の無効化メカニズム
- Pin/Unpin、fini()のカスケード処理
- JIT/AOTコンパイル最適化
- 複雑な文字列ABI（UTF-8、length prefixed等）
- WASI I/O インターフェース（file、network等）

## References & Dependencies  
- **Phase 6**: RefNew/RefGet/RefSet MIR命令 (実装済み)
- **Phase 7**: FutureNew/Await MIR命令 (実装済み)
- docs/予定/native-plan/README.md（Phase 8詳細）
- docs/説明書/wasm/* (WASM関連ドキュメント)
- [WebAssembly Specification](https://webassembly.github.io/spec/)
