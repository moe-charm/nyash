//! JIT minimal ABI types independent from VM internals

#[derive(Debug, Clone, Copy)]
pub enum JitValue {
    I64(i64),
    F64(f64),
    Bool(bool),
    /// Opaque handle for host objects (future use)
    Handle(u64),
}

impl JitValue {
    pub fn as_i64(&self) -> Option<i64> {
        if let JitValue::I64(v) = self {
            Some(*v)
        } else {
            None
        }
    }
}

/// Adapter between VMValue and JitValue — keeps JIT decoupled from VM internals
pub mod adapter {
    use super::JitValue;
    use crate::backend::vm::VMValue;

    pub fn to_jit_values(args: &[VMValue]) -> Vec<JitValue> {
        args.iter()
            .map(|v| match v {
                VMValue::Integer(i) => JitValue::I64(*i),
                VMValue::Float(f) => JitValue::F64(*f),
                VMValue::Bool(b) => JitValue::Bool(*b),
                VMValue::BoxRef(arc) => {
                    let h = crate::jit::rt::handles::to_handle(arc.clone());
                    JitValue::Handle(h)
                }
                // For now, map others to handle via boxing where reasonable
                VMValue::String(s) => {
                    let bx = Box::new(crate::box_trait::StringBox::new(s));
                    let bx_dyn: Box<dyn crate::box_trait::NyashBox> = bx;
                    let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> =
                        std::sync::Arc::from(bx_dyn);
                    let h = crate::jit::rt::handles::to_handle(arc);
                    JitValue::Handle(h)
                }
                VMValue::Void => JitValue::Handle(0),
                VMValue::Future(f) => {
                    let bx_dyn: Box<dyn crate::box_trait::NyashBox> = Box::new(f.clone());
                    let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> =
                        std::sync::Arc::from(bx_dyn);
                    let h = crate::jit::rt::handles::to_handle(arc);
                    JitValue::Handle(h)
                }
            })
            .collect()
    }

    pub fn from_jit_value(v: JitValue) -> VMValue {
        match v {
            JitValue::I64(i) => VMValue::Integer(i),
            JitValue::F64(f) => VMValue::Float(f),
            JitValue::Bool(b) => VMValue::Bool(b),
            JitValue::Handle(h) => {
                if let Some(arc) = crate::jit::rt::handles::get(h) {
                    VMValue::BoxRef(arc)
                } else {
                    VMValue::Void
                }
            }
        }
    }
}
