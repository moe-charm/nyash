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
}

impl NyashRuntime {
    /// Create a new runtime with defaults
    pub fn new() -> Self {
        Self {
            box_registry: create_default_registry(),
            box_declarations: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// Builder for NyashRuntime allowing DI without globals (future-proof)
pub struct NyashRuntimeBuilder {
    box_registry: Option<Arc<Mutex<UnifiedBoxRegistry>>>,
    box_declarations: Option<Arc<RwLock<HashMap<String, BoxDeclaration>>>>,
}

impl NyashRuntimeBuilder {
    pub fn new() -> Self {
        Self { box_registry: None, box_declarations: None }
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
        NyashRuntime {
            box_registry: self.box_registry.unwrap_or_else(|| create_default_registry()),
            box_declarations: self.box_declarations.unwrap_or_else(|| Arc::new(RwLock::new(HashMap::new()))),
        }
    }
}

fn create_default_registry() -> Arc<Mutex<UnifiedBoxRegistry>> {
    let mut registry = UnifiedBoxRegistry::new();
    registry.register(Arc::new(BuiltinBoxFactory::new()));
    #[cfg(feature = "plugins")]
    {
        registry.register(Arc::new(PluginBoxFactory::new()));
    }
    Arc::new(Mutex::new(registry))
}
