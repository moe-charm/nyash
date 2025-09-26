use std::collections::HashMap;

use super::Semantics;
use crate::backend::vm::VMValue;
use crate::mir::{BasicBlockId, ValueId};

/// Minimal Semantics for Cranelift skeleton (Const/Add/Return)
pub struct ClifSemanticsSkeleton {
    pub mem: HashMap<ValueId, VMValue>,
    pub ret_val: Option<VMValue>,
}

impl ClifSemanticsSkeleton {
    pub fn new() -> Self {
        Self {
            mem: HashMap::new(),
            ret_val: None,
        }
    }
}

impl Semantics for ClifSemanticsSkeleton {
    type Val = VMValue;
    type Ptr = ValueId;
    type BB = BasicBlockId;

    // Constants
    fn const_i64(&mut self, v: i64) -> Self::Val {
        VMValue::Integer(v)
    }
    fn const_f64(&mut self, v: f64) -> Self::Val {
        VMValue::Float(v)
    }
    fn const_bool(&mut self, v: bool) -> Self::Val {
        VMValue::Bool(v)
    }
    fn const_null(&mut self) -> Self::Val {
        VMValue::Void
    }
    fn const_str(&mut self, s: &str) -> Self::Val {
        VMValue::String(s.to_string())
    }

    // Unary/Binary/Compare (minimal)
    fn neg(&mut self, x: Self::Val) -> Self::Val {
        match x {
            VMValue::Integer(i) => VMValue::Integer(-i),
            VMValue::Float(f) => VMValue::Float(-f),
            v => v,
        }
    }
    fn not(&mut self, x: Self::Val) -> Self::Val {
        VMValue::Bool(matches!(
            x,
            VMValue::Bool(false) | VMValue::Integer(0) | VMValue::Void
        ))
    }
    fn bit_not(&mut self, x: Self::Val) -> Self::Val {
        if let VMValue::Integer(i) = x {
            VMValue::Integer(!i)
        } else {
            x
        }
    }
    fn add(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        use VMValue as V;
        match (a, b) {
            (V::Integer(x), V::Integer(y)) => V::Integer(x + y),
            (V::Float(x), V::Float(y)) => V::Float(x + y),
            (V::Float(x), V::Integer(y)) => V::Float(x + y as f64),
            (V::Integer(x), V::Float(y)) => V::Float(x as f64 + y),
            (V::String(s), V::String(t)) => V::String(format!("{}{}", s, t)),
            (V::String(s), V::Integer(y)) => V::String(format!("{}{}", s, y)),
            (V::Integer(x), V::String(t)) => V::String(format!("{}{}", x, t)),
            (l, r) => {
                let s = format!("{}{}", l.to_string(), r.to_string());
                V::String(s)
            }
        }
    }
    fn sub(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        match (a, b) {
            (VMValue::Integer(x), VMValue::Integer(y)) => VMValue::Integer(x - y),
            _ => VMValue::Void,
        }
    }
    fn mul(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        match (a, b) {
            (VMValue::Integer(x), VMValue::Integer(y)) => VMValue::Integer(x * y),
            _ => VMValue::Void,
        }
    }
    fn div(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        match (a, b) {
            (VMValue::Integer(x), VMValue::Integer(y)) if y != 0 => VMValue::Integer(x / y),
            _ => VMValue::Void,
        }
    }
    fn modulo(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        match (a, b) {
            (VMValue::Integer(x), VMValue::Integer(y)) if y != 0 => VMValue::Integer(x % y),
            _ => VMValue::Void,
        }
    }
    fn cmp_eq(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        VMValue::Bool(a.to_string() == b.to_string())
    }
    fn cmp_ne(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        VMValue::Bool(a.to_string() != b.to_string())
    }
    fn cmp_lt(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        match (a, b) {
            (VMValue::Integer(x), VMValue::Integer(y)) => VMValue::Bool(x < y),
            _ => VMValue::Bool(false),
        }
    }
    fn cmp_le(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        match (a, b) {
            (VMValue::Integer(x), VMValue::Integer(y)) => VMValue::Bool(x <= y),
            _ => VMValue::Bool(false),
        }
    }
    fn cmp_gt(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        match (a, b) {
            (VMValue::Integer(x), VMValue::Integer(y)) => VMValue::Bool(x > y),
            _ => VMValue::Bool(false),
        }
    }
    fn cmp_ge(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        match (a, b) {
            (VMValue::Integer(x), VMValue::Integer(y)) => VMValue::Bool(x >= y),
            _ => VMValue::Bool(false),
        }
    }

    // Memory & control (minimal)
    fn alloca_ptr(&mut self, vid: ValueId) -> Self::Ptr {
        vid
    }
    fn load(&mut self, ptr: &Self::Ptr) -> Self::Val {
        self.mem.get(ptr).cloned().unwrap_or(VMValue::Void)
    }
    fn store(&mut self, ptr: &Self::Ptr, v: Self::Val) {
        self.mem.insert(*ptr, v);
    }
    fn jump(&mut self, _target: BasicBlockId) {}
    fn branch(&mut self, _cond: Self::Val, _then_bb: BasicBlockId, _else_bb: BasicBlockId) {}
    fn phi_select(&mut self, _incoming: &[(BasicBlockId, Self::Val)]) -> Self::Val {
        VMValue::Void
    }
    fn ret(&mut self, v: Option<Self::Val>) {
        self.ret_val = v;
    }

    // Host/Box calls (unimplemented in skeleton)
    fn new_box(&mut self, _type_id: i64, _args: &[Self::Val]) -> Self::Val {
        VMValue::Void
    }
    fn box_call_tagged(
        &mut self,
        _type_id: i64,
        _method_id: i64,
        _recv: Self::Val,
        _argv: &[Self::Val],
        _tags: &[i64],
    ) -> Self::Val {
        VMValue::Void
    }
    fn extern_call(&mut self, _iface: &str, _method: &str, _args: &[Self::Val]) -> Self::Val {
        VMValue::Void
    }
}
