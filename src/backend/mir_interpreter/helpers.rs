use super::*;

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

    pub(super) fn eval_binop(
        &self,
        op: BinaryOp,
        a: VMValue,
        b: VMValue,
    ) -> Result<VMValue, VMError> {
        use BinaryOp::*;
        use VMValue::*;
        Ok(match (op, a, b) {
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
            (Lt, VMValue::String(ref s), VMValue::String(ref t)) => s < t,
            (Le, VMValue::String(ref s), VMValue::String(ref t)) => s <= t,
            (Gt, VMValue::String(ref s), VMValue::String(ref t)) => s > t,
            (Ge, VMValue::String(ref s), VMValue::String(ref t)) => s >= t,
            (opk, va, vb) => {
                return Err(VMError::TypeError(format!(
                    "unsupported compare {:?} on {:?} and {:?}",
                    opk, va, vb
                )))
            }
        })
    }
}
