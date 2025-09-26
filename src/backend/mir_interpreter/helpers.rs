use super::*;
use std::string::String as StdString;

impl MirInterpreter {
    pub(super) fn reg_load(&self, id: ValueId) -> Result<VMValue, VMError> {
        match self.regs.get(&id).cloned() {
            Some(v) => Ok(v),
            None => {
                if std::env::var("NYASH_VM_TRACE").ok().as_deref() == Some("1")
                    || std::env::var("NYASH_VM_TRACE_EXEC").ok().as_deref() == Some("1")
                {
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
                Err(VMError::InvalidValue(format!(
                    "use of undefined value {:?}",
                    id
                )))
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
        Ok(match (op, a, b) {
            // Safety valve: treat Void as 0 for + (dev fallback for scanners)
            (Add, VMValue::Void, Integer(y)) => Integer(y),
            (Add, Integer(x), VMValue::Void) => Integer(x),
            (Add, VMValue::Void, Float(y)) => Float(y),
            (Add, Float(x), VMValue::Void) => Float(x),
            // Dev-only safety valve: treat Void as empty string on string concatenation
            // Guarded by NYASH_VM_TOLERATE_VOID=1
            (Add, String(s), VMValue::Void) | (Add, VMValue::Void, String(s))
                if std::env::var("NYASH_VM_TOLERATE_VOID").ok().as_deref() == Some("1") =>
            {
                String(s)
            }
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
        // Dev-only safety valve: tolerate Void in comparisons when enabled
        // NYASH_VM_TOLERATE_VOID=1 → treat Void as 0 for numeric, empty for string
        let (a2, b2) = if std::env::var("NYASH_VM_TOLERATE_VOID").ok().as_deref() == Some("1") {
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
        let result = match (op, &a2, &b2) {
            (Eq, _, _) => eq_vm(&a2, &b2),
            (Ne, _, _) => !eq_vm(&a2, &b2),
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
                if std::env::var("NYASH_VM_TRACE").ok().as_deref() == Some("1") {
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
