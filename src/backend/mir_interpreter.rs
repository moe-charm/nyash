/*!
 * Minimal MIR Interpreter
 *
 * Executes a subset of MIR instructions for fast iteration without LLVM/JIT.
 * Supported: Const, BinOp(Add/Sub/Mul/Div/Mod), Compare, Load/Store, Branch, Jump, Return,
 * Print/Debug (best-effort), Barrier/Safepoint (no-op).
 */

use std::collections::HashMap;

use crate::backend::vm::{VMValue, VMError};
use crate::box_trait::NyashBox;
use crate::mir::{MirModule, MirFunction, MirInstruction, ConstValue, BinaryOp, CompareOp, ValueId, BasicBlockId};
use crate::backend::abi_util::{to_bool_vm, eq_vm};

pub struct MirInterpreter {
    // SSA value table
    regs: HashMap<ValueId, VMValue>,
    // Simple local memory for Load/Store where `ptr` is a ValueId token
    mem: HashMap<ValueId, VMValue>,
    // Object field storage for RefGet/RefSet (keyed by reference ValueId)
    obj_fields: HashMap<ValueId, HashMap<String, VMValue>>,
}

impl MirInterpreter {
    pub fn new() -> Self { Self { regs: HashMap::new(), mem: HashMap::new(), obj_fields: HashMap::new() } }

    /// Execute module entry (main) and return boxed result
    pub fn execute_module(&mut self, module: &MirModule) -> Result<Box<dyn NyashBox>, VMError> {
        let func = module.functions.get("main").ok_or_else(|| VMError::InvalidInstruction("missing main".into()))?;
        let ret = self.execute_function(func)?;
        Ok(ret.to_nyash_box())
    }

