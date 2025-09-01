/*!
 * VM Backend - Execute MIR instructions in a virtual machine
 *
 * Purpose: Core VM (execute loop, storage, control-flow, integration glue)
 * Responsibilities: fetch/dispatch instructions, manage values/blocks, stats hooks
 * Key APIs: VM::execute_module, execute_instruction, get_value/set_value
 * Typical Callers: runner (VM backend), instruction handlers (vm_instructions)
 */

use crate::mir::{MirModule, MirFunction, MirInstruction, ConstValue, BinaryOp, CompareOp, UnaryOp, ValueId, BasicBlockId};
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
    values: Vec<Option<VMValue>>,
    /// Current function being executed
    current_function: Option<String>,
    /// Frame state (current block, pc, last result)
    frame: ExecutionFrame,
    /// Previous basic block (for phi node resolution)
    previous_block: Option<BasicBlockId>,
    /// Simple field storage for objects (maps reference -> field -> value)
    pub(super) object_fields: HashMap<ValueId, HashMap<String, VMValue>>,
    /// Class name mapping for objects (for visibility checks)
    pub(super) object_class: HashMap<ValueId, String>,
    /// Marks ValueIds that represent internal (me/this) references within the current function
    pub(super) object_internal: std::collections::HashSet<ValueId>,
    /// Loop executor for handling phi nodes and loop-specific logic
    loop_executor: LoopExecutor,
    /// Shared runtime for box creation and declarations
    pub(super) runtime: NyashRuntime,
    /// Scope tracker for calling fini on scope exit
    scope_tracker: ScopeTracker,
    /// Active MIR module during execution (for function calls)
    module: Option<MirModule>,
    /// Instruction execution counters (by MIR opcode)
    pub(super) instr_counter: std::collections::HashMap<&'static str, usize>,
    /// Execution start time for optional stats
    pub(super) exec_start: Option<Instant>,
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
    /// Enter a GC root region and return a guard that leaves on drop
    pub(super) fn enter_root_region(&mut self) {
        self.scope_tracker.enter_root_region();
    }

    /// Pin a slice of VMValue as roots in the current region
    pub(super) fn pin_roots<'a>(&mut self, values: impl IntoIterator<Item = &'a VMValue>) {
        for v in values {
            self.scope_tracker.pin_root(v);
        }
    }

    /// Leave current GC root region
    pub(super) fn leave_root_region(&mut self) { self.scope_tracker.leave_root_region(); }

    /// Site info for GC logs: (func, bb, pc)
    pub(super) fn gc_site_info(&self) -> (String, i64, i64) {
        let func = self.current_function.as_deref().unwrap_or("<none>").to_string();
        let bb = self.frame.current_block.map(|b| b.0 as i64).unwrap_or(-1);
        let pc = self.frame.pc as i64;
        (func, bb, pc)
    }

    /// Print a simple breakdown of root VMValue kinds and top BoxRef types
    fn gc_print_roots_breakdown(&self) {
        use std::collections::HashMap;
        let roots = self.scope_tracker.roots_snapshot();
        let mut kinds: HashMap<&'static str, u64> = HashMap::new();
        let mut box_types: HashMap<String, u64> = HashMap::new();
        for v in &roots {
            match v {
                VMValue::Integer(_) => *kinds.entry("Integer").or_insert(0) += 1,
                VMValue::Float(_) => *kinds.entry("Float").or_insert(0) += 1,
                VMValue::Bool(_) => *kinds.entry("Bool").or_insert(0) += 1,
                VMValue::String(_) => *kinds.entry("String").or_insert(0) += 1,
                VMValue::Future(_) => *kinds.entry("Future").or_insert(0) += 1,
                VMValue::Void => *kinds.entry("Void").or_insert(0) += 1,
                VMValue::BoxRef(b) => {
                    let tn = b.type_name().to_string();
                    *box_types.entry(tn).or_insert(0) += 1;
                }
            }
        }
        eprintln!("[GC] roots_breakdown: kinds={:?}", kinds);
        let mut top: Vec<(String, u64)> = box_types.into_iter().collect();
        top.sort_by(|a, b| b.1.cmp(&a.1));
        top.truncate(5);
        eprintln!("[GC] roots_boxref_top5: {:?}", top);
    }
    fn gc_print_reachability_depth2(&self) {
        use std::collections::HashMap;
        let roots = self.scope_tracker.roots_snapshot();
        let mut child_types: HashMap<String, u64> = HashMap::new();
        let mut child_count = 0u64;
        for v in &roots {
            if let VMValue::BoxRef(b) = v {
                if let Some(arr) = b.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                    if let Ok(items) = arr.items.read() {
                        for item in items.iter() {
                            let tn = item.type_name().to_string();
                            *child_types.entry(tn).or_insert(0) += 1;
                            child_count += 1;
                        }
                    }
                }
                if let Some(map) = b.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                    let vals = map.values();
                    if let Some(arr2) = vals.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                        if let Ok(items) = arr2.items.read() {
                            for item in items.iter() {
                                let tn = item.type_name().to_string();
                                *child_types.entry(tn).or_insert(0) += 1;
                                child_count += 1;
                            }
                        }
                    }
                }
            }
        }
        let mut top: Vec<(String, u64)> = child_types.into_iter().collect();
        top.sort_by(|a, b| b.1.cmp(&a.1));
        top.truncate(5);
        eprintln!("[GC] depth2_children: total={} top5={:?}", child_count, top);
    }
    fn jit_threshold_from_env() -> u32 {
        std::env::var("NYASH_JIT_THRESHOLD")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|&v| v > 0)
            .unwrap_or(64)
    }
    /// Helper: execute phi via LoopExecutor with previous_block-based selection (Step2 skeleton)
    pub(super) fn loop_execute_phi(&mut self, dst: ValueId, inputs: &[(BasicBlockId, ValueId)]) -> Result<VMValue, VMError> {
        if inputs.is_empty() {
            return Err(VMError::InvalidInstruction("Phi node has no inputs".to_string()));
        }
        let debug_phi = std::env::var("NYASH_VM_DEBUG").ok().as_deref() == Some("1")
            || std::env::var("NYASH_VM_DEBUG_PHI").ok().as_deref() == Some("1");
        if debug_phi { eprintln!("[VM] phi-select (delegated) prev={:?} inputs={:?}", self.previous_block, inputs); }
        // Borrow just the values storage immutably to avoid borrowing entire &self in closure
        let values_ref = &self.values;
        let res = self.loop_executor.execute_phi(dst, inputs, |val_id| {
            let index = val_id.to_usize();
            if index < values_ref.len() {
                if let Some(ref value) = values_ref[index] {
                    Ok(value.clone())
                } else {
                    Err(VMError::InvalidValue(format!("Value {} not set", val_id)))
                }
            } else {
                Err(VMError::InvalidValue(format!("Value {} out of bounds", val_id)))
            }
        });
        if debug_phi {
            match &res {
                Ok(v) => eprintln!("[VM] phi-result -> {:?}", v),
                Err(e) => eprintln!("[VM] phi-error -> {:?}", e),
            }
        }
        res
    }
    /// Create a new VM instance
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            current_function: None,
            frame: ExecutionFrame::new(),
            previous_block: None,
            object_fields: HashMap::new(),
            object_class: HashMap::new(),
            object_internal: std::collections::HashSet::new(),
            loop_executor: LoopExecutor::new(),
            runtime: NyashRuntime::new(),
            scope_tracker: ScopeTracker::new(),
            module: None,
            instr_counter: std::collections::HashMap::new(),
            exec_start: None,
            boxcall_pic_hits: std::collections::HashMap::new(),
            boxcall_pic_funcname: std::collections::HashMap::new(),
            boxcall_poly_pic: std::collections::HashMap::new(),
            boxcall_vtable_funcname: std::collections::HashMap::new(),
            type_versions: std::collections::HashMap::new(),
            jit_manager: Some(crate::jit::manager::JitManager::new(Self::jit_threshold_from_env())),
            // TODO: Re-enable when interpreter refactoring is complete
            // box_registry: Arc::new(UnifiedBoxRegistry::new()),
            // #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
            // plugin_loader: None,
            // scope_tracker: ScopeTracker::new(),
            // box_declarations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a VM with an external runtime (dependency injection)
    pub fn with_runtime(runtime: NyashRuntime) -> Self {
        Self {
            values: Vec::new(),
            current_function: None,
            frame: ExecutionFrame::new(),
            previous_block: None,
            object_fields: HashMap::new(),
            object_class: HashMap::new(),
            object_internal: std::collections::HashSet::new(),
            loop_executor: LoopExecutor::new(),
            runtime,
            scope_tracker: ScopeTracker::new(),
            module: None,
            instr_counter: std::collections::HashMap::new(),
            exec_start: None,
            boxcall_pic_hits: std::collections::HashMap::new(),
            boxcall_pic_funcname: std::collections::HashMap::new(),
            boxcall_poly_pic: std::collections::HashMap::new(),
            boxcall_vtable_funcname: std::collections::HashMap::new(),
            type_versions: std::collections::HashMap::new(),
            jit_manager: Some(crate::jit::manager::JitManager::new(Self::jit_threshold_from_env())),
        }
    }
    
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
    
    /// Execute a MIR module
    pub fn execute_module(&mut self, module: &MirModule) -> Result<Box<dyn NyashBox>, VMError> {
        // Store module for nested calls
        self.module = Some(module.clone());
        // Optional: dump registry for debugging
        if std::env::var("NYASH_REG_DUMP").ok().as_deref() == Some("1") {
            crate::runtime::type_meta::dump_registry();
        }
        // Reset stats
        self.instr_counter.clear();
        self.exec_start = Some(Instant::now());
        // Find main function
        let main_function = module.get_function("main")
            .ok_or_else(|| VMError::InvalidInstruction("No main function found".to_string()))?;
        
        // Execute main function
        let result = self.execute_function(main_function)?;
        
        // Optional: print VM stats
        self.maybe_print_stats();
        // Optional: print concise JIT unified stats
        self.maybe_print_jit_unified_stats();

        // Optional: print cache stats summary
        if std::env::var("NYASH_VM_PIC_STATS").ok().as_deref() == Some("1") {
            self.print_cache_stats_summary();
        }

        // Optional: print JIT detailed summary (top functions)
        if let Some(jm) = &self.jit_manager { jm.print_summary(); }

        // Optional: GC diagnostics if enabled
        if let Ok(val) = std::env::var("NYASH_GC_TRACE") {
            if val == "1" || val == "2" || val == "3" {
                if let Some((sp, rd, wr)) = self.runtime.gc.snapshot_counters() {
                    eprintln!("[GC] counters: safepoints={} read_barriers={} write_barriers={}", sp, rd, wr);
                }
                let roots_total = self.scope_tracker.root_count_total();
                let root_regions = self.scope_tracker.root_regions();
                let field_slots: usize = self.object_fields.values().map(|m| m.len()).sum();
                eprintln!("[GC] mock_mark: roots_total={} regions={} object_field_slots={}", roots_total, root_regions, field_slots);
                if val == "2" || val == "3" { self.gc_print_roots_breakdown(); }
                if val == "3" { self.gc_print_reachability_depth2(); }
            }
        }

        // Convert result to NyashBox
        Ok(result.to_nyash_box())
    }

    fn print_cache_stats_summary(&self) {
        let sites_poly = self.boxcall_poly_pic.len();
        let entries_poly: usize = self.boxcall_poly_pic.values().map(|v| v.len()).sum();
        let avg_entries = if sites_poly > 0 { (entries_poly as f64) / (sites_poly as f64) } else { 0.0 };
        let sites_mono = self.boxcall_pic_funcname.len();
        let hits_total: u64 = self.boxcall_pic_hits.values().map(|v| *v as u64).sum();
        let vt_entries = self.boxcall_vtable_funcname.len();
        eprintln!(
            "[VM] PIC/VT summary: poly_sites={} avg_entries={:.2} mono_sites={} hits_total={} vt_entries={}",
            sites_poly, avg_entries, sites_mono, hits_total, vt_entries
        );
        // Top sites by hits (up to 5)
        let mut hits: Vec<(&String, &u32)> = self.boxcall_pic_hits.iter().collect();
        hits.sort_by(|a, b| b.1.cmp(a.1));
        for (i, (k, v)) in hits.into_iter().take(5).enumerate() {
            eprintln!("  #{} {} hits={}", i+1, k, v);
        }
    }

    /// Call a MIR function by name with VMValue arguments
    pub(super) fn call_function_by_name(&mut self, func_name: &str, args: Vec<VMValue>) -> Result<VMValue, VMError> {
        // Root region: ensure args stay rooted during nested call
        self.enter_root_region();
        self.pin_roots(args.iter());
        let module_ref = self.module.as_ref().ok_or_else(|| VMError::InvalidInstruction("No active module".to_string()))?;
        let function_ref = module_ref.get_function(func_name)
            .ok_or_else(|| VMError::InvalidInstruction(format!("Function '{}' not found", func_name)))?;
        // Clone function to avoid borrowing conflicts during execution
        let function = function_ref.clone();

        // Save current frame
        let saved_values = std::mem::take(&mut self.values);
        let saved_current_function = self.current_function.clone();
        let saved_current_block = self.frame.current_block;
        let saved_previous_block = self.previous_block;
        let saved_pc = self.frame.pc;
        let saved_last_result = self.frame.last_result;

        // Bind parameters
        for (i, param_id) in function.params.iter().enumerate() {
            if let Some(arg) = args.get(i) {
                self.set_value(*param_id, arg.clone());
            }
        }

        // Heuristic: map `me` (first param) to class name parsed from function name (e.g., User.method/N)
        if let Some(first) = function.params.get(0) {
            if let Some((class_part, _rest)) = func_name.split_once('.') {
                // Record class for internal field visibility checks
                self.object_class.insert(*first, class_part.to_string());
                // Mark internal reference
                self.object_internal.insert(*first);
            }
        }

        // Execute the function
        let result = self.execute_function(&function);

        // Restore frame
        self.values = saved_values;
        self.current_function = saved_current_function;
        self.frame.current_block = saved_current_block;
        self.previous_block = saved_previous_block;
        self.frame.pc = saved_pc;
        self.frame.last_result = saved_last_result;
        // Leave GC root region
        self.scope_tracker.leave_root_region();
        result
    }
    
    /// Execute a single function
    fn execute_function(&mut self, function: &MirFunction) -> Result<VMValue, VMError> {
        self.current_function = Some(function.signature.name.clone());
        // Phase 10_a: JIT profiling (function entry)
        if let Some(jm) = &mut self.jit_manager {
            // Allow threshold to react to env updates (e.g., DebugConfigBox.apply at runtime)
            if let Ok(s) = std::env::var("NYASH_JIT_THRESHOLD") {
                if let Ok(t) = s.parse::<u32>() { if t > 0 { jm.set_threshold(t); } }
            }
            jm.record_entry(&function.signature.name);
            // Try compile if hot (no-op for now, returns fake handle)
            let _ = jm.maybe_compile(&function.signature.name, function);
            // Record per-function lower stats captured during last JIT lower (if any)
            // Note: The current engine encapsulates its LowerCore; expose via last_stats on a new instance as needed.
            if jm.is_compiled(&function.signature.name) && std::env::var("NYASH_JIT_STATS").ok().as_deref() == Some("1") {
                if let Some(h) = jm.handle_of(&function.signature.name) {
                    eprintln!("[JIT] dispatch would go to handle={} for {} (stub)", h, function.signature.name);
                }
            }
        }
        
        // Initialize loop executor for this function
        self.loop_executor.initialize();
        
        // Enter a new scope for this function
        self.scope_tracker.push_scope();
        crate::runtime::global_hooks::push_task_scope();
        
        // Phase 10_c: try a JIT dispatch when enabled; fallback to VM on trap/miss
        // Prepare arguments from current frame params before borrowing jit_manager mutably
        let args_vec: Vec<VMValue> = function
            .params
            .iter()
            .filter_map(|pid| self.get_value(*pid).ok())
            .collect();
        if std::env::var("NYASH_JIT_EXEC").ok().as_deref() == Some("1") {
            let jit_only = std::env::var("NYASH_JIT_ONLY").ok().as_deref() == Some("1");
            // Root regionize args for JIT call
            self.enter_root_region();
            self.pin_roots(args_vec.iter());
            if let Some(jm_mut) = self.jit_manager.as_mut() {
                if jm_mut.is_compiled(&function.signature.name) {
                    if let Some(val) = jm_mut.execute_compiled(&function.signature.name, &function.signature.return_type, &args_vec) {
                        // Exit scope before returning
                        self.leave_root_region();
                        self.scope_tracker.pop_scope();
                        crate::runtime::global_hooks::pop_task_scope();
                        return Ok(val);
                    } else if std::env::var("NYASH_JIT_STATS").ok().as_deref() == Some("1") ||
                              std::env::var("NYASH_JIT_TRAP_LOG").ok().as_deref() == Some("1") {
                        eprintln!("[JIT] fallback: VM path taken for {}", function.signature.name);
                        if jit_only {
                            self.leave_root_region();
                            self.scope_tracker.pop_scope();
                            crate::runtime::global_hooks::pop_task_scope();
                            return Err(VMError::InvalidInstruction(format!("JIT-only enabled and JIT trap occurred for {}", function.signature.name)));
                        }
                    }
                } else if jit_only {
                    // Try to compile now and execute; if not possible, error out
                    let _ = jm_mut.maybe_compile(&function.signature.name, function);
                    if jm_mut.is_compiled(&function.signature.name) {
                        if let Some(val) = jm_mut.execute_compiled(&function.signature.name, &function.signature.return_type, &args_vec) {
                            self.leave_root_region();
                            self.scope_tracker.pop_scope();
                            crate::runtime::global_hooks::pop_task_scope();
                            return Ok(val);
                        } else {
                            self.leave_root_region();
                            self.scope_tracker.pop_scope();
                            crate::runtime::global_hooks::pop_task_scope();
                            return Err(VMError::InvalidInstruction(format!("JIT-only enabled and JIT execution failed for {}", function.signature.name)));
                        }
                    } else {
                        self.leave_root_region();
                        self.scope_tracker.pop_scope();
                        crate::runtime::global_hooks::pop_task_scope();
                        return Err(VMError::InvalidInstruction(format!("JIT-only enabled but function not compiled: {}", function.signature.name)));
                    }
                }
            }
            // Leave root region if not compiled or after fallback
            self.leave_root_region();
        } else {
            if let Some(jm_mut) = &mut self.jit_manager {
                let argc = function.params.len();
                let _would = jm_mut.maybe_dispatch(&function.signature.name, argc);
            }
        }

        // Start at entry block
        let mut current_block = function.entry_block;
        
        loop {
            let block = function.get_block(current_block)
                .ok_or_else(|| VMError::InvalidBasicBlock(format!("Block {} not found", current_block)))?;
            
            self.frame.current_block = Some(current_block);
            self.frame.pc = 0;
            
            let mut next_block = None;
            let mut should_return = None;
            
            // Execute instructions in this block (including terminator)
            let all_instructions: Vec<_> = block.all_instructions().collect();
            for (index, instruction) in all_instructions.iter().enumerate() {
                self.frame.pc = index;
                
                match self.execute_instruction(instruction)? {
                    ControlFlow::Continue => continue,
                    ControlFlow::Jump(target) => {
                        next_block = Some(target);
                        break;
                    },
                    ControlFlow::Return(value) => {
                        should_return = Some(value);
                        break;
                    },
                }
            }
            
            // Handle control flow
            if let Some(return_value) = should_return {
                // Exit scope before returning
                self.scope_tracker.pop_scope();
                crate::runtime::global_hooks::pop_task_scope();
                return Ok(return_value);
            } else if let Some(target) = next_block {
                // Update previous block before jumping and record transition via control_flow helper
                control_flow::record_transition(&mut self.previous_block, &mut self.loop_executor, current_block, target).ok();
                current_block = target;
            } else {
                // Block ended without terminator - this shouldn't happen in well-formed MIR
                // but let's handle it gracefully by returning void
                // Exit scope before returning
                self.scope_tracker.pop_scope();
                crate::runtime::global_hooks::pop_task_scope();
                return Ok(VMValue::Void);
            }
        }
    }
    
    /// Execute a single instruction
    fn execute_instruction(&mut self, instruction: &MirInstruction) -> Result<ControlFlow, VMError> {
        // Record instruction for stats
        let debug_global = std::env::var("NYASH_VM_DEBUG").ok().as_deref() == Some("1");
        let debug_exec = debug_global || std::env::var("NYASH_VM_DEBUG_EXEC").ok().as_deref() == Some("1");
        if debug_exec { eprintln!("[VM] execute_instruction: {:?}", instruction); }
        self.record_instruction(instruction);
        super::dispatch::execute_instruction(self, instruction, debug_global)
        
    }
       

    
    /// Get a value from storage
    pub(super) fn get_value(&self, value_id: ValueId) -> Result<VMValue, VMError> {
        let index = value_id.to_usize();
        if index < self.values.len() {
            if let Some(ref value) = self.values[index] {
                Ok(value.clone())
            } else {
                Err(VMError::InvalidValue(format!("Value {} not set", value_id)))
            }
        } else {
            Err(VMError::InvalidValue(format!("Value {} out of bounds", value_id)))
        }
    }
    
    /// Set a value in the VM storage
    pub(super) fn set_value(&mut self, value_id: ValueId, value: VMValue) {
        let index = value_id.to_usize();
        // Resize Vec if necessary
        if index >= self.values.len() {
            self.values.resize(index + 1, None);
        }
        self.values[index] = Some(value);
    }
    

    /// Record an instruction execution for statistics
    pub(super) fn record_instruction(&mut self, instruction: &MirInstruction) {
        let key: &'static str = match instruction {
            MirInstruction::Const { .. } => "Const",
            MirInstruction::BinOp { .. } => "BinOp",
            MirInstruction::UnaryOp { .. } => "UnaryOp",
            MirInstruction::Compare { .. } => "Compare",
            MirInstruction::Load { .. } => "Load",
            MirInstruction::Store { .. } => "Store",
            MirInstruction::Call { .. } => "Call",
            MirInstruction::BoxCall { .. } => "BoxCall",
            MirInstruction::Branch { .. } => "Branch",
            MirInstruction::Jump { .. } => "Jump",
            MirInstruction::Return { .. } => "Return",
            MirInstruction::Phi { .. } => "Phi",
            MirInstruction::NewBox { .. } => "NewBox",
            MirInstruction::TypeCheck { .. } => "TypeCheck",
            MirInstruction::Cast { .. } => "Cast",
            MirInstruction::TypeOp { .. } => "TypeOp",
            MirInstruction::ArrayGet { .. } => "ArrayGet",
            MirInstruction::ArraySet { .. } => "ArraySet",
            MirInstruction::Copy { .. } => "Copy",
            MirInstruction::Debug { .. } => "Debug",
            MirInstruction::Print { .. } => "Print",
            MirInstruction::Nop => "Nop",
            MirInstruction::Throw { .. } => "Throw",
            MirInstruction::Catch { .. } => "Catch",
            MirInstruction::Safepoint => "Safepoint",
            MirInstruction::RefNew { .. } => "RefNew",
            MirInstruction::RefGet { .. } => "RefGet",
            MirInstruction::RefSet { .. } => "RefSet",
            MirInstruction::WeakNew { .. } => "WeakNew",
            MirInstruction::WeakLoad { .. } => "WeakLoad",
            MirInstruction::BarrierRead { .. } => "BarrierRead",
            MirInstruction::BarrierWrite { .. } => "BarrierWrite",
            MirInstruction::WeakRef { .. } => "WeakRef",
            MirInstruction::Barrier { .. } => "Barrier",
            MirInstruction::FutureNew { .. } => "FutureNew",
            MirInstruction::FutureSet { .. } => "FutureSet",
            MirInstruction::Await { .. } => "Await",
            MirInstruction::ExternCall { .. } => "ExternCall",
            MirInstruction::PluginInvoke { .. } => "PluginInvoke",
        };
        *self.instr_counter.entry(key).or_insert(0) += 1;
    }

    
    /// Phase 9.78a: Unified method dispatch for all Box types
    fn call_unified_method(&self, box_value: Box<dyn NyashBox>, method: &str, args: Vec<Box<dyn NyashBox>>) -> Result<Box<dyn NyashBox>, VMError> {
        // For now, we use the simplified method dispatch
        // In a full implementation, this would check for InstanceBox and dispatch appropriately
        self.call_box_method_impl(box_value, method, args)
    }
    
    /// Call a method on a Box - simplified version of interpreter method dispatch
    pub(super) fn call_box_method(&self, box_value: Box<dyn NyashBox>, method: &str, mut _args: Vec<Box<dyn NyashBox>>) -> Result<Box<dyn NyashBox>, VMError> {
        // For now, implement basic methods for common box types
        // This is a simplified version - real implementation would need full method dispatch

        // 🌟 Universal methods pre-dispatch (non-invasive)
        match method {
            "toString" => {
                if !_args.is_empty() {
                    return Ok(Box::new(StringBox::new(format!("Error: toString() expects 0 arguments, got {}", _args.len()))));
                }
                return Ok(Box::new(StringBox::new(box_value.to_string_box().value)));
            }
            "type" => {
                if !_args.is_empty() {
                    return Ok(Box::new(StringBox::new(format!("Error: type() expects 0 arguments, got {}", _args.len()))));
                }
                return Ok(Box::new(StringBox::new(box_value.type_name())));
            }
            "equals" => {
                if _args.len() != 1 {
                    return Ok(Box::new(StringBox::new(format!("Error: equals() expects 1 argument, got {}", _args.len()))));
                }
                let rhs = _args.remove(0);
                let eq = box_value.equals(&*rhs);
                return Ok(Box::new(eq));
            }
            "clone" => {
                if !_args.is_empty() {
                    return Ok(Box::new(StringBox::new(format!("Error: clone() expects 0 arguments, got {}", _args.len()))));
                }
                return Ok(box_value.clone_box());
            }
            _ => {}
        }

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
        
        // Default: return void for any unrecognized box type or method
        Ok(Box::new(VoidBox::new()))
    }
}

/// RAII guard for GC root regions
// Root region guard removed in favor of explicit enter/leave to avoid borrow conflicts

/// Control flow result from instruction execution
pub(super) enum ControlFlow {
    Continue,
    Jump(BasicBlockId),
    Return(VMValue),
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{MirModule, MirFunction, FunctionSignature, MirType, EffectMask, BasicBlock};
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
