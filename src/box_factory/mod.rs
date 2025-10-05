/*!
 * Unified Box Factory Architecture
 *
 * Phase 9.78: 統合BoxFactoryアーキテクチャ
 * すべてのBox生成（ビルトイン、ユーザー定義、プラグイン）を統一的に扱う
 *
 * Design principles:
 * - "Everything is Box" 哲学の実装レベルでの体現
 * - birth/finiライフサイクルの明確な責務分離
 * - 保守性と拡張性の劇的向上
 */

use crate::box_trait::NyashBox;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Factory Priority Policy for Box creation (Phase 15.5 "Everything is Plugin")
///
/// Determines the order in which different Box factories are consulted
/// during Box creation to solve the StringBox/IntegerBox plugin priority issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactoryPolicy {
    /// Strict Plugin Priority: plugins > user > builtin
    /// ⚡ SOLVES THE CORE PROBLEM: Plugins have highest priority
    /// Use when plugins should completely replace builtins (Phase 15.5)
    StrictPluginFirst,

    /// Compatible Plugin Priority: plugins > builtin > user
    /// 🔧 Compatibility mode: Plugins first, but builtins before user-defined
    /// Use for gradual migration scenarios
    CompatPluginFirst,

    /// Legacy Builtin Priority: builtin > user > plugin (CURRENT DEFAULT)
    /// ⚠️ PROBLEMATIC: Plugins can never override builtins
    /// Only use for compatibility with existing setups
    BuiltinFirst,
}

/// Factory type classification for policy-based ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactoryType {
    /// Built-in factory (StringBox, IntegerBox, etc.)
    Builtin,
    /// User-defined Box factory
    User,
    /// Plugin-provided Box factory
    Plugin,
}

/// Runtime error types for Box operations
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("invalid operation: {message}")]
    InvalidOperation { message: String },
    #[error("type error: {message}")]
    TypeError { message: String },
}

/// Shared state for interpreter context (legacy compatibility)
#[derive(Debug, Default, Clone)]
pub struct SharedState;

impl SharedState {
    pub fn new() -> Self {
        Self
    }
}

/// Unified interface for all Box creation
pub trait BoxFactory: Send + Sync {
    /// Create a new Box instance with given arguments
    fn create_box(
        &self,
        name: &str,
        args: &[Box<dyn NyashBox>],
    ) -> Result<Box<dyn NyashBox>, RuntimeError>;

    /// Check if this factory is currently available
    fn is_available(&self) -> bool {
        true
    }

    /// Get list of Box types this factory can create
    fn box_types(&self) -> Vec<&str>;

    /// Check if this factory supports birth/fini lifecycle
    fn supports_birth(&self) -> bool {
        true
    }

    /// Identify builtin factory to enforce reserved-name protections
    fn is_builtin_factory(&self) -> bool {
        false
    }

    /// Identify factory type for policy-based priority ordering
    fn factory_type(&self) -> FactoryType {
        if self.is_builtin_factory() {
            FactoryType::Builtin
        } else {
            FactoryType::Plugin // Default assumption for external factories
        }
    }
}

/// Registry that manages all BoxFactory implementations
pub struct UnifiedBoxRegistry {
    /// Ordered list of factories with policy-based priority
    pub factories: Vec<Arc<dyn BoxFactory>>,

    /// Quick lookup cache for performance
    type_cache: RwLock<HashMap<String, usize>>, // maps type name to factory index

    /// Factory priority policy (Phase 15.5: Everything is Plugin)
    policy: FactoryPolicy,
}

impl UnifiedBoxRegistry {
    /// Create a new empty registry with default policy
    pub fn new() -> Self {
        Self::with_policy(FactoryPolicy::BuiltinFirst)
    }

    /// Create a new empty registry with specified policy
    pub fn with_policy(policy: FactoryPolicy) -> Self {
        Self {
            factories: Vec::new(),
            type_cache: RwLock::new(HashMap::new()),
            policy,
        }
    }