    fn execute_function(&mut self, func: &MirFunction) -> Result<VMValue, VMError> {
        let mut cur = func.entry_block;
        let mut last_pred: Option<BasicBlockId> = None;
        loop {
            let block = func.blocks.get(&cur).ok_or_else(|| VMError::InvalidBasicBlock(format!("bb {:?} not found", cur)))?;
            // Resolve incoming phi nodes using predecessor
            for inst in &block.instructions {
                if let MirInstruction::Phi { dst, inputs } = inst {
                    if let Some(pred) = last_pred {
                        if let Some((_, val)) = inputs.iter().find(|(bb, _)| *bb == pred) {
                            let v = self.reg_load(*val)?;
                            self.regs.insert(*dst, v);
                        }
                    } else {
                        // Entry block PHI: pick first input as a pragmatic default
                        if let Some((_, val)) = inputs.first() {
                            let v = self.reg_load(*val)?;
                            self.regs.insert(*dst, v);
                        }
                    }
                }
            }
            // Execute non-phi, non-terminator instructions
            for inst in block.non_phi_instructions() {
                match inst {
                    MirInstruction::Const { dst, value } => {
                        let v = match value {
                            ConstValue::Integer(i) => VMValue::Integer(*i),
                            ConstValue::Float(f) => VMValue::Float(*f),
                            ConstValue::Bool(b) => VMValue::Bool(*b),
                            ConstValue::String(s) => VMValue::String(s.clone()),
                            ConstValue::Null | ConstValue::Void => VMValue::Void,
                        };
                        self.regs.insert(*dst, v);
                    }
                    MirInstruction::NewBox { dst, box_type, args } => {
                        // Build arg boxes
                        let mut a: Vec<Box<dyn crate::box_trait::NyashBox>> = Vec::new();
                        for vid in args { a.push(self.reg_load(*vid)?.to_nyash_box()); }
                        // Use unified global registry (plugins already initialized by runner)
                        let reg = crate::runtime::unified_registry::get_global_unified_registry();
                        let created = reg.lock().unwrap().create_box(box_type, &a)
                            .map_err(|e| VMError::InvalidInstruction(format!("NewBox {} failed: {}", box_type, e)))?;
                        self.regs.insert(*dst, VMValue::from_nyash_box(created));
                    }
                    MirInstruction::PluginInvoke { dst, box_val, method, args, .. } => {
                        // Resolve receiver
                        let recv = self.reg_load(*box_val)?;
                        let recv_box: Box<dyn crate::box_trait::NyashBox> = match recv.clone() {
                            VMValue::BoxRef(b) => b.share_box(),
                            other => other.to_nyash_box(),
                        };
                        // If PluginBoxV2 → invoke via unified plugin host
                        if let Some(p) = recv_box.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                            let host = crate::runtime::plugin_loader_unified::get_global_plugin_host();
                            let host = host.read().unwrap();
                            let mut argv: Vec<Box<dyn crate::box_trait::NyashBox>> = Vec::new();
                            for a in args { argv.push(self.reg_load(*a)?.to_nyash_box()); }
                            match host.invoke_instance_method(&p.box_type, method, p.inner.instance_id, &argv) {
                                Ok(Some(ret)) => {
                                    if let Some(d) = dst { self.regs.insert(*d, VMValue::from_nyash_box(ret)); }
                                }
                                Ok(None) => { if let Some(d) = dst { self.regs.insert(*d, VMValue::Void); } }
                                Err(e) => { return Err(VMError::InvalidInstruction(format!("PluginInvoke {}.{} failed: {:?}", p.box_type, method, e))); }
                            }
                        } else {
                            // Minimal fallback: toString
                            if method == "toString" {
                                if let Some(d) = dst { self.regs.insert(*d, VMValue::String(recv_box.to_string_box().value)); }
                            } else {
                                return Err(VMError::InvalidInstruction(format!("PluginInvoke unsupported on {} for method {}", recv_box.type_name(), method)));
                            }
                        }
                    }
                    MirInstruction::BoxCall { dst, box_val, method, args, .. } => {
                        // Support getField/setField for normalized RefGet/RefSet
                        if method == "getField" {
                            if args.len() != 1 { return Err(VMError::InvalidInstruction("getField expects 1 arg".into())); }
                            let fname = match self.reg_load(args[0].clone())? { VMValue::String(s) => s, v => v.to_string() };
                            let v = self.obj_fields.get(box_val).and_then(|m| m.get(&fname)).cloned().unwrap_or(VMValue::Void);
                            if let Some(d) = dst { self.regs.insert(*d, v); }
                            continue;
                        } else if method == "setField" {
                            if args.len() != 2 { return Err(VMError::InvalidInstruction("setField expects 2 args".into())); }
                            let fname = match self.reg_load(args[0].clone())? { VMValue::String(s) => s, v => v.to_string() };
                            let valv = self.reg_load(args[1].clone())?;
                            self.obj_fields.entry(*box_val).or_default().insert(fname, valv);
                            continue;
                        }
                        // Fallback: treat like PluginInvoke for plugin-backed boxes
                        let recv = self.reg_load(*box_val)?;
                        let recv_box: Box<dyn crate::box_trait::NyashBox> = match recv.clone() {
                            VMValue::BoxRef(b) => b.share_box(),
                            other => other.to_nyash_box(),
                        };
                        if let Some(p) = recv_box.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                            // Special-case: ConsoleBox.readLine → stdin fallback if not provided by plugin
                            if p.box_type == "ConsoleBox" && method == "readLine" {
                                use std::io::{self, Read};
                                let mut s = String::new();
                                let mut stdin = io::stdin();
                                // Read a single line (blocking)
                                let mut buf = [0u8; 1];
                                while let Ok(n) = stdin.read(&mut buf) {
                                    if n == 0 { break; }
                                    let ch = buf[0] as char;
                                    if ch == '\n' { break; }
                                    s.push(ch);
                                    if s.len() > 1_000_000 { break; }
                                }
                                if let Some(d) = dst { self.regs.insert(*d, VMValue::String(s)); }
                                continue;
                            }
                            let host = crate::runtime::plugin_loader_unified::get_global_plugin_host();
                            let host = host.read().unwrap();
                            let mut argv: Vec<Box<dyn crate::box_trait::NyashBox>> = Vec::new();
                            for a in args { argv.push(self.reg_load(*a)?.to_nyash_box()); }
                            match host.invoke_instance_method(&p.box_type, method, p.inner.instance_id, &argv) {
                                Ok(Some(ret)) => { if let Some(d) = dst { self.regs.insert(*d, VMValue::from_nyash_box(ret)); } }
                                Ok(None) => { if let Some(d) = dst { self.regs.insert(*d, VMValue::Void); } }
                                Err(e) => { return Err(VMError::InvalidInstruction(format!("BoxCall {}.{} failed: {:?}", p.box_type, method, e))); }
                            }
                        } else {
                            return Err(VMError::InvalidInstruction(format!("BoxCall unsupported on {}.{}", recv_box.type_name(), method)));
                        }
                    }
                    MirInstruction::ExternCall { dst, iface_name, method_name, args, .. } => {
                        // Minimal env.console.log bridge
                        if iface_name == "env.console" && method_name == "log" {
                            if let Some(a0) = args.get(0) {
                                let v = self.reg_load(*a0)?;
                                println!("{}", v.to_string());
                            }
                            if let Some(d) = dst { self.regs.insert(*d, VMValue::Void); }
                        } else {
                            return Err(VMError::InvalidInstruction(format!("ExternCall {}.{} not supported", iface_name, method_name)));
                        }
                    }
                    MirInstruction::RefSet { reference, field, value } => {
                        let v = self.reg_load(*value)?;
                        self.obj_fields.entry(*reference).or_default().insert(field.clone(), v);
                    }
                    MirInstruction::RefGet { dst, reference, field } => {
                        let v = self.obj_fields.get(reference).and_then(|m| m.get(field)).cloned().unwrap_or(VMValue::Void);
                        self.regs.insert(*dst, v);
                    }
                    MirInstruction::BinOp { dst, op, lhs, rhs } => {
                        let a = self.reg_load(*lhs)?; let b = self.reg_load(*rhs)?;
                        let v = self.eval_binop(*op, a, b)?;
                        self.regs.insert(*dst, v);
                    }
                    MirInstruction::UnaryOp { dst, op, operand } => {
                        let x = self.reg_load(*operand)?;
                        let v = match op {
                            crate::mir::UnaryOp::Neg => match x { VMValue::Integer(i) => VMValue::Integer(-i), VMValue::Float(f)=>VMValue::Float(-f), _=> return Err(VMError::TypeError(format!("neg expects number, got {:?}", x))) },
                            crate::mir::UnaryOp::Not => VMValue::Bool(!to_bool_vm(&x).map_err(|e| VMError::TypeError(e))?),
                            crate::mir::UnaryOp::BitNot => match x { VMValue::Integer(i) => VMValue::Integer(!i), _=> return Err(VMError::TypeError(format!("bitnot expects integer, got {:?}", x))) },
                        };
                        self.regs.insert(*dst, v);
                    }
                    MirInstruction::Compare { dst, op, lhs, rhs } => {
                        let a = self.reg_load(*lhs)?; let b = self.reg_load(*rhs)?;
                        let v = self.eval_cmp(*op, a, b)?;
                        self.regs.insert(*dst, VMValue::Bool(v));
                    }
                    MirInstruction::Load { dst, ptr } => {
                        let v = self.mem.get(ptr).cloned().unwrap_or(VMValue::Void);
                        self.regs.insert(*dst, v);
                    }
                    MirInstruction::Store { value, ptr } => {
                        let v = self.reg_load(*value)?;
                        self.mem.insert(*ptr, v);
                    }
                    MirInstruction::Copy { dst, src } => {
                        let v = self.reg_load(*src)?; self.regs.insert(*dst, v);
                    }
                    MirInstruction::Debug { value, message } => {
                        let v = self.reg_load(*value)?;
                        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                            eprintln!("[mir-debug] {} => {:?}", message, v);
                        }
                    }
                    MirInstruction::Print { value, .. } => {
                        let v = self.reg_load(*value)?; println!("{}", v.to_string());
                    }
                    // No-ops in the interpreter for now
                    MirInstruction::BarrierRead { .. } | MirInstruction::BarrierWrite { .. } | MirInstruction::Barrier { .. } | MirInstruction::Safepoint | MirInstruction::Nop => {}
                    // Unimplemented but recognized — return clear error for visibility
                    other => return Err(VMError::InvalidInstruction(format!("MIR interp: unimplemented instruction: {:?}", other))),
                }
            }
            // Handle terminator
            match &block.terminator {
                Some(MirInstruction::Return { value }) => {
                    if let Some(v) = value { return self.reg_load(*v); } else { return Ok(VMValue::Void); }
                }
                Some(MirInstruction::Jump { target }) => {
                    last_pred = Some(block.id); cur = *target; continue;
                }
                Some(MirInstruction::Branch { condition, then_bb, else_bb }) => {
                    let c = self.reg_load(*condition)?; let t = to_bool_vm(&c).map_err(|e| VMError::TypeError(e))?;
                    last_pred = Some(block.id); cur = if t { *then_bb } else { *else_bb }; continue;
                }
                None => return Err(VMError::InvalidBasicBlock(format!("unterminated block {:?}", block.id))),
                Some(other) => return Err(VMError::InvalidInstruction(format!("invalid terminator in MIR interp: {:?}", other))),
            }
        }
    }

    fn reg_load(&self, id: ValueId) -> Result<VMValue, VMError> {
        self.regs.get(&id).cloned().ok_or_else(|| VMError::InvalidValue(format!("use of undefined value {:?}", id)))
    }

    fn eval_binop(&self, op: BinaryOp, a: VMValue, b: VMValue) -> Result<VMValue, VMError> {
        use BinaryOp::*; use VMValue::*;
        Ok(match (op, a, b) {
            (Add, Integer(x), Integer(y)) => Integer(x + y),
            // String concat: ifいずれかがStringなら文字列連結
            (Add, String(s), Integer(y)) => String(format!("{}{}", s, y)),
            (Add, String(s), Float(y)) => String(format!("{}{}", s, y)),
            (Add, String(s), Bool(y)) => String(format!("{}{}", s, y)),
            (Add, String(s), String(t)) => String(format!("{}{}", s, t)),
            (Add, Integer(x), String(t)) => String(format!("{}{}", x, t)),
            (Add, Float(x), String(t)) => String(format!("{}{}", x, t)),
            (Add, Bool(x), String(t)) => String(format!("{}{}", x, t)),
            (Sub, Integer(x), Integer(y)) => Integer(x - y),
            (Mul, Integer(x), Integer(y)) => Integer(x * y),
            (Div, Integer(_), Integer(0)) => return Err(VMError::DivisionByZero),
            (Div, Integer(x), Integer(y)) => Integer(x / y),
            (Mod, Integer(_), Integer(0)) => return Err(VMError::DivisionByZero),
            (Mod, Integer(x), Integer(y)) => Integer(x % y),
            // Float ops (best-effort)
            (Add, Float(x), Float(y)) => Float(x + y),
            (Sub, Float(x), Float(y)) => Float(x - y),
            (Mul, Float(x), Float(y)) => Float(x * y),
            (Div, Float(_), Float(y)) if y == 0.0 => return Err(VMError::DivisionByZero),
            (Div, Float(x), Float(y)) => Float(x / y),
            (Mod, Float(x), Float(y)) => Float(x % y),
            // Logical/bitwise on integers
            (BitAnd, Integer(x), Integer(y)) => Integer(x & y),
            (BitOr, Integer(x), Integer(y)) => Integer(x | y),
            (BitXor, Integer(x), Integer(y)) => Integer(x ^ y),
            (Shl, Integer(x), Integer(y)) => Integer(x.wrapping_shl(y as u32)),
            (Shr, Integer(x), Integer(y)) => Integer(x.wrapping_shr(y as u32)),
            // Fallbacks not yet supported
            (opk, va, vb) => return Err(VMError::TypeError(format!("unsupported binop {:?} on {:?} and {:?}", opk, va, vb))),
        })
    }

    fn eval_cmp(&self, op: CompareOp, a: VMValue, b: VMValue) -> Result<bool, VMError> {
        use CompareOp::*; use VMValue::*;
        Ok(match (op, &a, &b) {
            (Eq, _, _) => eq_vm(&a, &b),
            (Ne, _, _) => !eq_vm(&a, &b),
            (Lt, Integer(x), Integer(y)) => x < y,
            (Le, Integer(x), Integer(y)) => x <= y,
            (Gt, Integer(x), Integer(y)) => x > y,
            (Ge, Integer(x), Integer(y)) => x >= y,
            (Lt, Float(x), Float(y)) => x < y,
            (Le, Float(x), Float(y)) => x <= y,
            (Gt, Float(x), Float(y)) => x > y,
            (Ge, Float(x), Float(y)) => x >= y,
            (opk, va, vb) => return Err(VMError::TypeError(format!("unsupported compare {:?} on {:?} and {:?}", opk, va, vb))),
        })
    }
}
