use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, FloatValue, IntValue, PointerValue};
use inkwell::AddressSpace;

use crate::mir::MirType;

use crate::mir::CompareOp;
pub(crate) fn map_type<'ctx>(
    ctx: &'ctx Context,
    ty: &MirType,
) -> Result<BasicTypeEnum<'ctx>, String> {
    Ok(match ty {
        MirType::Integer => ctx.i64_type().into(),
        MirType::Float => ctx.f64_type().into(),
        MirType::Bool => ctx.bool_type().into(),
        MirType::String => ctx
            .i8_type()
            .ptr_type(inkwell::AddressSpace::from(0))
            .into(),
        MirType::Void => return Err("Void has no value type".to_string()),
        MirType::Box(_) => ctx
            .i8_type()
            .ptr_type(inkwell::AddressSpace::from(0))
            .into(),
        MirType::Array(_) | MirType::Future(_) | MirType::Unknown => ctx
            .i8_type()
            .ptr_type(inkwell::AddressSpace::from(0))
            .into(),
    })
}

pub(crate) fn as_int<'ctx>(v: BasicValueEnum<'ctx>) -> Option<IntValue<'ctx>> {
    if let BasicValueEnum::IntValue(iv) = v {
        Some(iv)
    } else {
        None
    }
}

pub(crate) fn as_float<'ctx>(v: BasicValueEnum<'ctx>) -> Option<FloatValue<'ctx>> {
    if let BasicValueEnum::FloatValue(fv) = v {
        Some(fv)
    } else {
        None
    }
}

pub(crate) fn to_i64_any<'ctx>(
    ctx: &'ctx Context,
    builder: &Builder<'ctx>,
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
            let slot = builder
                .get_insert_block()
                .and_then(|bb| bb.get_parent())
                .and_then(|f| f.get_first_basic_block())
                .map(|entry| {
                    let eb = ctx.create_builder();
                    eb.position_at_end(entry);
                    eb
                })
                .unwrap_or_else(|| ctx.create_builder());
            let i64p = i64t.ptr_type(AddressSpace::from(0));
            let tmp = slot
                .build_alloca(i64t, "f2i_tmp")
                .map_err(|e| e.to_string())?;
            let fptr_ty = ctx.f64_type().ptr_type(AddressSpace::from(0));
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

pub(crate) fn i64_to_ptr<'ctx>(
    ctx: &'ctx Context,
    builder: &Builder<'ctx>,
    iv: IntValue<'ctx>,
) -> Result<PointerValue<'ctx>, String> {
    let pty = ctx.i8_type().ptr_type(AddressSpace::from(0));
    builder
        .build_int_to_ptr(iv, pty, "i64_to_ptr")
        .map_err(|e| e.to_string())
}

pub(crate) fn classify_tag<'ctx>(v: BasicValueEnum<'ctx>) -> i64 {
    match v {
        BasicValueEnum::FloatValue(_) => 5,   // float
        BasicValueEnum::PointerValue(_) => 8, // handle/ptr
        BasicValueEnum::IntValue(_) => 3,     // integer/bool
        _ => 0,
    }
}

pub(crate) fn to_bool<'ctx>(
    ctx: &'ctx Context,
    b: BasicValueEnum<'ctx>,
    builder: &Builder<'ctx>,
) -> Result<IntValue<'ctx>, String> {
    if let Some(bb) = as_int(b) {
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

pub(crate) fn cmp_eq_ne_any<'ctx>(
    ctx: &'ctx Context,
    builder: &Builder<'ctx>,
    op: &CompareOp,
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
                .build_int_compare(pred, li, ri, "pcmp_any")
                .map_err(|e| e.to_string())?
                .into())
        }
        _ => Err("compare type mismatch".to_string()),
    }
}

pub(crate) fn map_mirtype_to_basic<'ctx>(ctx: &'ctx Context, ty: &MirType) -> BasicTypeEnum<'ctx> {
    match ty {
        MirType::Integer => ctx.i64_type().into(),
        MirType::Float => ctx.f64_type().into(),
        MirType::Bool => ctx.bool_type().into(),
        MirType::String => ctx.i8_type().ptr_type(AddressSpace::from(0)).into(),
        MirType::Box(_) | MirType::Array(_) | MirType::Future(_) | MirType::Unknown => {
            ctx.i8_type().ptr_type(AddressSpace::from(0)).into()
        }
        MirType::Void => ctx.i64_type().into(),
    }
}