    /// Create registry with policy from environment variable (Phase 15.5 setup)
    pub fn with_env_policy() -> Self {
        let mut policy = match std::env::var("NYASH_BOX_FACTORY_POLICY").ok().as_deref() {
            Some("compat_plugin_first") => FactoryPolicy::CompatPluginFirst,
            Some("builtin_first") => FactoryPolicy::BuiltinFirst,
            Some("strict_plugin_first") | _ => FactoryPolicy::StrictPluginFirst, // Phase 15.5: Plugin First DEFAULT!
        };

        // When plugins are globally disabled, prefer builtin factory ordering to reduce overhead.
        if std::env::var("NYASH_DISABLE_PLUGINS").ok().as_deref() == Some("1") {
            policy = FactoryPolicy::BuiltinFirst;
        }

        // Quiet in JSON-only/quiet modes to avoid polluting child JSON output
        let is_quiet = std::env::var("NYASH_JSON_ONLY").ok().as_deref() == Some("1")
            || std::env::var("NYASH_QUIET").ok().as_deref() == Some("1")
            || std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("0");
        if !is_quiet {
            eprintln!(
                "[UnifiedBoxRegistry] 🎯 Factory Policy: {:?} (Phase 15.5: Everything is Plugin!)",
                policy
            );
        }
        Self::with_policy(policy)
    }

    /// Get current factory policy
    pub fn get_policy(&self) -> FactoryPolicy {
        self.policy
    }

    /// Set factory policy and rebuild cache to reflect new priorities
    pub fn set_policy(&mut self, policy: FactoryPolicy) {
        if self.policy != policy {
            self.policy = policy;
            self.rebuild_cache();
        }
    }

    /// Rebuild type cache based on current policy
    /// Rebuild type cache based on current policy and environment
    ///
    /// Public because runner may update environment toggles
    /// (e.g., NYASH_USE_PLUGIN_BUILTINS / NYASH_PLUGIN_OVERRIDE_TYPES)
    /// after the registry has been initialized. Rebuilding ensures newly
    /// allowed plugin-backed core boxes (ArrayBox/MapBox/...) are picked up.
    pub fn rebuild_cache(&mut self) {
        // Clear existing cache
        let mut cache = self.type_cache.write().unwrap();
        cache.clear();

        // Get factory priority order based on policy
        let factory_order = self.get_factory_order_by_policy();

        // Re-register types with policy-based priority
        for &factory_index in factory_order.iter() {
            if let Some(factory) = self.factories.get(factory_index) {
                let types = factory.box_types();

                // Reserved core types that must remain builtin-owned
                fn is_reserved_type(name: &str) -> bool {
                    // Phase 15.5: 環境変数でプラグイン優先モード時は保護解除
                    if std::env::var("NYASH_USE_PLUGIN_BUILTINS").is_ok() {
                        if let Ok(types) = std::env::var("NYASH_PLUGIN_OVERRIDE_TYPES") {
                            if types.split(',').any(|t| t.trim() == name) {
                                return false;  // 予約型として扱わない
                            }
                        }
                    }
                    matches!(
                        name,
                        // Core value types
                        "StringBox" | "IntegerBox" | "BoolBox" | "FloatBox" | "NullBox"
                            // Core containers and result
                            | "ArrayBox" | "MapBox" | "ResultBox"
                            // Core method indirection
                            | "MethodBox"
                    )
                }

                for type_name in types {
                    // Enforce reserved names: only builtin factory may claim them
                    if is_reserved_type(type_name) && !factory.is_builtin_factory() {
                        // Quiet noisy logs in JSON-only/quiet modes
                        let is_quiet = std::env::var("NYASH_JSON_ONLY").ok().as_deref() == Some("1")
                            || std::env::var("NYASH_QUIET").ok().as_deref() == Some("1")
                            || std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("0");
                        if !is_quiet {
                            eprintln!(
                                "[UnifiedBoxRegistry] ❌ Rejecting registration of reserved type '{}' by non-builtin factory #{}",
                                type_name, factory_index
                            );
                        }
                        continue;
                    }

                    // Policy-based priority: first in order wins
                    let entry = cache.entry(type_name.to_string());
                    use std::collections::hash_map::Entry;
                    match entry {
                        Entry::Occupied(existing) => {
                            // Collision: type already claimed by higher-priority factory
                            // Quiet noisy logs in JSON-only/quiet modes
                            let is_quiet = std::env::var("NYASH_JSON_ONLY").ok().as_deref() == Some("1")
                                || std::env::var("NYASH_QUIET").ok().as_deref() == Some("1")
                                || std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("0");
                            if !is_quiet {
                                eprintln!(
                                    "[UnifiedBoxRegistry] ⚠️ Policy '{}': type '{}' kept by higher priority factory #{}, ignoring factory #{}",
                                    format!("{:?}", self.policy),
                                    existing.key(),
                                    existing.get(),
                                    factory_index
                                );
                            }
                        }
                        Entry::Vacant(v) => {
                            v.insert(factory_index);
                        }
                    }
                }
            }
        }
    }

