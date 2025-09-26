/*!
 * Call Target Types
 *
 * Type-safe call target specification for unified call system
 * Part of Phase 15.5 MIR Call unification
 */

use crate::mir::ValueId;

/// Call target specification for emit_unified_call
/// Provides type-safe target resolution at the builder level
#[derive(Debug, Clone)]
pub enum CallTarget {
    /// Global function (print, panic, etc.)
    Global(String),

    /// Method call (box.method)
    Method {
        box_type: Option<String>,  // None = infer from value
        method: String,
        receiver: ValueId,
    },

    /// Constructor (new BoxType)
    Constructor(String),

    /// External function (nyash.*)
    Extern(String),

    /// Dynamic function value
    Value(ValueId),

    /// Closure creation
    Closure {
        params: Vec<String>,
        captures: Vec<(String, ValueId)>,
        me_capture: Option<ValueId>,
    },
}

impl CallTarget {
    /// Check if this target is a constructor
    pub fn is_constructor(&self) -> bool {
        matches!(self, CallTarget::Constructor(_))
    }

    /// Get the name of the target for debugging
    pub fn name(&self) -> String {
        match self {
            CallTarget::Global(name) => name.clone(),
            CallTarget::Method { method, .. } => method.clone(),
            CallTarget::Constructor(box_type) => format!("new {}", box_type),
            CallTarget::Extern(name) => name.clone(),
            CallTarget::Value(_) => "<dynamic>".to_string(),
            CallTarget::Closure { .. } => "<closure>".to_string(),
        }
    }
}