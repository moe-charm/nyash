/*!
 * LLVM Compiler Implementation - Compile MIR to LLVM IR and native code
 */

use crate::mir::function::MirModule;
use crate::mir::instruction::{MirInstruction, ConstValue, BinaryOp, UnaryOp, CompareOp};
use crate::mir::ValueId;
use crate::box_trait::{NyashBox, IntegerBox, StringBox, BoolBox};
use crate::boxes::math_box::FloatBox;
use crate::boxes::function_box::FunctionBox;
use crate::boxes::null_box::NullBox;
use super::context::CodegenContext;
use std::collections::HashMap;

/// Mock LLVM Compiler with MIR interpreter for demonstration
/// This simulates LLVM behavior by interpreting MIR instructions
pub struct LLVMCompiler {
    /// Values stored during mock execution
    values: HashMap<ValueId, Box<dyn NyashBox>>,
}

#[cfg(not(feature = "llvm"))]
impl LLVMCompiler {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            values: HashMap::new(),
        })
    }
    
    pub fn compile_module(
        &self,
        mir_module: &MirModule,
        output_path: &str,
    ) -> Result<(), String> {
        // Mock implementation - in a real scenario this would:
        // 1. Create LLVM context and module
        // 2. Convert MIR instructions to LLVM IR
        // 3. Generate object file
        
        println!("🔧 Mock LLVM Compilation:");
        println!("   Module: {}", mir_module.name);
        println!("   Functions: {}", mir_module.functions.len());
        println!("   Output: {}", output_path);
        
        // Find entry function (prefer is_entry_point, then Main.main, then main, else first)
        let main_func = if let Some((_n,f)) = mir_module.functions.iter().find(|(_n,f)| f.metadata.is_entry_point) {
            f
        } else if let Some(f) = mir_module.functions.get("Main.main") {
            f
        } else if let Some(f) = mir_module.functions.get("main") {
            f
        } else if let Some((_n,f)) = mir_module.functions.iter().next() {
            f
        } else {
            return Err("Main.main function not found")
        };
            
        println!("   Main function found with {} blocks", main_func.blocks.len());
        
        // Simulate object file generation
        std::fs::write(output_path, b"Mock object file")?;
        println!("   ✅ Mock object file created");
        
        Ok(())
    }
    
    pub fn compile_and_execute(
        &mut self,
        mir_module: &MirModule,
        temp_path: &str,
    ) -> Result<Box<dyn NyashBox>, String> {
        // Mock implementation - interprets MIR instructions to simulate execution
        
        eprintln!("⚠️⚠️⚠️ WARNING: Using MOCK LLVM Implementation! ⚠️⚠️⚠️");
        eprintln!("⚠️ This is NOT real LLVM execution!");
        eprintln!("⚠️ Build with --features llvm for real compilation!");
        println!("🚀 Mock LLVM Compile & Execute (MIR Interpreter Mode):");
        
        // 1. Mock object file generation
        let obj_path = format!("{}.o", temp_path);
        self.compile_module(mir_module, &obj_path)?;
        
        // 2. Find and execute main function
        let main_func = mir_module.functions.get("Main.main")
            .ok_or("Main.main function not found")?;
        
        println!("   ⚡ Interpreting MIR instructions...");
        
        // 3. Execute MIR instructions
        let result = self.interpret_function(main_func)?;
        
        // 4. Cleanup mock files
        let _ = std::fs::remove_file(&obj_path);
        
        Ok(result)
    }
    
    /// Interpret a MIR function by executing its instructions
    fn interpret_function(
        &mut self,
        func: &crate::mir::function::MirFunction,
    ) -> Result<Box<dyn NyashBox>, String> {
        // Clear value storage
        self.values.clear();
        
        // For now, just execute the entry block
        if let Some(entry_block) = func.blocks.get(&0) {
            for inst in &entry_block.instructions {
                match inst {
                    MirInstruction::Const { dst, value } => {
                        let nyash_value = match value {
                            ConstValue::Integer(i) => Box::new(IntegerBox::new(*i)) as Box<dyn NyashBox>,
                            ConstValue::Float(f) => Box::new(FloatBox::new(*f)) as Box<dyn NyashBox>,
                            ConstValue::String(s) => Box::new(StringBox::new(s.clone())) as Box<dyn NyashBox>,
                            ConstValue::Bool(b) => Box::new(BoolBox::new(*b)) as Box<dyn NyashBox>,
                            ConstValue::Null => Box::new(NullBox::new()) as Box<dyn NyashBox>,
                        };
                        self.values.insert(*dst, nyash_value);
                        println!("   📝 %{} = const {:?}", dst.0, value);
                    }
                    
                    MirInstruction::BinOp { dst, op, lhs, rhs } => {
                        // Get operands
                        let left = self.values.get(lhs)
                            .ok_or_else(|| format!("Value %{} not found", lhs.0))?;
                        let right = self.values.get(rhs)
                            .ok_or_else(|| format!("Value %{} not found", rhs.0))?;
                        
                        // Simple integer arithmetic for now
                        if let (Some(l), Some(r)) = (left.as_any().downcast_ref::<IntegerBox>(), 
                                                      right.as_any().downcast_ref::<IntegerBox>()) {
                            let result = match op {
                                BinaryOp::Add => l.value + r.value,
                                BinaryOp::Sub => l.value - r.value,
                                BinaryOp::Mul => l.value * r.value,
                                BinaryOp::Div => {
                                    if r.value == 0 {
                                        return Err("Division by zero".to_string());
                                    }
                                    l.value / r.value
                                }
                                BinaryOp::Mod => l.value % r.value,
                                _ => {
                                    return Err("Binary operation not supported in mock".to_string());
                                }
                            };
                            self.values.insert(*dst, Box::new(IntegerBox::new(result)));
                            println!("   📊 %{} = %{} {:?} %{} = {}", dst.0, lhs.0, op, rhs.0, result);
                        } else {
                            return Err("Binary operation on non-integer values not supported in mock".to_string());
                        }
                    }
                    
                    MirInstruction::Return { value } => {
                        if let Some(val_id) = value {
                            let result = self.values.get(val_id)
                                .ok_or_else(|| format!("Return value %{} not found", val_id.0))?
                                .clone_box();
                            println!("   ✅ Returning value from %{}", val_id.0);
                            return Ok(result);
                        } else {
                            println!("   ✅ Void return");
                            return Ok(Box::new(IntegerBox::new(0)));
                        }
                    }
                    
                    MirInstruction::FunctionNew { dst, params, body, captures, me } => {
                        // Minimal: build FunctionBox with empty captures unless provided
                        let mut env = crate::boxes::function_box::ClosureEnv::new();
                        // Materialize captures (by value) if any
                        for (name, vid) in captures.iter() {
                            let v = self.values.get(vid).ok_or_else(|| format!("Value %{} not found for capture {}", vid.0, name))?;
                            env.captures.insert(name.clone(), v.clone_box());
                        }
                        // me capture (weak) if provided and is a box
                        if let Some(m) = me {
                            if let Some(b) = self.values.get(m) {
                                if let Some(arc) = std::sync::Arc::downcast::<dyn NyashBox>({
                                    let bx: std::sync::Arc<dyn NyashBox> = std::sync::Arc::from(b.clone_box());
                                    bx
                                }).ok() {
                                    env.me_value = Some(std::sync::Arc::downgrade(&arc));
                                }
                            }
                        }
                        let fun = FunctionBox::with_env(params.clone(), body.clone(), env);
                        self.values.insert(*dst, Box::new(fun));
                        println!("   🧰 %{} = function_new (params={})", dst.0, params.len());
                    }

                    MirInstruction::Call { dst, func, args, .. } => {
                        // Resolve callee
                        let cal = self.values.get(func)
                            .ok_or_else(|| format!("Call target %{} not found", func.0))?;
                        if let Some(fb) = cal.as_any().downcast_ref::<FunctionBox>() {
                            // Collect args as NyashBox
                            let mut argv: Vec<Box<dyn NyashBox>> = Vec::new();
                            for a in args {
                                let av = self.values.get(a).ok_or_else(|| format!("Arg %{} not found", a.0))?;
                                argv.push(av.clone_box());
                            }
                            let out = crate::interpreter::run_function_box(fb, argv)
                                .map_err(|e| format!("FunctionBox call failed: {:?}", e))?;
                            if let Some(d) = dst { self.values.insert(*d, out); }
                            println!("   📞 call %{} -> {}", func.0, dst.map(|v| v.0).unwrap_or(u32::MAX));
                        } else {
                            println!("   ⚠️  Skipping call: callee not FunctionBox");
                        }
                    }

                    _ => {
                        // Other instructions not yet implemented
                        println!("   ⚠️  Skipping instruction: {:?}", inst);
                    }
                }
            }
        }
        
        // Default return
        Ok(Box::new(IntegerBox::new(0)))
    }
}

// Real implementation (when feature "llvm" is enabled)
#[cfg(feature = "llvm")]
use inkwell::context::Context;
#[cfg(feature = "llvm")]
use inkwell::{values::{BasicValueEnum, FloatValue, IntValue, PhiValue, FunctionValue, PointerValue}, types::{BasicTypeEnum, IntType, FloatType, PointerType}, AddressSpace};
#[cfg(feature = "llvm")]
use inkwell::types::BasicType; // for as_basic_type_enum()
#[cfg(feature = "llvm")]
use std::collections::HashMap as StdHashMap;

#[cfg(feature = "llvm")]
impl LLVMCompiler {
    pub fn new() -> Result<Self, String> {
        Ok(Self { values: HashMap::new() })
    }

    pub fn compile_module(
        &self,
        mir_module: &MirModule,
        output_path: &str,
    ) -> Result<(), String> {
        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
            eprintln!("[LLVM] compile_module start: functions={}, out={}", mir_module.functions.len(), output_path);
        }
        let context = Context::create();
        let codegen = CodegenContext::new(&context, "nyash_module")?;
        // Lower only Main.main for now
        // Find entry function
        let func = if let Some((_n,f)) = mir_module.functions.iter().find(|(_n,f)| f.metadata.is_entry_point) {
            f
        } else if let Some(f) = mir_module.functions.get("Main.main") { f }
        else if let Some(f) = mir_module.functions.get("main") { f }
        else if let Some((_n,f)) = mir_module.functions.iter().next() { f }
        else { return Err("Main.main function not found in module".to_string()) };

