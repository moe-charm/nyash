//! CallBoundaryBox: unify JIT→VM return conversion in one place
use super::abi::JitValue;
use crate::backend::vm::VMValue;

pub struct CallBoundaryBox;

impl CallBoundaryBox {
    pub fn to_vm(ret_ty: &crate::mir::MirType, v: JitValue) -> VMValue {
        match ret_ty {
            crate::mir::MirType::Float => match v {
                JitValue::F64(f) => VMValue::Float(f),
                JitValue::I64(i) => VMValue::Float(i as f64),
                JitValue::Bool(b) => VMValue::Float(if b { 1.0 } else { 0.0 }),
                JitValue::Handle(h) => {
                    if let Some(_) = crate::jit::rt::handles::get(h) {
                        VMValue::Float(0.0)
                    } else {
                        VMValue::Float(0.0)
                    }
                }
            },
            crate::mir::MirType::Integer => match v {
                JitValue::I64(i) => VMValue::Integer(i),
                JitValue::F64(f) => VMValue::Integer(f as i64),
                JitValue::Bool(b) => VMValue::Integer(if b { 1 } else { 0 }),
                JitValue::Handle(h) => {
                    if let Some(_) = crate::jit::rt::handles::get(h) {
                        VMValue::Integer(0)
                    } else {
                        VMValue::Integer(0)
                    }
                }
            },
            crate::mir::MirType::Bool => match v {
                JitValue::Bool(b) => VMValue::Bool(b),
                JitValue::I64(i) => VMValue::Bool(i != 0),
                JitValue::F64(f) => VMValue::Bool(f != 0.0),
                JitValue::Handle(h) => {
                    if let Some(_) = crate::jit::rt::handles::get(h) {
                        VMValue::Bool(true)
                    } else {
                        VMValue::Bool(false)
                    }
                }
            },
            // Box-like returns: if we received a handle id (encoded as I64), resolve to BoxRef; also honor explicit Handle
            crate::mir::MirType::Box(_)
            | crate::mir::MirType::String
            | crate::mir::MirType::Array(_)
            | crate::mir::MirType::Future(_) => match v {
                JitValue::I64(i) => {
                    let h = i as u64;
                    if let Some(arc) = crate::jit::rt::handles::get(h) {
                        VMValue::BoxRef(arc)
                    } else {
                        VMValue::Integer(i)
                    }
                }
                JitValue::Handle(h) => {
                    if let Some(arc) = crate::jit::rt::handles::get(h) {
                        VMValue::BoxRef(arc)
                    } else {
                        VMValue::Void
                    }
                }
                JitValue::F64(f) => VMValue::Float(f),
                JitValue::Bool(b) => VMValue::Bool(b),
            },
            _ => {
                // Default adapter with heuristic: treat I64 matching a known handle as BoxRef
                match v {
                    JitValue::I64(i) => {
                        let h = i as u64;
                        if let Some(arc) = crate::jit::rt::handles::get(h) {
                            VMValue::BoxRef(arc)
                        } else {
                            super::abi::adapter::from_jit_value(JitValue::I64(i))
                        }
                    }
                    _ => super::abi::adapter::from_jit_value(v),
                }
            }
        }
    }
}
