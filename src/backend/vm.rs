/*!
 * VM Backend - Execute MIR instructions in a virtual machine
 *
 * Purpose: Core VM (execute loop, storage, control-flow, integration glue)
 * Responsibilities: fetch/dispatch instructions, manage values/blocks, stats hooks
 * Key APIs: VM::execute_module, execute_instruction, get_value/set_value
 * Typical Callers: runner (VM backend), instruction handlers (vm_instructions)
 */

use crate::mir::{ConstValue, ValueId, BasicBlockId, MirModule, MirFunction, MirInstruction};
use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox, VoidBox};
use std::collections::HashMap;
use std::sync::Arc;
use crate::runtime::NyashRuntime;
use crate::scope_tracker::ScopeTracker;
// MirModule is already imported via crate::mir at top
use crate::instance_v2::InstanceBox;
use super::vm_phi::LoopExecutor;
use std::time::Instant;
use super::frame::ExecutionFrame;
use super::control_flow;

// Phase 9.78a: Import necessary components for unified Box handling
// TODO: Re-enable when interpreter refactoring is complete
// use crate::box_factory::UnifiedBoxRegistry;
// use crate::instance_v2::InstanceBox;
// use crate::interpreter::BoxDeclaration;
// use crate::scope_tracker::ScopeTracker;
// #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
// use crate::runtime::plugin_loader_v2::PluginLoaderV2;

/// VM execution error
#[derive(Debug)]
pub enum VMError {
    InvalidValue(String),
    InvalidInstruction(String),
    InvalidBasicBlock(String),
    DivisionByZero,
    StackUnderflow,
    TypeError(String),
}

impl std::fmt::Display for VMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VMError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            VMError::InvalidInstruction(msg) => write!(f, "Invalid instruction: {}", msg),
            VMError::InvalidBasicBlock(msg) => write!(f, "Invalid basic block: {}", msg),
            VMError::DivisionByZero => write!(f, "Division by zero"),
            VMError::StackUnderflow => write!(f, "Stack underflow"),
            VMError::TypeError(msg) => write!(f, "Type error: {}", msg),
        }
    }
}

impl std::error::Error for VMError {}

/// VM value representation
#[derive(Debug, Clone)]
pub enum VMValue {
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Future(crate::boxes::future::FutureBox),
    Void,
    // Phase 9.78a: Add BoxRef for complex Box types
    BoxRef(Arc<dyn NyashBox>),
}

// Manual PartialEq implementation to avoid requiring PartialEq on FutureBox
impl PartialEq for VMValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (VMValue::Integer(a), VMValue::Integer(b)) => a == b,
            (VMValue::Float(a), VMValue::Float(b)) => a == b,
            (VMValue::Bool(a), VMValue::Bool(b)) => a == b,
            (VMValue::String(a), VMValue::String(b)) => a == b,
            (VMValue::Void, VMValue::Void) => true,
            // Future equality semantics are not defined; treat distinct futures as not equal
            (VMValue::Future(_), VMValue::Future(_)) => false,
            // BoxRef equality by reference
            (VMValue::BoxRef(_), VMValue::BoxRef(_)) => false,
            _ => false,
        }
    }
}

impl VMValue {
    /// Convert to NyashBox for output
    pub fn to_nyash_box(&self) -> Box<dyn NyashBox> {
        match self {
            VMValue::Integer(i) => Box::new(IntegerBox::new(*i)),
            VMValue::Float(f) => Box::new(crate::boxes::FloatBox::new(*f)),
            VMValue::Bool(b) => Box::new(BoolBox::new(*b)),
            VMValue::String(s) => Box::new(StringBox::new(s)),
            VMValue::Future(f) => Box::new(f.clone()),
            VMValue::Void => Box::new(VoidBox::new()),
            // BoxRef returns a shared handle (do NOT birth a new instance)
            VMValue::BoxRef(arc_box) => arc_box.share_box(),
        }
    }
    
    
    /// Get string representation for printing
    pub fn to_string(&self) -> String {
        match self {
            VMValue::Integer(i) => i.to_string(),
            VMValue::Float(f) => f.to_string(),
            VMValue::Bool(b) => b.to_string(),
            VMValue::String(s) => s.clone(),
            VMValue::Future(f) => f.to_string_box().value,
            VMValue::Void => "void".to_string(),
            VMValue::BoxRef(arc_box) => arc_box.to_string_box().value,
        }
    }
    
    /// Attempt to convert to integer
    pub fn as_integer(&self) -> Result<i64, VMError> {
        match self {
            VMValue::Integer(i) => Ok(*i),
            _ => Err(VMError::TypeError(format!("Expected integer, got {:?}", self))),
        }
    }
    