    /// Get factory indices ordered by current policy priority
    fn get_factory_order_by_policy(&self) -> Vec<usize> {
        let mut factory_indices: Vec<usize> = (0..self.factories.len()).collect();

        // Sort by factory type according to policy
        factory_indices.sort_by_key(|&index| {
            if let Some(factory) = self.factories.get(index) {
                let factory_type = factory.factory_type();

                match self.policy {
                    FactoryPolicy::StrictPluginFirst => match factory_type {
                        FactoryType::Plugin => 0,   // Highest priority
                        FactoryType::User => 1,     // Medium priority
                        FactoryType::Builtin => 2,  // Lowest priority
                    },
                    FactoryPolicy::CompatPluginFirst => match factory_type {
                        FactoryType::Plugin => 0,   // Highest priority
                        FactoryType::Builtin => 1,  // Medium priority
                        FactoryType::User => 2,     // Lowest priority
                    },
                    FactoryPolicy::BuiltinFirst => match factory_type {
                        FactoryType::Builtin => 0,  // Highest priority (current default)
                        FactoryType::User => 1,     // Medium priority
                        FactoryType::Plugin => 2,   // Lowest priority
                    },
                }
            } else {
                999 // Invalid factory index, put at end
            }
        });

        factory_indices
    }

    /// Register a new factory (policy-aware)
    pub fn register(&mut self, factory: Arc<dyn BoxFactory>) {
        // Simply add to the factory list
        self.factories.push(factory);

        // Rebuild cache to apply policy-based priority ordering
        // This ensures new factory is properly integrated with current policy
        self.rebuild_cache();
    }

