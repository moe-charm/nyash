use std::collections::HashMap;

use crate::backend::vm::VMValue;
use crate::mir::{BasicBlockId, ValueId};

use super::Semantics;

/// Minimal VM-backed Semantics implementation used for parity checks.
/// This is not a full VM; it only materializes constants/binary ops and records last return.
pub struct VmSemantics {
    pub vals: HashMap<ValueId, VMValue>,
    pub last_ret: Option<VMValue>,
}

impl VmSemantics {
    pub fn new() -> Self {
        Self {
            vals: HashMap::new(),
            last_ret: None,
        }
    }
}

impl Semantics for VmSemantics {
    type Val = VMValue;
    type Ptr = ValueId; // address by MIR value id (local slot semantics)
    type BB = BasicBlockId;

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

    fn neg(&mut self, x: Self::Val) -> Self::Val {
        match x {
            VMValue::Integer(i) => VMValue::Integer(-i),
            VMValue::Float(f) => VMValue::Float(-f),
            _ => VMValue::Integer(0),
        }
    }
    fn not(&mut self, x: Self::Val) -> Self::Val {
        match x {
            VMValue::Bool(b) => VMValue::Bool(!b),
            VMValue::Integer(i) => VMValue::Bool(i == 0),
            _ => VMValue::Bool(false),
        }
    }
    fn bit_not(&mut self, x: Self::Val) -> Self::Val {
        match x {
            VMValue::Integer(i) => VMValue::Integer(!i),
            _ => VMValue::Integer(0),
        }
    }
    fn add(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        match (a, b) {
            (VMValue::Integer(x), VMValue::Integer(y)) => VMValue::Integer(x + y),
            (VMValue::Float(x), VMValue::Float(y)) => VMValue::Float(x + y),
            _ => VMValue::Integer(0),
        }
    }
    fn sub(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        match (a, b) {
            (VMValue::Integer(x), VMValue::Integer(y)) => VMValue::Integer(x - y),
            (VMValue::Float(x), VMValue::Float(y)) => VMValue::Float(x - y),
            _ => VMValue::Integer(0),
        }
    }
    fn mul(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        match (a, b) {
            (VMValue::Integer(x), VMValue::Integer(y)) => VMValue::Integer(x * y),
            (VMValue::Float(x), VMValue::Float(y)) => VMValue::Float(x * y),
            _ => VMValue::Integer(0),
        }
    }
    fn div(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        match (a, b) {
            (VMValue::Integer(x), VMValue::Integer(y)) if y != 0 => VMValue::Integer(x / y),
            (VMValue::Float(x), VMValue::Float(y)) => VMValue::Float(x / y),
            _ => VMValue::Integer(0),
        }
    }
    fn modulo(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        match (a, b) {
            (VMValue::Integer(x), VMValue::Integer(y)) if y != 0 => VMValue::Integer(x % y),
            _ => VMValue::Integer(0),
        }
    }
    fn cmp_eq(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        VMValue::Bool(a == b)
    }
    fn cmp_ne(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        VMValue::Bool(a != b)
    }
    fn cmp_lt(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        match (a, b) {
            (VMValue::Integer(x), VMValue::Integer(y)) => VMValue::Bool(x < y),
            (VMValue::Float(x), VMValue::Float(y)) => VMValue::Bool(x < y),
            _ => VMValue::Bool(false),
        }
    }
    fn cmp_le(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        match (a, b) {
            (VMValue::Integer(x), VMValue::Integer(y)) => VMValue::Bool(x <= y),
            (VMValue::Float(x), VMValue::Float(y)) => VMValue::Bool(x <= y),
            _ => VMValue::Bool(false),
        }
    }
    fn cmp_gt(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        match (a, b) {
            (VMValue::Integer(x), VMValue::Integer(y)) => VMValue::Bool(x > y),
            (VMValue::Float(x), VMValue::Float(y)) => VMValue::Bool(x > y),
            _ => VMValue::Bool(false),
        }
    }
    fn cmp_ge(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        match (a, b) {
            (VMValue::Integer(x), VMValue::Integer(y)) => VMValue::Bool(x >= y),
            (VMValue::Float(x), VMValue::Float(y)) => VMValue::Bool(x >= y),
            _ => VMValue::Bool(false),
        }
    }

    fn alloca_ptr(&mut self, vid: ValueId) -> Self::Ptr {
        vid
    }
    fn load(&mut self, ptr: &Self::Ptr) -> Self::Val {
        self.vals.get(ptr).cloned().unwrap_or(VMValue::Integer(0))
    }
    fn store(&mut self, ptr: &Self::Ptr, v: Self::Val) {
        self.vals.insert(*ptr, v);
    }
    fn jump(&mut self, _target: BasicBlockId) {}
    fn branch(&mut self, _cond: Self::Val, _then_bb: BasicBlockId, _else_bb: BasicBlockId) {}
    fn phi_select(&mut self, incoming: &[(BasicBlockId, Self::Val)]) -> Self::Val {
        incoming
            .last()
            .map(|(_, v)| v.clone())
            .unwrap_or(VMValue::Integer(0))
    }
    fn ret(&mut self, v: Option<Self::Val>) {
        self.last_ret = v;
    }

    fn new_box(&mut self, _type_id: i64, _args: &[Self::Val]) -> Self::Val {
        VMValue::Integer(0)
    }
    fn box_call_tagged(
        &mut self,
        _type_id: i64,
        _method_id: i64,
        _recv: Self::Val,
        _argv: &[Self::Val],
        _tags: &[i64],
    ) -> Self::Val {
        VMValue::Integer(0)
    }
    fn extern_call(&mut self, _iface: &str, _method: &str, _args: &[Self::Val]) -> Self::Val {
        VMValue::Integer(0)
    }
}
