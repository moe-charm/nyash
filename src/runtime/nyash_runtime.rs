//! Minimal NyashRuntime skeleton shared by interpreter and VM
//!
//! Focused on dependency inversion: core models + runtime services,
//! while execution strategies live in interpreter/VM layers.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use crate::core::model::BoxDeclaration;
use crate::box_factory::{UnifiedBoxRegistry, BoxFactory};
use crate::box_factory::builtin::BuiltinBoxFactory;
#[cfg(feature = "plugins")]
use crate::box_factory::plugin::PluginBoxFactory;

/// Core runtime container for executing Nyash programs
pub struct NyashRuntime {
    /// Unified registry that can create any Box type
    pub box_registry: Arc<Mutex<UnifiedBoxRegistry>>,
    /// User-defined box declarations collected from source
    pub box_declarations: Arc<RwLock<HashMap<String, BoxDeclaration>>>,
    /// GC hooks (switchable runtime). Default is no-op.
    pub gc: Arc<dyn crate::runtime::gc::GcHooks>,
    /// Optional scheduler (single-thread by default is fine)
    pub scheduler: Option<Arc<dyn crate::runtime::scheduler::Scheduler>>, 
}

impl NyashRuntime {
    /// Create a new runtime with defaults
    pub fn new() -> Self {
        Self {
            box_registry: create_default_registry(),
            box_declarations: Arc::new(RwLock::new(HashMap::new())),
            gc: Arc::new(crate::runtime::gc::NullGc),
            scheduler: Some(Arc::new(crate::runtime::scheduler::SingleThreadScheduler::new())),
        }
    }
}

/// Builder for NyashRuntime allowing DI without globals (future-proof)
pub struct NyashRuntimeBuilder {
    box_registry: Option<Arc<Mutex<UnifiedBoxRegistry>>>,
    box_declarations: Option<Arc<RwLock<HashMap<String, BoxDeclaration>>>>,
    gc: Option<Arc<dyn crate::runtime::gc::GcHooks>>, 
    scheduler: Option<Arc<dyn crate::runtime::scheduler::Scheduler>>,
}

impl NyashRuntimeBuilder {
    pub fn new() -> Self {
        Self { box_registry: None, box_declarations: None, gc: None, scheduler: None }
    }

    /// Inject a BoxFactory implementation directly into a private registry
    pub fn with_factory(mut self, factory: Arc<dyn BoxFactory>) -> Self {
        let registry = self.box_registry.take().unwrap_or_else(|| create_default_registry());
        if let Ok(mut reg) = registry.lock() {
            reg.register(factory);
        }
        self.box_registry = Some(registry);
        self
    }

    pub fn with_box_declarations(
        mut self,
        decls: Arc<RwLock<HashMap<String, BoxDeclaration>>>,
    ) -> Self {
        self.box_declarations = Some(decls);
        self
    }

    pub fn build(self) -> NyashRuntime {
        let registry = self.box_registry.unwrap_or_else(|| create_default_registry());

        NyashRuntime {
            box_registry: registry,
            box_declarations: self.box_declarations.unwrap_or_else(|| Arc::new(RwLock::new(HashMap::new()))),
            gc: self.gc.unwrap_or_else(|| Arc::new(crate::runtime::gc::NullGc)),
            scheduler: Some(self.scheduler.unwrap_or_else(|| Arc::new(crate::runtime::scheduler::SingleThreadScheduler::new()))),
        }
    }
}

fn create_default_registry() -> Arc<Mutex<UnifiedBoxRegistry>> {
    let mut registry = UnifiedBoxRegistry::new();
    // Simple rule:
    // - Default: plugins-only (no builtins)
    // - wasm32: enable builtins
    // - tests: enable builtins
    // - feature "builtin-core": enable builtins manually
    #[cfg(any(test, target_arch = "wasm32", feature = "builtin-core"))]
    {
        registry.register(Arc::new(BuiltinBoxFactory::new()));
    }
    #[cfg(feature = "plugins")]
    {
        registry.register(Arc::new(PluginBoxFactory::new()));
    }
    Arc::new(Mutex::new(registry))
}

impl NyashRuntimeBuilder {
    /// Inject custom GC hooks (switchable runtime). Default is no-op.
    pub fn with_gc_hooks(mut self, gc: Arc<dyn crate::runtime::gc::GcHooks>) -> Self {
        self.gc = Some(gc);
        self
    }

    /// Convenience: use CountingGc for development metrics
    pub fn with_counting_gc(mut self) -> Self {
        let gc = Arc::new(crate::runtime::gc::CountingGc::new());
        self.gc = Some(gc);
        self
    }

    /// Inject a custom scheduler implementation
    pub fn with_scheduler(mut self, sched: Arc<dyn crate::runtime::scheduler::Scheduler>) -> Self {
        self.scheduler = Some(sched);
        self
    }

    /// Convenience: use SingleThreadScheduler
    pub fn with_single_thread_scheduler(mut self) -> Self {
        self.scheduler = Some(Arc::new(crate::runtime::scheduler::SingleThreadScheduler::new()));
        self
    }
}
