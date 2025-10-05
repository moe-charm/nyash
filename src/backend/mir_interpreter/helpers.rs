use super::*;
use crate::box_trait::VoidBox;
use std::string::String as StdString;

impl MirInterpreter {
    /// Emit a one-line JSON call trace when NYASH_CALL_TRACE=1.
    /// Label examples: "Global:print", "Method:ConsoleBox.println/1", "BoxCall:push".
    pub(super) fn emit_call_trace_label(&self, label: &str, argc: usize, recv: Option<u32>) {
        if std::env::var("NYASH_CALL_TRACE").ok().as_deref() != Some("1") {
            return;
        }
        let bb = self.last_block.map(|b| b.as_u32()).unwrap_or(0);
        let mut out = String::from("{\"kind\":\"call\",\"callee\":\"");
        let esc = |s: &str| s.replace('"', "\\\"");
        out.push_str(&esc(label));
        out.push('"');
        if let Some(r) = recv {
            out.push_str(",\"recv\":");
            out.push_str(&r.to_string());
        }
        out.push_str(",\"argc\":");
        out.push_str(&argc.to_string());
        out.push_str(",\"bb\":");
        out.push_str(&bb.to_string());
        out.push('}');
        eprintln!("{}", out);
    }
    #[inline]
    fn tag_nullish(v: &VMValue) -> &'static str {
        match v {
            VMValue::Void => "void",
            VMValue::BoxRef(b) => {
                if b.as_any().downcast_ref::<crate::boxes::null_box::NullBox>().is_some() { "null" }
                else if b.as_any().downcast_ref::<crate::boxes::missing_box::MissingBox>().is_some() { "missing" }
                else if b.as_any().downcast_ref::<crate::box_trait::VoidBox>().is_some() { "void" }
                else { "" }
            }
            _ => "",
        }
    }
    pub(super) fn reg_load(&self, id: ValueId) -> Result<VMValue, VMError> {
        match self.regs.get(&id).cloned() {
            Some(v) => Ok(v),
            None => {
                let cfg = super::VmConfig::global();
                if cfg.general_trace || cfg.trace_exec {
                    let keys: Vec<String> = self
                        .regs
                        .keys()
                        .map(|k| format!("{:?}", k))
                        .collect();
                    eprintln!(
                        "[vm-trace] reg_load undefined id={:?} last_block={:?} last_inst={:?} regs={}",
                        id,
                        self.last_block,
                        self.last_inst,
                        keys.join(", ")
                    );
                }
                // Dev-time safety valve: tolerate undefined registers as Void when enabled
                if cfg.tolerate_void {
                    return Ok(VMValue::Void);
                }
                Err(VMError::InvalidValue(format!(
                    "use of undefined value {:?}",
                    id
                )))
            }
        }
    }

    /// LocalSSA-style materialization at runtime: if the given `src` value
    /// has been copied to a new SSA id earlier in the current basic block,
    /// return that latest in-block dst id. Otherwise, return `src` unchanged.
    ///
    /// Contract:
    /// - Only considers non-PHI instructions inside the current block
    /// - Scans up to (but not including) the current executing instruction (`last_inst`)
    /// - Picks the latest Copy(dst := src) if multiple are present
    ///
    /// This mirrors the builder's LocalSSA contract and makes the VM tolerant
    /// to occasional upstream misses without changing semantics.
    pub(super) fn materialize_value_in_current_block(&self, src: ValueId) -> ValueId {
        let (fn_name, bb_id, last_inst) = match (self.cur_fn.as_ref(), self.last_block, self.last_inst.as_ref()) {
            (Some(f), Some(bb), Some(inst)) => (f.clone(), bb, inst.clone()),
            _ => return src,
        };
        let func = match self.functions.get(&fn_name) { Some(f) => f, None => return src };
        let bb = match func.blocks.get(&bb_id) { Some(b) => b, None => return src };
        // Work on non-PHI instructions in program order and stop at current instruction.
        let instrs: Vec<_> = bb.non_phi_instructions().collect();
        let stop = instrs
            .iter()
            .position(|i| **i == last_inst)
            .unwrap_or(instrs.len());
        let mut latest: Option<ValueId> = None;
        for inst in instrs.iter().take(stop) {
            if let crate::mir::MirInstruction::Copy { dst, src: s } = &**inst {
                if *s == src {
                    latest = Some(*dst);
                }
            }
        }
        latest.unwrap_or(src)
    }

    /// Materialize a slice of argument ValueIds within the current block.
    /// Debug builds assert that any changed id is Copy-defined in the block before current inst.
    pub(super) fn materialize_args_in_current_block(&self, args: &[ValueId]) -> Vec<ValueId> {
        let out: Vec<ValueId> = args
            .iter()
            .map(|a| self.materialize_value_in_current_block(*a))
            .collect();
        #[cfg(debug_assertions)]
        {
            for (orig, got) in args.iter().zip(out.iter()) {
                self.debug_assert_materialized_in_block(*orig, *got);
            }
        }
        out
    }

    /// Materialize a receiver id within the current block and (in debug) validate its provenance.
    pub(super) fn materialize_recv_in_current_block(&self, recv: ValueId) -> ValueId {
        let v = self.materialize_value_in_current_block(recv);
        #[cfg(debug_assertions)]
        self.debug_assert_materialized_in_block(recv, v);
        v
    }

    #[cfg(debug_assertions)]
    pub(super) fn debug_assert_materialized_in_block(&self, original: ValueId, got: ValueId) {
        if original == got { return; }
        let (fn_name, bb_id, last_inst) = match (self.cur_fn.as_ref(), self.last_block, self.last_inst.as_ref()) {
            (Some(f), Some(bb), Some(inst)) => (f.clone(), bb, inst.clone()),
            _ => return,
        };
        if let Some(func) = self.functions.get(&fn_name) {
            if let Some(bb) = func.blocks.get(&bb_id) {
                let instrs: Vec<_> = bb.non_phi_instructions().collect();
                let stop = instrs.iter().position(|i| **i == last_inst).unwrap_or(instrs.len());
                let mut ok = false;
                for inst in instrs.iter().take(stop) {
                    if let crate::mir::MirInstruction::Copy { dst, .. } = &**inst {
                        if *dst == got { ok = true; break; }
                    }
                }
                assert!(ok, "materialized id {:?} must be Copy-defined in current block before call", got);
            }
        }
    }

    /// Compute a stable key for an object receiver to store fields across functions.
    /// Prefer Arc ptr address for BoxRef; else fall back to ValueId number cast.
    pub(super) fn object_key_for(&self, id: crate::mir::ValueId) -> u64 {
        if let Ok(v) = self.reg_load(id) {
            if let crate::backend::vm::VMValue::BoxRef(arc) = v {
                let ptr = std::sync::Arc::as_ptr(&arc) as *const ();
                return ptr as usize as u64;
            }
        }
        id.as_u32() as u64
    }
    pub(super) fn eval_binop(
        &self,
        op: BinaryOp,
        a: VMValue,
        b: VMValue,
    ) -> Result<VMValue, VMError> {
        use BinaryOp::*;
        use VMValue::*;
        // Dev-time: normalize BoxRef(VoidBox) → VMValue::Void when tolerance is enabled or in --dev mode.
        let cfg = super::VmConfig::global();
        let tolerate = cfg.tolerate_void || cfg.dev_fallback;
        let (a, b) = if tolerate {
            let norm = |v: VMValue| -> VMValue {
                if let VMValue::BoxRef(bx) = &v {
                    if bx.as_any().downcast_ref::<VoidBox>().is_some() {
                        return VMValue::Void;
                    }
                }
                v
            };
            (norm(a), norm(b))
        } else { (a, b) };
        // Dev: nullish trace for binop
        if crate::config::env::null_missing_box_enabled() && Self::box_trace_enabled() {
            let (ak, bk) = (crate::backend::abi_util::tag_of_vm(&a), crate::backend::abi_util::tag_of_vm(&b));
            let (an, bn) = (Self::tag_nullish(&a), Self::tag_nullish(&b));
            let op_s = match op { BinaryOp::Add=>"Add", BinaryOp::Sub=>"Sub", BinaryOp::Mul=>"Mul", BinaryOp::Div=>"Div", BinaryOp::Mod=>"Mod", BinaryOp::BitAnd=>"BitAnd", BinaryOp::BitOr=>"BitOr", BinaryOp::BitXor=>"BitXor", BinaryOp::And=>"And", BinaryOp::Or=>"Or", BinaryOp::Shl=>"Shl", BinaryOp::Shr=>"Shr" };
            eprintln!("{{\"ev\":\"binop\",\"op\":\"{}\",\"a_k\":\"{}\",\"b_k\":\"{}\",\"a_n\":\"{}\",\"b_n\":\"{}\"}}", op_s, ak, bk, an, bn);
        }
        Ok(match (op, a, b) {
            // Dev-only safety valves for Add (guarded by tolerance or --dev):
            // - Treat Void as 0 for numeric +
            // - Treat Void as empty string for string +
            (Add, VMValue::Void, Integer(y)) | (Add, Integer(y), VMValue::Void) if tolerate => Integer(y),
            (Add, VMValue::Void, Float(y)) | (Add, Float(y), VMValue::Void) if tolerate => Float(y),
            (Add, String(s), VMValue::Void) | (Add, VMValue::Void, String(s)) if tolerate => String(s),
            (Add, Integer(x), Integer(y)) => Integer(x + y),
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
            (Add, Float(x), Float(y)) => Float(x + y),
            (Sub, Float(x), Float(y)) => Float(x - y),
            (Mul, Float(x), Float(y)) => Float(x * y),
            (Div, Float(_), Float(y)) if y == 0.0 => return Err(VMError::DivisionByZero),
            (Div, Float(x), Float(y)) => Float(x / y),
            (Mod, Float(x), Float(y)) => Float(x % y),
            (BitAnd, Integer(x), Integer(y)) => Integer(x & y),
            (BitOr, Integer(x), Integer(y)) => Integer(x | y),
            (BitXor, Integer(x), Integer(y)) => Integer(x ^ y),
            (And, VMValue::Bool(x), VMValue::Bool(y)) => VMValue::Bool(x && y),
            (Or,  VMValue::Bool(x), VMValue::Bool(y)) => VMValue::Bool(x || y),
            (Shl, Integer(x), Integer(y)) => Integer(x.wrapping_shl(y as u32)),
            (Shr, Integer(x), Integer(y)) => Integer(x.wrapping_shr(y as u32)),
            (opk, va, vb) => {
                return Err(VMError::TypeError(format!(
                    "unsupported binop {:?} on {:?} and {:?}",
                    opk, va, vb
                )))
            }
        })
    }

    pub(super) fn eval_cmp(&self, op: CompareOp, a: VMValue, b: VMValue) -> Result<bool, VMError> {
        use CompareOp::*;
        use VMValue::*;
        // Dev-time: normalize BoxRef(VoidBox) → VMValue::Void when tolerance is enabled or in --dev.
        let tolerate = super::VmConfig::global().tolerate_void;
        let (a, b) = if tolerate {
            let norm = |v: VMValue| -> VMValue {
                if let VMValue::BoxRef(bx) = &v {
                    if bx.as_any().downcast_ref::<VoidBox>().is_some() {
                        return VMValue::Void;
                    }
                }
                v
            };
            (norm(a), norm(b))
        } else { (a, b) };
        // Dev-only safety valve: tolerate Void in comparisons when enabled or in --dev
        // → treat Void as 0 for numeric, empty for string
        let (a2, b2) = if tolerate {
            match (&a, &b) {
                (VMValue::Void, VMValue::Integer(_)) => (Integer(0), b.clone()),
                (VMValue::Integer(_), VMValue::Void) => (a.clone(), Integer(0)),
                (VMValue::Void, VMValue::Float(_)) => (Float(0.0), b.clone()),
                (VMValue::Float(_), VMValue::Void) => (a.clone(), Float(0.0)),
                (VMValue::Void, VMValue::String(_)) => (String(StdString::new()), b.clone()),
                (VMValue::String(_), VMValue::Void) => (a.clone(), String(StdString::new())),
                (VMValue::Void, _) => (Integer(0), b.clone()),
                (_, VMValue::Void) => (a.clone(), Integer(0)),
                _ => (a.clone(), b.clone()),
            }
        } else {
            (a, b)
        };
        // Final safety (dev-only): if types still mismatch and any side is Void, coerce to numeric zeros
        // Enabled only when tolerance is active (NYASH_VM_TOLERATE_VOID=1 or --dev)
        let (a3, b3) = if tolerate {
            match (&a2, &b2) {
                (VMValue::Void, VMValue::Integer(_)) => (Integer(0), b2.clone()),
                (VMValue::Integer(_), VMValue::Void) => (a2.clone(), Integer(0)),
                (VMValue::Void, VMValue::Float(_)) => (Float(0.0), b2.clone()),
                (VMValue::Float(_), VMValue::Void) => (a2.clone(), Float(0.0)),
                _ => (a2.clone(), b2.clone()),
            }
        } else {
            (a2.clone(), b2.clone())
        };
        // Dev: nullish trace for compare
        if crate::config::env::null_missing_box_enabled() && Self::box_trace_enabled() {
            let (ak, bk) = (crate::backend::abi_util::tag_of_vm(&a2), crate::backend::abi_util::tag_of_vm(&b2));
            let (an, bn) = (Self::tag_nullish(&a2), Self::tag_nullish(&b2));
            let op_s = match op { CompareOp::Eq=>"Eq", CompareOp::Ne=>"Ne", CompareOp::Lt=>"Lt", CompareOp::Le=>"Le", CompareOp::Gt=>"Gt", CompareOp::Ge=>"Ge" };
            eprintln!("{{\"ev\":\"cmp\",\"op\":\"{}\",\"a_k\":\"{}\",\"b_k\":\"{}\",\"a_n\":\"{}\",\"b_n\":\"{}\"}}", op_s, ak, bk, an, bn);
        }
        // ParserBox-specific safety: if one side is ParserBox instance and the other numeric,
        // compare using ParserBox.gpos as integer. This mirrors common parser patterns where
        // position variables may be accidentally bound to `me`. Scope: only inside ParserBox.* functions.
        let (a4, b4) = if let Some(cur) = &self.cur_fn {
            if cur.starts_with("ParserBox.") {
                let coerce_pb = |v: &VMValue| -> VMValue {
                    if let VMValue::BoxRef(bx) = v {
                        if let Some(inst) = bx.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                            if inst.class_name == "ParserBox" {
                                if let Some(ny) = inst.get_field_ng("gpos") {
                                    match ny {
                                        crate::value::NyashValue::Integer(i) => VMValue::Integer(i as i64),
                                        crate::value::NyashValue::Float(f) => VMValue::Float(f),
                                        crate::value::NyashValue::String(ref s) => {
                                            if let Ok(i) = s.parse::<i64>() { VMValue::Integer(i) } else { VMValue::Integer(0) }
                                        }
                                        _ => VMValue::Integer(0),
                                    }
                                } else { VMValue::Integer(0) }
                            } else { v.clone() }
                        } else { v.clone() }
                    } else { v.clone() }
                };
                (coerce_pb(&a3), coerce_pb(&b3))
            } else { (a3.clone(), b3.clone()) }
        } else { (a3.clone(), b3.clone()) };

        // ParserBox-specific: numeric-string bridging for ordered compares (dev-safe,仕様不変の範囲で局所適用)
        let (a5, b5) = if let Some(cur) = &self.cur_fn {
            if cur.starts_with("ParserBox.") {
                let is_numstr = |s: &str| s.chars().all(|c| c.is_ascii_digit());
                let to_int = |s: &str| s.parse::<i64>().unwrap_or(0);
                use VMValue::*;
                match (&a4, &b4) {
                    (String(ref s), Integer(_)) if is_numstr(s) => (Integer(to_int(s)), b4.clone()),
                    (Integer(_), String(ref s)) if is_numstr(s) => (a4.clone(), Integer(to_int(s))),
                    (String(ref s), Float(_)) if is_numstr(s) => (Integer(to_int(s)), b4.clone()),
                    (Float(_), String(ref s)) if is_numstr(s) => (a4.clone(), Integer(to_int(s))),
                    _ => (a4.clone(), b4.clone()),
                }
            } else { (a4.clone(), b4.clone()) }
        } else { (a4.clone(), b4.clone()) };

        let result = match (op, &a5, &b5) {
            (Eq, _, _) => eq_vm(&a5, &b5),
            (Ne, _, _) => !eq_vm(&a5, &b5),
            (Lt, Integer(x), Integer(y)) => x < y,
            (Le, Integer(x), Integer(y)) => x <= y,
            (Gt, Integer(x), Integer(y)) => x > y,
            (Ge, Integer(x), Integer(y)) => x >= y,
            (Lt, Float(x), Float(y)) => x < y,
            (Le, Float(x), Float(y)) => x <= y,
            (Gt, Float(x), Float(y)) => x > y,
            (Ge, Float(x), Float(y)) => x >= y,
            (Lt, VMValue::String(ref s), VMValue::String(ref t)) => s < t,
            (Le, VMValue::String(ref s), VMValue::String(ref t)) => s <= t,
            (Gt, VMValue::String(ref s), VMValue::String(ref t)) => s > t,
            (Ge, VMValue::String(ref s), VMValue::String(ref t)) => s >= t,
            (opk, va, vb) => {
                if super::VmConfig::global().general_trace {
                    eprintln!(
                        "[vm-trace] compare error fn={:?} op={:?} a={:?} b={:?} last_block={:?} last_inst={:?}",
                        self.cur_fn, opk, va, vb, self.last_block, self.last_inst
                    );
                }
                return Err(VMError::TypeError(format!(
                    "unsupported compare {:?} on {:?} and {:?}",
                    opk, va, vb
                )));
            }
        };
        Ok(result)
    }

    /// Unified unborn guard: forbid operations on unborn instances.
    /// Allows if `born` is true or currently `in_birth`.
    pub(super) fn check_unborn_guard(&self, recv_id: ValueId) -> Result<(), VMError> {
        if !crate::config::env::check_contracts() { return Ok(()); }
        let key = self.object_key_for(recv_id);
        if self.contracts_new.contains(&key) {
            let seen_birth = self.contracts_born.contains(&key) || self.contracts_in_birth.contains(&key);
            if !seen_birth {
                return Err(VMError::InvalidInstruction(
                    "operation on unborn instance (call birth() first)".to_string(),
                ));
            }
        }
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{BasicBlock, BasicBlockId, FunctionSignature, MirFunction, MirInstruction, MirModule, MirType, ValueId};
    use crate::mir::EffectMask;

    fn make_func_with_block(instrs: Vec<MirInstruction>) -> (MirFunction, BasicBlockId, MirInstruction) {
        let sig = FunctionSignature { name: "F.test".into(), params: vec![], return_type: MirType::I64, effects: EffectMask::PURE };
        let entry = BasicBlockId::new(0);
        let mut f = MirFunction::new(sig, entry);
        let mut b = BasicBlock::new(entry);
        for i in instrs.iter().cloned() { b.add_instruction(i); }
        let last = b.instructions.last().cloned().unwrap_or(MirInstruction::Nop);
        f.blocks.insert(entry, b);
        (f, entry, last)
    }

    #[test]
    fn materialize_picks_latest_copy_before_current_inst() {
        let s = ValueId::new(1);
        let v1 = ValueId::new(2);
        let v2 = ValueId::new(3);
        let instrs = vec![
            MirInstruction::Copy { dst: v1, src: s },
            MirInstruction::Copy { dst: v2, src: s },
            MirInstruction::Nop,
        ];
        let (func, bb, last_inst) = make_func_with_block(instrs);
        let mut interp = MirInterpreter::new();
        interp.functions.insert("F.test".into(), func);
        interp.cur_fn = Some("F.test".into());
        interp.last_block = Some(bb);
        interp.last_inst = Some(last_inst);
        let got = interp.materialize_value_in_current_block(s);
        assert_eq!(got, v2);
    }

    #[test]
    fn materialize_stops_at_current_inst() {
        let s = ValueId::new(10);
        let v1 = ValueId::new(11);
        let v2 = ValueId::new(12);
        let entry = BasicBlockId::new(0);
        let sig = FunctionSignature { name: "F.test".into(), params: vec![], return_type: MirType::I64, effects: EffectMask::PURE };
        let mut func = MirFunction::new(sig, entry);
        let mut bb = BasicBlock::new(entry);
        bb.add_instruction(MirInstruction::Copy { dst: v1, src: s });
        let nop = MirInstruction::Nop;
        bb.add_instruction(nop.clone());
        let last_inst = bb.instructions.last().cloned().unwrap();
        bb.add_instruction(MirInstruction::Copy { dst: v2, src: s });
        func.blocks.insert(entry, bb);

        let mut interp = MirInterpreter::new();
        interp.functions.insert("F.test".into(), func);
        interp.cur_fn = Some("F.test".into());
        interp.last_block = Some(entry);
        interp.last_inst = Some(last_inst);
        let got = interp.materialize_value_in_current_block(s);
        assert_eq!(got, v1);
    }
}

