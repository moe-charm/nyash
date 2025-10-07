use super::super::*;
use crate::backend::mir_interpreter::VmConfig;
use crate::box_trait::VoidBox;
use std::string::String as StdString;

impl MirInterpreter {
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

    pub(in crate::backend::mir_interpreter) fn reg_load(&self, id: ValueId) -> Result<VMValue, VMError> {
        match self.regs.get(&id).cloned() {
            Some(v) => Ok(v),
            None => {
                let cfg = VmConfig::global();
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

    pub(in crate::backend::mir_interpreter) fn eval_binop(
        &self,
        op: BinaryOp,
        a: VMValue,
        b: VMValue,
    ) -> Result<VMValue, VMError> {
        use BinaryOp::*;
        use VMValue::*;
        // Dev-time: normalize BoxRef(VoidBox) → VMValue::Void when tolerance is enabled or in --dev mode.
        let cfg = VmConfig::global();
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
        let nyk_stub = std::env::var("NYASH_ENABLE_NYKERNEL_STUB").ok().as_deref() == Some("1");
        Ok(match (op, a, b) {
            // Dev-only safety valves for Add (guarded by tolerance or --dev):
            // - Treat Void as 0 for numeric +
            // - Treat Void as empty string for string +
            (Mul, VMValue::Void, VMValue::Integer(_)) | (Mul, VMValue::Integer(_), VMValue::Void) if tolerate || nyk_stub => Integer(0),
            (Mul, VMValue::Void, VMValue::Float(_)) | (Mul, VMValue::Float(_), VMValue::Void) if tolerate || nyk_stub => Float(0.0),
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

    pub(in crate::backend::mir_interpreter) fn eval_cmp(&self, op: CompareOp, a: VMValue, b: VMValue) -> Result<bool, VMError> {
        use CompareOp::*;
        use VMValue::*;
        // Dev-time: normalize BoxRef(VoidBox) → VMValue::Void when tolerance is enabled or in --dev.
        let tolerate = VmConfig::global().tolerate_void;
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
                if VmConfig::global().general_trace {
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
}
