# WASM実装の問題と改善計画

## 🚨 現状の問題

### 1. **2つのWASM実装が存在**
- **Rust→WASM**: `wasm-pack build`でNyashインタープリター全体をWASMに（動作する）
- **MIR→WASM**: `--compile-wasm`でNyashコードをWASMに変換（ほぼ動かない）

### 2. **MIR→WASM実装の問題点**
```rust
// src/backend/wasm/codegen.rs より
pub fn generate_module(...) -> Result<WasmModule, WasmError> {
    // 基本的な命令しか実装されていない
    // - 算術演算
    // - 制御フロー
    // - print文（ホスト関数呼び出し）
    
    // 未実装:
    // - Box操作（NewBox, BoxCall, PluginInvoke）
    // - 配列操作
    // - プラグインシステム
    // - GC/メモリ管理
}
```

### 3. **根本的な設計問題**
- **Box抽象の表現困難**: Everything is BoxをWASMの型システムで表現できない
- **動的ディスパッチ**: BoxCallやPluginInvokeの実装が困難
- **GCの不在**: WASMにはGCがない（WasmGC提案はまだ実験的）
- **プラグインFFI**: C ABIをWASM環境で実現できない

## 📊 現状の実装状況

### 実装済み（動作するもの）
```nyash
// 基本的な算術
function add(a, b) {
    return a + b
}

// 単純な制御フロー
function factorial(n) {
    if n <= 1 { return 1 }
    return n * factorial(n - 1)
}

// print文（ホスト関数経由）
print("Hello WASM")
```

### 未実装（動作しないもの）
```nyash
// Box操作
local str = new StringBox("hello")  // ❌ NewBox未実装
str.toUpperCase()                   // ❌ BoxCall未実装

// 配列
local arr = [1, 2, 3]              // ❌ 配列リテラル未実装
arr.push(4)                        // ❌ ArrayBox未実装

// プラグイン
local file = new FileBox()         // ❌ PluginInvoke未実装
```

## 🤔 なぜRust→WASMは動くのか

```toml
# Cargo.toml
[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
```

- **すべてのBox実装がそのままWASMに**: Arc<Mutex>も含めて
- **wasm-bindgenの魔法**: JavaScript↔Rust境界を自動生成
- **制限事項**: 一部のBox（TimerBox、FileBox等）は除外

## 🚀 改善案

### Option 1: MIR→WASM実装の完成（困難）
```wat
;; BoxをWASMテーブルで管理
(table $boxes 1000 externref)
(global $next_box_id (mut i32) (i32.const 0))

;; NewBox実装
(func $new_string_box (param $str i32) (result i32)
  ;; 新しいBox IDを割り当て
  (local $box_id i32)
  (local.set $box_id (global.get $next_box_id))
  
  ;; JavaScriptでStringBoxを作成
  (table.set $boxes
    (local.get $box_id)
    (call $js_create_string_box (local.get $str)))
  
  ;; IDを返す
  (local.get $box_id)
)
```

**問題点**:
- JavaScript側にBox実装が必要
- 性能オーバーヘッドが大きい
- プラグインシステムとの統合困難

### Option 2: Rust→WASMの活用（現実的）
```rust
// NyashコードをRustに変換してからWASMに
nyash_code → rust_code → wasm

// 例：
// Nyash: local s = new StringBox("hello")
// Rust:  let s = Box::new(StringBox::new("hello".to_string()));
// WASM:  (自動生成)
```

### Option 3: WASMランタイムの埋め込み（革新的）
```wat
;; 最小VMをWASMに埋め込む
(module
  ;; MIRバイトコードを格納
  (data (i32.const 0) "\01\02\03...")  
  
  ;; VMインタープリター
  (func $vm_execute
    ;; MIR命令をデコード・実行
  )
  
  ;; エントリーポイント
  (func (export "main")
    (call $vm_execute)
  )
)
```

## 🎯 推奨アプローチ

### Phase 1: 現状維持
- **Rust→WASM**: ブラウザでNyashを動かす用途で活用
- **MIR→WASM**: 実験的機能として残す

### Phase 2: Nyash→Rust変換
- NyashコードをRustに変換する仕組みを作る
- 生成されたRustコードをwasm-packでビルド

### Phase 3: WasmGC待ち
- WasmGC仕様が安定したら本格実装
- Box型システムをWasmGCで表現

## 📝 結論

現在のMIR→WASM実装は**実験的**なもので、実用レベルには達していません。一方、Rust→WASMは**すでに動作**しており、ブラウザでNyashを体験してもらうには十分です。

**当面は**：
1. Rust→WASMでプレイグラウンド提供
2. ネイティブ実行（VM/JIT/AOT）に注力
3. WasmGCの成熟を待つ

これが現実的な戦略です！