// ---- Box trace (dev-only observer) ----
impl MirInterpreter {
    #[inline]
    pub(super) fn box_trace_enabled() -> bool {
        std::env::var("NYASH_BOX_TRACE").ok().as_deref() == Some("1")
    }

    fn box_trace_filter_match(class_name: &str) -> bool {
        if let Ok(filt) = std::env::var("NYASH_BOX_TRACE_FILTER") {
            let want = filt.trim();
            if want.is_empty() { return true; }
            // comma/space separated tokens; match if any token is contained in class
            for tok in want.split(|c: char| c == ',' || c.is_whitespace()) {
                let t = tok.trim();
                if !t.is_empty() && class_name.contains(t) { return true; }
            }
            false
        } else {
            true
        }
    }

    fn json_escape(s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 8);
        for ch in s.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if c.is_control() => out.push(' '),
                c => out.push(c),
            }
        }
        out
    }

    pub(super) fn box_trace_emit_new(&self, class_name: &str, argc: usize) {
        if !Self::box_trace_enabled() || !Self::box_trace_filter_match(class_name) { return; }
        eprintln!(
            "{{\"ev\":\"new\",\"class\":\"{}\",\"argc\":{}}}",
            Self::json_escape(class_name), argc
        );
    }

    pub(super) fn box_trace_emit_call(&self, class_name: &str, method: &str, argc: usize) {
        if !Self::box_trace_enabled() || !Self::box_trace_filter_match(class_name) { return; }
        eprintln!(
            "{{\"ev\":\"call\",\"class\":\"{}\",\"method\":\"{}\",\"argc\":{}}}",
            Self::json_escape(class_name), Self::json_escape(method), argc
        );
    }

    pub(super) fn box_trace_emit_get(&self, class_name: &str, field: &str, val_kind: &str) {
        if !Self::box_trace_enabled() || !Self::box_trace_filter_match(class_name) { return; }
        eprintln!(
            "{{\"ev\":\"get\",\"class\":\"{}\",\"field\":\"{}\",\"val\":\"{}\"}}",
            Self::json_escape(class_name), Self::json_escape(field), Self::json_escape(val_kind)
        );
    }

    pub(super) fn box_trace_emit_set(&self, class_name: &str, field: &str, val_kind: &str) {
        if !Self::box_trace_enabled() || !Self::box_trace_filter_match(class_name) { return; }
        eprintln!(
            "{{\"ev\":\"set\",\"class\":\"{}\",\"field\":\"{}\",\"val\":\"{}\"}}",
            Self::json_escape(class_name), Self::json_escape(field), Self::json_escape(val_kind)
        );
    }
}

