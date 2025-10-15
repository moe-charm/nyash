# WASMバックエンド完全実装計画

## 📋 概要

**発見日**: 2025-09-30
**優先度**: 🟢 低（Phase 18で実装予定）
**影響範囲**: WASMバックエンド・ブラウザ実行

## 🎯 問題

WASMバックエンドが部分的実装で停止中：

### 該当箇所（5個）

#### 1. `src/backend/wasm/mod.rs:11,16` - Executor無効化
```rust
// mod executor; // TODO: Fix WASM executor build errors
// ...
// pub use executor::WasmExecutor; // TODO: Fix WASM executor build errors
```

**問題**: コンパイルエラーによりexecutorモジュール無効化

#### 2. `src/backend/wasm/codegen.rs:289,307` - フィールドオフセット未実装（2箇所）
```rust
// TODO: Add proper field offset calculation
```

**問題**: FieldAccessでオフセット計算が未実装

#### 3. `src/backend/wasm/runtime.rs:294` - Import未実装
```rust
// TODO: Implement {} import
```

**問題**: 外部関数インポートが未完成

## 💡 解決策案

### Phase A: Executor修復（最重要）

#### 問題分析
```bash
# エラー確認
cargo check --features wasm 2>&1 | grep "error\["
```

#### 推定される問題
1. wasmer API変更（バージョン非互換）
2. 型不一致（WasmValue関連）
3. モジュールローディング失敗

#### 解決アプローチ
```rust
// src/backend/wasm/executor.rs（修正版）
use wasmer::{Store, Module, Instance, imports, Function};

pub struct WasmExecutor {
    store: Store,
    instance: Instance,
}

impl WasmExecutor {
    pub fn new(wasm_bytes: &[u8]) -> Result<Self, String> {
        let mut store = Store::default();
        let module = Module::new(&store, wasm_bytes)
            .map_err(|e| format!("Module load error: {}", e))?;

        // Import関数登録
        let import_object = imports! {
            "env" => {
                "print" => Function::new_typed(&mut store, |x: i32| {
                    println!("{}", x);
                }),
            }
        };

        let instance = Instance::new(&mut store, &module, &import_object)
            .map_err(|e| format!("Instance error: {}", e))?;

        Ok(Self { store, instance })
    }

    pub fn call_main(&mut self) -> Result<(), String> {
        let main = self.instance.exports.get_function("main")
            .map_err(|e| format!("main not found: {}", e))?;

        main.call(&mut self.store, &[])
            .map_err(|e| format!("Execution error: {}", e))?;

        Ok(())
    }
}
```

**実装時間**: 2-3時間

### Phase B: フィールドオフセット計算

#### 問題
```rust
ASTNode::FieldAccess { object, field, .. } => {
    let obj_idx = self.lower_expression(object)?;
    // TODO: Add proper field offset calculation
    // 現状: 固定オフセット0を使用
}
```

#### 解決策
```rust
pub struct FieldOffsetTable {
    // box_name -> field_name -> offset
    offsets: HashMap<String, HashMap<String, u32>>,
}

impl FieldOffsetTable {
    pub fn build_from_mir(mir: &MirProgram) -> Self {
        let mut table = Self::default();

        for box_decl in &mir.box_declarations {
            let mut offset = 0;
            for field in &box_decl.fields {
                table.set_offset(&box_decl.name, &field.name, offset);
                offset += field_size(&field.type_name); // 型ごとのサイズ
            }
        }

        table
    }

    pub fn get_offset(&self, box_name: &str, field: &str) -> Option<u32> {
        self.offsets.get(box_name)?.get(field).copied()
    }
}

// codegen.rsで使用
ASTNode::FieldAccess { object, field, .. } => {
    let obj_idx = self.lower_expression(object)?;
    let box_type = self.get_type(obj_idx)?; // 型推論必要

    let offset = self.field_table.get_offset(&box_type, field)
        .ok_or_else(|| format!("Field {} not found in {}", field, box_type))?;

    // i32.load offset=<offset>
    self.wasm.i32_load(MemArg {
        align: 4,
        offset: offset as u64,
        memory: 0,
    });
}
```

**実装時間**: 3-4時間

### Phase C: Runtime Import完全実装

#### 問題
```rust
// TODO: Implement {} import
```

#### 解決策
```rust
// runtime.rs
pub fn build_import_object(store: &mut Store) -> Imports {
    imports! {
        "env" => {
            // 基本I/O
            "print" => Function::new_typed(store, print_impl),
            "error" => Function::new_typed(store, error_impl),

            // Box操作
            "box_new" => Function::new_typed(store, box_new_impl),
            "box_call" => Function::new_typed(store, box_call_impl),

            // メモリ管理
            "alloc" => Function::new_typed(store, alloc_impl),
            "dealloc" => Function::new_typed(store, dealloc_impl),

            // 外部呼び出し
            "extern_call" => Function::new_typed(store, extern_call_impl),
        }
    }
}

fn print_impl(mut env: FunctionEnvMut<WasmEnv>, ptr: i32, len: i32) {
    let memory = env.data().memory.view(&env);
    let bytes = memory.read(ptr as u64, len as usize).unwrap();
    let s = String::from_utf8_lossy(&bytes);
    println!("{}", s);
}

// 他の実装も同様...
```

**実装時間**: 4-6時間

## 🚀 実装ステップ（推奨順）

### Step 1: Phase A（Executor修復） - 最優先 ✅
**時間**: 2-3時間
**理由**: 全体が動かない根本原因

### Step 2: Phase C（Runtime Import） - 重要 ⚠️
**時間**: 4-6時間
**理由**: 実行に必須機能

### Step 3: Phase B（フィールドオフセット） - 機能追加 🆕
**時間**: 3-4時間
**理由**: Box操作に必要

## 📊 影響範囲

### 修正必要ファイル
- `src/backend/wasm/executor.rs` - Executor修復
- `src/backend/wasm/codegen.rs` - フィールドオフセット実装
- `src/backend/wasm/runtime.rs` - Import完全実装
- `src/backend/wasm/mod.rs` - executor再有効化
- `Cargo.toml` - wasmerバージョン確認

### テスト追加
- `tests/wasm_executor_basic.rs` - 基本実行
- `tests/wasm_field_access.rs` - フィールドアクセス
- `tests/wasm_runtime_imports.rs` - Runtime Import
- スモークテスト: WASMバックエンド全機能

## 🎯 成功基準

- ✅ WASMバックエンドがビルド成功
- ✅ 基本プログラムがWASM実行可能
- ✅ フィールドアクセスが正しく動作
- ✅ Runtime Importすべて実装
- ✅ ブラウザでも実行可能

## 🔗 関連資料

- [Phase 18計画](../../../../development/roadmap/phases/phase-18/)
- [WASMバックエンド設計](../../../../reference/backends/wasm-design.md)
- [wasmerドキュメント](https://docs.rs/wasmer/)

## 📝 補足

**優先度判断**:
- Phase 15（セルフホスティング）では不要
- Phase 18（ブラウザ実行）で必須
- 現時点では**最低優先度**

**実装タイミング**: Phase 17完了後、Phase 18で実装推奨

**代替手段**: 現時点では以下で代用可能：
```bash
# VM/LLVMバックエンドを使用
./target/release/nyash --backend vm program.nyash
./target/release/nyash --backend llvm program.nyash
```

**注意**: WASMバックエンドは実験的機能。レガシーインタープリター削除により、
一部の機能が動作しない可能性あり。Phase 18で全面見直し予定。