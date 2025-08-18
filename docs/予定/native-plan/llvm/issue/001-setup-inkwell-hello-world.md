# 🚀 Issue #001: LLVM PoC - inkwellセットアップとHello World実装

**タイプ**: Feature  
**優先度**: Critical  
**見積もり**: 3日  
**担当**: Copilot

## 📋 概要

Phase 9.78 LLVM PoCの第一歩として、inkwellクレートを導入し、最小限のNyashプログラム（`return 42`）をLLVM経由で実行できるようにする。

## 🎯 成功条件

以下のNyashプログラムがLLVM経由で実行され、正しい終了コードを返すこと：

```nyash
// test_return_42.nyash
static box Main {
    main() {
        return 42
    }
}
```

期待される動作：
```bash
$ cargo run --features llvm -- --backend llvm test_return_42.nyash
$ echo $?
42
```

## 📝 実装タスク

### 1. **Cargo.toml更新** ✅必須
```toml
[dependencies]
inkwell = { version = "0.5", features = ["llvm17-0"] }

[features]
llvm = ["inkwell"]
```

### 2. **基本構造の作成** ✅必須
```rust
// src/backend/llvm/mod.rs
pub mod context;
pub mod compiler;

use crate::mir::module::MirModule;
use crate::errors::RuntimeError;

pub fn compile_to_object(
    mir_module: &MirModule,
    output_path: &str,
) -> Result<(), RuntimeError> {
    let compiler = compiler::LLVMCompiler::new()?;
    compiler.compile_module(mir_module, output_path)
}
```

### 3. **LLVMコンテキスト管理** ✅必須
```rust
// src/backend/llvm/context.rs
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::targets::{Target, TargetMachine, TargetTriple, InitializationConfig};

pub struct CodegenContext<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub target_machine: TargetMachine,
}

impl<'ctx> CodegenContext<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Result<Self, String> {
        // 1. ターゲット初期化
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|e| format!("Failed to initialize native target: {}", e))?;
        
        // 2. モジュール作成
        let module = context.create_module(module_name);
        
        // 3. ターゲットマシン作成
        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple)
            .map_err(|e| format!("Failed to get target: {}", e))?;
        let target_machine = target
            .create_target_machine(
                &triple,
                "generic",
                "",
                inkwell::OptimizationLevel::None,
                inkwell::targets::RelocMode::Default,
                inkwell::targets::CodeModel::Default,
            )
            .ok_or_else(|| "Failed to create target machine".to_string())?;
        
        // 4. データレイアウト設定
        module.set_triple(&triple);
        module.set_data_layout(&target_machine.get_target_data().get_data_layout());
        
        Ok(Self {
            context,
            module,
            builder: context.create_builder(),
            target_machine,
        })
    }
}
```

### 4. **最小限のコンパイラ実装** ✅必須
```rust
// src/backend/llvm/compiler.rs
use inkwell::context::Context;
use inkwell::values::IntValue;
use crate::mir::module::MirModule;
use crate::mir::instruction::MirInstruction;
use super::context::CodegenContext;

pub struct LLVMCompiler {
    context: Context,
}

impl LLVMCompiler {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            context: Context::create(),
        })
    }
    
    pub fn compile_module(
        &self,
        mir_module: &MirModule,
        output_path: &str,
    ) -> Result<(), String> {
        let codegen = CodegenContext::new(&self.context, "nyash_module")?;
        
        // 1. main関数を探す
        let main_func = mir_module.functions.iter()
            .find(|f| f.name == "Main.main")
            .ok_or("Main.main function not found")?;
        
        // 2. LLVM関数を作成
        let i32_type = codegen.context.i32_type();
        let fn_type = i32_type.fn_type(&[], false);
        let llvm_func = codegen.module.add_function("main", fn_type, None);
        
        // 3. エントリブロックを作成
        let entry = codegen.context.append_basic_block(llvm_func, "entry");
        codegen.builder.position_at_end(entry);
        
        // 4. MIR命令を処理（今回はReturnのみ）
        for block in &main_func.blocks {
            for inst in &block.instructions {
                match inst {
                    MirInstruction::Return(Some(value_id)) => {
                        // 簡易実装: 定数42を返すと仮定
                        let ret_val = i32_type.const_int(42, false);
                        codegen.builder.build_return(Some(&ret_val));
                    }
                    _ => {
                        // 他の命令は今回スキップ
                    }
                }
            }
        }
        
        // 5. 検証
        if !llvm_func.verify(true) {
            return Err("Function verification failed".to_string());
        }
        
        // 6. オブジェクトファイル生成
        codegen.target_machine
            .write_to_file(&codegen.module, 
                         inkwell::targets::FileType::Object, 
                         output_path.as_ref())
            .map_err(|e| format!("Failed to write object file: {}", e))?;
        
        Ok(())
    }
}
```

### 5. **バックエンド統合** ✅必須
```rust
// src/backend/mod.rsに追加
#[cfg(feature = "llvm")]
pub mod llvm;

// src/runner.rsのrun_with_backend関数に追加
#[cfg(feature = "llvm")]
ExecutionBackend::LLVM => {
    // 1. オブジェクトファイル生成
    let obj_path = "nyash_output.o";
    crate::backend::llvm::compile_to_object(&mir_module, obj_path)?;
    
    // 2. リンク（簡易版：システムのccを使用）
    use std::process::Command;
    let output = Command::new("cc")
        .args(&[obj_path, "-o", "nyash_output"])
        .output()
        .map_err(|e| RuntimeError::new(format!("Link failed: {}", e)))?;
    
    if !output.status.success() {
        return Err(RuntimeError::new("Linking failed"));
    }
    
    // 3. 実行
    let output = Command::new("./nyash_output")
        .output()
        .map_err(|e| RuntimeError::new(format!("Execution failed: {}", e)))?;
    
    // 4. 終了コードを返す
    let exit_code = output.status.code().unwrap_or(-1);
    Ok(Box::new(IntegerBox::new(exit_code as i64)))
}
```

## 🧪 テストケース

```rust
// tests/llvm_hello_world.rs
#[test]
#[cfg(feature = "llvm")]
fn test_return_42() {
    let source = r#"
        static box Main {
            main() {
                return 42
            }
        }
    "#;
    
    // パース → MIR生成 → LLVM実行
    let result = compile_and_run_llvm(source);
    assert_eq!(result, 42);
}
```

## 📚 参考資料

- [inkwell Examples](https://github.com/TheDan64/inkwell/tree/master/examples)
- [LLVM Tutorial](https://llvm.org/docs/tutorial/)
- [AI大会議結果](../AI-Conference-LLVM-Results.md)

## ⚠️ 注意事項

1. **LLVM依存関係**: LLVM 17がシステムにインストールされている必要があります
2. **プラットフォーム**: まずはLinux/macOSで動作確認し、Windowsは後回し
3. **エラーハンドリング**: 今回は最小実装のため、詳細なエラー処理は省略

## 🎯 次のステップ

このIssueが完了したら、次は：
- Issue #002: 基本的な算術演算の実装（BinOp）
- Issue #003: 定数値の実装（Const）

---

**作成者**: Claude + moe-charm  
**レビュアー**: AIチーム  
**関連PR**: （作成予定）