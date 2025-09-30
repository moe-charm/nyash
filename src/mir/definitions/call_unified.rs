/*!
 * Unified MIR Call Definitions - ChatGPT5 Pro A++ Design
 *
 * This module contains all call-related definitions for the unified MIR Call instruction
 * that replaces Call/BoxCall/PluginInvoke/ExternCall/NewBox/NewClosure
 */

use crate::mir::{Effect, EffectMask, ValueId};

/// Certainty of callee type information for method calls
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeCertainty {
    /// Receiver class is known (from origin propagation or static context)
    Known,
    /// Receiver may be a union/merged flow; class not uniquely known
    Union,
}

/// Call target specification for type-safe function resolution
/// Replaces runtime string-based resolution with compile-time typed targets
#[derive(Debug, Clone, PartialEq)]
pub enum Callee {
    /// Global function call (e.g., nyash.builtin.print)
    /// Resolves to built-in or global functions at compile time
    Global(String),

    /// Module function call (e.g., ParserBox.starts_with/3)
    /// Fully qualified function present in the MIR module function table
    ModuleFunction(String),

    /// Box method call with explicit receiver
    /// Enables static resolution of box.method() patterns
    Method {
        box_name: String,           // "StringBox", "ConsoleStd", etc.
        method: String,             // "upper", "print", etc.
        receiver: Option<ValueId>,  // Some(obj) for instance, None for static/constructor
        certainty: TypeCertainty,   // Phase 3: known vs union
    },

    /// Constructor call (NewBox equivalent)
    /// Creates new Box instances with birth() method
    Constructor {
        box_type: String,           // "StringBox", "ArrayBox", etc.
        // Constructor doesn't have a receiver
    },

    /// Closure creation (NewClosure equivalent)
    /// Creates function values with captured variables
    Closure {
        params: Vec<String>,
        captures: Vec<(String, ValueId)>,
        me_capture: Option<ValueId>,  // Optional 'me' weak capture
    },

    /// Dynamic function value call
    /// Preserves first-class function semantics for variables containing functions
    Value(ValueId),

    /// External C ABI function call
    /// Direct mapping to host/runtime functions
    Extern(String),
}

impl Callee {
    /// Check if this is a constructor call
    pub fn is_constructor(&self) -> bool {
        matches!(self, Callee::Constructor { .. } | Callee::Closure { .. })
    }

    /// Check if this is a method call with receiver
    pub fn has_receiver(&self) -> bool {
        match self {
            Callee::Method { receiver, .. } => receiver.is_some(),
            _ => false,
        }
    }

    /// Get the receiver if this is a method call
    pub fn receiver(&self) -> Option<ValueId> {
        match self {
            Callee::Method { receiver, .. } => *receiver,
            _ => None,
        }
    }
}

/// Call flags for unified MIR Call instruction
/// Controls call behavior and optimization hints
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallFlags {
    /// Tail call optimization hint
    pub tail_call: bool,
    /// Function never returns (e.g., panic, exit)
    pub no_return: bool,
    /// Call can be inlined
    pub can_inline: bool,
    /// Call is a constructor (NewBox/NewClosure)
    pub is_constructor: bool,
}

impl CallFlags {
    /// Create default call flags (all false)
    pub const fn new() -> Self {
        CallFlags {
            tail_call: false,
            no_return: false,
            can_inline: false,
            is_constructor: false,
        }
    }

    /// Create flags for constructor calls
    pub const fn constructor() -> Self {
        CallFlags {
            tail_call: false,
            no_return: false,
            can_inline: false,
            is_constructor: true,
        }
    }

    /// Create flags for no-return calls (panic, exit)
    pub const fn no_return() -> Self {
        CallFlags {
            tail_call: false,
            no_return: true,
            can_inline: false,
            is_constructor: false,
        }
    }

    /// Builder method to set tail call flag
    pub fn with_tail_call(mut self) -> Self {
        self.tail_call = true;
        self
    }

    /// Builder method to set inline flag
    pub fn with_inline(mut self) -> Self {
        self.can_inline = true;
        self
    }
}

impl Default for CallFlags {
    fn default() -> Self {
        CallFlags::new()
    }
}

/// Unified MIR Call instruction - replaces Call/BoxCall/PluginInvoke/ExternCall/NewBox/NewClosure
/// ChatGPT5 Pro A++ design for complete call unification
#[derive(Debug, Clone, PartialEq)]
pub struct MirCall {
    /// Destination value for result (None for void calls)
    pub dst: Option<ValueId>,
    /// Call target specification (includes receiver for methods)
    pub callee: Callee,
    /// Arguments to the call (receiver NOT included here if method call)
    pub args: Vec<ValueId>,
    /// Call behavior flags
    pub flags: CallFlags,
    /// Effect mask for optimization and analysis
    pub effects: EffectMask,
}

