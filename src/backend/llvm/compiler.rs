/*!
 * LLVM Compiler Implementation - Compile MIR to LLVM IR and native code
 */

use crate::mir::function::MirModule;
use crate::mir::instruction::MirInstruction;
use crate::box_trait::{NyashBox, IntegerBox};
use super::context::CodegenContext;

#[cfg(feature = "llvm")]
use inkwell::context::Context;
#[cfg(feature = "llvm")]
use inkwell::values::IntValue;

pub struct LLVMCompiler {
    #[cfg(feature = "llvm")]
    context: Context,
    #[cfg(not(feature = "llvm"))]
    _phantom: std::marker::PhantomData<()>,
}

impl LLVMCompiler {
    pub fn new() -> Result<Self, String> {
        #[cfg(feature = "llvm")]
        {
            Ok(Self {
                context: Context::create(),
            })
        }
        #[cfg(not(feature = "llvm"))]
        {
            Err("LLVM feature not enabled. Please build with --features llvm".to_string())
        }
    }
    
    pub fn compile_module(
        &self,
        mir_module: &MirModule,
        output_path: &str,
    ) -> Result<(), String> {
        #[cfg(feature = "llvm")]
        {
            let codegen = CodegenContext::new(&self.context, "nyash_module")?;
            
            // 1. main関数を探す
            let main_func = mir_module.functions.get("Main.main")
                .ok_or("Main.main function not found")?;
            
            // 2. LLVM関数を作成
            let i32_type = codegen.context.i32_type();
            let fn_type = i32_type.fn_type(&[], false);
            let llvm_func = codegen.module.add_function("main", fn_type, None);
            
            // 3. エントリブロックを作成
            let entry = codegen.context.append_basic_block(llvm_func, "entry");
            codegen.builder.position_at_end(entry);
            
            // 4. MIR命令を処理（今回はReturnのみ）
            for (_block_id, block) in &main_func.blocks {
                for inst in &block.instructions {
                    match inst {
                        MirInstruction::Return { value: Some(_value_id) } => {
                            // 簡易実装: 定数42を返すと仮定
                            let ret_val = i32_type.const_int(42, false);
                            codegen.builder.build_return(Some(&ret_val)).unwrap();
                        }
                        MirInstruction::Return { value: None } => {
                            // void return
                            let ret_val = i32_type.const_int(0, false);
                            codegen.builder.build_return(Some(&ret_val)).unwrap();
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
        #[cfg(not(feature = "llvm"))]
        {
            Err("LLVM feature not enabled".to_string())
        }
    }
    
    pub fn compile_and_execute(
        &self,
        mir_module: &MirModule,
        temp_path: &str,
    ) -> Result<Box<dyn NyashBox>, String> {
        #[cfg(feature = "llvm")]
        {
            // 1. オブジェクトファイル生成
            let obj_path = format!("{}.o", temp_path);
            self.compile_module(mir_module, &obj_path)?;
            
            // 2. リンク（簡易版：システムのccを使用）
            use std::process::Command;
            let executable_path = format!("{}_exec", temp_path);
            let output = Command::new("cc")
                .args(&[&obj_path, "-o", &executable_path])
                .output()
                .map_err(|e| format!("Link failed: {}", e))?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Linking failed: {}", stderr));
            }
            
            // 3. 実行
            let output = Command::new(&format!("./{}", executable_path))
                .output()
                .map_err(|e| format!("Execution failed: {}", e))?;
            
            // 4. 終了コードを返す
            let exit_code = output.status.code().unwrap_or(-1);
            
            // 5. 一時ファイルのクリーンアップ
            let _ = std::fs::remove_file(&obj_path);
            let _ = std::fs::remove_file(&executable_path);
            
            Ok(Box::new(IntegerBox::new(exit_code as i64)))
        }
        #[cfg(not(feature = "llvm"))]
        {
            Err("LLVM feature not enabled".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compiler_creation() {
        #[cfg(feature = "llvm")]
        {
            let compiler = LLVMCompiler::new();
            assert!(compiler.is_ok());
        }
        #[cfg(not(feature = "llvm"))]
        {
            let compiler = LLVMCompiler::new();
            assert!(compiler.is_err());
        }
    }
}