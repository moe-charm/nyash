use std::collections::HashMap;

use inkwell::values::{BasicValueEnum as BVE, FunctionValue};

use crate::backend::llvm::context::CodegenContext;
use crate::mir::{function::MirFunction, ValueId};

/// Lower a direct Call where callee is provided as a const string ValueId in MIR14.
///
/// Requirements:
/// - `const_strs`: mapping from ValueId to the string literal value within the same function.
/// - `llvm_funcs`: predeclared LLVM functions keyed by MIR function name (same string as const).
pub(in super::super) fn lower_call<'ctx>(
    codegen: &CodegenContext<'ctx>,
    _func: &MirFunction,
    vmap: &mut HashMap<ValueId, BVE<'ctx>>,
    dst: &Option<ValueId>,
    callee: &ValueId,
    args: &[ValueId],
    const_strs: &HashMap<ValueId, String>,
    llvm_funcs: &HashMap<String, FunctionValue<'ctx>>,
) -> Result<(), String> {
    let name_s = const_strs
        .get(callee)
        .ok_or_else(|| format!("call: callee value {} not a const string", callee.as_u32()))?;
    let target = llvm_funcs
        .get(name_s)
        .ok_or_else(|| format!("call: function not predeclared: {}", name_s))?;

    // Collect args in order
    let mut avs: Vec<BVE<'ctx>> = Vec::with_capacity(args.len());
    for a in args {
        let v = *vmap
            .get(a)
            .ok_or_else(|| format!("call arg missing: {}", a.as_u32()))?;
        avs.push(v);
    }
    let params: Vec<inkwell::values::BasicMetadataValueEnum> =
        avs.iter().map(|v| (*v).into()).collect();
    let call = codegen
        .builder
        .build_call(*target, &params, "call")
        .map_err(|e| e.to_string())?;
    if let Some(d) = dst {
        if let Some(rv) = call.try_as_basic_value().left() {
            vmap.insert(*d, rv);
        }
    }
    Ok(())
}

