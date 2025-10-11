//! MethodRef pseudo-method handler.
//!
//! Keeps the logic that materialises a `CallableBox` behind a small
//! dedicated boundary so the main router stays focused on dispatch.

use crate::backend::vm_types::{VMError, VMValue};
use crate::boxes::callable::CallableBox;

pub struct MethodRefBox;

impl MethodRefBox {
    /// Returns `Some(Result)` when the call was recognised as a methodRef request.
    /// Falls back to the caller by yielding `None` for all other methods.
    pub fn try_route(
        receiver: &VMValue,
        method: &str,
        args: &[VMValue],
    ) -> Option<Result<VMValue, VMError>> {
        if method != "methodRef" {
            return None;
        }

        Some(Self::create_callable(receiver, args))
    }

    fn create_callable(receiver: &VMValue, args: &[VMValue]) -> Result<VMValue, VMError> {
        if args.len() != 2 {
            return Err(VMError::InvalidInstruction(crate::common::diagnostics::msg::no_method_arity(
                "<receiver>", "methodRef", args.len(), &[2]
            )));
        }

        let bound_box = match receiver {
            VMValue::BoxRef(bx) => bx,
            _ => {
                return Err(VMError::InvalidInstruction(
                    "methodRef requires Box receiver".into(),
                ))
            }
        };

        let method_name = match args.get(0) {
            Some(VMValue::String(s)) => s.clone(),
            _ => {
                return Err(VMError::InvalidInstruction(
                    "methodRef: name must be String".into(),
                ))
            }
        };

        let arity_i64 = match args.get(1) {
            Some(VMValue::Integer(i)) => *i,
            _ => {
                return Err(VMError::InvalidInstruction(
                    "methodRef: arity must be Integer".into(),
                ))
            }
        };

        if arity_i64 < 0 {
            return Err(VMError::InvalidInstruction(
                "methodRef: arity must be >= 0".into(),
            ));
        }

        let callable = CallableBox::new(Some(bound_box.share_box()), method_name, arity_i64 as usize);
        Ok(VMValue::from_nyash_box(Box::new(callable)))
    }
}
