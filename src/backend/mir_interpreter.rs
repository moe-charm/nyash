/*!
 * Minimal MIR Interpreter
 *
 * Executes a subset of MIR instructions for fast iteration without LLVM/JIT.
 * Supported: Const, BinOp(Add/Sub/Mul/Div/Mod), Compare, Load/Store, Branch, Jump, Return,
 * Print/Debug (best-effort), Barrier/Safepoint (no-op).
 */

use std::collections::HashMap;

use crate::backend::abi_util::{eq_vm, to_bool_vm};
use crate::backend::vm::{VMError, VMValue};
use crate::box_trait::NyashBox;
use crate::mir::{
    BasicBlockId, BinaryOp, Callee, CompareOp, ConstValue, MirFunction, MirInstruction, MirModule, ValueId,
};

pub struct MirInterpreter {
    // SSA value table (per-function; swapped on call)
    regs: HashMap<ValueId, VMValue>,
    // Simple local memory for Load/Store where `ptr` is a ValueId token
    mem: HashMap<ValueId, VMValue>,
    // Object field storage for RefGet/RefSet (keyed by reference ValueId)
    obj_fields: HashMap<ValueId, HashMap<String, VMValue>>,
    // Function table (current module)
    functions: HashMap<String, MirFunction>,
    // Currently executing function name (for call resolution preferences)
    cur_fn: Option<String>,
}

impl MirInterpreter {
    pub fn new() -> Self {
        Self {
            regs: HashMap::new(),
            mem: HashMap::new(),
            obj_fields: HashMap::new(),
            functions: HashMap::new(),
            cur_fn: None,
        }
    }

    /// Execute module entry (main) and return boxed result
    pub fn execute_module(&mut self, module: &MirModule) -> Result<Box<dyn NyashBox>, VMError> {
        // Snapshot functions for call resolution
        self.functions = module.functions.clone();
        let func = module
            .functions
            .get("main")
            .ok_or_else(|| VMError::InvalidInstruction("missing main".into()))?;
        let ret = self.execute_function(func)?;
        Ok(ret.to_nyash_box())
    }

    fn execute_function(&mut self, func: &MirFunction) -> Result<VMValue, VMError> {
        self._exec_function_inner(func, None)
    }