    /// Attempt to convert to bool
    pub fn as_bool(&self) -> Result<bool, VMError> {
        match self {
            VMValue::Bool(b) => Ok(*b),
            VMValue::Integer(i) => Ok(*i != 0),
            // Pragmatic coercions for dynamic boxes
            VMValue::BoxRef(b) => {
                // BoolBox → bool
                if let Some(bb) = b.as_any().downcast_ref::<BoolBox>() {
                    return Ok(bb.value);
                }
                // IntegerBox → truthy if non-zero (legacy and new)
                if let Some(ib) = b.as_any().downcast_ref::<IntegerBox>() { return Ok(ib.value != 0); }
                if let Some(ib) = b.as_any().downcast_ref::<crate::boxes::integer_box::IntegerBox>() { return Ok(ib.value != 0); }
                // VoidBox → false (nullish false)
                if b.as_any().downcast_ref::<VoidBox>().is_some() {
                    return Ok(false);
                }
                Err(VMError::TypeError(format!("Expected bool, got BoxRef({})", b.type_name())))
            }
            // Treat plain Void as false for logical contexts
            VMValue::Void => Ok(false),
            _ => Err(VMError::TypeError(format!("Expected bool, got {:?}", self))),
        }
    }
    
    /// Convert from NyashBox to VMValue  
    pub fn from_nyash_box(nyash_box: Box<dyn crate::box_trait::NyashBox>) -> VMValue {
        // Try to downcast to known types for optimization
        if let Some(int_box) = nyash_box.as_any().downcast_ref::<IntegerBox>() {
            VMValue::Integer(int_box.value)
        } else if let Some(bool_box) = nyash_box.as_any().downcast_ref::<BoolBox>() {
            VMValue::Bool(bool_box.value)
        } else if let Some(string_box) = nyash_box.as_any().downcast_ref::<StringBox>() {
            VMValue::String(string_box.value.clone())
        } else if let Some(future_box) = nyash_box.as_any().downcast_ref::<crate::boxes::future::FutureBox>() {
            VMValue::Future(future_box.clone())
        } else {
            // Phase 9.78a: For all other Box types (user-defined, plugin), store as BoxRef
            VMValue::BoxRef(Arc::from(nyash_box))
        }
    }
}

impl From<&ConstValue> for VMValue {
    fn from(const_val: &ConstValue) -> Self {
        match const_val {
            ConstValue::Integer(i) => VMValue::Integer(*i),
            ConstValue::Float(f) => VMValue::Float(*f),
            ConstValue::Bool(b) => VMValue::Bool(*b),
            ConstValue::String(s) => VMValue::String(s.clone()),
            ConstValue::Null => VMValue::Void, // Simplified
            ConstValue::Void => VMValue::Void,
        }
    }
}

/// Virtual Machine state
pub struct VM {
    /// Value storage (uses ValueId as direct index into Vec for O(1) access)
    pub(super) values: Vec<Option<VMValue>>,
    /// Current function being executed
    pub(super) current_function: Option<String>,
    /// Frame state (current block, pc, last result)
    pub(super) frame: ExecutionFrame,
    /// Previous basic block (for phi node resolution)
    pub(super) previous_block: Option<BasicBlockId>,
    /// Simple field storage for objects (maps reference -> field -> value)
    pub(super) object_fields: HashMap<ValueId, HashMap<String, VMValue>>,
    /// Class name mapping for objects (for visibility checks)
    pub(super) object_class: HashMap<ValueId, String>,
    /// Marks ValueIds that represent internal (me/this) references within the current function
    pub(super) object_internal: std::collections::HashSet<ValueId>,
    /// Loop executor for handling phi nodes and loop-specific logic
    pub(super) loop_executor: LoopExecutor,
    /// Shared runtime for box creation and declarations
    pub(super) runtime: NyashRuntime,
    /// Scope tracker for calling fini on scope exit
    pub(super) scope_tracker: ScopeTracker,
    /// Active MIR module during execution (for function calls)
    pub(super) module: Option<MirModule>,
    /// Instruction execution counters (by MIR opcode)
    pub(super) instr_counter: std::collections::HashMap<&'static str, usize>,
    /// Execution start time for optional stats
    pub(super) exec_start: Option<Instant>,
    /// Stats: number of BoxCall hits via VTable path
    pub(super) boxcall_hits_vtable: u64,
    /// Stats: number of BoxCall hits via Poly-PIC path
    pub(super) boxcall_hits_poly_pic: u64,
    /// Stats: number of BoxCall hits via Mono-PIC path
    pub(super) boxcall_hits_mono_pic: u64,
    /// Stats: number of BoxCall hits via generic fallback path
    pub(super) boxcall_hits_generic: u64,
    /// Mono-PIC skeleton: global hit counters keyed by (recv_type, method_id/name)
    pub(super) boxcall_pic_hits: std::collections::HashMap<String, u32>,
    /// Mono-PIC: cached direct targets (currently InstanceBox function name)
    pub(super) boxcall_pic_funcname: std::collections::HashMap<String, String>,
    /// Poly-PIC: per call-site up to 4 entries of (label, version, func_name)
    pub(super) boxcall_poly_pic: std::collections::HashMap<String, Vec<(String, u32, String)>>,
    /// VTable-like cache: (type, method_id, arity) → direct target (InstanceBox method)
    pub(super) boxcall_vtable_funcname: std::collections::HashMap<String, String>,
    /// Version map for cache invalidation: label -> version
    pub(super) type_versions: std::collections::HashMap<String, u32>,
    /// Optional JIT manager (Phase 10_a skeleton)
    pub(super) jit_manager: Option<crate::jit::manager::JitManager>,
    // Phase 9.78a: Add unified Box handling components
    // TODO: Re-enable when interpreter refactoring is complete
    // /// Box registry for creating all Box types
    // box_registry: Arc<UnifiedBoxRegistry>,
    // /// Plugin loader for external Box types
    // #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
    // plugin_loader: Option<Arc<PluginLoaderV2>>,
    // Scope tracker for lifecycle management
    // scope_tracker: ScopeTracker,
    // /// Box declarations from the AST
    // box_declarations: Arc<RwLock<HashMap<String, BoxDeclaration>>>,
}

