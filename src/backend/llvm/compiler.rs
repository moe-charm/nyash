/*!
 * LLVM Compiler Implementation - Compile MIR to LLVM IR and native code
 */

use crate::mir::function::MirModule;
use crate::mir::instruction::{MirInstruction, ConstValue, BinaryOp, UnaryOp, CompareOp};
use crate::mir::ValueId;
use crate::box_trait::{NyashBox, IntegerBox, StringBox, BoolBox};
use crate::boxes::math_box::FloatBox;
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
                                BinaryOp::Add => l.value() + r.value(),
                                BinaryOp::Sub => l.value() - r.value(),
                                BinaryOp::Mul => l.value() * r.value(),
                                BinaryOp::Div => {
                                    if r.value() == 0 {
                                        return Err("Division by zero".to_string());
                                    }
                                    l.value() / r.value()
                                }
                                BinaryOp::Mod => l.value() % r.value(),
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

        // Load box type-id mapping from nyash.toml (for NewBox lowering)
        let mut box_type_ids: StdHashMap<String, i64> = StdHashMap::new();
        if let Ok(cfg) = std::fs::read_to_string("nyash.toml") {
            if let Ok(doc) = toml::from_str::<toml::Value>(&cfg) {
                if let Some(bt) = doc.get("box_types").and_then(|v| v.as_table()) {
                    for (k, v) in bt {
                        if let Some(id) = v.as_integer() { box_type_ids.insert(k.clone(), id as i64); }
                    }
                }
            }
        }

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

        // Helper ops
        fn as_int<'ctx>(v: BasicValueEnum<'ctx>) -> Option<IntValue<'ctx>> { if let BasicValueEnum::IntValue(iv) = v { Some(iv) } else { None } }
        fn as_float<'ctx>(v: BasicValueEnum<'ctx>) -> Option<FloatValue<'ctx>> { if let BasicValueEnum::FloatValue(fv) = v { Some(fv) } else { None } }
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
                        let recv_p = if let BasicValueEnum::PointerValue(pv) = recv_v { pv } else { return Err("box receiver must be pointer".to_string()); };
                        let recv_h = codegen.builder.build_ptr_to_int(recv_p, i64t, "recv_p2i").map_err(|e| e.to_string())?;
                        // Resolve type_id from metadata (Box("Type")) via nyash.toml
                        let type_id: i64 = if let Some(crate::mir::MirType::Box(bname)) = func.metadata.value_types.get(box_val) {
                            *box_type_ids.get(bname).unwrap_or(&0)
                        } else if let Some(crate::mir::MirType::String) = func.metadata.value_types.get(box_val) {
                            *box_type_ids.get("StringBox").unwrap_or(&0)
                        } else { 0 };

                        // Special-case ArrayBox get/set until general by-id is widely annotated
                        if let Some(crate::mir::MirType::Box(bname)) = func.metadata.value_types.get(box_val) {
                            if bname == "ArrayBox" && (method == "get" || method == "set") {
                                match method.as_str() {
                                    "get" => {
                                        if args.len() != 1 { return Err("ArrayBox.get expects 1 arg".to_string()); }
                                        let idx_v = *vmap.get(&args[0]).ok_or("array.get index missing")?;
                                        let idx_i = if let BasicValueEnum::IntValue(iv) = idx_v { iv } else { return Err("array.get index must be int".to_string()); };
                                        let fnty = i64t.fn_type(&[i64t.into(), i64t.into()], false);
                                        let callee = codegen.module.get_function("nyash_array_get_h").unwrap_or_else(|| codegen.module.add_function("nyash_array_get_h", fnty, None));
                                        let call = codegen.builder.build_call(callee, &[recv_h.into(), idx_i.into()], "aget").map_err(|e| e.to_string())?;
                                        if let Some(d) = dst { let rv = call.try_as_basic_value().left().ok_or("array_get_h returned void".to_string())?; vmap.insert(*d, rv); }
                                        return Ok(());
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
                                        return Ok(());
                                    }
                                    _ => {}
                                }
                            }
                        }

                        // General by-id invoke when method_id is available
                        if let Some(mid) = method_id {
                            // Prepare up to 2 args for now (extend later)
                            let argc_val = i64t.const_int(args.len() as u64, false);
                            let mut a1 = i64t.const_zero();
                            let mut a2 = i64t.const_zero();
                            let mut get_i64 = |vid: ValueId| -> Result<inkwell::values::IntValue, String> {
                                let v = *vmap.get(&vid).ok_or("arg missing")?;
                                Ok(match v {
                                    BasicValueEnum::IntValue(iv) => iv,
                                    BasicValueEnum::PointerValue(pv) => codegen.builder.build_ptr_to_int(pv, i64t, "p2i").map_err(|e| e.to_string())?,
                                    _ => return Err("unsupported arg value (expect int or handle ptr)".to_string()),
                                })
                            };
                            if args.len() >= 1 { a1 = get_i64(args[0])?; }
                            if args.len() >= 2 { a2 = get_i64(args[1])?; }
                            // declare i64 @nyash_plugin_invoke3_i64(i64 type_id, i64 method_id, i64 argc, i64 a0, i64 a1, i64 a2)
                            let fnty = i64t.fn_type(&[i64t.into(), i64t.into(), i64t.into(), i64t.into(), i64t.into(), i64t.into()], false);
                            let callee = codegen.module.get_function("nyash_plugin_invoke3_i64").unwrap_or_else(|| codegen.module.add_function("nyash_plugin_invoke3_i64", fnty, None));
                            let tid = i64t.const_int(type_id as u64, true);
                            let midv = i64t.const_int((*mid) as u64, false);
                            let call = codegen.builder.build_call(callee, &[tid.into(), midv.into(), argc_val.into(), recv_h.into(), a1.into(), a2.into()], "pinvoke").map_err(|e| e.to_string())?;
                            if let Some(d) = dst {
                                let rv = call.try_as_basic_value().left().ok_or("invoke3_i64 returned void".to_string())?;
                                // Decide return lowering by dst annotated type
                                if let Some(mt) = func.metadata.value_types.get(d) {
                                    match mt {
                                        crate::mir::MirType::Integer | crate::mir::MirType::Bool => { vmap.insert(*d, rv); }
                                        crate::mir::MirType::Box(_) | crate::mir::MirType::String | crate::mir::MirType::Array(_) | crate::mir::MirType::Future(_) | crate::mir::MirType::Unknown => {
                                            // rv is i64 handle; convert to i8*
                                            let h = if let BasicValueEnum::IntValue(iv) = rv { iv } else { return Err("invoke ret expected i64".to_string()); };
                                            let pty = codegen.context.i8_type().ptr_type(AddressSpace::from(0));
                                            let ptr = codegen.builder.build_int_to_ptr(h, pty, "ret_handle_to_ptr").map_err(|e| e.to_string())?;
                                            vmap.insert(*d, ptr.into());
                                        }
                                        _ => { vmap.insert(*d, rv); }
                                    }
                                } else {
                                    vmap.insert(*d, rv);
                                }
                            }
                        } else {
                            return Err("BoxCall without method_id not supported in by-id path (enable by-name wrapper if needed)".to_string());
                        }
                    }
                    MirInstruction::ExternCall { dst, iface_name, method_name, args, effects: _ } => {
                        // Minimal: map console.log / debug.trace to libc puts
                        if (iface_name == "env.console" && method_name == "log") || (iface_name == "env.debug" && method_name == "trace") {
                            if args.len() != 1 { return Err("console.log/debug.trace expect 1 arg (string)".to_string()); }
                            let av = *vmap.get(&args[0]).ok_or("extern arg missing")?;
                            let sp = if let BasicValueEnum::PointerValue(pv) = av { pv } else { return Err("extern arg must be string pointer".to_string()); };
                            // declare i32 @puts(i8*)
                            let i8p = codegen.context.i8_type().ptr_type(AddressSpace::from(0));
                            let puts_ty = codegen.context.i32_type().fn_type(&[i8p.into()], false);
                            let puts = codegen.module.get_function("puts").unwrap_or_else(|| codegen.module.add_function("puts", puts_ty, None));
                            let _ = codegen.builder.build_call(puts, &[sp.into()], "puts").map_err(|e| e.to_string())?;
                            if let Some(d) = dst { vmap.insert(*d, codegen.context.i64_type().const_zero().into()); }
                        } else {
                            return Err("ExternCall lowering supports only env.console.log/env.debug.trace in 11.2 minimal".to_string());
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
                        } else {
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
        codegen.target_machine.write_to_file(&codegen.module, inkwell::targets::FileType::Object, output_path.as_ref()).map_err(|e| format!("Failed to write object file: {}", e))?;
        Ok(())
    }

    pub fn compile_and_execute(
        &mut self,
        mir_module: &MirModule,
        temp_path: &str,
    ) -> Result<Box<dyn NyashBox>, String> {
        let obj_path = format!("{}.o", temp_path);
        self.compile_module(mir_module, &obj_path)?;
        // For now, return 0 as IntegerBox (skeleton)
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
