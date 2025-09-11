use super::helpers::{as_float, as_int, map_type};
use super::LLVMCompiler;
use crate::backend::llvm::context::CodegenContext;
use crate::mir::function::MirModule;
use crate::mir::instruction::{ConstValue, MirInstruction, UnaryOp};
use crate::mir::ValueId;
use inkwell::context::Context;
use inkwell::{
    types::{BasicTypeEnum, FloatType, IntType, PointerType},
    values::{BasicValueEnum, FloatValue, FunctionValue, IntValue, PhiValue, PointerValue},
    AddressSpace,
};
use std::collections::HashMap;

// Submodules: helpers for type conversion/classification used by lowering
mod types;
use self::types::{
    classify_tag, cmp_eq_ne_any, i64_to_ptr, map_mirtype_to_basic, to_bool, to_i64_any,
};
mod instructions;

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
        // Lower only Main.main for now
        // Find entry function
        let func = if let Some((_n, f)) = mir_module
            .functions
            .iter()
            .find(|(_n, f)| f.metadata.is_entry_point)
        {
            f
        } else if let Some(f) = mir_module.functions.get("Main.main") {
            f
        } else if let Some(f) = mir_module.functions.get("main") {
            f
        } else if let Some((_n, f)) = mir_module.functions.iter().next() {
            f
        } else {
            return Err("Main.main function not found in module".to_string());
        };

        // Map MIR types to LLVM types via helpers

        // Load box type-id mapping from nyash_box.toml (central plugin registry)
        let box_type_ids = crate::backend::llvm::box_types::load_box_type_ids();

        // Function type
        let ret_type = match func.signature.return_type {
            crate::mir::MirType::Void => None,
            ref t => Some(map_type(codegen.context, t)?),
        };
        let fn_type = match ret_type {
            Some(BasicTypeEnum::IntType(t)) => t.fn_type(&[], false),
            Some(BasicTypeEnum::FloatType(t)) => t.fn_type(&[], false),
            Some(BasicTypeEnum::PointerType(t)) => t.fn_type(&[], false),
            Some(_) => return Err("Unsupported return basic type".to_string()),
            None => codegen.context.void_type().fn_type(&[], false),
        };
        let llvm_func = codegen.module.add_function("ny_main", fn_type, None);

        // Create LLVM basic blocks: ensure entry is created first to be function entry
        let (mut bb_map, entry_bb) = instructions::create_basic_blocks(&codegen, llvm_func, func);

        // Position at entry
        codegen.builder.position_at_end(entry_bb);

        // SSA value map
        let mut vmap: HashMap<ValueId, BasicValueEnum> = HashMap::new();

        // Helper ops are now provided by codegen/types.rs

        // Pre-create allocas for locals on demand (entry-only builder)
        let mut allocas: HashMap<ValueId, PointerValue> = HashMap::new();
        let entry_builder = codegen.context.create_builder();
        entry_builder.position_at_end(entry_bb);

        // Helper: map MirType to LLVM basic type (value type) is provided by types::map_mirtype_to_basic

        // Helper: create (or get) an alloca for a given pointer-typed SSA value id
        let mut alloca_elem_types: HashMap<ValueId, BasicTypeEnum> = HashMap::new();

        // Pre-create PHI nodes for all blocks (so we can add incoming from predecessors)
        let mut phis_by_block: HashMap<
            crate::mir::BasicBlockId,
            Vec<(ValueId, PhiValue, Vec<(crate::mir::BasicBlockId, ValueId)>)>,
        > = HashMap::new();
        for bid in func.block_ids() {
            let bb = *bb_map.get(&bid).ok_or("missing bb in map")?;
            // Position at start of the block (no instructions emitted yet)
            codegen.builder.position_at_end(bb);
            let block = func.blocks.get(&bid).unwrap();
            for inst in block
                .instructions
                .iter()
                .take_while(|i| matches!(i, MirInstruction::Phi { .. }))
            {
                if let MirInstruction::Phi { dst, inputs } = inst {
                    // Decide PHI type: prefer annotated value type; fallback to first input's annotated type; finally i64
                    let mut phi_ty: Option<BasicTypeEnum> = None;
                    if let Some(mt) = func.metadata.value_types.get(dst) {
                        phi_ty = Some(map_mirtype_to_basic(codegen.context, mt));
                    } else if let Some((_, iv)) = inputs.first() {
                        if let Some(mt) = func.metadata.value_types.get(iv) {
                            phi_ty = Some(map_mirtype_to_basic(codegen.context, mt));
                        }
                    }
                    let phi_ty = phi_ty.unwrap_or_else(|| codegen.context.i64_type().into());
                    let phi = codegen
                        .builder
                        .build_phi(phi_ty, &format!("phi_{}", dst.as_u32()))
                        .map_err(|e| e.to_string())?;
                    vmap.insert(*dst, phi.as_basic_value());
                    phis_by_block
                        .entry(bid)
                        .or_default()
                        .push((*dst, phi, inputs.clone()));
                }
            }
        }

        // Lower in block order
        for bid in func.block_ids() {
            let bb = *bb_map.get(&bid).unwrap();
            if codegen
                .builder
                .get_insert_block()
                .map(|b| b != bb)
                .unwrap_or(true)
            {
                codegen.builder.position_at_end(bb);
            }
            let block = func.blocks.get(&bid).unwrap();
            for inst in &block.instructions {
                match inst {
                    MirInstruction::NewBox {
                        dst,
                        box_type,
                        args,
                    } => {
                        match (box_type.as_str(), args.len()) {
                            ("StringBox", 1) => {
                                // Pass-through: if arg was built as Const String, its lowering produced i8* already
                                let av = *vmap.get(&args[0]).ok_or("StringBox arg missing")?;
                                vmap.insert(*dst, av);
                            }
                            ("IntegerBox", 1) => {
                                // Pass-through integer payload as i64
                                let av = *vmap.get(&args[0]).ok_or("IntegerBox arg missing")?;
                                vmap.insert(*dst, av);
                            }
                            // Minimal birth_i64 path for 1-2 args (i64 or handle-as-i64)
                            (_, n) if n == 1 || n == 2 => {
                                let type_id = *box_type_ids.get(box_type).unwrap_or(&0);
                                let i64t = codegen.context.i64_type();
                                let fnty = i64t.fn_type(
                                    &[i64t.into(), i64t.into(), i64t.into(), i64t.into()],
                                    false,
                                );
                                let callee = codegen
                                    .module
                                    .get_function("nyash.box.birth_i64")
                                    .unwrap_or_else(|| {
                                        codegen.module.add_function(
                                            "nyash.box.birth_i64",
                                            fnty,
                                            None,
                                        )
                                    });
                                // argc
                                let argc = i64t.const_int(args.len() as u64, false);
                                // a1/a2 as i64
                                let mut a1 = i64t.const_zero();
                                let mut a2 = i64t.const_zero();
                                if args.len() >= 1 {
                                    let v = *vmap.get(&args[0]).ok_or("newbox arg[0] missing")?;
                                    a1 = match v {
                                        BasicValueEnum::IntValue(iv) => iv,
                                        BasicValueEnum::PointerValue(pv) => codegen.builder.build_ptr_to_int(pv, i64t, "arg0_p2i").map_err(|e| e.to_string())?,
                                        _ => return Err("newbox arg[0]: unsupported type (expect int or handle ptr)".to_string()),
                                    };
                                }
                                if args.len() >= 2 {
                                    let v = *vmap.get(&args[1]).ok_or("newbox arg[1] missing")?;
                                    a2 = match v {
                                        BasicValueEnum::IntValue(iv) => iv,
                                        BasicValueEnum::PointerValue(pv) => codegen.builder.build_ptr_to_int(pv, i64t, "arg1_p2i").map_err(|e| e.to_string())?,
                                        _ => return Err("newbox arg[1]: unsupported type (expect int or handle ptr)".to_string()),
                                    };
                                }
                                let tid = i64t.const_int(type_id as u64, true);
                                let call = codegen
                                    .builder
                                    .build_call(
                                        callee,
                                        &[tid.into(), argc.into(), a1.into(), a2.into()],
                                        "birth_i64",
                                    )
                                    .map_err(|e| e.to_string())?;
                                let h = call
                                    .try_as_basic_value()
                                    .left()
                                    .ok_or("birth_i64 returned void".to_string())?
                                    .into_int_value();
                                let pty = codegen.context.ptr_type(AddressSpace::from(0));
                                let ptr = codegen
                                    .builder
                                    .build_int_to_ptr(h, pty, "handle_to_ptr")
                                    .map_err(|e| e.to_string())?;
                                vmap.insert(*dst, ptr.into());
                            }
                            _ => {
                                // No-arg birth via central type registry
                                if !args.is_empty() {
                                    return Err(
                                        "NewBox with >2 args not yet supported in LLVM lowering"
                                            .to_string(),
                                    );
                                }
                                let type_id = *box_type_ids.get(box_type).unwrap_or(&0);
                                let i64t = codegen.context.i64_type();
                                // declare i64 @nyash.box.birth_h(i64)
                                let fn_ty = i64t.fn_type(&[i64t.into()], false);
                                let callee = codegen
                                    .module
                                    .get_function("nyash.box.birth_h")
                                    .unwrap_or_else(|| {
                                        codegen.module.add_function(
                                            "nyash.box.birth_h",
                                            fn_ty,
                                            None,
                                        )
                                    });
                                let tid = i64t.const_int(type_id as u64, true);
                                let call = codegen
                                    .builder
                                    .build_call(callee, &[tid.into()], "birth")
                                    .map_err(|e| e.to_string())?;
                                // Handle is i64; represent Box as opaque i8* via inttoptr
                                let h_i64 = call
                                    .try_as_basic_value()
                                    .left()
                                    .ok_or("birth_h returned void".to_string())?
                                    .into_int_value();
                                let pty = codegen.context.ptr_type(AddressSpace::from(0));
                                let ptr = codegen
                                    .builder
                                    .build_int_to_ptr(h_i64, pty, "handle_to_ptr")
                                    .map_err(|e| e.to_string())?;
                                vmap.insert(*dst, ptr.into());
                            }
                        }
                    }
                    MirInstruction::Const { dst, value } => {
                        let bval = match value {
                            ConstValue::Integer(i) => {
                                codegen.context.i64_type().const_int(*i as u64, true).into()
                            }
                            ConstValue::Float(f) => {
                                codegen.context.f64_type().const_float(*f).into()
                            }
                            ConstValue::Bool(b) => codegen
                                .context
                                .bool_type()
                                .const_int(*b as u64, false)
                                .into(),
                            ConstValue::String(s) => {
                                let gv = codegen
                                    .builder
                                    .build_global_string_ptr(s, "str")
                                    .map_err(|e| e.to_string())?;
                                let len =
                                    codegen.context.i32_type().const_int(s.len() as u64, false);
                                // declare i8* @nyash_string_new(i8*, i32)
                                let rt = codegen
                                    .context
                                    .ptr_type(inkwell::AddressSpace::from(0));
                                let fn_ty = rt.fn_type(
                                    &[
                                        codegen
                                            .context
                                            .ptr_type(inkwell::AddressSpace::from(0))
                                            .into(),
                                        codegen.context.i32_type().into(),
                                    ],
                                    false,
                                );
                                let callee = codegen
                                    .module
                                    .get_function("nyash_string_new")
                                    .unwrap_or_else(|| {
                                        codegen.module.add_function("nyash_string_new", fn_ty, None)
                                    });
                                let call = codegen
                                    .builder
                                    .build_call(
                                        callee,
                                        &[gv.as_pointer_value().into(), len.into()],
                                        "strnew",
                                    )
                                    .map_err(|e| e.to_string())?;
                                call.try_as_basic_value()
                                    .left()
                                    .ok_or("nyash_string_new returned void".to_string())?
                            }
                            ConstValue::Null => codegen
                                .context
                                .ptr_type(inkwell::AddressSpace::from(0))
                                .const_zero()
                                .into(),
                            ConstValue::Void => return Err("Const Void unsupported".to_string()),
                        };
                        vmap.insert(*dst, bval);
                    }
                    MirInstruction::BoxCall {
                        dst,
                        box_val,
                        method,
                        method_id,
                        args,
                        effects: _,
                    } => {
                        let i64t = codegen.context.i64_type();
                        // Receiver handle (i64)
                        let recv_v = *vmap.get(box_val).ok_or("box receiver missing")?;
                        // Accept either an opaque pointer (i8*) or an i64 handle for the receiver
                        let recv_p = match recv_v {
                            BasicValueEnum::PointerValue(pv) => pv,
                            BasicValueEnum::IntValue(iv) => {
                                // Treat as Nyash handle and convert to opaque pointer
                                let pty = codegen.context.ptr_type(AddressSpace::from(0));
                                codegen
                                    .builder
                                    .build_int_to_ptr(iv, pty, "recv_i2p")
                                    .map_err(|e| e.to_string())?
                            }
                            _ => {
                                return Err("box receiver must be pointer or i64 handle".to_string())
                            }
                        };
                        let recv_h = codegen
                            .builder
                            .build_ptr_to_int(recv_p, i64t, "recv_p2i")
                            .map_err(|e| e.to_string())?;
                        // Resolve type_id from metadata (Box("Type")) using box_type_ids
                        let type_id: i64 = if let Some(crate::mir::MirType::Box(bname)) =
                            func.metadata.value_types.get(box_val)
                        {
                            *box_type_ids.get(bname).unwrap_or(&0)
                        } else if let Some(crate::mir::MirType::String) =
                            func.metadata.value_types.get(box_val)
                        {
                            *box_type_ids.get("StringBox").unwrap_or(&0)
                        } else {
                            0
                        };

                        // Special-case ArrayBox get/set/push/length until general by-id is widely annotated
                        if let Some(crate::mir::MirType::Box(bname)) =
                            func.metadata.value_types.get(box_val)
                        {
                            if bname == "ArrayBox"
                                && (method == "get"
                                    || method == "set"
                                    || method == "push"
                                    || method == "length")
                            {
                                match method.as_str() {
                                    "get" => {
                                        if args.len() != 1 {
                                            return Err("ArrayBox.get expects 1 arg".to_string());
                                        }
                                        let idx_v =
                                            *vmap.get(&args[0]).ok_or("array.get index missing")?;
                                        let idx_i = if let BasicValueEnum::IntValue(iv) = idx_v {
                                            iv
                                        } else {
                                            return Err("array.get index must be int".to_string());
                                        };
                                        let fnty = i64t.fn_type(&[i64t.into(), i64t.into()], false);
                                        let callee = codegen
                                            .module
                                            .get_function("nyash_array_get_h")
                                            .unwrap_or_else(|| {
                                                codegen.module.add_function(
                                                    "nyash_array_get_h",
                                                    fnty,
                                                    None,
                                                )
                                            });
                                        let call = codegen
                                            .builder
                                            .build_call(
                                                callee,
                                                &[recv_h.into(), idx_i.into()],
                                                "aget",
                                            )
                                            .map_err(|e| e.to_string())?;
                                        if let Some(d) = dst {
                                            let rv = call
                                                .try_as_basic_value()
                                                .left()
                                                .ok_or("array_get_h returned void".to_string())?;
                                            vmap.insert(*d, rv);
                                        }
                                    }
                                    "set" => {
                                        if args.len() != 2 {
                                            return Err("ArrayBox.set expects 2 arg".to_string());
                                        }
                                        let idx_v =
                                            *vmap.get(&args[0]).ok_or("array.set index missing")?;
                                        let val_v =
                                            *vmap.get(&args[1]).ok_or("array.set value missing")?;
                                        let idx_i = if let BasicValueEnum::IntValue(iv) = idx_v {
                                            iv
                                        } else {
                                            return Err("array.set index must be int".to_string());
                                        };
                                        let val_i = if let BasicValueEnum::IntValue(iv) = val_v {
                                            iv
                                        } else {
                                            return Err("array.set value must be int".to_string());
                                        };
                                        let fnty = i64t.fn_type(
                                            &[i64t.into(), i64t.into(), i64t.into()],
                                            false,
                                        );
                                        let callee = codegen
                                            .module
                                            .get_function("nyash_array_set_h")
                                            .unwrap_or_else(|| {
                                                codegen.module.add_function(
                                                    "nyash_array_set_h",
                                                    fnty,
                                                    None,
                                                )
                                            });
                                        let _ = codegen
                                            .builder
                                            .build_call(
                                                callee,
                                                &[recv_h.into(), idx_i.into(), val_i.into()],
                                                "aset",
                                            )
                                            .map_err(|e| e.to_string())?;
                                    }
                                    "push" => {
                                        if args.len() != 1 {
                                            return Err("ArrayBox.push expects 1 arg".to_string());
                                        }
                                        let val_v = *vmap
                                            .get(&args[0])
                                            .ok_or("array.push value missing")?;
                                        let val_i =
                                            match val_v {
                                                BasicValueEnum::IntValue(iv) => iv,
                                                BasicValueEnum::PointerValue(pv) => codegen
                                                    .builder
                                                    .build_ptr_to_int(pv, i64t, "val_p2i")
                                                    .map_err(|e| e.to_string())?,
                                                _ => return Err(
                                                    "array.push value must be int or handle ptr"
                                                        .to_string(),
                                                ),
                                            };
                                        let fnty = i64t.fn_type(&[i64t.into(), i64t.into()], false);
                                        let callee = codegen
                                            .module
                                            .get_function("nyash_array_push_h")
                                            .unwrap_or_else(|| {
                                                codegen.module.add_function(
                                                    "nyash_array_push_h",
                                                    fnty,
                                                    None,
                                                )
                                            });
                                        let _ = codegen
                                            .builder
                                            .build_call(
                                                callee,
                                                &[recv_h.into(), val_i.into()],
                                                "apush",
                                            )
                                            .map_err(|e| e.to_string())?;
                                    }
                                    "length" => {
                                        if !args.is_empty() {
                                            return Err("ArrayBox.length expects 0 arg".to_string());
                                        }
                                        let fnty = i64t.fn_type(&[i64t.into()], false);
                                        let callee = codegen
                                            .module
                                            .get_function("nyash_array_length_h")
                                            .unwrap_or_else(|| {
                                                codegen.module.add_function(
                                                    "nyash_array_length_h",
                                                    fnty,
                                                    None,
                                                )
                                            });
                                        let call = codegen
                                            .builder
                                            .build_call(callee, &[recv_h.into()], "alen")
                                            .map_err(|e| e.to_string())?;
                                        if let Some(d) = dst {
                                            let rv = call.try_as_basic_value().left().ok_or(
                                                "array_length_h returned void".to_string(),
                                            )?;
                                            vmap.insert(*d, rv);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }

                        // Instance field helpers: getField/setField (safe path)
                        if method == "getField" {
                            if args.len() != 1 {
                                return Err("getField expects 1 arg (name)".to_string());
                            }
                            let name_v = *vmap.get(&args[0]).ok_or("getField name missing")?;
                            let name_p = if let BasicValueEnum::PointerValue(pv) = name_v {
                                pv
                            } else {
                                return Err("getField name must be pointer".to_string());
                            };
                            let i8p = codegen.context.ptr_type(AddressSpace::from(0));
                            let fnty = i64t.fn_type(&[i64t.into(), i8p.into()], false);
                            let callee = codegen
                                .module
                                .get_function("nyash.instance.get_field_h")
                                .unwrap_or_else(|| {
                                    codegen.module.add_function(
                                        "nyash.instance.get_field_h",
                                        fnty,
                                        None,
                                    )
                                });
                            let call = codegen
                                .builder
                                .build_call(callee, &[recv_h.into(), name_p.into()], "getField")
                                .map_err(|e| e.to_string())?;
                            if let Some(d) = dst {
                                let rv = call
                                    .try_as_basic_value()
                                    .left()
                                    .ok_or("get_field returned void".to_string())?;
                                // rv is i64 handle; convert to i8*
                                let h = if let BasicValueEnum::IntValue(iv) = rv {
                                    iv
                                } else {
                                    return Err("get_field ret expected i64".to_string());
                                };
                                let pty = codegen.context.ptr_type(AddressSpace::from(0));
                                let ptr = codegen
                                    .builder
                                    .build_int_to_ptr(h, pty, "gf_handle_to_ptr")
                                    .map_err(|e| e.to_string())?;
                                vmap.insert(*d, ptr.into());
                            }
                            // no early return; continue lowering
                        }
                        if method == "setField" {
                            if args.len() != 2 {
                                return Err("setField expects 2 args (name, value)".to_string());
                            }
                            let name_v = *vmap.get(&args[0]).ok_or("setField name missing")?;
                            let val_v = *vmap.get(&args[1]).ok_or("setField value missing")?;
                            let name_p = if let BasicValueEnum::PointerValue(pv) = name_v {
                                pv
                            } else {
                                return Err("setField name must be pointer".to_string());
                            };
                            let val_h = match val_v {
                                BasicValueEnum::PointerValue(pv) => codegen
                                    .builder
                                    .build_ptr_to_int(pv, i64t, "val_p2i")
                                    .map_err(|e| e.to_string())?,
                                BasicValueEnum::IntValue(iv) => iv,
                                _ => {
                                    return Err(
                                        "setField value must be handle/ptr or i64".to_string()
                                    )
                                }
                            };
                            let i8p = codegen.context.ptr_type(AddressSpace::from(0));
                            let fnty = i64t.fn_type(&[i64t.into(), i8p.into(), i64t.into()], false);
                            let callee = codegen
                                .module
                                .get_function("nyash.instance.set_field_h")
                                .unwrap_or_else(|| {
                                    codegen.module.add_function(
                                        "nyash.instance.set_field_h",
                                        fnty,
                                        None,
                                    )
                                });
                            let _ = codegen
                                .builder
                                .build_call(
                                    callee,
                                    &[recv_h.into(), name_p.into(), val_h.into()],
                                    "setField",
                                )
                                .map_err(|e| e.to_string())?;
                            // no early return; continue lowering
                        }

                        // General by-id invoke when method_id is available
                        if let Some(mid) = method_id {
                            // Prepare up to 4 args (i64 or f64 bits or handle)
                            let argc_val = i64t.const_int(args.len() as u64, false);
                            let mut a1 = i64t.const_zero();
                            let mut a2 = i64t.const_zero();
                            let mut a3 = i64t.const_zero();
                            let mut a4 = i64t.const_zero();
                            let get_i64 =
                                |vid: ValueId| -> Result<inkwell::values::IntValue, String> {
                                    let v = *vmap.get(&vid).ok_or("arg missing")?;
                                    to_i64_any(codegen.context, &codegen.builder, v)
                                };
                            if args.len() >= 1 {
                                a1 = get_i64(args[0])?;
                            }
                            if args.len() >= 2 {
                                a2 = get_i64(args[1])?;
                            }
                            if args.len() >= 3 {
                                a3 = get_i64(args[2])?;
                            }
                            if args.len() >= 4 {
                                a4 = get_i64(args[3])?;
                            }
                            // Choose return ABI by dst annotated type
                            let dst_ty =
                                dst.as_ref().and_then(|d| func.metadata.value_types.get(d));
                            let use_f64_ret = matches!(dst_ty, Some(crate::mir::MirType::Float));
                            if use_f64_ret {
                                // declare double @nyash_plugin_invoke3_f64(i64,i64,i64,i64,i64,i64)
                                let fnty = codegen.context.f64_type().fn_type(
                                    &[
                                        i64t.into(),
                                        i64t.into(),
                                        i64t.into(),
                                        i64t.into(),
                                        i64t.into(),
                                        i64t.into(),
                                    ],
                                    false,
                                );
                                let callee = codegen
                                    .module
                                    .get_function("nyash_plugin_invoke3_f64")
                                    .unwrap_or_else(|| {
                                        codegen.module.add_function(
                                            "nyash_plugin_invoke3_f64",
                                            fnty,
                                            None,
                                        )
                                    });
                                let tid = i64t.const_int(type_id as u64, true);
                                let midv = i64t.const_int((*mid) as u64, false);
                                let call = codegen
                                    .builder
                                    .build_call(
                                        callee,
                                        &[
                                            tid.into(),
                                            midv.into(),
                                            argc_val.into(),
                                            recv_h.into(),
                                            a1.into(),
                                            a2.into(),
                                        ],
                                        "pinvoke_f64",
                                    )
                                    .map_err(|e| e.to_string())?;
                                if let Some(d) = dst {
                                    let rv = call
                                        .try_as_basic_value()
                                        .left()
                                        .ok_or("invoke3_f64 returned void".to_string())?;
                                    vmap.insert(*d, rv);
                                }
                                return Ok(());
                            }
                            // For argument typing, use tagged variant to allow f64/handle
                            // Prepare tags for a1..a4: 5=float, 8=handle(ptr), 3=int
                            let mut tag1 = i64t.const_int(3, false);
                            let mut tag2 = i64t.const_int(3, false);
                            let mut tag3 = i64t.const_int(3, false);
                            let mut tag4 = i64t.const_int(3, false);
                            let classify = |vid: ValueId| -> Option<i64> {
                                vmap.get(&vid).map(|v| classify_tag(*v))
                            };
                            if args.len() >= 1 {
                                if let Some(t) = classify(args[0]) {
                                    tag1 = i64t.const_int(t as u64, false);
                                }
                            }
                            if args.len() >= 2 {
                                if let Some(t) = classify(args[1]) {
                                    tag2 = i64t.const_int(t as u64, false);
                                }
                            }
                            if args.len() >= 3 {
                                if let Some(t) = classify(args[2]) {
                                    tag3 = i64t.const_int(t as u64, false);
                                }
                            }
                            if args.len() >= 4 {
                                if let Some(t) = classify(args[3]) {
                                    tag4 = i64t.const_int(t as u64, false);
                                }
                            }
                            if args.len() <= 4 {
                                // Call fixed-arity tagged shim (up to 4 args)
                                let fnty = i64t.fn_type(
                                    &[
                                        i64t.into(),
                                        i64t.into(),
                                        i64t.into(),
                                        i64t.into(),
                                        i64t.into(),
                                        i64t.into(),
                                        i64t.into(),
                                        i64t.into(),
                                        i64t.into(),
                                        i64t.into(),
                                        i64t.into(),
                                        i64t.into(),
                                    ],
                                    false,
                                );
                                let callee = codegen
                                    .module
                                    .get_function("nyash_plugin_invoke3_tagged_i64")
                                    .unwrap_or_else(|| {
                                        codegen.module.add_function(
                                            "nyash_plugin_invoke3_tagged_i64",
                                            fnty,
                                            None,
                                        )
                                    });
                                let tid = i64t.const_int(type_id as u64, true);
                                let midv = i64t.const_int((*mid) as u64, false);
                                let call = codegen
                                    .builder
                                    .build_call(
                                        callee,
                                        &[
                                            tid.into(),
                                            midv.into(),
                                            argc_val.into(),
                                            recv_h.into(),
                                            a1.into(),
                                            tag1.into(),
                                            a2.into(),
                                            tag2.into(),
                                            a3.into(),
                                            tag3.into(),
                                            a4.into(),
                                            tag4.into(),
                                        ],
                                        "pinvoke_tagged",
                                    )
                                    .map_err(|e| e.to_string())?;
                                if let Some(d) = dst {
                                    let rv = call
                                        .try_as_basic_value()
                                        .left()
                                        .ok_or("invoke3_i64 returned void".to_string())?;
                                    // Decide return lowering by dst annotated type
                                    if let Some(mt) = func.metadata.value_types.get(d) {
                                        match mt {
                                            crate::mir::MirType::Integer
                                            | crate::mir::MirType::Bool => {
                                                vmap.insert(*d, rv);
                                            }
                                            crate::mir::MirType::Box(_)
                                            | crate::mir::MirType::String
                                            | crate::mir::MirType::Array(_)
                                            | crate::mir::MirType::Future(_)
                                            | crate::mir::MirType::Unknown => {
                                                let h = if let BasicValueEnum::IntValue(iv) = rv {
                                                    iv
                                                } else {
                                                    return Err(
                                                        "invoke ret expected i64".to_string()
                                                    );
                                                };
                                                let pty = codegen
                                                    .context
                                                    .ptr_type(AddressSpace::from(0));
                                                let ptr = codegen
                                                    .builder
                                                    .build_int_to_ptr(h, pty, "ret_handle_to_ptr")
                                                    .map_err(|e| e.to_string())?;
                                                vmap.insert(*d, ptr.into());
                                            }
                                            _ => {
                                                vmap.insert(*d, rv);
                                            }
                                        }
                                    } else {
                                        vmap.insert(*d, rv);
                                    }
                                }
                            } else {
                                // Variable-length path: build arrays of values/tags and call vector shim
                                let n = args.len() as u32;
                                // alloca [N x i64] for vals and tags
                                let arr_ty = i64t.array_type(n);
                                let vals_arr = entry_builder
                                    .build_alloca(arr_ty, "vals_arr")
                                    .map_err(|e| e.to_string())?;
                                let tags_arr = entry_builder
                                    .build_alloca(arr_ty, "tags_arr")
                                    .map_err(|e| e.to_string())?;
                                for (i, vid) in args.iter().enumerate() {
                                    let idx = [
                                        codegen.context.i32_type().const_zero(),
                                        codegen.context.i32_type().const_int(i as u64, false),
                                    ];
                                    let gep_v = unsafe {
                                        codegen
                                            .builder
                                            .build_in_bounds_gep(
                                                arr_ty,
                                                vals_arr,
                                                &idx,
                                                &format!("v_gep_{}", i),
                                            )
                                            .map_err(|e| e.to_string())?
                                    };
                                    let gep_t = unsafe {
                                        codegen
                                            .builder
                                            .build_in_bounds_gep(
                                                arr_ty,
                                                tags_arr,
                                                &idx,
                                                &format!("t_gep_{}", i),
                                            )
                                            .map_err(|e| e.to_string())?
                                    };
                                    let vi = get_i64(*vid)?;
                                    let tag = classify(*vid).unwrap_or(3);
                                    let tagv = i64t.const_int(tag as u64, false);
                                    codegen
                                        .builder
                                        .build_store(gep_v, vi)
                                        .map_err(|e| e.to_string())?;
                                    codegen
                                        .builder
                                        .build_store(gep_t, tagv)
                                        .map_err(|e| e.to_string())?;
                                }
                                // cast to i64* pointers
                                let i64p = codegen.context.ptr_type(AddressSpace::from(0));
                                let vals_ptr = codegen
                                    .builder
                                    .build_pointer_cast(vals_arr, i64p, "vals_ptr")
                                    .map_err(|e| e.to_string())?;
                                let tags_ptr = codegen
                                    .builder
                                    .build_pointer_cast(tags_arr, i64p, "tags_ptr")
                                    .map_err(|e| e.to_string())?;
                                // declare i64 @nyash.plugin.invoke_tagged_v_i64(i64,i64,i64,i64,i64*,i64*)
                                let fnty = i64t.fn_type(
                                    &[
                                        i64t.into(),
                                        i64t.into(),
                                        i64t.into(),
                                        i64t.into(),
                                        i64p.into(),
                                        i64p.into(),
                                    ],
                                    false,
                                );
                                let callee = codegen
                                    .module
                                    .get_function("nyash.plugin.invoke_tagged_v_i64")
                                    .unwrap_or_else(|| {
                                        codegen.module.add_function(
                                            "nyash.plugin.invoke_tagged_v_i64",
                                            fnty,
                                            None,
                                        )
                                    });
                                let tid = i64t.const_int(type_id as u64, true);
                                let midv = i64t.const_int((*mid) as u64, false);
                                let call = codegen
                                    .builder
                                    .build_call(
                                        callee,
                                        &[
                                            tid.into(),
                                            midv.into(),
                                            argc_val.into(),
                                            recv_h.into(),
                                            vals_ptr.into(),
                                            tags_ptr.into(),
                                        ],
                                        "pinvoke_tagged_v",
                                    )
                                    .map_err(|e| e.to_string())?;
                                if let Some(d) = dst {
                                    let rv = call
                                        .try_as_basic_value()
                                        .left()
                                        .ok_or("invoke_v returned void".to_string())?;
                                    if let Some(mt) = func.metadata.value_types.get(d) {
                                        match mt {
                                            crate::mir::MirType::Integer
                                            | crate::mir::MirType::Bool => {
                                                vmap.insert(*d, rv);
                                            }
                                            crate::mir::MirType::Box(_)
                                            | crate::mir::MirType::String
                                            | crate::mir::MirType::Array(_)
                                            | crate::mir::MirType::Future(_)
                                            | crate::mir::MirType::Unknown => {
                                                let h = if let BasicValueEnum::IntValue(iv) = rv {
                                                    iv
                                                } else {
                                                    return Err(
                                                        "invoke ret expected i64".to_string()
                                                    );
                                                };
                                                let pty = codegen
                                                    .context
                                                    .ptr_type(AddressSpace::from(0));
                                                let ptr = codegen
                                                    .builder
                                                    .build_int_to_ptr(h, pty, "ret_handle_to_ptr")
                                                    .map_err(|e| e.to_string())?;
                                                vmap.insert(*d, ptr.into());
                                            }
                                            _ => {
                                                vmap.insert(*d, rv);
                                            }
                                        }
                                    } else {
                                        vmap.insert(*d, rv);
                                    }
                                }
                            }
                            // handled above per-branch
                        } else {
                            return Err(format!("BoxCall requires method_id for method '{}'. The method_id should be automatically injected during MIR compilation.", method));
                        }
                    }
                    MirInstruction::ExternCall { dst, iface_name, method_name, args, effects: _ } => {
                        instructions::lower_externcall(&codegen, func, &mut vmap, dst, iface_name, method_name, args)?;
                    }
                    MirInstruction::UnaryOp { dst, op, operand } => {
                        let v = *vmap.get(operand).ok_or("operand missing")?;
                        let out = match op {
                            UnaryOp::Neg => {
                                if let Some(iv) = as_int(v) {
                                    codegen
                                        .builder
                                        .build_int_neg(iv, "ineg")
                                        .map_err(|e| e.to_string())?
                                        .into()
                                } else if let Some(fv) = as_float(v) {
                                    codegen
                                        .builder
                                        .build_float_neg(fv, "fneg")
                                        .map_err(|e| e.to_string())?
                                        .into()
                                } else {
                                    return Err("neg on non-number".to_string());
                                }
                            }
                            UnaryOp::Not | UnaryOp::BitNot => {
                                if let Some(iv) = as_int(v) {
                                    codegen
                                        .builder
                                        .build_not(iv, "inot")
                                        .map_err(|e| e.to_string())?
                                        .into()
                                } else {
                                    return Err("not on non-int".to_string());
                                }
                            }
                        };
                        vmap.insert(*dst, out);
                    }
                    MirInstruction::BinOp { dst, op, lhs, rhs } => {
                        let lv = *vmap.get(lhs).ok_or("lhs missing")?;
                        let rv = *vmap.get(rhs).ok_or("rhs missing")?;
                        let mut handled_concat = false;
                        // String-like concat handling: if either side is a pointer (i8*),
                        // and op is Add, route to NyRT concat helpers
                        if let crate::mir::BinaryOp::Add = op {
                            let i8p = codegen.context.ptr_type(AddressSpace::from(0));
                            match (lv, rv) {
                                (
                                    BasicValueEnum::PointerValue(lp),
                                    BasicValueEnum::PointerValue(rp),
                                ) => {
                                    let fnty = i8p.fn_type(&[i8p.into(), i8p.into()], false);
                                    let callee = codegen
                                        .module
                                        .get_function("nyash.string.concat_ss")
                                        .unwrap_or_else(|| {
                                            codegen.module.add_function(
                                                "nyash.string.concat_ss",
                                                fnty,
                                                None,
                                            )
                                        });
                                    let call = codegen
                                        .builder
                                        .build_call(callee, &[lp.into(), rp.into()], "concat_ss")
                                        .map_err(|e| e.to_string())?;
                                    let rv = call
                                        .try_as_basic_value()
                                        .left()
                                        .ok_or("concat_ss returned void".to_string())?;
                                    vmap.insert(*dst, rv);
                                    handled_concat = true;
                                }
                                (
                                    BasicValueEnum::PointerValue(lp),
                                    BasicValueEnum::IntValue(ri),
                                ) => {
                                    let i64t = codegen.context.i64_type();
                                    let fnty = i8p.fn_type(&[i8p.into(), i64t.into()], false);
                                    let callee = codegen
                                        .module
                                        .get_function("nyash.string.concat_si")
                                        .unwrap_or_else(|| {
                                            codegen.module.add_function(
                                                "nyash.string.concat_si",
                                                fnty,
                                                None,
                                            )
                                        });
                                    let call = codegen
                                        .builder
                                        .build_call(callee, &[lp.into(), ri.into()], "concat_si")
                                        .map_err(|e| e.to_string())?;
                                    let rv = call
                                        .try_as_basic_value()
                                        .left()
                                        .ok_or("concat_si returned void".to_string())?;
                                    vmap.insert(*dst, rv);
                                    handled_concat = true;
                                }
                                (
                                    BasicValueEnum::IntValue(li),
                                    BasicValueEnum::PointerValue(rp),
                                ) => {
                                    let i64t = codegen.context.i64_type();
                                    let fnty = i8p.fn_type(&[i64t.into(), i8p.into()], false);
                                    let callee = codegen
                                        .module
                                        .get_function("nyash.string.concat_is")
                                        .unwrap_or_else(|| {
                                            codegen.module.add_function(
                                                "nyash.string.concat_is",
                                                fnty,
                                                None,
                                            )
                                        });
                                    let call = codegen
                                        .builder
                                        .build_call(callee, &[li.into(), rp.into()], "concat_is")
                                        .map_err(|e| e.to_string())?;
                                    let rv = call
                                        .try_as_basic_value()
                                        .left()
                                        .ok_or("concat_is returned void".to_string())?;
                                    vmap.insert(*dst, rv);
                                    handled_concat = true;
                                }
                                _ => {}
                            }
                        }
                        if handled_concat {
                            // Concat already lowered and dst set
                        } else {
                            let out = if let (Some(li), Some(ri)) = (as_int(lv), as_int(rv)) {
                                use crate::mir::BinaryOp as B;
                                match op {
                                    B::Add => codegen
                                        .builder
                                        .build_int_add(li, ri, "iadd")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::Sub => codegen
                                        .builder
                                        .build_int_sub(li, ri, "isub")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::Mul => codegen
                                        .builder
                                        .build_int_mul(li, ri, "imul")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::Div => codegen
                                        .builder
                                        .build_int_signed_div(li, ri, "idiv")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::Mod => codegen
                                        .builder
                                        .build_int_signed_rem(li, ri, "imod")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::BitAnd => codegen
                                        .builder
                                        .build_and(li, ri, "iand")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::BitOr => codegen
                                        .builder
                                        .build_or(li, ri, "ior")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::BitXor => codegen
                                        .builder
                                        .build_xor(li, ri, "ixor")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::Shl => codegen
                                        .builder
                                        .build_left_shift(li, ri, "ishl")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::Shr => codegen
                                        .builder
                                        .build_right_shift(li, ri, false, "ishr")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::And | B::Or => {
                                        // Treat as logical on integers: convert to i1 and and/or
                                        let lb =
                                            to_bool(codegen.context, li.into(), &codegen.builder)?;
                                        let rb =
                                            to_bool(codegen.context, ri.into(), &codegen.builder)?;
                                        match op {
                                            B::And => codegen
                                                .builder
                                                .build_and(lb, rb, "land")
                                                .map_err(|e| e.to_string())?
                                                .into(),
                                            _ => codegen
                                                .builder
                                                .build_or(lb, rb, "lor")
                                                .map_err(|e| e.to_string())?
                                                .into(),
                                        }
                                    }
                                }
                            } else if let (Some(lf), Some(rf)) = (as_float(lv), as_float(rv)) {
                                use crate::mir::BinaryOp as B;
                                match op {
                                    B::Add => codegen
                                        .builder
                                        .build_float_add(lf, rf, "fadd")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::Sub => codegen
                                        .builder
                                        .build_float_sub(lf, rf, "fsub")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::Mul => codegen
                                        .builder
                                        .build_float_mul(lf, rf, "fmul")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::Div => codegen
                                        .builder
                                        .build_float_div(lf, rf, "fdiv")
                                        .map_err(|e| e.to_string())?
                                        .into(),
                                    B::Mod => return Err("fmod not supported yet".to_string()),
                                    _ => return Err("bit/logic ops on float".to_string()),
                                }
                            } else {
                                return Err("binop type mismatch".to_string());
                            };
                            vmap.insert(*dst, out);
                        }
                    }
                    MirInstruction::Compare { dst, op, lhs, rhs } => {
                        let out = instructions::lower_compare(&codegen, &vmap, op, lhs, rhs)?;
                        vmap.insert(*dst, out);
                    }
                    MirInstruction::Store { value, ptr } => {
                        let val = *vmap.get(value).ok_or("store value missing")?;
                        // Determine or create the alloca for this ptr, using current value type
                        let elem_ty = match val {
                            BasicValueEnum::IntValue(iv) => BasicTypeEnum::IntType(iv.get_type()),
                            BasicValueEnum::FloatValue(fv) => {
                                BasicTypeEnum::FloatType(fv.get_type())
                            }
                            BasicValueEnum::PointerValue(pv) => {
                                BasicTypeEnum::PointerType(pv.get_type())
                            }
                            _ => return Err("unsupported store value type".to_string()),
                        };
                        if let Some(existing) = allocas.get(ptr).copied() {
                            // If types mismatch (e.g., i1 vs i64), try simple widen/narrow for ints; pointer->pointer cast
                            let existing_elem = *alloca_elem_types
                                .get(ptr)
                                .ok_or("alloca elem type missing")?;
                            if existing_elem != elem_ty {
                                match (val, existing_elem) {
                                    (BasicValueEnum::IntValue(iv), BasicTypeEnum::IntType(t)) => {
                                        let bw_src = iv.get_type().get_bit_width();
                                        let bw_dst = t.get_bit_width();
                                        if bw_src < bw_dst {
                                            let adj = codegen
                                                .builder
                                                .build_int_z_extend(iv, t, "zext")
                                                .map_err(|e| e.to_string())?;
                                            codegen
                                                .builder
                                                .build_store(existing, adj)
                                                .map_err(|e| e.to_string())?;
                                        } else if bw_src > bw_dst {
                                            let adj = codegen
                                                .builder
                                                .build_int_truncate(iv, t, "trunc")
                                                .map_err(|e| e.to_string())?;
                                            codegen
                                                .builder
                                                .build_store(existing, adj)
                                                .map_err(|e| e.to_string())?;
                                        } else {
                                            codegen
                                                .builder
                                                .build_store(existing, iv)
                                                .map_err(|e| e.to_string())?;
                                        }
                                    }
                                    (
                                        BasicValueEnum::PointerValue(pv),
                                        BasicTypeEnum::PointerType(pt),
                                    ) => {
                                        let adj = codegen
                                            .builder
                                            .build_pointer_cast(pv, pt, "pcast")
                                            .map_err(|e| e.to_string())?;
                                        codegen
                                            .builder
                                            .build_store(existing, adj)
                                            .map_err(|e| e.to_string())?;
                                    }
                                    (
                                        BasicValueEnum::FloatValue(fv),
                                        BasicTypeEnum::FloatType(ft),
                                    ) => {
                                        if fv.get_type() == ft {
                                            codegen
                                                .builder
                                                .build_store(existing, fv)
                                                .map_err(|e| e.to_string())?;
                                        } else {
                                            return Err("float width mismatch in store".to_string());
                                        }
                                    }
                                    _ => return Err("store type mismatch".to_string()),
                                };
                            } else {
                                match val {
                                    BasicValueEnum::IntValue(iv) => {
                                        codegen
                                            .builder
                                            .build_store(existing, iv)
                                            .map_err(|e| e.to_string())?;
                                    }
                                    BasicValueEnum::FloatValue(fv) => {
                                        codegen
                                            .builder
                                            .build_store(existing, fv)
                                            .map_err(|e| e.to_string())?;
                                    }
                                    BasicValueEnum::PointerValue(pv) => {
                                        codegen
                                            .builder
                                            .build_store(existing, pv)
                                            .map_err(|e| e.to_string())?;
                                    }
                                    _ => return Err("unsupported store value type".to_string()),
                                }
                            }
                        } else {
                            // Create new alloca at entry
                            let slot = entry_builder
                                .build_alloca(elem_ty, &format!("slot_{}", ptr.as_u32()))
                                .map_err(|e| e.to_string())?;
                            // Initialize to zero/null
                            let zero_val: BasicValueEnum = match elem_ty {
                                BasicTypeEnum::IntType(t) => t.const_zero().into(),
                                BasicTypeEnum::FloatType(t) => t.const_float(0.0).into(),
                                BasicTypeEnum::PointerType(t) => t.const_zero().into(),
                                _ => return Err("Unsupported alloca element type".to_string()),
                            };
                            entry_builder
                                .build_store(slot, zero_val)
                                .map_err(|e| e.to_string())?;
                            allocas.insert(*ptr, slot);
                            alloca_elem_types.insert(*ptr, elem_ty);
                            match val {
                                BasicValueEnum::IntValue(iv) => {
                                    codegen
                                        .builder
                                        .build_store(slot, iv)
                                        .map_err(|e| e.to_string())?;
                                }
                                BasicValueEnum::FloatValue(fv) => {
                                    codegen
                                        .builder
                                        .build_store(slot, fv)
                                        .map_err(|e| e.to_string())?;
                                }
                                BasicValueEnum::PointerValue(pv) => {
                                    codegen
                                        .builder
                                        .build_store(slot, pv)
                                        .map_err(|e| e.to_string())?;
                                }
                                _ => return Err("unsupported store value type".to_string()),
                            }
                        }
                    }
                    MirInstruction::Load { dst, ptr } => {
                        // Ensure alloca exists; if not, try to infer from annotated dst type, else default i64
                        let (slot, elem_ty) = if let Some(p) = allocas.get(ptr).copied() {
                            let ety = *alloca_elem_types
                                .get(ptr)
                                .ok_or("alloca elem type missing")?;
                            (p, ety)
                        } else {
                            let elem_ty = if let Some(mt) = func.metadata.value_types.get(dst) {
                                map_mirtype_to_basic(codegen.context, mt)
                            } else {
                                codegen.context.i64_type().into()
                            };
                            // Create new alloca at entry
                            let slot = entry_builder
                                .build_alloca(elem_ty, &format!("slot_{}", ptr.as_u32()))
                                .map_err(|e| e.to_string())?;
                            let zero_val: BasicValueEnum = match elem_ty {
                                BasicTypeEnum::IntType(t) => t.const_zero().into(),
                                BasicTypeEnum::FloatType(t) => t.const_float(0.0).into(),
                                BasicTypeEnum::PointerType(t) => t.const_zero().into(),
                                _ => return Err("Unsupported alloca element type".to_string()),
                            };
                            entry_builder
                                .build_store(slot, zero_val)
                                .map_err(|e| e.to_string())?;
                            allocas.insert(*ptr, slot);
                            alloca_elem_types.insert(*ptr, elem_ty);
                            (slot, elem_ty)
                        };
                        let lv = codegen
                            .builder
                            .build_load(elem_ty, slot, &format!("load_{}", dst.as_u32()))
                            .map_err(|e| e.to_string())?;
                        vmap.insert(*dst, lv);
                    }
                    MirInstruction::Phi { .. } => {
                        // Already created in pre-pass; nothing to do here.
                    }
                    _ => { /* ignore other ops for 11.1 */ }
                }
            }
            if let Some(term) = &block.terminator {
                match term {
                    MirInstruction::Return { value } => {
                        instructions::emit_return(&codegen, func, &vmap, value)?;
                    }
                    MirInstruction::Jump { target } => {
                        instructions::emit_jump(&codegen, bid, target, &bb_map, &phis_by_block, &vmap)?;
                    }
                    MirInstruction::Branch { condition, then_bb, else_bb } => {
                        instructions::emit_branch(&codegen, bid, condition, then_bb, else_bb, &bb_map, &phis_by_block, &vmap)?;
                    }
                    _ => {}
                }
            }
        }

        // Verify and emit
        if !llvm_func.verify(true) {
            return Err("Function verification failed".to_string());
        }
        // Try writing via file API first; if it succeeds but file is missing due to env/FS quirks,
        // also write via memory buffer as a fallback to ensure presence.
        let verbose = std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1");
        if verbose {
            eprintln!("[LLVM] emitting object to {} (begin)", output_path);
        }
        match codegen.target_machine.write_to_file(
            &codegen.module,
            inkwell::targets::FileType::Object,
            std::path::Path::new(output_path),
        ) {
            Ok(_) => {
                // Verify; if missing, fallback to memory buffer write
                if std::fs::metadata(output_path).is_err() {
                    let buf = codegen
                        .target_machine
                        .write_to_memory_buffer(&codegen.module, inkwell::targets::FileType::Object)
                        .map_err(|e| format!("Failed to get object buffer: {}", e))?;
                    std::fs::write(output_path, buf.as_slice()).map_err(|e| {
                        format!("Failed to write object to '{}': {}", output_path, e)
                    })?;
                    if verbose {
                        eprintln!(
                            "[LLVM] wrote object via memory buffer fallback: {} ({} bytes)",
                            output_path,
                            buf.get_size()
                        );
                    }
                } else if verbose {
                    if let Ok(meta) = std::fs::metadata(output_path) {
                        eprintln!(
                            "[LLVM] wrote object via file API: {} ({} bytes)",
                            output_path,
                            meta.len()
                        );
                    }
                }
                if verbose {
                    eprintln!("[LLVM] emit complete (Ok branch) for {}", output_path);
                }
                Ok(())
            }
            Err(e) => {
                // Fallback: memory buffer
                let buf = codegen
                    .target_machine
                    .write_to_memory_buffer(&codegen.module, inkwell::targets::FileType::Object)
                    .map_err(|ee| {
                        format!(
                            "Failed to write object ({}); and memory buffer failed: {}",
                            e, ee
                        )
                    })?;
                std::fs::write(output_path, buf.as_slice()).map_err(|ee| {
                    format!(
                        "Failed to write object to '{}': {} (original error: {})",
                        output_path, ee, e
                    )
                })?;
                if verbose {
                    eprintln!(
                        "[LLVM] wrote object via error fallback: {} ({} bytes)",
                        output_path,
                        buf.get_size()
                    );
                }
                if verbose {
                    eprintln!(
                        "[LLVM] emit complete (Err branch handled) for {}",
                        output_path
                    );
                }
                Ok(())
            }
        }
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