impl MirCall {
    /// Create a new MirCall with default flags and pure effects
    pub fn new(dst: Option<ValueId>, callee: Callee, args: Vec<ValueId>) -> Self {
        MirCall {
            dst,
            callee,
            args,
            flags: CallFlags::new(),
            effects: EffectMask::PURE,
        }
    }

    /// Create a global function call
    pub fn global(dst: Option<ValueId>, name: String, args: Vec<ValueId>) -> Self {
        MirCall::new(dst, Callee::Global(name), args)
    }

    /// Create a method call
    pub fn method(
        dst: Option<ValueId>,
        box_name: String,
        method: String,
        receiver: ValueId,
        args: Vec<ValueId>,
    ) -> Self {
        MirCall::new(
            dst,
            Callee::Method {
                box_name,
                method,
                receiver: Some(receiver),
                certainty: TypeCertainty::Known,
            },
            args,
        )
    }

    /// Create an external call
    pub fn external(dst: Option<ValueId>, name: String, args: Vec<ValueId>) -> Self {
        let mut call = MirCall::new(dst, Callee::Extern(name), args);
        call.effects = EffectMask::IO;  // External calls have I/O effects
        call
    }

    /// Create a constructor call (NewBox equivalent)
    pub fn constructor(dst: ValueId, box_type: String, args: Vec<ValueId>) -> Self {
        let mut call = MirCall::new(
            Some(dst),
            Callee::Constructor { box_type },
            args,
        );
        call.flags = CallFlags::constructor();
        call.effects = EffectMask::PURE.add(Effect::Alloc);
        call
    }

    /// Create a closure call (NewClosure equivalent)
    pub fn closure(
        dst: ValueId,
        params: Vec<String>,
        captures: Vec<(String, ValueId)>,
        me_capture: Option<ValueId>,
    ) -> Self {
        let mut call = MirCall::new(
            Some(dst),
            Callee::Closure {
                params,
                captures,
                me_capture,
            },
            vec![],  // Closures don't have regular args at creation
        );
        call.flags = CallFlags::constructor();
        call.effects = EffectMask::PURE.add(Effect::Alloc);
        call
    }

    /// Set tail call flag
    pub fn with_tail_call(mut self) -> Self {
        self.flags.tail_call = true;
        self
    }

    /// Set effects
    pub fn with_effects(mut self, effects: EffectMask) -> Self {
        self.effects = effects;
        self
    }

    /// Check if this call produces a value
    pub fn has_result(&self) -> bool {
        self.dst.is_some()
    }

    /// Get the effective effects for this call
    pub fn effective_effects(&self) -> EffectMask {
        // Constructors always allocate
        if self.flags.is_constructor {
            self.effects.add(Effect::Alloc)
        } else {
            self.effects
        }
    }
}

/// Helper functions for MirCall migration
pub mod migration {
    use super::*;

    /// Convert legacy Call instruction to MirCall
    pub fn from_legacy_call(
        dst: Option<ValueId>,
        func: ValueId,
        callee: Option<Callee>,
        args: Vec<ValueId>,
        effects: EffectMask,
    ) -> MirCall {
        // If new callee is provided, use it
        if let Some(callee) = callee {
            let mut call = MirCall::new(dst, callee, args);
            call.effects = effects;
            call
        } else {
            // Fall back to Value call for legacy
            let mut call = MirCall::new(dst, Callee::Value(func), args);
            call.effects = effects;
            call
        }
    }

    /// Convert BoxCall to MirCall
    pub fn from_box_call(
        dst: Option<ValueId>,
        box_val: ValueId,
        method: String,
        args: Vec<ValueId>,
        effects: EffectMask,
    ) -> MirCall {
        // For BoxCall, we need to infer the box type
        // Mark certainty as Union (unknown at this stage)
        let mut call = MirCall::new(
            dst,
            Callee::Method {
                box_name: "UnknownBox".to_string(),
                method,
                receiver: Some(box_val),
                certainty: TypeCertainty::Union,
            },
            args,
        );
        call.effects = effects;
        call
    }

    /// Convert NewBox to MirCall
    pub fn from_new_box(
        dst: ValueId,
        box_type: String,
        args: Vec<ValueId>,
    ) -> MirCall {
        MirCall::constructor(dst, box_type, args)
    }

    /// Convert ExternCall to MirCall
    pub fn from_extern_call(
        dst: Option<ValueId>,
        iface_name: String,
        method_name: String,
        args: Vec<ValueId>,
        effects: EffectMask,
    ) -> MirCall {
        let full_name = format!("{}.{}", iface_name, method_name);
        MirCall::external(dst, full_name, args).with_effects(effects)
    }
}
