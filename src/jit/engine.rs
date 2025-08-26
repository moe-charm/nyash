//! JIT Engine skeleton
//!
//! Phase 10_a: Provide a placeholder engine interface that later hosts
//! Cranelift contexts and compiled function handles.

use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct JitEngine {
    // In the future: isa, module, context, fn table, etc.
    initialized: bool,
    next_handle: u64,
    /// Stub function table: handle -> callable closure
    fntab: HashMap<u64, Arc<dyn Fn(&[crate::backend::vm::VMValue]) -> crate::backend::vm::VMValue + Send + Sync>>,
    /// Host externs by symbol name (Phase 10_d)
    externs: HashMap<String, Arc<dyn Fn(&[crate::backend::vm::VMValue]) -> crate::backend::vm::VMValue + Send + Sync>>,
    #[cfg(feature = "cranelift-jit")]
    isa: Option<cranelift_codegen::isa::OwnedTargetIsa>,
}

impl JitEngine {
    pub fn new() -> Self {
        let mut this = Self {
            initialized: true,
            next_handle: 1,
            fntab: HashMap::new(),
            externs: HashMap::new(),
            #[cfg(feature = "cranelift-jit")] isa: None,
        };
        #[cfg(feature = "cranelift-jit")]
        { this.isa = None; }
        this.register_default_externs();
        this
    }

    /// Compile a function if supported; returns an opaque handle id
    pub fn compile_function(&mut self, func_name: &str, mir: &crate::mir::MirFunction) -> Option<u64> {
        // Phase 10_b skeleton: walk MIR with LowerCore and report coverage
        let mut lower = crate::jit::lower::core::LowerCore::new();
        #[cfg(feature = "cranelift-jit")]
        let mut builder = crate::jit::lower::builder::CraneliftBuilder::new();
        #[cfg(not(feature = "cranelift-jit"))]
        let mut builder = crate::jit::lower::builder::NoopBuilder::new();
        if let Err(e) = lower.lower_function(mir, &mut builder) {
            eprintln!("[JIT] lower failed for {}: {}", func_name, e);
            return None;
        }
        if std::env::var("NYASH_JIT_DUMP").ok().as_deref() == Some("1") {
            #[cfg(feature = "cranelift-jit")]
            {
                let s = builder.stats;
                eprintln!("[JIT] lower {}: covered={} unsupported={} (consts={}, binops={}, cmps={}, branches={}, rets={})",
                    func_name, lower.covered, lower.unsupported,
                    s.0, s.1, s.2, s.3, s.4);
            }
            #[cfg(not(feature = "cranelift-jit"))]
            {
                eprintln!("[JIT] lower {}: covered={} unsupported={} (consts={}, binops={}, cmps={}, branches={}, rets={})",
                    func_name, lower.covered, lower.unsupported,
                    builder.consts, builder.binops, builder.cmps, builder.branches, builder.rets);
            }
        }
        // Create a handle and register an executable closure
        let h = self.next_handle;
        self.next_handle = self.next_handle.saturating_add(1);
        #[cfg(feature = "cranelift-jit")]
        {
            if let Some(closure) = builder.take_compiled_closure() {
                self.fntab.insert(h, closure);
                return Some(h);
            }
        }
        // Fallback: insert a stub closure
        self.fntab.insert(h, Arc::new(|_args: &[crate::backend::vm::VMValue]| {
            crate::backend::vm::VMValue::Void
        }));
        Some(h)
    }

    /// Execute compiled function by handle (stub). Returns Some(VMValue) if found.
    pub fn execute_handle(&self, handle: u64, args: &[crate::backend::vm::VMValue]) -> Option<crate::backend::vm::VMValue> {
        self.fntab.get(&handle).map(|f| f(args))
    }

    /// Register built-in externs (collections)
    fn register_default_externs(&mut self) {
        use crate::jit::r#extern::collections as c;
        self.register_extern(c::SYM_ARRAY_LEN, Arc::new(|args| c::array_len(args)));
        self.register_extern(c::SYM_ARRAY_GET, Arc::new(|args| c::array_get(args)));
        self.register_extern(c::SYM_ARRAY_SET, Arc::new(|args| c::array_set(args)));
        self.register_extern(c::SYM_ARRAY_PUSH, Arc::new(|args| c::array_push(args)));
        self.register_extern(c::SYM_MAP_GET, Arc::new(|args| c::map_get(args)));
        self.register_extern(c::SYM_MAP_SET, Arc::new(|args| c::map_set(args)));
        self.register_extern(c::SYM_MAP_SIZE, Arc::new(|args| c::map_size(args)));
    }

    pub fn register_extern(&mut self, name: &str, f: Arc<dyn Fn(&[crate::backend::vm::VMValue]) -> crate::backend::vm::VMValue + Send + Sync>) {
        self.externs.insert(name.to_string(), f);
    }

    /// Lookup an extern symbol (to be used by the lowering once call emission is added)
    pub fn lookup_extern(&self, name: &str) -> Option<Arc<dyn Fn(&[crate::backend::vm::VMValue]) -> crate::backend::vm::VMValue + Send + Sync>> {
        self.externs.get(name).cloned()
    }
}
