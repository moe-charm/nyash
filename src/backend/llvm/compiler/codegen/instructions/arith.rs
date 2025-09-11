use std::collections::HashMap;

use inkwell::values::BasicValueEnum;

use crate::backend::llvm::context::CodegenContext;
use crate::mir::{CompareOp, ValueId};

/// Compare lowering: return the resulting BasicValueEnum (i1)
pub(in super::super) fn lower_compare<'ctx>(
    codegen: &CodegenContext<'ctx>,
    vmap: &HashMap<ValueId, BasicValueEnum<'ctx>>,
    op: &CompareOp,
    lhs: &ValueId,
    rhs: &ValueId,
) -> Result<BasicValueEnum<'ctx>, String> {
    use crate::backend::llvm::compiler::helpers::{as_float, as_int};
    let lv = *vmap.get(lhs).ok_or("lhs missing")?;
    let rv = *vmap.get(rhs).ok_or("rhs missing")?;
    let out = if let (Some(li), Some(ri)) = (as_int(lv), as_int(rv)) {
        use CompareOp as C;
        let pred = match op {
            C::Eq => inkwell::IntPredicate::EQ,
            C::Ne => inkwell::IntPredicate::NE,
            C::Lt => inkwell::IntPredicate::SLT,
            C::Le => inkwell::IntPredicate::SLE,
            C::Gt => inkwell::IntPredicate::SGT,
            C::Ge => inkwell::IntPredicate::SGE,
        };
        codegen
            .builder
            .build_int_compare(pred, li, ri, "icmp")
            .map_err(|e| e.to_string())?
            .into()
    } else if let (Some(lf), Some(rf)) = (as_float(lv), as_float(rv)) {
        use CompareOp as C;
        let pred = match op {
            C::Eq => inkwell::FloatPredicate::OEQ,
            C::Ne => inkwell::FloatPredicate::ONE,
            C::Lt => inkwell::FloatPredicate::OLT,
            C::Le => inkwell::FloatPredicate::OLE,
            C::Gt => inkwell::FloatPredicate::OGT,
            C::Ge => inkwell::FloatPredicate::OGE,
        };
        codegen
            .builder
            .build_float_compare(pred, lf, rf, "fcmp")
            .map_err(|e| e.to_string())?
            .into()
    } else if let (BasicValueEnum::PointerValue(lp), BasicValueEnum::PointerValue(rp)) = (lv, rv) {
        // Support pointer equality/inequality comparisons
        use CompareOp as C;
        match op {
            C::Eq | C::Ne => {
                let i64t = codegen.context.i64_type();
                let li = codegen
                    .builder
                    .build_ptr_to_int(lp, i64t, "pi_l")
                    .map_err(|e| e.to_string())?;
                let ri = codegen
                    .builder
                    .build_ptr_to_int(rp, i64t, "pi_r")
                    .map_err(|e| e.to_string())?;
                let pred = if matches!(op, C::Eq) {
                    inkwell::IntPredicate::EQ
                } else {
                    inkwell::IntPredicate::NE
                };
                codegen
                    .builder
                    .build_int_compare(pred, li, ri, "pcmp")
                    .map_err(|e| e.to_string())?
                    .into()
            }
            _ => return Err("unsupported pointer comparison (only Eq/Ne)".to_string()),
        }
    } else if let (BasicValueEnum::PointerValue(lp), BasicValueEnum::IntValue(ri)) = (lv, rv) {
        use CompareOp as C;
        match op {
            C::Eq | C::Ne => {
                let i64t = codegen.context.i64_type();
                let li = codegen
                    .builder
                    .build_ptr_to_int(lp, i64t, "pi_l")
                    .map_err(|e| e.to_string())?;
                let pred = if matches!(op, C::Eq) {
                    inkwell::IntPredicate::EQ
                } else {
                    inkwell::IntPredicate::NE
                };
                codegen
                    .builder
                    .build_int_compare(pred, li, ri, "pcmpi")
                    .map_err(|e| e.to_string())?
                    .into()
            }
            _ => return Err("unsupported pointer-int comparison (only Eq/Ne)".to_string()),
        }
    } else if let (BasicValueEnum::IntValue(li), BasicValueEnum::PointerValue(rp)) = (lv, rv) {
        use CompareOp as C;
        match op {
            C::Eq | C::Ne => {
                let i64t = codegen.context.i64_type();
                let ri = codegen
                    .builder
                    .build_ptr_to_int(rp, i64t, "pi_r")
                    .map_err(|e| e.to_string())?;
                let pred = if matches!(op, C::Eq) {
                    inkwell::IntPredicate::EQ
                } else {
                    inkwell::IntPredicate::NE
                };
                codegen
                    .builder
                    .build_int_compare(pred, li, ri, "pcmpi")
                    .map_err(|e| e.to_string())?
                    .into()
            }
            _ => return Err("unsupported int-pointer comparison (only Eq/Ne)".to_string()),
        }
    } else {
        return Err("compare type mismatch".to_string());
    };
    Ok(out)
}
