use inkwell::context::Context;
use inkwell::{
    types::BasicTypeEnum,
    values::{BasicValueEnum, FloatValue, IntValue, PhiValue, PointerValue},
    AddressSpace,
};

use crate::mir;

use super::super::helpers::{as_float, as_int};

// Helper conversions and classifiers extracted from the monolithic codegen.
// Kept signature-compatible to minimize churn in compile_module.

pub(super) fn to_i64_any<'ctx>(
    ctx: &'ctx Context,
    builder: &inkwell::builder::Builder<'ctx>,
    v: BasicValueEnum<'ctx>,
) -> Result<IntValue<'ctx>, String> {
    let i64t = ctx.i64_type();
    Ok(match v {
        BasicValueEnum::IntValue(iv) => {
            if iv.get_type().get_bit_width() == 64 {
                iv
            } else if iv.get_type().get_bit_width() < 64 {
                builder
                    .build_int_z_extend(iv, i64t, "zext_i64")
                    .map_err(|e| e.to_string())?
            } else {
                builder
                    .build_int_truncate(iv, i64t, "trunc_i64")
                    .map_err(|e| e.to_string())?
            }
        }
        BasicValueEnum::PointerValue(pv) => builder
            .build_ptr_to_int(pv, i64t, "p2i64")
            .map_err(|e| e.to_string())?,
        BasicValueEnum::FloatValue(fv) => {
            // Bitcast f64 -> i64 via stack slot
            let slot_builder = builder
                .get_insert_block()
                .and_then(|bb| bb.get_parent())
                .and_then(|f| f.get_first_basic_block())
                .map(|entry| {
                    let eb = ctx.create_builder();
                    eb.position_at_end(entry);
                    eb
                })
                .unwrap_or_else(|| ctx.create_builder());
            let tmp = slot_builder
                .build_alloca(i64t, "f2i_tmp")
                .map_err(|e| e.to_string())?;
            let fptr_ty = ctx.ptr_type(AddressSpace::from(0));
            let castp = builder
                .build_pointer_cast(tmp, fptr_ty, "i64p_to_f64p")
                .map_err(|e| e.to_string())?;
            builder.build_store(castp, fv).map_err(|e| e.to_string())?;
            builder
                .build_load(i64t, tmp, "ld_f2i")
                .map_err(|e| e.to_string())?
                .into_int_value()
        }
        _ => return Err("unsupported value for i64 conversion".to_string()),
    })
}

pub(super) fn i64_to_ptr<'ctx>(
    ctx: &'ctx Context,
    builder: &inkwell::builder::Builder<'ctx>,
    iv: IntValue<'ctx>,
) -> Result<PointerValue<'ctx>, String> {
    let pty = ctx.ptr_type(AddressSpace::from(0));
    builder
        .build_int_to_ptr(iv, pty, "i64_to_ptr")
        .map_err(|e| e.to_string())
}

pub(super) fn classify_tag<'ctx>(v: BasicValueEnum<'ctx>) -> i64 {
    match v {
        BasicValueEnum::FloatValue(_) => 5,   // float
        BasicValueEnum::PointerValue(_) => 8, // handle/ptr
        BasicValueEnum::IntValue(_) => 3,     // integer/bool
        _ => 3,
    }
}

pub(super) fn to_bool<'ctx>(
    ctx: &'ctx Context,
    b: BasicValueEnum<'ctx>,
    builder: &inkwell::builder::Builder<'ctx>,
) -> Result<IntValue<'ctx>, String> {
    if let Some(bb) = as_int(b) {
        // If not i1, compare != 0
        if bb.get_type().get_bit_width() == 1 {
            Ok(bb)
        } else {
            Ok(builder
                .build_int_compare(
                    inkwell::IntPredicate::NE,
                    bb,
                    bb.get_type().const_zero(),
                    "tobool",
                )
                .map_err(|e| e.to_string())?)
        }
    } else if let Some(fv) = as_float(b) {
        let zero = fv.get_type().const_float(0.0);
        Ok(builder
            .build_float_compare(inkwell::FloatPredicate::ONE, fv, zero, "toboolf")
            .map_err(|e| e.to_string())?)
    } else if let BasicValueEnum::PointerValue(pv) = b {
        let i64t = ctx.i64_type();
        let p2i = builder
            .build_ptr_to_int(pv, i64t, "p2i")
            .map_err(|e| e.to_string())?;
        Ok(builder
            .build_int_compare(inkwell::IntPredicate::NE, p2i, i64t.const_zero(), "toboolp")
            .map_err(|e| e.to_string())?)
    } else {
        Err("Unsupported value for boolean conversion".to_string())
    }
}

pub(super) fn cmp_eq_ne_any<'ctx>(
    ctx: &'ctx Context,
    builder: &inkwell::builder::Builder<'ctx>,
    op: &crate::mir::CompareOp,
    lv: BasicValueEnum<'ctx>,
    rv: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    use crate::mir::CompareOp as C;
    match (lv, rv) {
        (BasicValueEnum::IntValue(li), BasicValueEnum::IntValue(ri)) => {
            let pred = if matches!(op, C::Eq) {
                inkwell::IntPredicate::EQ
            } else {
                inkwell::IntPredicate::NE
            };
            Ok(builder
                .build_int_compare(pred, li, ri, "icmp")
                .map_err(|e| e.to_string())?
                .into())
        }
        (BasicValueEnum::FloatValue(lf), BasicValueEnum::FloatValue(rf)) => {
            let pred = if matches!(op, C::Eq) {
                inkwell::FloatPredicate::OEQ
            } else {
                inkwell::FloatPredicate::ONE
            };
            Ok(builder
                .build_float_compare(pred, lf, rf, "fcmp")
                .map_err(|e| e.to_string())?
                .into())
        }
        (BasicValueEnum::PointerValue(_), _) | (_, BasicValueEnum::PointerValue(_)) => {
            let li = to_i64_any(ctx, builder, lv)?;
            let ri = to_i64_any(ctx, builder, rv)?;
            let pred = if matches!(op, C::Eq) {
                inkwell::IntPredicate::EQ
            } else {
                inkwell::IntPredicate::NE
            };
            Ok(builder
                .build_int_compare(pred, li, ri, "icmp_any")
                .map_err(|e| e.to_string())?
                .into())
        }
        _ => Err("unsupported compare types".to_string()),
    }
}

pub(super) fn map_mirtype_to_basic<'ctx>(
    ctx: &'ctx Context,
    t: &mir::MirType,
) -> BasicTypeEnum<'ctx> {
    match t {
        mir::MirType::Integer => ctx.i64_type().into(),
        mir::MirType::Bool => ctx.bool_type().into(),
        mir::MirType::Float => ctx.f64_type().into(),
        mir::MirType::String => ctx.ptr_type(AddressSpace::from(0)).into(),
        mir::MirType::Box(_)
        | mir::MirType::Array(_)
        | mir::MirType::Future(_)
        | mir::MirType::Unknown => ctx.ptr_type(AddressSpace::from(0)).into(),
        mir::MirType::Void => ctx.i64_type().into(), // avoid void as a value type; default to i64
    }
}