impl VM {
    pub fn runtime_ref(&self) -> &NyashRuntime { &self.runtime }

    
    
    
    // TODO: Re-enable when interpreter refactoring is complete
    /*
    /// Create a new VM instance with Box registry and declarations
    pub fn new_with_registry(
        box_registry: Arc<UnifiedBoxRegistry>, 
        box_declarations: Arc<RwLock<HashMap<String, BoxDeclaration>>>
    ) -> Self {
        // Implementation pending interpreter refactoring
        unimplemented!()
    }
    
    /// Phase 9.78a: Create VM with plugin support
    #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
    pub fn new_with_plugins(
        box_registry: Arc<UnifiedBoxRegistry>,
        plugin_loader: Arc<PluginLoaderV2>,
        box_declarations: Arc<RwLock<HashMap<String, BoxDeclaration>>>,
    ) -> Self {
        // Implementation pending interpreter refactoring
        unimplemented!()
    }
    */
    
    

    
    
    
       

    
    
    

    

    
    
    
    // Call a method on a Box - moved to vm_methods.rs (wrapper now in vm_methods)
    // removed: old inline implementation
        /*
        // ResultBox (NyashResultBox - new)
        if let Some(result_box) = box_value.as_any().downcast_ref::<crate::boxes::result::NyashResultBox>() {
            match method {
                // Rust側の公開APIメソッド名に合わせたバリアントを許容
                "is_ok" | "isOk" => {
                    return Ok(result_box.is_ok());
                }
                "get_value" | "getValue" => {
                    return Ok(result_box.get_value());
                }
                "get_error" | "getError" => {
                    return Ok(result_box.get_error());
                }
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }

        // Legacy box_trait::ResultBox is no longer handled here (migration complete)

        // InstanceBox field access unification: getField/setField
        if let Some(instance) = box_value.as_any().downcast_ref::<InstanceBox>() {
            match method {
                "getField" => {
                    if _args.len() != 1 { return Ok(Box::new(StringBox::new("getField(name) requires 1 arg"))); }
                    let name = _args[0].to_string_box().value;
                    if let Some(shared) = instance.get_field(&name) {
                        return Ok(shared.clone_box());
                    }
                    return Ok(Box::new(VoidBox::new()));
                }
                "setField" => {
                    if _args.len() != 2 { return Ok(Box::new(StringBox::new("setField(name, value) requires 2 args"))); }
                    let name = _args[0].to_string_box().value;
                    let val_arc: crate::box_trait::SharedNyashBox = std::sync::Arc::from(_args[1].clone_or_share());
                    let _ = instance.set_field(&name, val_arc);
                    return Ok(Box::new(VoidBox::new()));
                }
                _ => {}
            }
        }

        // JitStatsBox methods (process-local JIT counters)
        if let Some(_jsb) = box_value.as_any().downcast_ref::<crate::boxes::jit_stats_box::JitStatsBox>() {
            match method {
                "toJson" | "toJSON" => {
                    return Ok(crate::boxes::jit_stats_box::JitStatsBox::new().to_json());
                }
                // Return detailed per-function stats as JSON array
                // Each item: { name, phi_total, phi_b1, ret_bool_hint, hits, compiled, handle }
                "perFunction" | "per_function" => {
                    if let Some(jm) = &self.jit_manager {
                        let v = jm.per_function_stats();
                        let arr: Vec<serde_json::Value> = v.into_iter().map(|(name, phi_t, phi_b1, rb, hits, compiled, handle)| {
                            serde_json::json!({
                                "name": name,
                                "phi_total": phi_t,
                                "phi_b1": phi_b1,
                                "ret_bool_hint": rb,
                                "hits": hits,
                                "compiled": compiled,
                                "handle": handle
                            })
                        }).collect();
                        let s = serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_string());
                        return Ok(Box::new(crate::box_trait::StringBox::new(s)));
                    }
                    return Ok(Box::new(crate::box_trait::StringBox::new("[]")));
                }
                "top5" => {
                    if let Some(jm) = &self.jit_manager {
                        let v = jm.top_hits(5);
                        let arr: Vec<serde_json::Value> = v.into_iter().map(|(name, hits, compiled, handle)| {
                            serde_json::json!({
                                "name": name,
                                "hits": hits,
                                "compiled": compiled,
                                "handle": handle
                            })
                        }).collect();
                        let s = serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_string());
                        return Ok(Box::new(crate::box_trait::StringBox::new(s)));
                    }
                    return Ok(Box::new(crate::box_trait::StringBox::new("[]")));
                }
                "summary" => {
                    let cfg = crate::jit::config::current();
                    let caps = crate::jit::config::probe_capabilities();
                    let abi_mode = if cfg.native_bool_abi && caps.supports_b1_sig { "b1_bool" } else { "i64_bool" };
                    let b1_norm = crate::jit::rt::b1_norm_get();
                    let ret_b1 = crate::jit::rt::ret_bool_hint_get();
                    let mut payload = serde_json::json!({
                        "abi_mode": abi_mode,
                        "abi_b1_enabled": cfg.native_bool_abi,
                        "abi_b1_supported": caps.supports_b1_sig,
                        "b1_norm_count": b1_norm,
                        "ret_bool_hint_count": ret_b1,
                        "top5": []
                    });
                    if let Some(jm) = &self.jit_manager {
                        let v = jm.top_hits(5);
                        let top5: Vec<serde_json::Value> = v.into_iter().map(|(name, hits, compiled, handle)| serde_json::json!({
                            "name": name, "hits": hits, "compiled": compiled, "handle": handle
                        })).collect();
                        if let Some(obj) = payload.as_object_mut() { obj.insert("top5".to_string(), serde_json::Value::Array(top5)); }
                    }
                    let s = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
                    return Ok(Box::new(crate::box_trait::StringBox::new(s)));
                }
                _ => return Ok(Box::new(crate::box_trait::VoidBox::new())),
            }
        }

        // StringBox methods
        if let Some(string_box) = box_value.as_any().downcast_ref::<StringBox>() {
            match method {
                "length" | "len" => {
                    return Ok(Box::new(IntegerBox::new(string_box.value.len() as i64)));
                },
                "toString" => {
                    return Ok(Box::new(StringBox::new(string_box.value.clone())));
                },
                "substring" => {
                    // substring(start, end) - simplified implementation
                    if _args.len() >= 2 {
                        if let (Some(start_box), Some(end_box)) = (_args.get(0), _args.get(1)) {
                            if let (Some(start_int), Some(end_int)) = (
                                start_box.as_any().downcast_ref::<IntegerBox>(),
                                end_box.as_any().downcast_ref::<IntegerBox>()
                            ) {
                                let start = start_int.value.max(0) as usize;
                                let end = end_int.value.max(0) as usize;
                                let len = string_box.value.len();
                                
                                if start <= len {
                                    let end_idx = end.min(len);
                                    if start <= end_idx {
                                        let substr = &string_box.value[start..end_idx];
                                        return Ok(Box::new(StringBox::new(substr)));
                                    }
                                }
                            }
                        }
                    }
                    return Ok(Box::new(StringBox::new(""))); // Return empty string on error
                },
                "concat" => {
                    // concat(other) - concatenate with another string
                    if let Some(other_box) = _args.get(0) {
                        let other_str = other_box.to_string_box().value;
                        let result = string_box.value.clone() + &other_str;
                        return Ok(Box::new(StringBox::new(result)));
                    }
                    return Ok(Box::new(StringBox::new(string_box.value.clone())));
                },
                _ => return Ok(Box::new(VoidBox::new())), // Unsupported method
            }
        }
        
        // ArrayBox methods (minimal set)
        if let Some(array_box) = box_value.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            match method {
                "push" => {
                    if let Some(v) = _args.get(0) { return Ok(array_box.push(v.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: push(value) requires 1 arg")));
                },
                "pop" => { return Ok(array_box.pop()); },
                "length" | "len" => { return Ok(array_box.length()); },
                "get" => {
                    if let Some(i) = _args.get(0) { return Ok(array_box.get(i.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: get(index) requires 1 arg")));
                },
                "set" => {
                    if _args.len() >= 2 { return Ok(array_box.set(_args[0].clone_or_share(), _args[1].clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: set(index, value) requires 2 args")));
                },
                "remove" => {
                    if let Some(i) = _args.get(0) { return Ok(array_box.remove(i.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: remove(index) requires 1 arg")));
                },
                "contains" => {
                    if let Some(v) = _args.get(0) { return Ok(array_box.contains(v.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: contains(value) requires 1 arg")));
                },
                "indexOf" => {
                    if let Some(v) = _args.get(0) { return Ok(array_box.indexOf(v.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: indexOf(value) requires 1 arg")));
                },
                "clear" => { return Ok(array_box.clear()); },
                "join" => {
                    if let Some(sep) = _args.get(0) { return Ok(array_box.join(sep.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: join(sep) requires 1 arg")));
                },
                "sort" => { return Ok(array_box.sort()); },
                "reverse" => { return Ok(array_box.reverse()); },
                "slice" => {
                    if _args.len() >= 2 { return Ok(array_box.slice(_args[0].clone_or_share(), _args[1].clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: slice(start, end) requires 2 args")));
                },
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }

        // MapBox methods (minimal set)
        if let Some(map_box) = box_value.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
            match method {
                "set" => {
                    if _args.len() >= 2 { return Ok(map_box.set(_args[0].clone_or_share(), _args[1].clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: set(key, value) requires 2 args")));
                },
                "get" => {
                    if let Some(k) = _args.get(0) { return Ok(map_box.get(k.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: get(key) requires 1 arg")));
                },
                "has" => {
                    if let Some(k) = _args.get(0) { return Ok(map_box.has(k.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: has(key) requires 1 arg")));
                },
                "delete" | "remove" => {
                    if let Some(k) = _args.get(0) { return Ok(map_box.delete(k.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: delete(key) requires 1 arg")));
                },
                "keys" => { return Ok(map_box.keys()); },
                "values" => { return Ok(map_box.values()); },
                "size" => { return Ok(map_box.size()); },
                "clear" => { return Ok(map_box.clear()); },
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }

        // MathBox methods (minimal set)
        if let Some(math_box) = box_value.as_any().downcast_ref::<crate::boxes::math_box::MathBox>() {
            // Coerce numeric-like StringBox to FloatBox for function-style lowering path
            let mut coerce_num = |b: &Box<dyn NyashBox>| -> Box<dyn NyashBox> {
                if let Some(sb) = b.as_any().downcast_ref::<StringBox>() {
                    let s = sb.value.trim();
                    if let Ok(f) = s.parse::<f64>() { return Box::new(crate::boxes::FloatBox::new(f)); }
                    if let Ok(i) = s.parse::<i64>() { return Box::new(IntegerBox::new(i)); }
                }
                b.clone_or_share()
            };
            match method {
                "min" => {
                    if _args.len() >= 2 {
                        let a = coerce_num(&_args[0]);
                        let b = coerce_num(&_args[1]);
                        return Ok(math_box.min(a, b));
                    }
                    return Ok(Box::new(StringBox::new("Error: min(a,b) requires 2 args")));
                }
                "max" => {
                    if _args.len() >= 2 {
                        let a = coerce_num(&_args[0]);
                        let b = coerce_num(&_args[1]);
                        return Ok(math_box.max(a, b));
                    }
                    return Ok(Box::new(StringBox::new("Error: max(a,b) requires 2 args")));
                }
                "abs" => {
                    if let Some(v) = _args.get(0) { return Ok(math_box.abs(coerce_num(v))); }
                    return Ok(Box::new(StringBox::new("Error: abs(x) requires 1 arg")));
                }
                "sin" => {
                    if let Some(v) = _args.get(0) { return Ok(math_box.sin(coerce_num(v))); }
                    return Ok(Box::new(StringBox::new("Error: sin(x) requires 1 arg")));
                }
                "cos" => {
                    if let Some(v) = _args.get(0) { return Ok(math_box.cos(coerce_num(v))); }
                    return Ok(Box::new(StringBox::new("Error: cos(x) requires 1 arg")));
                }
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }

        // SocketBox methods (minimal set + timeout variants)
        if let Some(sock) = box_value.as_any().downcast_ref::<crate::boxes::socket_box::SocketBox>() {
            match method {
                "bind" => {
                    if _args.len() >= 2 { return Ok(sock.bind(_args[0].clone_or_share(), _args[1].clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: bind(address, port) requires 2 args")));
                },
                "listen" => {
                    if let Some(b) = _args.get(0) { return Ok(sock.listen(b.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: listen(backlog) requires 1 arg")));
                },
                "accept" => { return Ok(sock.accept()); },
                "acceptTimeout" | "accept_timeout" => {
                    if let Some(ms) = _args.get(0) { return Ok(sock.accept_timeout(ms.clone_or_share())); }
                    return Ok(Box::new(crate::box_trait::VoidBox::new()));
                },
                "connect" => {
                    if _args.len() >= 2 { return Ok(sock.connect(_args[0].clone_or_share(), _args[1].clone_or_share())); }
                    return Ok(Box::new(StringBox::new("Error: connect(address, port) requires 2 args")));
                },
                "read" => { return Ok(sock.read()); },
                "recvTimeout" | "recv_timeout" => {
                    if let Some(ms) = _args.get(0) { return Ok(sock.recv_timeout(ms.clone_or_share())); }
                    return Ok(Box::new(StringBox::new("")));
                },
                "write" => {
                    if let Some(d) = _args.get(0) { return Ok(sock.write(d.clone_or_share())); }
                    return Ok(Box::new(crate::box_trait::BoolBox::new(false)));
                },
                "close" => { return Ok(sock.close()); },
                "isServer" | "is_server" => { return Ok(sock.is_server()); },
                "isConnected" | "is_connected" => { return Ok(sock.is_connected()); },
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }
        
        // IntegerBox methods  
        if let Some(integer_box) = box_value.as_any().downcast_ref::<IntegerBox>() {
            match method {
                "toString" => {
                    return Ok(Box::new(StringBox::new(integer_box.value.to_string())));
                },
                "abs" => {
                    return Ok(Box::new(IntegerBox::new(integer_box.value.abs())));
                },
                _ => return Ok(Box::new(VoidBox::new())), // Unsupported method
            }
        }
        
        // BoolBox methods
        if let Some(bool_box) = box_value.as_any().downcast_ref::<BoolBox>() {
            match method {
                "toString" => {
                    return Ok(Box::new(StringBox::new(bool_box.value.to_string())));
                },
                _ => return Ok(Box::new(VoidBox::new())), // Unsupported method
            }
        }
        
        // ArrayBox methods - needed for kilo editor
        if let Some(array_box) = box_value.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            match method {
                "length" | "len" => {
                    let items = array_box.items.read().unwrap();
                    return Ok(Box::new(IntegerBox::new(items.len() as i64)));
                },
                "get" => {
                    // get(index) - get element at index
                    if let Some(index_box) = _args.get(0) {
                        if let Some(index_int) = index_box.as_any().downcast_ref::<IntegerBox>() {
                            let items = array_box.items.read().unwrap();
                            let index = index_int.value as usize;
                            if index < items.len() {
                                return Ok(items[index].clone_box());
                            }
                        }
                    }
                    return Ok(Box::new(VoidBox::new())); // Return void for out of bounds
                },
                "set" => {
                    // set(index, value) - simplified implementation
                    // Note: This is a read-only operation in the VM for now
                    // In a real implementation, we'd need mutable access
                    return Ok(Box::new(VoidBox::new()));
                },
                "push" => {
                    // push(value) - simplified implementation
                    // Note: This is a read-only operation in the VM for now
                    return Ok(Box::new(VoidBox::new()));
                },
                "insert" => {
                    // insert(index, value) - simplified implementation
                    // Note: This is a read-only operation in the VM for now
                    return Ok(Box::new(VoidBox::new()));
                },
                _ => return Ok(Box::new(VoidBox::new())), // Unsupported method
            }
        }
        
        // PluginBoxV2 support
        if let Some(plugin_box) = box_value.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
            // For toString on plugins, return a descriptive string
            if method == "toString" {
                return Ok(Box::new(StringBox::new(format!("{}(id={})", plugin_box.box_type, plugin_box.inner.instance_id))));
            }

            // Name-based fallback: delegate to unified PluginHost to invoke by (box_type, method)
            // This preserves existing semantics but actually performs the plugin call instead of no-op.
            let host = crate::runtime::get_global_plugin_host();
            if let Ok(h) = host.read() {
                // Prepare args as NyashBox list; they are already Box<dyn NyashBox>
                let nyash_args: Vec<Box<dyn NyashBox>> = _args.into_iter().map(|b| b).collect();
                match h.invoke_instance_method(&plugin_box.box_type, method, plugin_box.inner.instance_id, &nyash_args) {
                    Ok(Some(ret)) => return Ok(ret),
                    Ok(None) => return Ok(Box::new(VoidBox::new())),
                    Err(e) => {
                        eprintln!("[VM] Plugin invoke error: {}.{} -> {}", plugin_box.box_type, method, e.message());
                        return Ok(Box::new(VoidBox::new()));
                    }
                }
            }
            return Ok(Box::new(VoidBox::new()));
        }
        */
}

