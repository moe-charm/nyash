use std::collections::HashMap;

use inkwell::values::BasicValueEnum as BVE;
use inkwell::AddressSpace;

use crate::backend::llvm::context::CodegenContext;
use crate::mir::{BasicBlockId, ValueId};
use crate::backend::llvm::compiler::codegen::instructions::builder_cursor::BuilderCursor;

pub(super) fn lower_log_or_trace<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    cur_bid: BasicBlockId,
    vmap: &mut HashMap<ValueId, BVE<'ctx>>,
    dst: &Option<ValueId>,
    iface_name: &str,
    method_name: &str,
    args: &[ValueId],
) -> Result<(), String> {
    if args.len() != 1 {
        return Err(format!("{}.{} expects 1 arg", iface_name, method_name));
    }
    let av = *vmap.get(&args[0]).ok_or("extern arg missing")?;
    match av {
        // If argument is i8* (string), call string variant
        BVE::PointerValue(pv) => {
            let i8p = codegen.context.ptr_type(AddressSpace::from(0));
            let fnty = codegen.context.i64_type().fn_type(&[i8p.into()], false);
            let fname = if iface_name == "env.console" {
                match method_name {
                    "log" => "nyash.console.log",
                    "warn" => "nyash.console.warn",
                    _ => "nyash.console.error",
                }
            } else {
                "nyash.debug.trace"
            };
            let callee = codegen
                .module
                .get_function(fname)
                .unwrap_or_else(|| codegen.module.add_function(fname, fnty, None));
            let _ = cursor
                .emit_instr(cur_bid, |b| b.build_call(callee, &[pv.into()], "console_log_p"))
                .map_err(|e| e.to_string())?;
            if let Some(d) = dst {
                vmap.insert(*d, codegen.context.i64_type().const_zero().into());
            }
            Ok(())
        }
        // Otherwise, convert to i64 and call handle variant
        _ => {
            let arg_val = match av {
                BVE::IntValue(iv) => {
                    if iv.get_type() == codegen.context.bool_type() {
                        cursor
                            .emit_instr(cur_bid, |b| b.build_int_z_extend(iv, codegen.context.i64_type(), "bool2i64"))
                            .map_err(|e| e.to_string())?
                    } else if iv.get_type() == codegen.context.i64_type() {
                        iv
                    } else {
                        cursor
                            .emit_instr(cur_bid, |b| b.build_int_s_extend(iv, codegen.context.i64_type(), "int2i64"))
                            .map_err(|e| e.to_string())?
                    }
                }
                BVE::PointerValue(_) => unreachable!(),
                _ => return Err("console.log arg conversion failed".to_string()),
            };
            let fnty = codegen
                .context
                .i64_type()
                .fn_type(&[codegen.context.i64_type().into()], false);
            let fname = if iface_name == "env.console" {
                match method_name {
                    "log" => "nyash.console.log_handle",
                    "warn" => "nyash.console.warn_handle",
                    _ => "nyash.console.error_handle",
                }
            } else {
                "nyash.debug.trace_handle"
            };
            let callee = codegen
                .module
                .get_function(fname)
                .unwrap_or_else(|| codegen.module.add_function(fname, fnty, None));
            let _ = cursor
                .emit_instr(cur_bid, |b| b.build_call(callee, &[arg_val.into()], "console_log_h"))
                .map_err(|e| e.to_string())?;
            if let Some(d) = dst {
                vmap.insert(*d, codegen.context.i64_type().const_zero().into());
            }
            Ok(())
        }
    }
}

pub(super) fn lower_readline<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    cur_bid: BasicBlockId,
    vmap: &mut HashMap<ValueId, BVE<'ctx>>,
    dst: &Option<ValueId>,
    args: &[ValueId],
) -> Result<(), String> {
    if !args.is_empty() {
        return Err("console.readLine expects 0 args".to_string());
    }
    let i8p = codegen.context.ptr_type(AddressSpace::from(0));
    let fnty = i8p.fn_type(&[], false);
    let callee = codegen
        .module
        .get_function("nyash.console.readline")
        .unwrap_or_else(|| codegen.module.add_function("nyash.console.readline", fnty, None));
    let call = cursor
        .emit_instr(cur_bid, |b| b.build_call(callee, &[], "readline"))
        .map_err(|e| e.to_string())?;
    if let Some(d) = dst {
        let rv = call
            .try_as_basic_value()
            .left()
            .ok_or("readline returned void".to_string())?;
        vmap.insert(*d, rv);
    }
    Ok(())
}