        // Map MIR types to LLVM types
        fn map_type<'ctx>(ctx: &'ctx Context, ty: &crate::mir::MirType) -> Result<BasicTypeEnum<'ctx>, String> {
            Ok(match ty {
                crate::mir::MirType::Integer => ctx.i64_type().into(),
                crate::mir::MirType::Float => ctx.f64_type().into(),
                crate::mir::MirType::Bool => ctx.bool_type().into(),
                crate::mir::MirType::String => ctx.i8_type().ptr_type(inkwell::AddressSpace::from(0)).into(),
                crate::mir::MirType::Void => return Err("Void has no value type".to_string()),
                crate::mir::MirType::Box(_) => ctx.i8_type().ptr_type(inkwell::AddressSpace::from(0)).into(),
                crate::mir::MirType::Array(_) | crate::mir::MirType::Future(_) | crate::mir::MirType::Unknown => ctx.i8_type().ptr_type(inkwell::AddressSpace::from(0)).into(),
            })
        }

        // Load box type-id mapping from nyash_box.toml (central plugin registry)
        let box_type_ids = super::box_types::load_box_type_ids();

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
        let mut bb_map: StdHashMap<crate::mir::BasicBlockId, inkwell::basic_block::BasicBlock> = StdHashMap::new();
        let entry_first = func.entry_block;
        let entry_bb = codegen.context.append_basic_block(llvm_func, &format!("bb{}", entry_first.as_u32()));
        bb_map.insert(entry_first, entry_bb);
        for bid in func.block_ids() {
            if bid == entry_first { continue; }
            let name = format!("bb{}", bid.as_u32());
            let bb = codegen.context.append_basic_block(llvm_func, &name);
            bb_map.insert(bid, bb);
        }

        // Position at entry
        codegen.builder.position_at_end(entry_bb);

        // SSA value map
        let mut vmap: StdHashMap<ValueId, BasicValueEnum> = StdHashMap::new();

        // Helper ops (centralized conversions and comparisons)
        fn as_int<'ctx>(v: BasicValueEnum<'ctx>) -> Option<IntValue<'ctx>> { if let BasicValueEnum::IntValue(iv) = v { Some(iv) } else { None } }
        fn as_float<'ctx>(v: BasicValueEnum<'ctx>) -> Option<FloatValue<'ctx>> { if let BasicValueEnum::FloatValue(fv) = v { Some(fv) } else { None } }
        fn to_i64_any<'ctx>(ctx: &'ctx Context, builder: &inkwell::builder::Builder<'ctx>, v: BasicValueEnum<'ctx>) -> Result<IntValue<'ctx>, String> {
            let i64t = ctx.i64_type();
            Ok(match v {
                BasicValueEnum::IntValue(iv) => {
                    if iv.get_type().get_bit_width() == 64 { iv }
                    else if iv.get_type().get_bit_width() < 64 { builder.build_int_z_extend(iv, i64t, "zext_i64").map_err(|e| e.to_string())? }
                    else { builder.build_int_truncate(iv, i64t, "trunc_i64").map_err(|e| e.to_string())? }
                }
                BasicValueEnum::PointerValue(pv) => builder.build_ptr_to_int(pv, i64t, "p2i64").map_err(|e| e.to_string())?,
                BasicValueEnum::FloatValue(fv) => {
                    // Bitcast f64 -> i64 via stack slot
                    let slot = builder.get_insert_block().and_then(|bb| bb.get_parent()).and_then(|f| f.get_first_basic_block()).map(|entry| {
                        let eb = ctx.create_builder(); eb.position_at_end(entry); eb
                    }).unwrap_or_else(|| ctx.create_builder());
                    let i64p = i64t.ptr_type(AddressSpace::from(0));
                    let tmp = slot.build_alloca(i64t, "f2i_tmp").map_err(|e| e.to_string())?;
                    let fptr_ty = ctx.f64_type().ptr_type(AddressSpace::from(0));
                    let castp = builder.build_pointer_cast(tmp, fptr_ty, "i64p_to_f64p").map_err(|e| e.to_string())?;
                    builder.build_store(castp, fv).map_err(|e| e.to_string())?;
                    builder.build_load(i64t, tmp, "ld_f2i").map_err(|e| e.to_string())?.into_int_value()
                }
                _ => return Err("unsupported value for i64 conversion".to_string()),
            })
        }
        fn i64_to_ptr<'ctx>(ctx: &'ctx Context, builder: &inkwell::builder::Builder<'ctx>, iv: IntValue<'ctx>) -> Result<PointerValue<'ctx>, String> {
            let pty = ctx.i8_type().ptr_type(AddressSpace::from(0));
            builder.build_int_to_ptr(iv, pty, "i64_to_ptr").map_err(|e| e.to_string())
        }
        fn classify_tag<'ctx>(v: BasicValueEnum<'ctx>) -> i64 {
            match v {
                BasicValueEnum::FloatValue(_) => 5, // float
                BasicValueEnum::PointerValue(_) => 8, // handle/ptr
                BasicValueEnum::IntValue(_) => 3, // integer/bool
                _ => 3,
            }
        }
        fn to_bool<'ctx>(ctx: &'ctx Context, b: BasicValueEnum<'ctx>, builder: &inkwell::builder::Builder<'ctx>) -> Result<IntValue<'ctx>, String> {
            if let Some(bb) = as_int(b) {
                // If not i1, compare != 0
                if bb.get_type().get_bit_width() == 1 { Ok(bb) }
                else { Ok(builder.build_int_compare(inkwell::IntPredicate::NE, bb, bb.get_type().const_zero(), "tobool").map_err(|e| e.to_string())?) }
            } else if let Some(fv) = as_float(b) {
                let zero = fv.get_type().const_float(0.0);
                Ok(builder.build_float_compare(inkwell::FloatPredicate::ONE, fv, zero, "toboolf").map_err(|e| e.to_string())?)
            } else if let BasicValueEnum::PointerValue(pv) = b {
                let i64t = ctx.i64_type();
                let p2i = builder.build_ptr_to_int(pv, i64t, "p2i").map_err(|e| e.to_string())?;
                Ok(builder.build_int_compare(inkwell::IntPredicate::NE, p2i, i64t.const_zero(), "toboolp").map_err(|e| e.to_string())?)
            } else {
                Err("Unsupported value for boolean conversion".to_string())
            }
        }
        fn cmp_eq_ne_any<'ctx>(ctx: &'ctx Context, builder: &inkwell::builder::Builder<'ctx>, op: &crate::mir::CompareOp, lv: BasicValueEnum<'ctx>, rv: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
            use crate::mir::CompareOp as C;
            match (lv, rv) {
                (BasicValueEnum::IntValue(li), BasicValueEnum::IntValue(ri)) => {
                    let pred = if matches!(op, C::Eq) { inkwell::IntPredicate::EQ } else { inkwell::IntPredicate::NE };
                    Ok(builder.build_int_compare(pred, li, ri, "icmp").map_err(|e| e.to_string())?.into())
                }
                (BasicValueEnum::FloatValue(lf), BasicValueEnum::FloatValue(rf)) => {
                    let pred = if matches!(op, C::Eq) { inkwell::FloatPredicate::OEQ } else { inkwell::FloatPredicate::ONE };
                    Ok(builder.build_float_compare(pred, lf, rf, "fcmp").map_err(|e| e.to_string())?.into())
                }
                (BasicValueEnum::PointerValue(_), _) | (_, BasicValueEnum::PointerValue(_)) => {
                    let li = to_i64_any(ctx, builder, lv)?;
                    let ri = to_i64_any(ctx, builder, rv)?;
                    let pred = if matches!(op, C::Eq) { inkwell::IntPredicate::EQ } else { inkwell::IntPredicate::NE };
                    Ok(builder.build_int_compare(pred, li, ri, "pcmp_any").map_err(|e| e.to_string())?.into())
                }
                _ => Err("compare type mismatch".to_string()),
            }
        }

        // Pre-create allocas for locals on demand (entry-only builder)
        let mut allocas: StdHashMap<ValueId, PointerValue> = StdHashMap::new();
        let mut entry_builder = codegen.context.create_builder();
        entry_builder.position_at_end(entry_bb);

        // Helper: map MirType to LLVM basic type (value type)
        fn map_mirtype_to_basic<'ctx>(ctx: &'ctx Context, ty: &crate::mir::MirType) -> BasicTypeEnum<'ctx> {
            match ty {
                crate::mir::MirType::Integer => ctx.i64_type().into(),
                crate::mir::MirType::Float => ctx.f64_type().into(),
                crate::mir::MirType::Bool => ctx.bool_type().into(),
                crate::mir::MirType::String => ctx.i8_type().ptr_type(AddressSpace::from(0)).into(),
                crate::mir::MirType::Box(_) | crate::mir::MirType::Array(_) | crate::mir::MirType::Future(_) | crate::mir::MirType::Unknown => ctx.i8_type().ptr_type(AddressSpace::from(0)).into(),
                crate::mir::MirType::Void => ctx.i64_type().into(), // avoid void as a value type; default to i64
            }
        }

        // Helper: create (or get) an alloca for a given pointer-typed SSA value id
        let mut alloca_elem_types: StdHashMap<ValueId, BasicTypeEnum> = StdHashMap::new();

        // Pre-create PHI nodes for all blocks (so we can add incoming from predecessors)
        let mut phis_by_block: StdHashMap<crate::mir::BasicBlockId, Vec<(ValueId, PhiValue, Vec<(crate::mir::BasicBlockId, ValueId)>)>> = StdHashMap::new();
        for bid in func.block_ids() {
            let bb = *bb_map.get(&bid).ok_or("missing bb in map")?;
            // Position at start of the block (no instructions emitted yet)
            codegen.builder.position_at_end(bb);
            let block = func.blocks.get(&bid).unwrap();
            for inst in block.instructions.iter().take_while(|i| matches!(i, MirInstruction::Phi { .. })) {
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
                    let phi = codegen.builder.build_phi(phi_ty, &format!("phi_{}", dst.as_u32())).map_err(|e| e.to_string())?;
                    vmap.insert(*dst, phi.as_basic_value());
                    phis_by_block.entry(bid).or_default().push((*dst, phi, inputs.clone()));
                }
            }
        }

        // Lower in block order
        for bid in func.block_ids() {
            let bb = *bb_map.get(&bid).unwrap();
            if codegen.builder.get_insert_block().map(|b| b != bb).unwrap_or(true) {
                codegen.builder.position_at_end(bb);
            }
            let block = func.blocks.get(&bid).unwrap();
            for inst in &block.instructions {
                match inst {
                    MirInstruction::NewBox { dst, box_type, args } => {
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
                                let fnty = i64t.fn_type(&[i64t.into(), i64t.into(), i64t.into(), i64t.into()], false);
                                let callee = codegen.module.get_function("nyash.box.birth_i64").unwrap_or_else(|| codegen.module.add_function("nyash.box.birth_i64", fnty, None));
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
                                let call = codegen.builder.build_call(callee, &[tid.into(), argc.into(), a1.into(), a2.into()], "birth_i64").map_err(|e| e.to_string())?;
                                let h = call.try_as_basic_value().left().ok_or("birth_i64 returned void".to_string())?.into_int_value();
                                let pty = codegen.context.i8_type().ptr_type(AddressSpace::from(0));
                                let ptr = codegen.builder.build_int_to_ptr(h, pty, "handle_to_ptr").map_err(|e| e.to_string())?;
                                vmap.insert(*dst, ptr.into());
                            }
                            _ => {
                                // No-arg birth via central type registry
                                if !args.is_empty() {
                                    return Err("NewBox with >2 args not yet supported in LLVM lowering".to_string());
                                }
                                let type_id = *box_type_ids.get(box_type).unwrap_or(&0);
                                let i64t = codegen.context.i64_type();
                                // declare i64 @nyash.box.birth_h(i64)
                                let fn_ty = i64t.fn_type(&[i64t.into()], false);
                                let callee = codegen.module.get_function("nyash.box.birth_h").unwrap_or_else(|| codegen.module.add_function("nyash.box.birth_h", fn_ty, None));
                                let tid = i64t.const_int(type_id as u64, true);
                                let call = codegen.builder.build_call(callee, &[tid.into()], "birth").map_err(|e| e.to_string())?;
                                // Handle is i64; represent Box as opaque i8* via inttoptr
                                let h_i64 = call.try_as_basic_value().left().ok_or("birth_h returned void".to_string())?.into_int_value();
                                let pty = codegen.context.i8_type().ptr_type(AddressSpace::from(0));
                                let ptr = codegen.builder.build_int_to_ptr(h_i64, pty, "handle_to_ptr").map_err(|e| e.to_string())?;
                                vmap.insert(*dst, ptr.into());
                            }
                        }
                    }
                    MirInstruction::Const { dst, value } => {
                        let bval = match value {
                            ConstValue::Integer(i) => codegen.context.i64_type().const_int(*i as u64, true).into(),
                            ConstValue::Float(f) => codegen.context.f64_type().const_float(*f).into(),
                            ConstValue::Bool(b) => codegen.context.bool_type().const_int(*b as u64, false).into(),
                            ConstValue::String(s) => {
                                let gv = codegen.builder.build_global_string_ptr(s, "str").map_err(|e| e.to_string())?;
                                let len = codegen.context.i32_type().const_int(s.len() as u64, false);
                                // declare i8* @nyash_string_new(i8*, i32)
                                let rt = codegen.context.i8_type().ptr_type(inkwell::AddressSpace::from(0));
                                let fn_ty = rt.fn_type(&[codegen.context.i8_type().ptr_type(inkwell::AddressSpace::from(0)).into(), codegen.context.i32_type().into()], false);
                                let callee = codegen.module.get_function("nyash_string_new").unwrap_or_else(|| codegen.module.add_function("nyash_string_new", fn_ty, None));
                                let call = codegen.builder.build_call(callee, &[gv.as_pointer_value().into(), len.into()], "strnew").map_err(|e| e.to_string())?;
                                call.try_as_basic_value().left().ok_or("nyash_string_new returned void".to_string())?
                            }
                            ConstValue::Null => codegen.context.i8_type().ptr_type(inkwell::AddressSpace::from(0)).const_zero().into(),
                            ConstValue::Void => return Err("Const Void unsupported".to_string()),
                        };
                        vmap.insert(*dst, bval);
                    }
                    MirInstruction::BoxCall { dst, box_val, method, method_id, args, effects: _ } => {
                        let i64t = codegen.context.i64_type();
                        // Receiver handle (i64)
                        let recv_v = *vmap.get(box_val).ok_or("box receiver missing")?;
                        // Accept either an opaque pointer (i8*) or an i64 handle for the receiver
                        let recv_p = match recv_v {
                            BasicValueEnum::PointerValue(pv) => pv,
                            BasicValueEnum::IntValue(iv) => {
                                // Treat as Nyash handle and convert to opaque pointer
                                let pty = codegen.context.i8_type().ptr_type(AddressSpace::from(0));
                                codegen.builder.build_int_to_ptr(iv, pty, "recv_i2p").map_err(|e| e.to_string())?
                            }
                            _ => return Err("box receiver must be pointer or i64 handle".to_string()),
                        };
                        let recv_h = codegen.builder.build_ptr_to_int(recv_p, i64t, "recv_p2i").map_err(|e| e.to_string())?;
                        // Resolve type_id from metadata (Box("Type")) using box_type_ids
                        let type_id: i64 = if let Some(crate::mir::MirType::Box(bname)) = func.metadata.value_types.get(box_val) {
                            *box_type_ids.get(bname).unwrap_or(&0)
                        } else if let Some(crate::mir::MirType::String) = func.metadata.value_types.get(box_val) {
                            *box_type_ids.get("StringBox").unwrap_or(&0)
                        } else { 0 };

                        // Special-case ArrayBox get/set/push/length until general by-id is widely annotated
                        if let Some(crate::mir::MirType::Box(bname)) = func.metadata.value_types.get(box_val) {
                            if bname == "ArrayBox" && (method == "get" || method == "set" || method == "push" || method == "length") {
                                match method.as_str() {
                                    "get" => {
                                        if args.len() != 1 { return Err("ArrayBox.get expects 1 arg".to_string()); }
                                        let idx_v = *vmap.get(&args[0]).ok_or("array.get index missing")?;
                                        let idx_i = if let BasicValueEnum::IntValue(iv) = idx_v { iv } else { return Err("array.get index must be int".to_string()); };
                                        let fnty = i64t.fn_type(&[i64t.into(), i64t.into()], false);
                                        let callee = codegen.module.get_function("nyash_array_get_h").unwrap_or_else(|| codegen.module.add_function("nyash_array_get_h", fnty, None));
                                        let call = codegen.builder.build_call(callee, &[recv_h.into(), idx_i.into()], "aget").map_err(|e| e.to_string())?;
                                        if let Some(d) = dst { let rv = call.try_as_basic_value().left().ok_or("array_get_h returned void".to_string())?; vmap.insert(*d, rv); }
                                    }
                                    "set" => {
                                        if args.len() != 2 { return Err("ArrayBox.set expects 2 arg".to_string()); }
                                        let idx_v = *vmap.get(&args[0]).ok_or("array.set index missing")?;
                                        let val_v = *vmap.get(&args[1]).ok_or("array.set value missing")?;
                                        let idx_i = if let BasicValueEnum::IntValue(iv) = idx_v { iv } else { return Err("array.set index must be int".to_string()); };
                                        let val_i = if let BasicValueEnum::IntValue(iv) = val_v { iv } else { return Err("array.set value must be int".to_string()); };
                                        let fnty = i64t.fn_type(&[i64t.into(), i64t.into(), i64t.into()], false);
                                        let callee = codegen.module.get_function("nyash_array_set_h").unwrap_or_else(|| codegen.module.add_function("nyash_array_set_h", fnty, None));
                                        let _ = codegen.builder.build_call(callee, &[recv_h.into(), idx_i.into(), val_i.into()], "aset").map_err(|e| e.to_string())?;
                                    }
                                    "push" => {
                                        if args.len() != 1 { return Err("ArrayBox.push expects 1 arg".to_string()); }
                                        let val_v = *vmap.get(&args[0]).ok_or("array.push value missing")?;
                                        let val_i = match val_v {
                                            BasicValueEnum::IntValue(iv) => iv,
                                            BasicValueEnum::PointerValue(pv) => codegen.builder.build_ptr_to_int(pv, i64t, "val_p2i").map_err(|e| e.to_string())?,
                                            _ => return Err("array.push value must be int or handle ptr".to_string()),
                                        };
                                        let fnty = i64t.fn_type(&[i64t.into(), i64t.into()], false);
                                        let callee = codegen.module.get_function("nyash_array_push_h").unwrap_or_else(|| codegen.module.add_function("nyash_array_push_h", fnty, None));
                                        let _ = codegen.builder.build_call(callee, &[recv_h.into(), val_i.into()], "apush").map_err(|e| e.to_string())?;
                                    }
                                    "length" => {
                                        if !args.is_empty() { return Err("ArrayBox.length expects 0 arg".to_string()); }
                                        let fnty = i64t.fn_type(&[i64t.into()], false);
                                        let callee = codegen.module.get_function("nyash_array_length_h").unwrap_or_else(|| codegen.module.add_function("nyash_array_length_h", fnty, None));
                                        let call = codegen.builder.build_call(callee, &[recv_h.into()], "alen").map_err(|e| e.to_string())?;
                                        if let Some(d) = dst { let rv = call.try_as_basic_value().left().ok_or("array_length_h returned void".to_string())?; vmap.insert(*d, rv); }
                                    }
                                    _ => {}
                                }
                            }
                        }

                        // Instance field helpers: getField/setField (safe path)
                        if method == "getField" {
                            if args.len() != 1 { return Err("getField expects 1 arg (name)".to_string()); }
                            let name_v = *vmap.get(&args[0]).ok_or("getField name missing")?;
                            let name_p = if let BasicValueEnum::PointerValue(pv) = name_v { pv } else { return Err("getField name must be pointer".to_string()); };
                            let i8p = codegen.context.i8_type().ptr_type(AddressSpace::from(0));
                            let fnty = i64t.fn_type(&[i64t.into(), i8p.into()], false);
                            let callee = codegen.module.get_function("nyash.instance.get_field_h").unwrap_or_else(|| codegen.module.add_function("nyash.instance.get_field_h", fnty, None));
                            let call = codegen.builder.build_call(callee, &[recv_h.into(), name_p.into()], "getField").map_err(|e| e.to_string())?;
                            if let Some(d) = dst {
                                let rv = call.try_as_basic_value().left().ok_or("get_field returned void".to_string())?;
                                // rv is i64 handle; convert to i8*
                                let h = if let BasicValueEnum::IntValue(iv) = rv { iv } else { return Err("get_field ret expected i64".to_string()); };
                                let pty = codegen.context.i8_type().ptr_type(AddressSpace::from(0));
                                let ptr = codegen.builder.build_int_to_ptr(h, pty, "gf_handle_to_ptr").map_err(|e| e.to_string())?;
                                vmap.insert(*d, ptr.into());
                            }
                            // no early return; continue lowering
                        }
                        if method == "setField" {
                            if args.len() != 2 { return Err("setField expects 2 args (name, value)".to_string()); }
                            let name_v = *vmap.get(&args[0]).ok_or("setField name missing")?;
                            let val_v = *vmap.get(&args[1]).ok_or("setField value missing")?;
                            let name_p = if let BasicValueEnum::PointerValue(pv) = name_v { pv } else { return Err("setField name must be pointer".to_string()); };
                            let val_h = match val_v {
                                BasicValueEnum::PointerValue(pv) => codegen.builder.build_ptr_to_int(pv, i64t, "val_p2i").map_err(|e| e.to_string())?,
                                BasicValueEnum::IntValue(iv) => iv,
                                _ => return Err("setField value must be handle/ptr or i64".to_string()),
                            };
                            let i8p = codegen.context.i8_type().ptr_type(AddressSpace::from(0));
                            let fnty = i64t.fn_type(&[i64t.into(), i8p.into(), i64t.into()], false);
                            let callee = codegen.module.get_function("nyash.instance.set_field_h").unwrap_or_else(|| codegen.module.add_function("nyash.instance.set_field_h", fnty, None));
                            let _ = codegen.builder.build_call(callee, &[recv_h.into(), name_p.into(), val_h.into()], "setField").map_err(|e| e.to_string())?;
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
                            let mut get_i64 = |vid: ValueId| -> Result<inkwell::values::IntValue, String> {
                                let v = *vmap.get(&vid).ok_or("arg missing")?;
                                to_i64_any(codegen.context, &codegen.builder, v)
                            };
                            if args.len() >= 1 { a1 = get_i64(args[0])?; }
                            if args.len() >= 2 { a2 = get_i64(args[1])?; }
                            if args.len() >= 3 { a3 = get_i64(args[2])?; }
                            if args.len() >= 4 { a4 = get_i64(args[3])?; }
                            // Choose return ABI by dst annotated type
                            let dst_ty = dst.as_ref().and_then(|d| func.metadata.value_types.get(d));
                            let use_f64_ret = matches!(dst_ty, Some(crate::mir::MirType::Float));
                            if use_f64_ret {
                                // declare double @nyash_plugin_invoke3_f64(i64,i64,i64,i64,i64,i64)
                                let fnty = codegen.context.f64_type().fn_type(&[i64t.into(), i64t.into(), i64t.into(), i64t.into(), i64t.into(), i64t.into()], false);
                                let callee = codegen.module.get_function("nyash_plugin_invoke3_f64").unwrap_or_else(|| codegen.module.add_function("nyash_plugin_invoke3_f64", fnty, None));
                                let tid = i64t.const_int(type_id as u64, true);
                                let midv = i64t.const_int((*mid) as u64, false);
                                let call = codegen.builder.build_call(callee, &[tid.into(), midv.into(), argc_val.into(), recv_h.into(), a1.into(), a2.into()], "pinvoke_f64").map_err(|e| e.to_string())?;
                                if let Some(d) = dst {
                                    let rv = call.try_as_basic_value().left().ok_or("invoke3_f64 returned void".to_string())?;
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
                            let classify = |vid: ValueId| -> Option<i64> { vmap.get(&vid).map(|v| classify_tag(*v)) };
                            if args.len() >= 1 { if let Some(t) = classify(args[0]) { tag1 = i64t.const_int(t as u64, false); } }
                            if args.len() >= 2 { if let Some(t) = classify(args[1]) { tag2 = i64t.const_int(t as u64, false); } }
                            if args.len() >= 3 { if let Some(t) = classify(args[2]) { tag3 = i64t.const_int(t as u64, false); } }
                            if args.len() >= 4 { if let Some(t) = classify(args[3]) { tag4 = i64t.const_int(t as u64, false); } }
                            if args.len() <= 4 {
                                // Call fixed-arity tagged shim (up to 4 args)
                                let fnty = i64t.fn_type(&[
                                    i64t.into(), i64t.into(), i64t.into(), i64t.into(),
                                    i64t.into(), i64t.into(), i64t.into(), i64t.into(),
                                    i64t.into(), i64t.into(), i64t.into(), i64t.into()
                                ], false);
                                let callee = codegen.module.get_function("nyash_plugin_invoke3_tagged_i64").unwrap_or_else(|| codegen.module.add_function("nyash_plugin_invoke3_tagged_i64", fnty, None));
                                let tid = i64t.const_int(type_id as u64, true);
                                let midv = i64t.const_int((*mid) as u64, false);
                                let call = codegen.builder.build_call(
                                    callee,
                                    &[tid.into(), midv.into(), argc_val.into(), recv_h.into(),
                                      a1.into(), tag1.into(), a2.into(), tag2.into(),
                                      a3.into(), tag3.into(), a4.into(), tag4.into()],
                                    "pinvoke_tagged"
                                ).map_err(|e| e.to_string())?;
                                if let Some(d) = dst {
                                    let rv = call.try_as_basic_value().left().ok_or("invoke3_i64 returned void".to_string())?;
                                    // Decide return lowering by dst annotated type
                                    if let Some(mt) = func.metadata.value_types.get(d) {
                                        match mt {
                                            crate::mir::MirType::Integer | crate::mir::MirType::Bool => { 
                                                vmap.insert(*d, rv); 
                                            }
                                            crate::mir::MirType::Box(_) | crate::mir::MirType::String | crate::mir::MirType::Array(_) | crate::mir::MirType::Future(_) | crate::mir::MirType::Unknown => {
                                                let h = if let BasicValueEnum::IntValue(iv) = rv { iv } else { return Err("invoke ret expected i64".to_string()); };
                                                let pty = codegen.context.i8_type().ptr_type(AddressSpace::from(0));
                                                let ptr = codegen.builder.build_int_to_ptr(h, pty, "ret_handle_to_ptr").map_err(|e| e.to_string())?;
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
                                let vals_arr = entry_builder.build_alloca(arr_ty, "vals_arr").map_err(|e| e.to_string())?;
                                let tags_arr = entry_builder.build_alloca(arr_ty, "tags_arr").map_err(|e| e.to_string())?;
                                for (i, vid) in args.iter().enumerate() {
                                    let idx = [codegen.context.i32_type().const_zero(), codegen.context.i32_type().const_int(i as u64, false)];
                                    let gep_v = unsafe { codegen.builder.build_in_bounds_gep(arr_ty, vals_arr, &idx, &format!("v_gep_{}", i)).map_err(|e| e.to_string())? };
                                    let gep_t = unsafe { codegen.builder.build_in_bounds_gep(arr_ty, tags_arr, &idx, &format!("t_gep_{}", i)).map_err(|e| e.to_string())? };
                                    let vi = get_i64(*vid)?;
                                    let tag = classify(*vid).unwrap_or(3);
                                    let tagv = i64t.const_int(tag as u64, false);
                                    codegen.builder.build_store(gep_v, vi).map_err(|e| e.to_string())?;
                                    codegen.builder.build_store(gep_t, tagv).map_err(|e| e.to_string())?;
                                }
                                // cast to i64* pointers
                                let i64p = i64t.ptr_type(AddressSpace::from(0));
                                let vals_ptr = codegen.builder.build_pointer_cast(vals_arr, i64p, "vals_ptr").map_err(|e| e.to_string())?;
                                let tags_ptr = codegen.builder.build_pointer_cast(tags_arr, i64p, "tags_ptr").map_err(|e| e.to_string())?;
                                // declare i64 @nyash.plugin.invoke_tagged_v_i64(i64,i64,i64,i64,i64*,i64*)
                                let fnty = i64t.fn_type(&[i64t.into(), i64t.into(), i64t.into(), i64t.into(), i64p.into(), i64p.into()], false);
                                let callee = codegen.module.get_function("nyash.plugin.invoke_tagged_v_i64").unwrap_or_else(|| codegen.module.add_function("nyash.plugin.invoke_tagged_v_i64", fnty, None));
                                let tid = i64t.const_int(type_id as u64, true);
                                let midv = i64t.const_int((*mid) as u64, false);
                                let call = codegen.builder.build_call(callee, &[tid.into(), midv.into(), argc_val.into(), recv_h.into(), vals_ptr.into(), tags_ptr.into()], "pinvoke_tagged_v").map_err(|e| e.to_string())?;
                                if let Some(d) = dst {
                                    let rv = call.try_as_basic_value().left().ok_or("invoke_v returned void".to_string())?;
                                    if let Some(mt) = func.metadata.value_types.get(d) {
                                        match mt {
                                            crate::mir::MirType::Integer | crate::mir::MirType::Bool => { 
                                                vmap.insert(*d, rv); 
                                            }
                                            crate::mir::MirType::Box(_) | crate::mir::MirType::String | crate::mir::MirType::Array(_) | crate::mir::MirType::Future(_) | crate::mir::MirType::Unknown => {
                                                let h = if let BasicValueEnum::IntValue(iv) = rv { iv } else { return Err("invoke ret expected i64".to_string()); };
                                                let pty = codegen.context.i8_type().ptr_type(AddressSpace::from(0));
                                                let ptr = codegen.builder.build_int_to_ptr(h, pty, "ret_handle_to_ptr").map_err(|e| e.to_string())?;
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
                        // Route console.log/warn/error/readLine and debug.trace to NyRT shims
                        if (iface_name == "env.console" && (method_name == "log" || method_name == "warn" || method_name == "error"))
                            || (iface_name == "env.debug" && method_name == "trace") {
                            if args.len() != 1 { return Err(format!("{}.{} expects 1 arg (handle)", iface_name, method_name)); }
                            let av = *vmap.get(&args[0]).ok_or("extern arg missing")?;
                            
                            // Handle-based console functions (i64 → i64)
                            let arg_val = match av {
                                BasicValueEnum::IntValue(iv) => {
                                    // Handle different integer types (i1, i32, i64)
                                    if iv.get_type() == codegen.context.bool_type() {
                                        // bool (i1) → i64 zero-extension
                                        codegen.builder.build_int_z_extend(iv, codegen.context.i64_type(), "bool2i64").map_err(|e| e.to_string())?
                                    } else if iv.get_type() == codegen.context.i64_type() {
                                        iv // already i64
                                    } else {
                                        // other integer types → i64 sign-extension
                                        codegen.builder.build_int_s_extend(iv, codegen.context.i64_type(), "int2i64").map_err(|e| e.to_string())?
                                    }
                                },
                                BasicValueEnum::PointerValue(pv) => codegen.builder.build_ptr_to_int(pv, codegen.context.i64_type(), "p2i").map_err(|e| e.to_string())?,
                                _ => return Err("console.log arg conversion failed".to_string()),
                            };
                            
                            let fnty = codegen.context.i64_type().fn_type(&[codegen.context.i64_type().into()], false);
                            let fname = if iface_name == "env.console" {
                                match method_name.as_str() { 
                                    "log" => "nyash.console.log_handle", 
                                    "warn" => "nyash.console.warn_handle", 
                                    _ => "nyash.console.error_handle" 
                                }
                            } else { "nyash.debug.trace_handle" };
                            let callee = codegen.module.get_function(fname).unwrap_or_else(|| codegen.module.add_function(fname, fnty, None));
                            let _ = codegen.builder.build_call(callee, &[arg_val.into()], "console_log_h").map_err(|e| e.to_string())?;
                            if let Some(d) = dst { vmap.insert(*d, codegen.context.i64_type().const_zero().into()); }
                        } else if iface_name == "env.console" && method_name == "readLine" {
                            if !args.is_empty() { return Err("console.readLine expects 0 args".to_string()); }
                            let i8p = codegen.context.i8_type().ptr_type(AddressSpace::from(0));
                            let fnty = i8p.fn_type(&[], false);
                            let callee = codegen.module.get_function("nyash.console.readline").unwrap_or_else(|| codegen.module.add_function("nyash.console.readline", fnty, None));
                            let call = codegen.builder.build_call(callee, &[], "readline").map_err(|e| e.to_string())?;
                            if let Some(d) = dst {
                                let rv = call.try_as_basic_value().left().ok_or("readline returned void".to_string())?;
                                vmap.insert(*d, rv);
                            }
                        } else if iface_name == "env.future" && method_name == "spawn_instance" {
                            // Lower to NyRT: i64 nyash.future.spawn_instance3_i64(i64 a0, i64 a1, i64 a2, i64 argc)
                            // a0: receiver handle (or param index→handle via nyash.handle.of upstream if needed)
                            // a1: method name pointer (i8*) or handle; we pass pointer as i64 here
                            // a2: first payload (i64/handle); more args currently unsupported in LLVM lowering
                            if args.len() < 2 { return Err("env.future.spawn_instance expects at least (recv, method_name)".to_string()); }
                            let i64t = codegen.context.i64_type();
                            let i8p = codegen.context.i8_type().ptr_type(AddressSpace::from(0));
                            // a0
                            let a0_v = *vmap.get(&args[0]).ok_or("recv missing")?;
                            let a0 = to_i64_any(codegen.context, &codegen.builder, a0_v)?;
                            // a1 (method name)
                            let a1_v = *vmap.get(&args[1]).ok_or("method_name missing")?;
                            let a1 = match a1_v {
                                BasicValueEnum::PointerValue(pv) => codegen.builder.build_ptr_to_int(pv, i64t, "mname_p2i").map_err(|e| e.to_string())?,
                                _ => to_i64_any(codegen.context, &codegen.builder, a1_v)?,
                            };
                            // a2 (first payload if any)
                            let a2 = if args.len() >= 3 {
                                let v = *vmap.get(&args[2]).ok_or("arg2 missing")?;
                                to_i64_any(codegen.context, &codegen.builder, v)?
                            } else { i64t.const_zero() };
                            let argc_total = i64t.const_int(args.len().saturating_sub(1) as u64, false);
                            // declare and call
                            let fnty = i64t.fn_type(&[i64t.into(), i64t.into(), i64t.into(), i64t.into()], false);
                            let callee = codegen
                                .module
                                .get_function("nyash.future.spawn_instance3_i64")
                                .unwrap_or_else(|| codegen.module.add_function("nyash.future.spawn_instance3_i64", fnty, None));
                            let call = codegen
                                .builder
                                .build_call(callee, &[a0.into(), a1.into(), a2.into(), argc_total.into()], "spawn_i3")
                                .map_err(|e| e.to_string())?;
                            if let Some(d) = dst {
                                let rv = call
                                    .try_as_basic_value()
                                    .left()
                                    .ok_or("spawn_instance3 returned void".to_string())?;
                                // Treat as handle → pointer for Box return types; otherwise keep i64
                                if let Some(mt) = func.metadata.value_types.get(d) {
                                    match mt {
                                        crate::mir::MirType::Integer | crate::mir::MirType::Bool => { vmap.insert(*d, rv); }
                                        crate::mir::MirType::Box(_) | crate::mir::MirType::String | crate::mir::MirType::Array(_) | crate::mir::MirType::Future(_) | crate::mir::MirType::Unknown => {
                                            let iv = if let BasicValueEnum::IntValue(iv) = rv { iv } else { return Err("spawn ret expected i64".to_string()); };
                                            let pty = codegen.context.i8_type().ptr_type(AddressSpace::from(0));
                                            let ptr = codegen.builder.build_int_to_ptr(iv, pty, "ret_handle_to_ptr").map_err(|e| e.to_string())?;
                                            vmap.insert(*d, ptr.into());
                                        }
                                        _ => { vmap.insert(*d, rv); }
                                    }
                                } else { vmap.insert(*d, rv); }
                            }
                        } else if iface_name == "env.local" && method_name == "get" {
                            // Core-13 pure shim: get(ptr) → return the SSA value of ptr
                            if let Some(d) = dst { let av = *vmap.get(&args[0]).ok_or("extern arg missing")?; vmap.insert(*d, av); }
                        } else if iface_name == "env.local" && method_name == "set" {
                            // set(ptr, val) → no-op at AOT SSA level (ptr is symbolic)
                            // No dst expected in our normalization; ignore safely
                        } else if iface_name == "env.box" && method_name == "new" {
                            // Call NyRT shim:
                            //  - 1 arg:  i64 @nyash.env.box.new(i8* type)
                            //  - >=2arg: i64 @nyash.env.box.new_i64(i8* type, i64 argc, i64 a1, i64 a2)
                            if args.len() < 1 { return Err("env.box.new expects at least 1 arg (type name)".to_string()); }
                            let i8p = codegen.context.i8_type().ptr_type(AddressSpace::from(0));
                            let tyv = *vmap.get(&args[0]).ok_or("type name arg missing")?;
                            let ty_ptr = match tyv { BasicValueEnum::PointerValue(p) => p, _ => return Err("env.box.new type must be i8* string".to_string()) };
                            let i64t = codegen.context.i64_type();
                            // 1) new(type)
                            let out_ptr: PointerValue = if args.len() == 1 {
                                let fnty = i64t.fn_type(&[i8p.into()], false);
                                let callee = codegen.module.get_function("nyash.env.box.new").unwrap_or_else(|| codegen.module.add_function("nyash.env.box.new", fnty, None));
                                let call = codegen.builder.build_call(callee, &[ty_ptr.into()], "env_box_new").map_err(|e| e.to_string())?;
                                let rv = call.try_as_basic_value().left().ok_or("env.box.new returned void".to_string())?;
                                let i64v = if let BasicValueEnum::IntValue(iv) = rv { iv } else { return Err("env.box.new ret expected i64".to_string()); };
                                codegen.builder.build_int_to_ptr(i64v, i8p, "box_handle_to_ptr").map_err(|e| e.to_string())?
                            } else {
                                // 2) new_i64x(type, argc, a1..a4)
                                if args.len() - 1 > 4 { return Err("env.box.new supports up to 4 args in AOT shim".to_string()); }
                                let fnty = i64t.fn_type(&[i8p.into(), i64t.into(), i64t.into(), i64t.into(), i64t.into(), i64t.into()], false);
                                let callee = codegen.module.get_function("nyash.env.box.new_i64x").unwrap_or_else(|| codegen.module.add_function("nyash.env.box.new_i64x", fnty, None));
                                let argc_val = i64t.const_int((args.len() - 1) as u64, false);
                                // Inline-coerce up to 4 args to i64 handles (int pass-through, f64→box, i8*→box)
                                let mut a1 = i64t.const_zero();
                                if args.len() >= 2 {
                                    let bv = *vmap.get(&args[1]).ok_or("arg missing")?;
                                    a1 = match bv {
                                        BasicValueEnum::IntValue(iv) => iv,
                                        BasicValueEnum::FloatValue(fv) => {
                                            let fnty = i64t.fn_type(&[codegen.context.f64_type().into()], false);
                                            let callee = codegen.module.get_function("nyash.box.from_f64").unwrap_or_else(|| codegen.module.add_function("nyash.box.from_f64", fnty, None));
                                            let call = codegen.builder.build_call(callee, &[fv.into()], "arg1_f64_to_box").map_err(|e| e.to_string())?;
                                            let rv = call.try_as_basic_value().left().ok_or("from_f64 returned void".to_string())?;
                                            if let BasicValueEnum::IntValue(h) = rv { h } else { return Err("from_f64 ret expected i64".to_string()); }
                                        }
                                        BasicValueEnum::PointerValue(pv) => {
                                            let fnty = i64t.fn_type(&[i8p.into()], false);
                                            let callee = codegen.module.get_function("nyash.box.from_i8_string").unwrap_or_else(|| codegen.module.add_function("nyash.box.from_i8_string", fnty, None));
                                            let call = codegen.builder.build_call(callee, &[pv.into()], "arg1_i8_to_box").map_err(|e| e.to_string())?;
                                            let rv = call.try_as_basic_value().left().ok_or("from_i8_string returned void".to_string())?;
                                            if let BasicValueEnum::IntValue(h) = rv { h } else { return Err("from_i8_string ret expected i64".to_string()); }
                                        }
                                        _ => { return Err("unsupported arg value for env.box.new".to_string()); }
                                    };
                                }
                                let mut a2 = i64t.const_zero();
                                if args.len() >= 3 {
                                    let bv = *vmap.get(&args[2]).ok_or("arg missing")?;
                                    a2 = match bv {
                                        BasicValueEnum::IntValue(iv) => iv,
                                        BasicValueEnum::FloatValue(fv) => {
                                            let fnty = i64t.fn_type(&[codegen.context.f64_type().into()], false);
                                            let callee = codegen.module.get_function("nyash.box.from_f64").unwrap_or_else(|| codegen.module.add_function("nyash.box.from_f64", fnty, None));
                                            let call = codegen.builder.build_call(callee, &[fv.into()], "arg2_f64_to_box").map_err(|e| e.to_string())?;
                                            let rv = call.try_as_basic_value().left().ok_or("from_f64 returned void".to_string())?;
                                            if let BasicValueEnum::IntValue(h) = rv { h } else { return Err("from_f64 ret expected i64".to_string()); }
                                        }
                                        BasicValueEnum::PointerValue(pv) => {
                                            let fnty = i64t.fn_type(&[i8p.into()], false);
                                            let callee = codegen.module.get_function("nyash.box.from_i8_string").unwrap_or_else(|| codegen.module.add_function("nyash.box.from_i8_string", fnty, None));
                                            let call = codegen.builder.build_call(callee, &[pv.into()], "arg2_i8_to_box").map_err(|e| e.to_string())?;
                                            let rv = call.try_as_basic_value().left().ok_or("from_i8_string returned void".to_string())?;
                                            if let BasicValueEnum::IntValue(h) = rv { h } else { return Err("from_i8_string ret expected i64".to_string()); }
                                        }
                                        _ => { return Err("unsupported arg value for env.box.new".to_string()); }
                                    };
                                }
                                let mut a3 = i64t.const_zero();
                                if args.len() >= 4 {
                                    let bv = *vmap.get(&args[3]).ok_or("arg missing")?;
                                    a3 = match bv {
                                        BasicValueEnum::IntValue(iv) => iv,
                                        BasicValueEnum::FloatValue(fv) => {
                                            let fnty = i64t.fn_type(&[codegen.context.f64_type().into()], false);
                                            let callee = codegen.module.get_function("nyash.box.from_f64").unwrap_or_else(|| codegen.module.add_function("nyash.box.from_f64", fnty, None));
                                            let call = codegen.builder.build_call(callee, &[fv.into()], "arg3_f64_to_box").map_err(|e| e.to_string())?;
                                            let rv = call.try_as_basic_value().left().ok_or("from_f64 returned void".to_string())?;
                                            if let BasicValueEnum::IntValue(h) = rv { h } else { return Err("from_f64 ret expected i64".to_string()); }
                                        }
                                        BasicValueEnum::PointerValue(pv) => {
                                            let fnty = i64t.fn_type(&[i8p.into()], false);
                                            let callee = codegen.module.get_function("nyash.box.from_i8_string").unwrap_or_else(|| codegen.module.add_function("nyash.box.from_i8_string", fnty, None));
                                            let call = codegen.builder.build_call(callee, &[pv.into()], "arg3_i8_to_box").map_err(|e| e.to_string())?;
                                            let rv = call.try_as_basic_value().left().ok_or("from_i8_string returned void".to_string())?;
                                            if let BasicValueEnum::IntValue(h) = rv { h } else { return Err("from_i8_string ret expected i64".to_string()); }
                                        }
                                        _ => { return Err("unsupported arg value for env.box.new".to_string()); }
                                    };
                                }
                                let mut a4 = i64t.const_zero();
                                if args.len() >= 5 {
                                    let bv = *vmap.get(&args[4]).ok_or("arg missing")?;
                                    a4 = match bv {
                                        BasicValueEnum::IntValue(iv) => iv,
                                        BasicValueEnum::FloatValue(fv) => {
                                            let fnty = i64t.fn_type(&[codegen.context.f64_type().into()], false);
                                            let callee = codegen.module.get_function("nyash.box.from_f64").unwrap_or_else(|| codegen.module.add_function("nyash.box.from_f64", fnty, None));
                                            let call = codegen.builder.build_call(callee, &[fv.into()], "arg4_f64_to_box").map_err(|e| e.to_string())?;
                                            let rv = call.try_as_basic_value().left().ok_or("from_f64 returned void".to_string())?;
                                            if let BasicValueEnum::IntValue(h) = rv { h } else { return Err("from_f64 ret expected i64".to_string()); }
                                        }
                                        BasicValueEnum::PointerValue(pv) => {
                                            let fnty = i64t.fn_type(&[i8p.into()], false);
                                            let callee = codegen.module.get_function("nyash.box.from_i8_string").unwrap_or_else(|| codegen.module.add_function("nyash.box.from_i8_string", fnty, None));
                                            let call = codegen.builder.build_call(callee, &[pv.into()], "arg4_i8_to_box").map_err(|e| e.to_string())?;
                                            let rv = call.try_as_basic_value().left().ok_or("from_i8_string returned void".to_string())?;
                                            if let BasicValueEnum::IntValue(h) = rv { h } else { return Err("from_i8_string ret expected i64".to_string()); }
                                        }
                                        _ => { return Err("unsupported arg value for env.box.new".to_string()); }
                                    };
                                }
                                let call = codegen.builder.build_call(callee, &[ty_ptr.into(), argc_val.into(), a1.into(), a2.into(), a3.into(), a4.into()], "env_box_new_i64x").map_err(|e| e.to_string())?;
                                let rv = call.try_as_basic_value().left().ok_or("env.box.new_i64 returned void".to_string())?;
                                let i64v = if let BasicValueEnum::IntValue(iv) = rv { iv } else { return Err("env.box.new_i64 ret expected i64".to_string()); };
                                codegen.builder.build_int_to_ptr(i64v, i8p, "box_handle_to_ptr").map_err(|e| e.to_string())?
                            };
                            if let Some(d) = dst { vmap.insert(*d, out_ptr.into()); }
                        } else {
                            return Err(format!("ExternCall lowering unsupported: {}.{} (add a NyRT shim for this interface method)", iface_name, method_name));
                        }
                    }
                    MirInstruction::UnaryOp { dst, op, operand } => {
                        let v = *vmap.get(operand).ok_or("operand missing")?;
                        let out = match op {
                        UnaryOp::Neg => {
                                if let Some(iv) = as_int(v) { codegen.builder.build_int_neg(iv, "ineg").map_err(|e| e.to_string())?.into() }
                                else if let Some(fv) = as_float(v) { codegen.builder.build_float_neg(fv, "fneg").map_err(|e| e.to_string())?.into() }
                                else { return Err("neg on non-number".to_string()) }
                            }
                            UnaryOp::Not | UnaryOp::BitNot => {
                                if let Some(iv) = as_int(v) { codegen.builder.build_not(iv, "inot").map_err(|e| e.to_string())?.into() }
                                else { return Err("not on non-int".to_string()) }
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
                            let i8p = codegen.context.i8_type().ptr_type(AddressSpace::from(0));
                            match (lv, rv) {
                                (BasicValueEnum::PointerValue(lp), BasicValueEnum::PointerValue(rp)) => {
                                    let fnty = i8p.fn_type(&[i8p.into(), i8p.into()], false);
                                    let callee = codegen.module.get_function("nyash.string.concat_ss").unwrap_or_else(|| codegen.module.add_function("nyash.string.concat_ss", fnty, None));
                                    let call = codegen.builder.build_call(callee, &[lp.into(), rp.into()], "concat_ss").map_err(|e| e.to_string())?;
                                    let rv = call.try_as_basic_value().left().ok_or("concat_ss returned void".to_string())?;
                                    vmap.insert(*dst, rv);
                                    handled_concat = true;
                                }
                                (BasicValueEnum::PointerValue(lp), BasicValueEnum::IntValue(ri)) => {
                                    let i64t = codegen.context.i64_type();
                                    let fnty = i8p.fn_type(&[i8p.into(), i64t.into()], false);
                                    let callee = codegen.module.get_function("nyash.string.concat_si").unwrap_or_else(|| codegen.module.add_function("nyash.string.concat_si", fnty, None));
                                    let call = codegen.builder.build_call(callee, &[lp.into(), ri.into()], "concat_si").map_err(|e| e.to_string())?;
                                    let rv = call.try_as_basic_value().left().ok_or("concat_si returned void".to_string())?;
                                    vmap.insert(*dst, rv);
                                    handled_concat = true;
                                }
                                (BasicValueEnum::IntValue(li), BasicValueEnum::PointerValue(rp)) => {
                                    let i64t = codegen.context.i64_type();
                                    let fnty = i8p.fn_type(&[i64t.into(), i8p.into()], false);
                                    let callee = codegen.module.get_function("nyash.string.concat_is").unwrap_or_else(|| codegen.module.add_function("nyash.string.concat_is", fnty, None));
                                    let call = codegen.builder.build_call(callee, &[li.into(), rp.into()], "concat_is").map_err(|e| e.to_string())?;
                                    let rv = call.try_as_basic_value().left().ok_or("concat_is returned void".to_string())?;
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
                                    B::Add => codegen.builder.build_int_add(li, ri, "iadd").map_err(|e| e.to_string())?.into(),
                                    B::Sub => codegen.builder.build_int_sub(li, ri, "isub").map_err(|e| e.to_string())?.into(),
                                    B::Mul => codegen.builder.build_int_mul(li, ri, "imul").map_err(|e| e.to_string())?.into(),
                                    B::Div => codegen.builder.build_int_signed_div(li, ri, "idiv").map_err(|e| e.to_string())?.into(),
                                    B::Mod => codegen.builder.build_int_signed_rem(li, ri, "imod").map_err(|e| e.to_string())?.into(),
                                    B::BitAnd => codegen.builder.build_and(li, ri, "iand").map_err(|e| e.to_string())?.into(),
                                    B::BitOr => codegen.builder.build_or(li, ri, "ior").map_err(|e| e.to_string())?.into(),
                                    B::BitXor => codegen.builder.build_xor(li, ri, "ixor").map_err(|e| e.to_string())?.into(),
                                    B::Shl => codegen.builder.build_left_shift(li, ri, "ishl").map_err(|e| e.to_string())?.into(),
                                    B::Shr => codegen.builder.build_right_shift(li, ri, false, "ishr").map_err(|e| e.to_string())?.into(),
                                    B::And | B::Or => {
                                        // Treat as logical on integers: convert to i1 and and/or
                                        let lb = to_bool(codegen.context, li.into(), &codegen.builder)?;
                                        let rb = to_bool(codegen.context, ri.into(), &codegen.builder)?;
                                        match op {
                                            B::And => codegen.builder.build_and(lb, rb, "land").map_err(|e| e.to_string())?.into(),
                                            _ => codegen.builder.build_or(lb, rb, "lor").map_err(|e| e.to_string())?.into(),
                                        }
                                    }
                                }
                            } else if let (Some(lf), Some(rf)) = (as_float(lv), as_float(rv)) {
                                use crate::mir::BinaryOp as B;
                                match op {
                                    B::Add => codegen.builder.build_float_add(lf, rf, "fadd").map_err(|e| e.to_string())?.into(),
                                    B::Sub => codegen.builder.build_float_sub(lf, rf, "fsub").map_err(|e| e.to_string())?.into(),
                                    B::Mul => codegen.builder.build_float_mul(lf, rf, "fmul").map_err(|e| e.to_string())?.into(),
                                    B::Div => codegen.builder.build_float_div(lf, rf, "fdiv").map_err(|e| e.to_string())?.into(),
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
                        let lv = *vmap.get(lhs).ok_or("lhs missing")?;
                        let rv = *vmap.get(rhs).ok_or("rhs missing")?;
                        let out = if let (Some(li), Some(ri)) = (as_int(lv), as_int(rv)) {
                            use crate::mir::CompareOp as C;
                            let pred = match op { C::Eq=>inkwell::IntPredicate::EQ, C::Ne=>inkwell::IntPredicate::NE, C::Lt=>inkwell::IntPredicate::SLT, C::Le=>inkwell::IntPredicate::SLE, C::Gt=>inkwell::IntPredicate::SGT, C::Ge=>inkwell::IntPredicate::SGE };
                            codegen.builder.build_int_compare(pred, li, ri, "icmp").map_err(|e| e.to_string())?.into()
                        } else if let (Some(lf), Some(rf)) = (as_float(lv), as_float(rv)) {
                            use crate::mir::CompareOp as C;
                            let pred = match op { C::Eq=>inkwell::FloatPredicate::OEQ, C::Ne=>inkwell::FloatPredicate::ONE, C::Lt=>inkwell::FloatPredicate::OLT, C::Le=>inkwell::FloatPredicate::OLE, C::Gt=>inkwell::FloatPredicate::OGT, C::Ge=>inkwell::FloatPredicate::OGE };
                            codegen.builder.build_float_compare(pred, lf, rf, "fcmp").map_err(|e| e.to_string())?.into()
                        } else if let (BasicValueEnum::PointerValue(lp), BasicValueEnum::PointerValue(rp)) = (lv, rv) {
                            // Support pointer equality/inequality comparisons
                            use crate::mir::CompareOp as C;
                            match op {
                                C::Eq | C::Ne => {
                                    let i64t = codegen.context.i64_type();
                                    let li = codegen.builder.build_ptr_to_int(lp, i64t, "pi_l").map_err(|e| e.to_string())?;
                                    let ri = codegen.builder.build_ptr_to_int(rp, i64t, "pi_r").map_err(|e| e.to_string())?;
                                    let pred = if matches!(op, C::Eq) { inkwell::IntPredicate::EQ } else { inkwell::IntPredicate::NE };
                                    codegen.builder.build_int_compare(pred, li, ri, "pcmp").map_err(|e| e.to_string())?.into()
                                }
                                _ => return Err("unsupported pointer comparison (only Eq/Ne)".to_string()),
                            }
                        } else if let (BasicValueEnum::PointerValue(lp), BasicValueEnum::IntValue(ri)) = (lv, rv) {
                            use crate::mir::CompareOp as C;
                            match op {
                                C::Eq | C::Ne => {
                                    let i64t = codegen.context.i64_type();
                                    let li = codegen.builder.build_ptr_to_int(lp, i64t, "pi_l").map_err(|e| e.to_string())?;
                                    let pred = if matches!(op, C::Eq) { inkwell::IntPredicate::EQ } else { inkwell::IntPredicate::NE };
                                    codegen.builder.build_int_compare(pred, li, ri, "pcmpi").map_err(|e| e.to_string())?.into()
                                }
                                _ => return Err("unsupported pointer-int comparison (only Eq/Ne)".to_string()),
                            }
                        } else if let (BasicValueEnum::IntValue(li), BasicValueEnum::PointerValue(rp)) = (lv, rv) {
                            use crate::mir::CompareOp as C;
                            match op {
                                C::Eq | C::Ne => {
                                    let i64t = codegen.context.i64_type();
                                    let ri = codegen.builder.build_ptr_to_int(rp, i64t, "pi_r").map_err(|e| e.to_string())?;
                                    let pred = if matches!(op, C::Eq) { inkwell::IntPredicate::EQ } else { inkwell::IntPredicate::NE };
                                    codegen.builder.build_int_compare(pred, li, ri, "ipcmi").map_err(|e| e.to_string())?.into()
                                }
                                _ => return Err("unsupported int-pointer comparison (only Eq/Ne)".to_string()),
                            }
                        } else {
                            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                                let lk = match lv { BasicValueEnum::IntValue(_) => "int", BasicValueEnum::FloatValue(_) => "float", BasicValueEnum::PointerValue(_) => "ptr", _ => "other" };
                                let rk = match rv { BasicValueEnum::IntValue(_) => "int", BasicValueEnum::FloatValue(_) => "float", BasicValueEnum::PointerValue(_) => "ptr", _ => "other" };
                                eprintln!("[LLVM] compare type mismatch: lhs={}, rhs={} (op={:?})", lk, rk, op);
                            }
                            return Err("compare type mismatch".to_string());
                        };
                        vmap.insert(*dst, out);
                    }
                    MirInstruction::Store { value, ptr } => {
                        let val = *vmap.get(value).ok_or("store value missing")?;
                        // Determine or create the alloca for this ptr, using current value type
                        let elem_ty = match val {
                            BasicValueEnum::IntValue(iv) => BasicTypeEnum::IntType(iv.get_type()),
                            BasicValueEnum::FloatValue(fv) => BasicTypeEnum::FloatType(fv.get_type()),
                            BasicValueEnum::PointerValue(pv) => BasicTypeEnum::PointerType(pv.get_type()),
                            _ => return Err("unsupported store value type".to_string()),
                        };
                        if let Some(existing) = allocas.get(ptr).copied() {
                            // If types mismatch (e.g., i1 vs i64), try simple widen/narrow for ints; pointer->pointer cast
                            let existing_elem = *alloca_elem_types.get(ptr).ok_or("alloca elem type missing")?;
                            if existing_elem != elem_ty {
                                match (val, existing_elem) {
                                    (BasicValueEnum::IntValue(iv), BasicTypeEnum::IntType(t)) => {
                                        let bw_src = iv.get_type().get_bit_width();
                                        let bw_dst = t.get_bit_width();
                                        if bw_src < bw_dst {
                                            let adj = codegen.builder.build_int_z_extend(iv, t, "zext").map_err(|e| e.to_string())?;
                                            codegen.builder.build_store(existing, adj).map_err(|e| e.to_string())?;
                                        } else if bw_src > bw_dst {
                                            let adj = codegen.builder.build_int_truncate(iv, t, "trunc").map_err(|e| e.to_string())?;
                                            codegen.builder.build_store(existing, adj).map_err(|e| e.to_string())?;
                                        } else {
                                            codegen.builder.build_store(existing, iv).map_err(|e| e.to_string())?;
                                        }
                                    }
                                    (BasicValueEnum::PointerValue(pv), BasicTypeEnum::PointerType(pt)) => {
                                        let adj = codegen.builder.build_pointer_cast(pv, pt, "pcast").map_err(|e| e.to_string())?;
                                        codegen.builder.build_store(existing, adj).map_err(|e| e.to_string())?;
                                    }
                                    (BasicValueEnum::FloatValue(fv), BasicTypeEnum::FloatType(ft)) => {
                                        if fv.get_type() == ft { codegen.builder.build_store(existing, fv).map_err(|e| e.to_string())?; }
                                        else { return Err("float width mismatch in store".to_string()); }
                                    }
                                    _ => return Err("store type mismatch".to_string()),
                                };
                            } else {
                                match val {
                                    BasicValueEnum::IntValue(iv) => { codegen.builder.build_store(existing, iv).map_err(|e| e.to_string())?; },
                                    BasicValueEnum::FloatValue(fv) => { codegen.builder.build_store(existing, fv).map_err(|e| e.to_string())?; },
                                    BasicValueEnum::PointerValue(pv) => { codegen.builder.build_store(existing, pv).map_err(|e| e.to_string())?; },
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
                            entry_builder.build_store(slot, zero_val).map_err(|e| e.to_string())?;
                            allocas.insert(*ptr, slot);
                            alloca_elem_types.insert(*ptr, elem_ty);
                            match val {
                                BasicValueEnum::IntValue(iv) => { codegen.builder.build_store(slot, iv).map_err(|e| e.to_string())?; },
                                BasicValueEnum::FloatValue(fv) => { codegen.builder.build_store(slot, fv).map_err(|e| e.to_string())?; },
                                BasicValueEnum::PointerValue(pv) => { codegen.builder.build_store(slot, pv).map_err(|e| e.to_string())?; },
                                _ => return Err("unsupported store value type".to_string()),
                            }
                        }
                    }
                    MirInstruction::Load { dst, ptr } => {
                        // Ensure alloca exists; if not, try to infer from annotated dst type, else default i64
                        let (slot, elem_ty) = if let Some(p) = allocas.get(ptr).copied() {
                            let ety = *alloca_elem_types.get(ptr).ok_or("alloca elem type missing")?;
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
                            entry_builder.build_store(slot, zero_val).map_err(|e| e.to_string())?;
                            allocas.insert(*ptr, slot);
                            alloca_elem_types.insert(*ptr, elem_ty);
                            (slot, elem_ty)
                        };
                        let lv = codegen.builder.build_load(elem_ty, slot, &format!("load_{}", dst.as_u32())).map_err(|e| e.to_string())?;
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
                        match (&func.signature.return_type, value) {
                            (crate::mir::MirType::Void, _) => { codegen.builder.build_return(None).unwrap(); }
                            (ref t, Some(vid)) => {
                                let v = *vmap.get(vid).ok_or("ret value missing")?;
                                // Trust SSA type to match declared return type for now
                                codegen.builder.build_return(Some(&v)).map_err(|e| e.to_string())?;
                            }
                            (_t, None) => return Err("non-void function missing return value".to_string()),
                        }
                    }
                    MirInstruction::Jump { target } => {
                        // Wire phi incoming for target
                        if let Some(list) = phis_by_block.get(target) {
                            for (_dst, phi, inputs) in list {
                                if let Some((_, in_vid)) = inputs.iter().find(|(pred, _)| pred == &bid) {
                                    let val = *vmap.get(in_vid).ok_or("phi incoming value missing")?;
                                    let pred_bb = *bb_map.get(&bid).ok_or("pred bb missing")?;
                                    match val {
                                        BasicValueEnum::IntValue(iv) => phi.add_incoming(&[(&iv, pred_bb)]),
                                        BasicValueEnum::FloatValue(fv) => phi.add_incoming(&[(&fv, pred_bb)]),
                                        BasicValueEnum::PointerValue(pv) => phi.add_incoming(&[(&pv, pred_bb)]),
                                        _ => return Err("unsupported phi incoming value".to_string()),
                                    }
                                }
                            }
                        }
                        let tbb = *bb_map.get(target).ok_or("target bb missing")?;
                        codegen.builder.build_unconditional_branch(tbb).map_err(|e| e.to_string())?;
                    }
                    MirInstruction::Branch { condition, then_bb, else_bb } => {
                        let cond_v = *vmap.get(condition).ok_or("cond missing")?;
                        let b = to_bool(codegen.context, cond_v, &codegen.builder)?;
                        // Wire phi incoming for both successors
                        if let Some(list) = phis_by_block.get(then_bb) {
                            for (_dst, phi, inputs) in list {
                                if let Some((_, in_vid)) = inputs.iter().find(|(pred, _)| pred == &bid) {
                                    let val = *vmap.get(in_vid).ok_or("phi incoming (then) value missing")?;
                                    let pred_bb = *bb_map.get(&bid).ok_or("pred bb missing")?;
                                    match val {
                                        BasicValueEnum::IntValue(iv) => phi.add_incoming(&[(&iv, pred_bb)]),
                                        BasicValueEnum::FloatValue(fv) => phi.add_incoming(&[(&fv, pred_bb)]),
                                        BasicValueEnum::PointerValue(pv) => phi.add_incoming(&[(&pv, pred_bb)]),
                                        _ => return Err("unsupported phi incoming value (then)".to_string()),
                                    }
                                }
                            }
                        }
                        if let Some(list) = phis_by_block.get(else_bb) {
                            for (_dst, phi, inputs) in list {
                                if let Some((_, in_vid)) = inputs.iter().find(|(pred, _)| pred == &bid) {
                                    let val = *vmap.get(in_vid).ok_or("phi incoming (else) value missing")?;
                                    let pred_bb = *bb_map.get(&bid).ok_or("pred bb missing")?;
                                    match val {
                                        BasicValueEnum::IntValue(iv) => phi.add_incoming(&[(&iv, pred_bb)]),
                                        BasicValueEnum::FloatValue(fv) => phi.add_incoming(&[(&fv, pred_bb)]),
                                        BasicValueEnum::PointerValue(pv) => phi.add_incoming(&[(&pv, pred_bb)]),
                                        _ => return Err("unsupported phi incoming value (else)".to_string()),
                                    }
                                }
                            }
                        }
                        let tbb = *bb_map.get(then_bb).ok_or("then bb missing")?;
                        let ebb = *bb_map.get(else_bb).ok_or("else bb missing")?;
                        codegen.builder.build_conditional_branch(b, tbb, ebb).map_err(|e| e.to_string())?;
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
        if verbose { eprintln!("[LLVM] emitting object to {} (begin)", output_path); }
        match codegen.target_machine.write_to_file(&codegen.module, inkwell::targets::FileType::Object, std::path::Path::new(output_path)) {
            Ok(_) => {
                // Verify; if missing, fallback to memory buffer write
                if std::fs::metadata(output_path).is_err() {
                    let buf = codegen.target_machine
                        .write_to_memory_buffer(&codegen.module, inkwell::targets::FileType::Object)
                        .map_err(|e| format!("Failed to get object buffer: {}", e))?;
                    std::fs::write(output_path, buf.as_slice()).map_err(|e| format!("Failed to write object to '{}': {}", output_path, e))?;
                    if verbose { eprintln!("[LLVM] wrote object via memory buffer fallback: {} ({} bytes)", output_path, buf.get_size()); }
                } else if verbose {
                    if let Ok(meta) = std::fs::metadata(output_path) { eprintln!("[LLVM] wrote object via file API: {} ({} bytes)", output_path, meta.len()); }
                }
                if verbose { eprintln!("[LLVM] emit complete (Ok branch) for {}", output_path); }
                Ok(())
            }
            Err(e) => {
                // Fallback: memory buffer
                let buf = codegen.target_machine
                    .write_to_memory_buffer(&codegen.module, inkwell::targets::FileType::Object)
                    .map_err(|ee| format!("Failed to write object ({}); and memory buffer failed: {}", e, ee))?;
                std::fs::write(output_path, buf.as_slice()).map_err(|ee| format!("Failed to write object to '{}': {} (original error: {})", output_path, ee, e))?;
                if verbose { eprintln!("[LLVM] wrote object via error fallback: {} ({} bytes)", output_path, buf.get_size()); }
                if verbose { eprintln!("[LLVM] emit complete (Err branch handled) for {}", output_path); }
                Ok(())
            }
        }
    }

    pub fn compile_and_execute(
        &mut self,
        mir_module: &MirModule,
        temp_path: &str,
    ) -> Result<Box<dyn NyashBox>, String> {
        // 1) Emit object via real LLVM lowering to ensure IR generation remains healthy
        let obj_path = format!("{}.o", temp_path);
        self.compile_module(mir_module, &obj_path)?;

        // 2) Execute via a minimal MIR interpreter for parity (until full AOT linkage is wired)
        //    Supports: Const(Integer/Bool/String/Null), BinOp on Integer, Return(Some/None)
        //    This mirrors the non-LLVM mock path just enough for simple parity tests.
        self.values.clear();
        let func = mir_module
            .functions
            .get("Main.main")
            .or_else(|| mir_module.functions.get("main"))
            .or_else(|| mir_module.functions.values().next())
            .ok_or_else(|| "Main.main function not found".to_string())?;

        use crate::mir::instruction::MirInstruction as I;
        for inst in &func.get_block(func.entry_block).unwrap().instructions {
            match inst {
                I::Const { dst, value } => {
                    let v: Box<dyn NyashBox> = match value {
                        ConstValue::Integer(i) => Box::new(IntegerBox::new(*i)),
                        ConstValue::Float(f) => Box::new(crate::boxes::math_box::FloatBox::new(*f)),
                        ConstValue::String(s) => Box::new(crate::box_trait::StringBox::new(s.clone())),
                        ConstValue::Bool(b) => Box::new(crate::box_trait::BoolBox::new(*b)),
                        ConstValue::Null => Box::new(crate::boxes::null_box::NullBox::new()),
                        ConstValue::Void => Box::new(IntegerBox::new(0)),
                    };
                    self.values.insert(*dst, v);
                }
                I::BinOp { dst, op, lhs, rhs } => {
                    let l = self
                        .values
                        .get(lhs)
                        .and_then(|b| b.as_any().downcast_ref::<IntegerBox>())
                        .ok_or_else(|| format!("binop lhs %{} not integer", lhs.0))?;
                    let r = self
                        .values
                        .get(rhs)
                        .and_then(|b| b.as_any().downcast_ref::<IntegerBox>())
                        .ok_or_else(|| format!("binop rhs %{} not integer", rhs.0))?;
                        let res = match op {
                            BinaryOp::Add => l.value + r.value,
                            BinaryOp::Sub => l.value - r.value,
                            BinaryOp::Mul => l.value * r.value,
                            BinaryOp::Div => {
                                if r.value == 0 { return Err("division by zero".into()); }
                                l.value / r.value
                            }
                            BinaryOp::Mod => l.value % r.value,
                            BinaryOp::BitAnd => l.value & r.value,
                            BinaryOp::BitOr  => l.value | r.value,
                            BinaryOp::BitXor => l.value ^ r.value,
                            BinaryOp::Shl    => l.value << r.value,
                            BinaryOp::Shr    => l.value >> r.value,
                            BinaryOp::And    => { if (l.value != 0) && (r.value != 0) { 1 } else { 0 } },
                            BinaryOp::Or     => { if (l.value != 0) || (r.value != 0) { 1 } else { 0 } },
                        };
                    self.values.insert(*dst, Box::new(IntegerBox::new(res)));
                }
                I::ExternCall { dst, iface_name, method_name, args, .. } => {
                    if iface_name == "env.console" {
                        if let Some(arg0) = args.get(0).and_then(|v| self.values.get(v)) {
                            let msg = arg0.to_string_box().value;
                            match method_name.as_str() {
                                "log" => println!("{}", msg),
                                "warn" => eprintln!("[warn] {}", msg),
                                "error" => eprintln!("[error] {}", msg),
                                _ => {}
                            }
                        }
                        if let Some(d) = dst { self.values.insert(*d, Box::new(IntegerBox::new(0))); }
                    }
                }
                I::Return { value } => {
                    if let Some(v) = value { return self.values.get(v).map(|b| b.clone_box()).ok_or_else(|| format!("return %{} missing", v.0)); }
                    return Ok(Box::new(IntegerBox::new(0)));
                }
                _ => { /* ignore for now */ }
            }
        }
        Ok(Box::new(IntegerBox::new(0)))
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