    fn _exec_function_inner(
        &mut self,
        func: &MirFunction,
        arg_vals: Option<&[VMValue]>,
    ) -> Result<VMValue, VMError> {
        // Swap in a fresh register file for this call
        let saved_regs = std::mem::take(&mut self.regs);
        let saved_fn = self.cur_fn.clone();
        self.cur_fn = Some(func.signature.name.clone());

        // Bind parameters if provided
        if let Some(args) = arg_vals {
            for (i, pid) in func.params.iter().enumerate() {
                let v = args.get(i).cloned().unwrap_or(VMValue::Void);
                self.regs.insert(*pid, v);
            }
        }

        let mut cur = func.entry_block;
        let mut last_pred: Option<BasicBlockId> = None;
        loop {
            let block = func
                .blocks
                .get(&cur)
                .ok_or_else(|| VMError::InvalidBasicBlock(format!("bb {:?} not found", cur)))?;
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
                    MirInstruction::NewBox {
                        dst,
                        box_type,
                        args,
                    } => {
                        // Build arg boxes
                        let mut a: Vec<Box<dyn crate::box_trait::NyashBox>> = Vec::new();
                        for vid in args {
                            a.push(self.reg_load(*vid)?.to_nyash_box());
                        }
                        // Use unified global registry (plugins already initialized by runner)
                        let reg = crate::runtime::unified_registry::get_global_unified_registry();
                        let created =
                            reg.lock().unwrap().create_box(box_type, &a).map_err(|e| {
                                VMError::InvalidInstruction(format!(
                                    "NewBox {} failed: {}",
                                    box_type, e
                                ))
                            })?;
                        self.regs.insert(*dst, VMValue::from_nyash_box(created));
                    }
                    MirInstruction::PluginInvoke {
                        dst,
                        box_val,
                        method,
                        args,
                        ..
                    } => {
                        // Resolve receiver
                        let recv = self.reg_load(*box_val)?;
                        let recv_box: Box<dyn crate::box_trait::NyashBox> = match recv.clone() {
                            VMValue::BoxRef(b) => b.share_box(),
                            other => other.to_nyash_box(),
                        };
                        // If PluginBoxV2 → invoke via unified plugin host
                        if let Some(p) = recv_box
                            .as_any()
                            .downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>(
                        ) {
                            let host =
                                crate::runtime::plugin_loader_unified::get_global_plugin_host();
                            let host = host.read().unwrap();
                            let mut argv: Vec<Box<dyn crate::box_trait::NyashBox>> = Vec::new();
                            for a in args {
                                argv.push(self.reg_load(*a)?.to_nyash_box());
                            }
                            match host.invoke_instance_method(
                                &p.box_type,
                                method,
                                p.inner.instance_id,
                                &argv,
                            ) {
                                Ok(Some(ret)) => {
                                    if let Some(d) = dst {
                                        self.regs.insert(*d, VMValue::from_nyash_box(ret));
                                    }
                                }
                                Ok(None) => {
                                    if let Some(d) = dst {
                                        self.regs.insert(*d, VMValue::Void);
                                    }
                                }
                                Err(e) => {
                                    return Err(VMError::InvalidInstruction(format!(
                                        "PluginInvoke {}.{} failed: {:?}",
                                        p.box_type, method, e
                                    )));
                                }
                            }
                        } else {
                            // Minimal fallback: toString
                            if method == "toString" {
                                if let Some(d) = dst {
                                    self.regs.insert(
                                        *d,
                                        VMValue::String(recv_box.to_string_box().value),
                                    );
                                }
                            } else {
                                return Err(VMError::InvalidInstruction(format!(
                                    "PluginInvoke unsupported on {} for method {}",
                                    recv_box.type_name(),
                                    method
                                )));
                            }
                        }
                    }
                    MirInstruction::BoxCall {
                        dst,
                        box_val,
                        method,
                        args,
                        ..
                    } => {
                        // Support getField/setField for normalized RefGet/RefSet
                        if method == "getField" {
                            if args.len() != 1 {
                                return Err(VMError::InvalidInstruction(
                                    "getField expects 1 arg".into(),
                                ));
                            }
                            let fname = match self.reg_load(args[0].clone())? {
                                VMValue::String(s) => s,
                                v => v.to_string(),
                            };
                            let v = self
                                .obj_fields
                                .get(box_val)
                                .and_then(|m| m.get(&fname))
                                .cloned()
                                .unwrap_or(VMValue::Void);
                            if let Some(d) = dst {
                                self.regs.insert(*d, v);
                            }
                            continue;
                        } else if method == "setField" {
                            if args.len() != 2 {
                                return Err(VMError::InvalidInstruction(
                                    "setField expects 2 args".into(),
                                ));
                            }
                            let fname = match self.reg_load(args[0].clone())? {
                                VMValue::String(s) => s,
                                v => v.to_string(),
                            };
                            let valv = self.reg_load(args[1].clone())?;
                            self.obj_fields
                                .entry(*box_val)
                                .or_default()
                                .insert(fname, valv);
                            continue;
                        }
                        // Builtin StringBox minimal methods (length/concat) to bridge plugin-first gaps
                        {
                            let recv = self.reg_load(*box_val)?;
                            let recv_box_any: Box<dyn crate::box_trait::NyashBox> = match recv.clone() {
                                VMValue::BoxRef(b) => b.share_box(),
                                other => other.to_nyash_box(),
                            };
                            if let Some(sb) = recv_box_any
                                .as_any()
                                .downcast_ref::<crate::box_trait::StringBox>()
                            {
                                match method.as_str() {
                                    "length" => {
                                        let ret = sb.length();
                                        if let Some(d) = dst {
                                            self.regs.insert(*d, VMValue::from_nyash_box(ret));
                                        }
                                        continue;
                                    }
                                    "concat" => {
                                        if args.len() != 1 {
                                            return Err(VMError::InvalidInstruction(
                                                "concat expects 1 arg".into(),
                                            ));
                                        }
                                        let rhs = self.reg_load(args[0])?;
                                        let new_s = format!("{}{}", sb.value, rhs.to_string());
                                        if let Some(d) = dst {
                                            self.regs.insert(
                                                *d,
                                                VMValue::from_nyash_box(Box::new(
                                                    crate::box_trait::StringBox::new(new_s),
                                                )),
                                            );
                                        }
                                        continue;
                                    }
                                    _ => { /* fallthrough to plugin or error */ }
                                }
                            }
                        }
                        // Fallback: treat like PluginInvoke for plugin-backed boxes
                        let recv = self.reg_load(*box_val)?;
                        let recv_box: Box<dyn crate::box_trait::NyashBox> = match recv.clone() {
                            VMValue::BoxRef(b) => b.share_box(),
                            other => other.to_nyash_box(),
                        };
                        if let Some(p) = recv_box
                            .as_any()
                            .downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>(
                        ) {
                            // Special-case: ConsoleBox.readLine → stdin fallback if not provided by plugin
                            if p.box_type == "ConsoleBox" && method == "readLine" {
                                use std::io::{self, Read};
                                let mut s = String::new();
                                let mut stdin = io::stdin();
                                // Read a single line (blocking)
                                let mut buf = [0u8; 1];
                                while let Ok(n) = stdin.read(&mut buf) {
                                    if n == 0 {
                                        break;
                                    }
                                    let ch = buf[0] as char;
                                    if ch == '\n' {
                                        break;
                                    }
                                    s.push(ch);
                                    if s.len() > 1_000_000 {
                                        break;
                                    }
                                }
                                if let Some(d) = dst {
                                    self.regs.insert(*d, VMValue::String(s));
                                }
                                continue;
                            }
                            let host =
                                crate::runtime::plugin_loader_unified::get_global_plugin_host();
                            let host = host.read().unwrap();
                            let mut argv: Vec<Box<dyn crate::box_trait::NyashBox>> = Vec::new();
                            for a in args {
                                argv.push(self.reg_load(*a)?.to_nyash_box());
                            }
                            match host.invoke_instance_method(
                                &p.box_type,
                                method,
                                p.inner.instance_id,
                                &argv,
                            ) {
                                Ok(Some(ret)) => {
                                    if let Some(d) = dst {
                                        self.regs.insert(*d, VMValue::from_nyash_box(ret));
                                    }
                                }
                                Ok(None) => {
                                    if let Some(d) = dst {
                                        self.regs.insert(*d, VMValue::Void);
                                    }
                                }
                                Err(e) => {
                                    return Err(VMError::InvalidInstruction(format!(
                                        "BoxCall {}.{} failed: {:?}",
                                        p.box_type, method, e
                                    )));
                                }
                            }
                        } else {
                            return Err(VMError::InvalidInstruction(format!(
                                "BoxCall unsupported on {}.{}",
                                recv_box.type_name(),
                                method
                            )));
                        }
                    }
                    MirInstruction::ExternCall {
                        dst,
                        iface_name,
                        method_name,
                        args,
                        ..
                    } => {
                        match (iface_name.as_str(), method_name.as_str()) {
                            ("env.console", "log") => {
                                if let Some(a0) = args.get(0) {
                                    let v = self.reg_load(*a0)?;
                                    println!("{}", v.to_string());
                                }
                                if let Some(d) = dst {
                                    self.regs.insert(*d, VMValue::Void);
                                }
                            }
                            ("env.future", "new") => {
                                let fut = crate::boxes::future::NyashFutureBox::new();
                                if let Some(a0) = args.get(0) {
                                    let v = self.reg_load(*a0)?;
                                    fut.set_result(v.to_nyash_box());
                                }
                                if let Some(d) = dst {
                                    self.regs.insert(*d, VMValue::Future(fut));
                                }
                            }
                            ("env.future", "set") => {
                                if args.len() >= 2 {
                                    let f = self.reg_load(args[0])?;
                                    let v = self.reg_load(args[1])?;
                                    if let VMValue::Future(fut) = f {
                                        fut.set_result(v.to_nyash_box());
                                    } else {
                                        return Err(VMError::TypeError("env.future.set expects Future".into()));
                                    }
                                }
                                if let Some(d) = dst {
                                    self.regs.insert(*d, VMValue::Void);
                                }
                            }
                            ("env.future", "await") => {
                                if let Some(a0) = args.get(0) {
                                    let f = self.reg_load(*a0)?;
                                    match f {
                                        VMValue::Future(fut) => {
                                            // Coarse safepoint while blocking
                                            let v = fut.get();
                                            if let Some(d) = dst {
                                                self.regs.insert(*d, VMValue::from_nyash_box(v));
                                            }
                                        }
                                        _ => return Err(VMError::TypeError("await expects Future".into())),
                                    }
                                }
                            }
                            ("env.runtime", "checkpoint") => {
                                crate::runtime::global_hooks::safepoint_and_poll();
                                if let Some(d) = dst {
                                    self.regs.insert(*d, VMValue::Void);
                                }
                            }
                            ("env.modules", "set") => {
                                if args.len() >= 2 {
                                    let k = self.reg_load(args[0])?.to_string();
                                    let v = self.reg_load(args[1])?.to_nyash_box();
                                    crate::runtime::modules_registry::set(k, v);
                                }
                                if let Some(d) = dst {
                                    self.regs.insert(*d, VMValue::Void);
                                }
                            }
                            ("env.modules", "get") => {
                                if let Some(a0) = args.get(0) {
                                    let k = self.reg_load(*a0)?.to_string();
                                    let vb = crate::runtime::modules_registry::get(&k)
                                        .unwrap_or_else(|| Box::new(crate::box_trait::VoidBox::new()));
                                    if let Some(d) = dst {
                                        self.regs.insert(*d, VMValue::from_nyash_box(vb));
                                    }
                                }
                            }
                            _ => {
                                return Err(VMError::InvalidInstruction(format!(
                                    "ExternCall {}.{} not supported",
                                    iface_name, method_name
                                )));
                            }
                        }
                    }
                    MirInstruction::RefSet {
                        reference,
                        field,
                        value,
                    } => {
                        let v = self.reg_load(*value)?;
                        self.obj_fields
                            .entry(*reference)
                            .or_default()
                            .insert(field.clone(), v);
                    }
                    MirInstruction::RefGet {
                        dst,
                        reference,
                        field,
                    } => {
                        let v = self
                            .obj_fields
                            .get(reference)
                            .and_then(|m| m.get(field))
                            .cloned()
                            .unwrap_or(VMValue::Void);
                        self.regs.insert(*dst, v);
                    }
                    MirInstruction::BinOp { dst, op, lhs, rhs } => {
                        let a = self.reg_load(*lhs)?;
                        let b = self.reg_load(*rhs)?;
                        let v = self.eval_binop(*op, a, b)?;
                        self.regs.insert(*dst, v);
                    }
                    MirInstruction::UnaryOp { dst, op, operand } => {
                        let x = self.reg_load(*operand)?;
                        let v = match op {
                            crate::mir::UnaryOp::Neg => match x {
                                VMValue::Integer(i) => VMValue::Integer(-i),
                                VMValue::Float(f) => VMValue::Float(-f),
                                _ => {
                                    return Err(VMError::TypeError(format!(
                                        "neg expects number, got {:?}",
                                        x
                                    )))
                                }
                            },
                            crate::mir::UnaryOp::Not => {
                                VMValue::Bool(!to_bool_vm(&x).map_err(|e| VMError::TypeError(e))?)
                            }
                            crate::mir::UnaryOp::BitNot => match x {
                                VMValue::Integer(i) => VMValue::Integer(!i),
                                _ => {
                                    return Err(VMError::TypeError(format!(
                                        "bitnot expects integer, got {:?}",
                                        x
                                    )))
                                }
                            },
                        };
                        self.regs.insert(*dst, v);
                    }
                    MirInstruction::Compare { dst, op, lhs, rhs } => {
                        let a = self.reg_load(*lhs)?;
                        let b = self.reg_load(*rhs)?;
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
                        let v = self.reg_load(*src)?;
                        self.regs.insert(*dst, v);
                    }
                    MirInstruction::Debug { value, message } => {
                        let v = self.reg_load(*value)?;
                        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                            eprintln!("[mir-debug] {} => {:?}", message, v);
                        }
                    }
                    MirInstruction::Print { value, .. } => {
                        let v = self.reg_load(*value)?;
                        println!("{}", v.to_string());
                    }
                    // No-ops in the interpreter for now
                    MirInstruction::BarrierRead { .. }
                    | MirInstruction::BarrierWrite { .. }
                    | MirInstruction::Barrier { .. }
                    | MirInstruction::Safepoint
                    | MirInstruction::Nop => {}
                    MirInstruction::Call {
                        dst,
                        func,
                        callee,
                        args,
                        ..
                    } => {
                        // VM実行器Callee対応 - ChatGPT5 Pro MIR革命の最終1%！

                        // Phase 1: 段階移行サポート - callee型安全解決を優先、フォールバックで従来解決
                        let call_result = if let Some(callee_type) = callee {
                            // NEW: 型安全Callee解決（ChatGPT5 Pro設計）
                            self.execute_callee_call(callee_type, args)?
                        } else {
                            // LEGACY: 従来の文字列ベース解決（func: ValueId）
                            self.execute_legacy_call(*func, args)?
                        };

                        if let Some(d) = dst {
                            self.regs.insert(*d, call_result);
                        }
                    }
                    // Unimplemented but recognized — return clear error for visibility
                    other => {
                        return Err(VMError::InvalidInstruction(format!(
                            "MIR interp: unimplemented instruction: {:?}",
                            other
                        )))
                    }
                }
            }
            // Handle terminator
            let out = match &block.terminator {
                Some(MirInstruction::Return { value }) => {
                    if let Some(v) = value {
                        self.reg_load(*v)
                    } else {
                        Ok(VMValue::Void)
                    }
                }
                Some(MirInstruction::Jump { target }) => {
                    last_pred = Some(block.id);
                    cur = *target;
                    continue;
                }
                Some(MirInstruction::Branch {
                    condition,
                    then_bb,
                    else_bb,
                }) => {
                    let c = self.reg_load(*condition)?;
                    let t = to_bool_vm(&c).map_err(|e| VMError::TypeError(e))?;
                    last_pred = Some(block.id);
                    cur = if t { *then_bb } else { *else_bb };
                    continue;
                }
                None => {
                    Err(VMError::InvalidBasicBlock(format!(
                        "unterminated block {:?}",
                        block.id
                    )))
                }
                Some(other) => {
                    Err(VMError::InvalidInstruction(format!(
                        "invalid terminator in MIR interp: {:?}",
                        other
                    )))
                }
            };
            // Function finished (return or error)
            // Restore previous register file and current function
            let result = out;
            self.cur_fn = saved_fn;
            self.regs = saved_regs;
            return result;
        }
    }

    fn reg_load(&self, id: ValueId) -> Result<VMValue, VMError> {
        self.regs
            .get(&id)
            .cloned()
            .ok_or_else(|| VMError::InvalidValue(format!("use of undefined value {:?}", id)))
    }

    fn eval_binop(&self, op: BinaryOp, a: VMValue, b: VMValue) -> Result<VMValue, VMError> {
        use BinaryOp::*;
        use VMValue::*;
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
            (opk, va, vb) => {
                return Err(VMError::TypeError(format!(
                    "unsupported binop {:?} on {:?} and {:?}",
                    opk, va, vb
                )))
            }
        })
    }

    fn eval_cmp(&self, op: CompareOp, a: VMValue, b: VMValue) -> Result<bool, VMError> {
        use CompareOp::*;
        use VMValue::*;
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
            (opk, va, vb) => {
                return Err(VMError::TypeError(format!(
                    "unsupported compare {:?} on {:?} and {:?}",
                    opk, va, vb
                )))
            }
        })
    }

    /// NEW: Callee型安全解決（ChatGPT5 Pro設計）
    fn execute_callee_call(&mut self, callee: &Callee, args: &[ValueId]) -> Result<VMValue, VMError> {
        match callee {
            Callee::Global(func_name) => {
                // グローバル関数呼び出し（nyash.builtin.print等）
                self.execute_global_function(func_name, args)
            }
            Callee::Method { box_name, method, receiver } => {
                // メソッド呼び出し（StringBox.concat等）
                if let Some(recv_id) = receiver {
                    let recv_val = self.reg_load(*recv_id)?;
                    self.execute_method_call(&recv_val, method, args)
                } else {
                    Err(VMError::InvalidInstruction(format!(
                        "Method call {}.{} missing receiver",
                        box_name, method
                    )))
                }
            }
            Callee::Constructor { box_type } => {
                // コンストラクタ呼び出し（NewBox相当）
                // TODO: 実際のBox生成実装（Phase 2で実装）
                Err(VMError::InvalidInstruction(format!(
                    "Constructor calls not yet implemented for {}",
                    box_type
                )))
            }
            Callee::Closure { params: _, captures: _, me_capture: _ } => {
                // クロージャ生成（NewClosure相当）
                // TODO: クロージャ生成実装（Phase 2で実装）
                Err(VMError::InvalidInstruction(
                    "Closure creation not yet implemented in VM".into()
                ))
            }
            Callee::Value(func_val_id) => {
                // 第一級関数呼び出し（クロージャ等）
                let _func_val = self.reg_load(*func_val_id)?;
                // TODO: 第一級関数呼び出し実装（Phase 2で拡張）
                Err(VMError::InvalidInstruction(
                    "First-class function calls not yet implemented in VM".into()
                ))
            }
            Callee::Extern(extern_name) => {
                // 外部C ABI関数呼び出し
                self.execute_extern_function(extern_name, args)
            }
        }
    }

    /// LEGACY: 従来の文字列ベース解決（後方互換性）
    fn execute_legacy_call(&mut self, func_id: ValueId, args: &[ValueId]) -> Result<VMValue, VMError> {
        // 1) 名前を取り出す
        let name_val = self.reg_load(func_id)?;
        let raw = match name_val {
            VMValue::String(ref s) => s.clone(),
            other => other.to_string(),
        };
        // 2) 直接一致を優先
        let mut pick: Option<String> = None;
        if self.functions.contains_key(&raw) {
            pick = Some(raw.clone());
        } else {
            let arity = args.len();
            let mut cands: Vec<String> = Vec::new();
            // a) 末尾サフィックス一致: ".name/arity"
            let suf = format!(".{}{}", raw, format!("/{}", arity));
            for k in self.functions.keys() {
                if k.ends_with(&suf) { cands.push(k.clone()); }
            }
            // b) raw に '/' が含まれ、完全名っぽい場合はそのままも候補に（既に上で除外）
            if cands.is_empty() && raw.contains('/') && self.functions.contains_key(&raw) {
                cands.push(raw.clone());
            }
            // c) 優先: 現在のボックス名と一致するもの
            if cands.len() > 1 {
                if let Some(cur) = &self.cur_fn {
                    let cur_box = cur.split('.').next().unwrap_or("");
                    let scoped: Vec<String> = cands
                        .iter()
                        .filter(|k| k.starts_with(&format!("{}.", cur_box)))
                        .cloned()
                        .collect();
                    if scoped.len() == 1 { cands = scoped; }
                }
            }
            if cands.len() == 1 {
                pick = Some(cands.remove(0));
            } else if cands.len() > 1 {
                cands.sort();
                pick = Some(cands[0].clone());
            }
        }
        let fname = pick.ok_or_else(|| VMError::InvalidInstruction(format!(
            "call unresolved: '{}' (arity={})",
            raw,
            args.len()
        )))?;
        if std::env::var("NYASH_VM_CALL_TRACE").ok().as_deref() == Some("1") {
            eprintln!("[vm] legacy-call resolved '{}' -> '{}'", raw, fname);
        }
        let callee = self
            .functions
            .get(&fname)
            .cloned()
            .ok_or_else(|| VMError::InvalidInstruction(format!("function not found: {}", fname)))?;
        // 3) 実引数の評価
        let mut argv: Vec<VMValue> = Vec::new();
        for a in args { argv.push(self.reg_load(*a)?); }
        // 4) 実行
        self._exec_function_inner(&callee, Some(&argv))
    }

    /// グローバル関数実行（nyash.builtin.*）
    fn execute_global_function(&mut self, func_name: &str, args: &[ValueId]) -> Result<VMValue, VMError> {
        match func_name {
            "nyash.builtin.print" | "print" => {
                if let Some(arg_id) = args.get(0) {
                    let val = self.reg_load(*arg_id)?;
                    println!("{}", val.to_string());
                }
                Ok(VMValue::Void)
            }
            "nyash.console.log" => {
                if let Some(arg_id) = args.get(0) {
                    let val = self.reg_load(*arg_id)?;
                    println!("{}", val.to_string());
                }
                Ok(VMValue::Void)
            }
            "nyash.builtin.error" => {
                if let Some(arg_id) = args.get(0) {
                    let val = self.reg_load(*arg_id)?;
                    eprintln!("Error: {}", val.to_string());
                }
                Ok(VMValue::Void)
            }
            _ => Err(VMError::InvalidInstruction(format!(
                "Unknown global function: {}",
                func_name
            )))
        }
    }

    /// メソッド呼び出し実行
    fn execute_method_call(&mut self, receiver: &VMValue, method: &str, args: &[ValueId]) -> Result<VMValue, VMError> {
        // 受信オブジェクトの型に基づいてメソッド実行
        match receiver {
            VMValue::String(s) => {
                match method {
                    "length" => Ok(VMValue::Integer(s.len() as i64)),
                    "concat" => {
                        if let Some(arg_id) = args.get(0) {
                            let arg_val = self.reg_load(*arg_id)?;
                            let new_str = format!("{}{}", s, arg_val.to_string());
                            Ok(VMValue::String(new_str))
                        } else {
                            Err(VMError::InvalidInstruction("concat requires 1 argument".into()))
                        }
                    }
                    _ => Err(VMError::InvalidInstruction(format!(
                        "Unknown String method: {}",
                        method
                    )))
                }
            }
            VMValue::BoxRef(box_ref) => {
                // プラグインBox経由でメソッド呼び出し
                if let Some(p) = box_ref.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                    let host = crate::runtime::plugin_loader_unified::get_global_plugin_host();
                    let host = host.read().unwrap();
                    let mut argv: Vec<Box<dyn crate::box_trait::NyashBox>> = Vec::new();
                    for a in args {
                        argv.push(self.reg_load(*a)?.to_nyash_box());
                    }
                    match host.invoke_instance_method(&p.box_type, method, p.inner.instance_id, &argv) {
                        Ok(Some(ret)) => Ok(VMValue::from_nyash_box(ret)),
                        Ok(None) => Ok(VMValue::Void),
                        Err(e) => Err(VMError::InvalidInstruction(format!(
                            "Plugin method {}.{} failed: {:?}",
                            p.box_type, method, e
                        )))
                    }
                } else {
                    Err(VMError::InvalidInstruction(format!(
                        "Method {} not supported on BoxRef({})",
                        method, box_ref.type_name()
                    )))
                }
            }
            _ => Err(VMError::InvalidInstruction(format!(
                "Method {} not supported on {:?}",
                method, receiver
            )))
        }
    }

    /// 外部関数実行（C ABI）
    fn execute_extern_function(&mut self, extern_name: &str, args: &[ValueId]) -> Result<VMValue, VMError> {
        match extern_name {
            "exit" => {
                let code = if let Some(arg_id) = args.get(0) {
                    self.reg_load(*arg_id)?.as_integer().unwrap_or(0)
                } else { 0 };
                std::process::exit(code as i32);
            }
            "panic" => {
                let msg = if let Some(arg_id) = args.get(0) {
                    self.reg_load(*arg_id)?.to_string()
                } else { "VM panic".to_string() };
                panic!("{}", msg);
            }
            _ => Err(VMError::InvalidInstruction(format!(
                "Unknown extern function: {}",
                extern_name
            )))
        }
    }
}