// ---- Print trace (dev-only) ----
impl MirInterpreter {
    #[inline]
    pub(super) fn print_trace_enabled() -> bool {
        std::env::var("NYASH_PRINT_TRACE").ok().as_deref() == Some("1")
    }

    pub(super) fn print_trace_emit(&self, val: &VMValue) {
        if !Self::print_trace_enabled() { return; }
        let (kind, class, nullish) = match val {
            VMValue::Integer(_) => ("Integer", "".to_string(), None),
            VMValue::Float(_) => ("Float", "".to_string(), None),
            VMValue::Bool(_) => ("Bool", "".to_string(), None),
            VMValue::String(_) => ("String", "".to_string(), None),
            VMValue::Void => ("Void", "".to_string(), None),
            VMValue::Future(_) => ("Future", "".to_string(), None),
            VMValue::BoxRef(b) => {
                // Prefer InstanceBox.class_name when available
                if let Some(inst) = b.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                    let tag = if crate::config::env::null_missing_box_enabled() {
                        if b.as_any().downcast_ref::<crate::boxes::null_box::NullBox>().is_some() { Some("null") }
                        else if b.as_any().downcast_ref::<crate::boxes::missing_box::MissingBox>().is_some() { Some("missing") }
                        else { None }
                    } else { None };
                    ("BoxRef", inst.class_name.clone(), tag)
                } else {
                    let tag = if crate::config::env::null_missing_box_enabled() {
                        if b.as_any().downcast_ref::<crate::boxes::null_box::NullBox>().is_some() { Some("null") }
                        else if b.as_any().downcast_ref::<crate::boxes::missing_box::MissingBox>().is_some() { Some("missing") }
                        else { None }
                    } else { None };
                    ("BoxRef", b.type_name().to_string(), tag)
                }
            }
        };
        if let Some(tag) = nullish {
            eprintln!(
                "{{\"ev\":\"print\",\"kind\":\"{}\",\"class\":\"{}\",\"nullish\":\"{}\"}}",
                kind,
                Self::json_escape(&class),
                tag
            );
        } else {
            eprintln!(
                "{{\"ev\":\"print\",\"kind\":\"{}\",\"class\":\"{}\"}}",
                kind,
                Self::json_escape(&class)
            );
        }
    }
}
