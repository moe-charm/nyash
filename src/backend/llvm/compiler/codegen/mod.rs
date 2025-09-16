//! LLVM Codegen Orchestrator
//!
//! Structure
//! - `function.rs`: per-function lowering (MIR → LLVM IR)
//! - `utils.rs`: helpers like `sanitize_symbol` and const-string maps
//! - `instructions/*`: focused lowerers and terminators (branch/jump/return, calls, boxcall, externcall)
//! - `types.rs`: MIR→LLVM type mapping and classifiers
//!
//! Keep this file slim: predeclare functions, delegate lowering, emit entry wrapper/object.
use super::helpers::map_type;
use super::LLVMCompiler;
pub(super) use crate::backend::llvm::context::CodegenContext;
use crate::mir::function::MirModule;
pub(super) use crate::mir::instruction::{ConstValue, MirInstruction, UnaryOp};
pub(super) use crate::mir::ValueId;
use inkwell::context::Context;
pub(super) use inkwell::{
    types::{BasicTypeEnum, FloatType, IntType, PointerType},
    values::{BasicValueEnum, FloatValue, FunctionValue, IntValue, PhiValue, PointerValue},
    AddressSpace,
};
use std::collections::HashMap;

mod types;
pub(super) use self::types::{
    classify_tag, cmp_eq_ne_any, i64_to_ptr, map_mirtype_to_basic, to_bool, to_i64_any,
};
mod function;
mod instructions;
mod object;
mod utils;

fn sanitize_symbol(name: &str) -> String {
    utils::sanitize_symbol(name)
}

impl LLVMCompiler {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            values: HashMap::new(),
        })
    }

    pub fn compile_module(&self, mir_module: &MirModule, output_path: &str) -> Result<(), String> {
        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
            eprintln!(
                "[LLVM] compile_module start: functions={}, out={}",
                mir_module.functions.len(),
                output_path
            );
        }
        let context = Context::create();
        let codegen = CodegenContext::new(&context, "nyash_module")?;
        let box_type_ids = crate::backend::llvm::box_types::load_box_type_ids();

        // Find entry function
        let (entry_name, _entry_func_ref) = if let Some((n, f)) = mir_module
            .functions
            .iter()
            .find(|(_n, f)| f.metadata.is_entry_point)
        {
            (n.clone(), f)
        } else if let Some(f) = mir_module.functions.get("Main.main") {
            ("Main.main".to_string(), f)
        } else if let Some(f) = mir_module.functions.get("main") {
            ("main".to_string(), f)
        } else if let Some((n, f)) = mir_module.functions.iter().next() {
            (n.clone(), f)
        } else {
            return Err("Main.main function not found in module".to_string());
        };

        // Predeclare all MIR functions as LLVM functions
        let mut llvm_funcs: HashMap<String, FunctionValue> = HashMap::new();
        for (name, f) in &mir_module.functions {
            let ret_bt = match f.signature.return_type {
                crate::mir::MirType::Void => codegen.context.i64_type().into(),
                ref t => map_type(codegen.context, t)?,
            };
            let mut params_bt: Vec<BasicTypeEnum> = Vec::new();
            for pt in &f.signature.params {
                params_bt.push(map_type(codegen.context, pt)?);
            }
            let param_vals: Vec<_> = params_bt.iter().map(|t| (*t).into()).collect();
            let ll_fn_ty = match ret_bt {
                BasicTypeEnum::IntType(t) => t.fn_type(&param_vals, false),
                BasicTypeEnum::FloatType(t) => t.fn_type(&param_vals, false),
                BasicTypeEnum::PointerType(t) => t.fn_type(&param_vals, false),
                _ => return Err("Unsupported return basic type".to_string()),
            };
            let sym = format!("ny_f_{}", sanitize_symbol(name));
            let lf = codegen.module.add_function(&sym, ll_fn_ty, None);
            llvm_funcs.insert(name.clone(), lf);
        }

        // Lower all functions
        for (name, func) in &mir_module.functions {
            let llvm_func = *llvm_funcs.get(name).ok_or("predecl not found")?;
            function::lower_one_function(
                &codegen,
                llvm_func,
                func,
                name,
                &box_type_ids,
                &llvm_funcs,
            )?;
        }

        // Build entry wrapper and emit object
        object::emit_wrapper_and_object(&codegen, &entry_name, output_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_compiler_creation() {
        let compiler = LLVMCompiler::new();
        assert!(compiler.is_ok());
    }
}