    /// Create a Box using the unified interface
    pub fn create_box(
        &self,
        name: &str,
        args: &[Box<dyn NyashBox>],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        // Prefer plugin-builtins when enabled and provider is available in v2 registry
        if std::env::var("NYASH_USE_PLUGIN_BUILTINS").ok().as_deref() == Some("1") {
            use crate::runtime::{get_global_registry, BoxProvider};
            // Allowlist types for override: env NYASH_PLUGIN_OVERRIDE_TYPES="ArrayBox,MapBox" (default: none)
            let allow: Vec<String> = if let Ok(list) = std::env::var("NYASH_PLUGIN_OVERRIDE_TYPES")
            {
                list.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            } else {
                vec![]
            };
            if allow.iter().any(|t| t == name) {
                let v2 = get_global_registry();
                if let Some(provider) = v2.get_provider(name) {
                    if let BoxProvider::Plugin(_lib) = provider {
                        return v2.create_box(name, args).map_err(|e| {
                            RuntimeError::InvalidOperation {
                                message: format!("Plugin Box creation failed: {}", e),
                            }
                        });
                    }
                }
            }
        }
        // Check cache first
        let cache = self.type_cache.read().unwrap();
        if let Some(&factory_index) = cache.get(name) {
            if let Some(factory) = self.factories.get(factory_index) {
                if factory.is_available() {
                    return factory.create_box(name, args);
                }
            }
        }
        drop(cache);

        // Linear search through all factories
        for (fi, factory) in self.factories.iter().enumerate() {
            if !factory.is_available() {
                continue;
            }

            // For factories that advertise types, check if they support this type
            let box_types = factory.box_types();
            if !box_types.is_empty() && !box_types.contains(&name) {
                continue;
            }

            // Try to create the box (factories with empty box_types() will always be tried)
            if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                eprintln!(
                    "[UnifiedBoxRegistry] try factory#{} {:?} for {}",
                    fi,
                    factory.factory_type(),
                    name
                );
            }
            match factory.create_box(name, args) {
                Ok(boxed) => return Ok(boxed),
                Err(_) => continue, // Try next factory
            }
        }
        // Final fallback: if v2 plugin registry has a provider for this name, try it once
        {
            let v2 = crate::runtime::get_global_registry();
            if let Some(_prov) = v2.get_provider(name) {
                if let Ok(b) = v2.create_box(name, args) {
                    return Ok(b);
                }
            }
        }

        {
            let host_arc = crate::runtime::plugin_loader_unified::get_global_plugin_host();
            let maybe = host_arc.read().ok().and_then(|h| h.create_box(name, args).ok());
            if let Some(b) = maybe { return Ok(b); }
        }

        
        if std::env::var("NYASH_PLUGIN_LOOKUP_LOCAL").ok().as_deref() == Some("1") {
            let candidates = ["hako.toml", "nyash.toml"];
            for cfg in candidates.iter() {
                if let Ok(ny) = crate::config::nyash_toml_v2::NyashConfigV2::from_file(cfg) {
                    if let Some((lib, def)) = ny.find_library_for_box(name) {
                        let maybe = {
                            let host_arc = crate::runtime::plugin_loader_unified::get_global_plugin_host();
                            host_arc.read().ok().and_then(|h| {
                                let _ = h.load_library_direct(lib, &def.path, &def.boxes);
                                h.create_box(name, args).ok()
                            })
                        };
                        if let Some(b) = maybe { return Ok(b); }
                    }
                }
            }
        }
Err(RuntimeError::InvalidOperation {
            message: format!("Unknown Box type: {}", name),
        })
    }

    /// Check whether a type name is known to the registry
    pub fn has_type(&self, name: &str) -> bool {
        // Check cache first
        {
            let cache = self.type_cache.read().unwrap();
            if let Some(&idx) = cache.get(name) {
                if let Some(factory) = self.factories.get(idx) {
                    if factory.is_available() {
                        return true;
                    }
                }
            }
        }
        // Fallback: scan factories that can enumerate types
        for factory in &self.factories {
            if !factory.is_available() {
                continue;
            }
            let types = factory.box_types();
            if !types.is_empty() && types.contains(&name) {
                return true;
            }
        }
        false
    }

    /// Get all available Box types
    pub fn available_types(&self) -> Vec<String> {
        let mut types = Vec::new();
        for factory in &self.factories {
            if factory.is_available() {
                for type_name in factory.box_types() {
                    types.push(type_name.to_string());
                }
            }
        }
        types.sort();
        types.dedup();
        types
    }
}

pub mod builtin;
pub mod plugin;
/// Re-export submodules
#[cfg(feature = "interpreter-legacy")]
pub mod user_defined;

// Phase 15.5: Separated builtin implementations for easy deletion
pub mod builtin_impls;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = UnifiedBoxRegistry::new();
        assert_eq!(registry.available_types().len(), 0);
    }
}