/// RAII guard for GC root regions
// Root region guard removed in favor of explicit enter/leave to avoid borrow conflicts

pub(super) use crate::backend::vm_control_flow::ControlFlow;

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{MirModule, MirFunction, FunctionSignature, MirType, EffectMask, BasicBlock, BinaryOp};
    use crate::parser::NyashParser;
    use crate::runtime::NyashRuntime;
    use crate::core::model::BoxDeclaration as CoreBoxDecl;
    use crate::interpreter::SharedState;
    use crate::box_factory::user_defined::UserDefinedBoxFactory;
    use std::sync::Arc;
    use std::collections::HashMap;
    
    #[test]
    fn test_basic_vm_execution() {
        let mut vm = VM::new();
        
        // Test constant loading
        let const_instr = MirInstruction::Const {
            dst: ValueId(1),
            value: ConstValue::Integer(42),
        };
        
        let result = vm.execute_instruction(&const_instr);
        assert!(result.is_ok());
        
        let value = vm.get_value(ValueId(1)).unwrap();
        assert_eq!(value.as_integer().unwrap(), 42);
    }
    
    #[test]
    fn test_binary_operations() {
        let mut vm = VM::new();
        
        // Load constants
        vm.set_value(ValueId(1), VMValue::Integer(10));
        vm.set_value(ValueId(2), VMValue::Integer(32));
        
        // Test addition
        let add_instr = MirInstruction::BinOp {
            dst: ValueId(3),
            op: BinaryOp::Add,
            lhs: ValueId(1),
            rhs: ValueId(2),
        };
        
        let result = vm.execute_instruction(&add_instr);
        assert!(result.is_ok());
        
        let value = vm.get_value(ValueId(3)).unwrap();
        assert_eq!(value.as_integer().unwrap(), 42);
    }

    fn collect_box_declarations(ast: &crate::ast::ASTNode, runtime: &NyashRuntime) {
        fn walk(node: &crate::ast::ASTNode, runtime: &NyashRuntime) {
            match node {
                crate::ast::ASTNode::Program { statements, .. } => {
                    for st in statements { walk(st, runtime); }
                }
                crate::ast::ASTNode::BoxDeclaration { name, fields, public_fields, private_fields, methods, constructors, init_fields, weak_fields, is_interface, extends, implements, type_parameters, .. } => {
                    let decl = CoreBoxDecl {
                        name: name.clone(),
                        fields: fields.clone(),
                        public_fields: public_fields.clone(),
                        private_fields: private_fields.clone(),
                        methods: methods.clone(),
                        constructors: constructors.clone(),
                        init_fields: init_fields.clone(),
                        weak_fields: weak_fields.clone(),
                        is_interface: *is_interface,
                        extends: extends.clone(),
                        implements: implements.clone(),
                        type_parameters: type_parameters.clone(),
                    };
                    if let Ok(mut map) = runtime.box_declarations.write() {
                        map.insert(name.clone(), decl);
                    }
                }
                _ => {}
            }
        }
        walk(ast, runtime);
    }

    #[test]
    fn test_vm_user_box_birth_and_method() {
        let code = r#"
box Person {
  init { name }
  birth(n) {
    me.name = n
  }
  greet() {
    return "Hello, " + me.name
  }
}

return new Person("Alice").greet()
"#;

        // Parse to AST
        let ast = NyashParser::parse_from_string(code).expect("parse failed");

        // Prepare runtime with user-defined declarations and factory
        let runtime = {
            let rt = NyashRuntime::new();
            collect_box_declarations(&ast, &rt);
            let mut shared = SharedState::new();
            shared.box_declarations = rt.box_declarations.clone();
            let udf = Arc::new(UserDefinedBoxFactory::new(shared));
            if let Ok(mut reg) = rt.box_registry.lock() { reg.register(udf); }
            rt
        };

        // Compile to MIR
        let mut compiler = crate::mir::MirCompiler::new();
        let compile_result = compiler.compile(ast).expect("mir compile failed");

        // Debug: Print MIR
        println!("=== MIR Output ===");
        let mut printer = crate::mir::MirPrinter::verbose();
        println!("{}", printer.print_module(&compile_result.module));
        println!("==================");

        // Execute with VM
        let mut vm = VM::with_runtime(runtime);
        let result = vm.execute_module(&compile_result.module).expect("vm exec failed");
        assert_eq!(result.to_string_box().value, "Hello, Alice");
    }

    #[test]
    fn test_vm_user_box_var_then_method() {
        let code = r#"
box Counter {
  init { x }
  birth(n) { me.x = n }
  inc() { me.x = me.x + 1 }
  get() { return me.x }
}

local c
c = new Counter(10)
c.inc()
c.get()
"#;
        let ast = NyashParser::parse_from_string(code).expect("parse failed");
        let runtime = {
            let rt = NyashRuntime::new();
            collect_box_declarations(&ast, &rt);
            let mut shared = SharedState::new();
            shared.box_declarations = rt.box_declarations.clone();
            let udf = Arc::new(UserDefinedBoxFactory::new(shared));
            if let Ok(mut reg) = rt.box_registry.lock() { reg.register(udf); }
            rt
        };
        let mut compiler = crate::mir::MirCompiler::new();
        let compile_result = compiler.compile(ast).expect("mir compile failed");
        let mut vm = VM::with_runtime(runtime);
        let result = vm.execute_module(&compile_result.module).expect("vm exec failed");
        assert_eq!(result.to_string_box().value, "11");
    }

    #[test]
    fn test_vm_extern_console_log() {
        let code = r#"
console.log("ok")
"#;
        let ast = NyashParser::parse_from_string(code).expect("parse failed");
        let runtime = NyashRuntime::new();
        let mut compiler = crate::mir::MirCompiler::new();
        let compile_result = compiler.compile(ast).expect("mir compile failed");
        let mut vm = VM::with_runtime(runtime);
        let result = vm.execute_module(&compile_result.module).expect("vm exec failed");
        assert_eq!(result.to_string_box().value, "void");
    }

    #[test]
    fn test_vm_user_box_vtable_caching_two_calls() {
        // Defines a simple user box and calls the same method twice; ensures correctness.
        // Builder reserves slots for user methods (from 4), so method_id should be present,
        // enabling vtable cache population on first call and (conceptually) direct call on second.
        let code = r#"
box Greeter {
  init { name }
  birth(n) { me.name = n }
  greet() { return "Hi, " + me.name }
}

local g
g = new Greeter("Bob")
g.greet()
g.greet()
"#;

        // Parse to AST
        let ast = crate::parser::NyashParser::parse_from_string(code).expect("parse failed");

        // Prepare runtime with user-defined declarations and factory
        let runtime = {
            let rt = crate::runtime::NyashRuntime::new();
            // Collect box declarations into runtime so that user-defined factory works
            fn collect_box_decls(ast: &crate::ast::ASTNode, runtime: &crate::runtime::NyashRuntime) {
                use crate::core::model::BoxDeclaration as CoreBoxDecl;
                fn walk(node: &crate::ast::ASTNode, runtime: &crate::runtime::NyashRuntime) {
                    match node {
                        crate::ast::ASTNode::Program { statements, .. } => for st in statements { walk(st, runtime); },
                        crate::ast::ASTNode::BoxDeclaration { name, fields, public_fields, private_fields, methods, constructors, init_fields, weak_fields, is_interface, extends, implements, type_parameters, .. } => {
                            let decl = CoreBoxDecl {
                                name: name.clone(),
                                fields: fields.clone(),
                                public_fields: public_fields.clone(),
                                private_fields: private_fields.clone(),
                                methods: methods.clone(),
                                constructors: constructors.clone(),
                                init_fields: init_fields.clone(),
                                weak_fields: weak_fields.clone(),
                                is_interface: *is_interface,
                                extends: extends.clone(),
                                implements: implements.clone(),
                                type_parameters: type_parameters.clone(),
                            };
                            if let Ok(mut map) = runtime.box_declarations.write() { map.insert(name.clone(), decl); }
                        }
                        _ => {}
                    }
                }
                walk(ast, runtime);
            }
            collect_box_decls(&ast, &rt);
            // Register user-defined factory
            let mut shared = crate::interpreter::SharedState::new();
            shared.box_declarations = rt.box_declarations.clone();
            let udf = std::sync::Arc::new(crate::box_factory::user_defined::UserDefinedBoxFactory::new(shared));
            if let Ok(mut reg) = rt.box_registry.lock() { reg.register(udf); }
            rt
        };

        // Compile and execute
        let mut compiler = crate::mir::MirCompiler::new();
        let compile_result = compiler.compile(ast).expect("mir compile failed");
        let mut vm = VM::with_runtime(runtime);
        let result = vm.execute_module(&compile_result.module).expect("vm exec failed");
        assert_eq!(result.to_string_box().value, "Hi, Bob");
    }

    #[test]
    fn test_vm_fastpath_universal_type_and_equals() {
        // toString/type/equals/cloneのうち、typeとequalsのfast-pathを検証
        // 1) type: (new ArrayBox()).type() == "ArrayBox"
        let code_type = r#"
return (new ArrayBox()).type()
"#;
        let ast = NyashParser::parse_from_string(code_type).expect("parse failed");
        let runtime = NyashRuntime::new();
        let mut compiler = crate::mir::MirCompiler::new();
        let compile_result = compiler.compile(ast).expect("mir compile failed");
        let mut vm = VM::with_runtime(runtime);
        let result = vm.execute_module(&compile_result.module).expect("vm exec failed");
        assert_eq!(result.to_string_box().value, "ArrayBox");

        // 2) equals: (new IntegerBox(5)).equals(5) == true
        let code_eq = r#"
return (new IntegerBox(5)).equals(5)
"#;
        let ast = NyashParser::parse_from_string(code_eq).expect("parse failed");
        let runtime = NyashRuntime::new();
        let mut compiler = crate::mir::MirCompiler::new();
        let compile_result = compiler.compile(ast).expect("mir compile failed");
        let mut vm = VM::with_runtime(runtime);
        let result = vm.execute_module(&compile_result.module).expect("vm exec failed");
        assert_eq!(result.to_string_box().value, "true");
    }
}
