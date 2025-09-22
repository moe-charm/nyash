//! JIT stub module for Phase 15 compilation compatibility
//! 
//! This is a temporary stub to maintain compilation compatibility while
//! JIT/Cranelift is archived. All functions return safe defaults or no-op.

pub mod config {
    #[derive(Debug, Clone, Default)]
    pub struct JitConfig {
        pub exec: bool,
        pub stats: bool,
        pub stats_json: bool,
        pub dump: bool,
        pub threshold: Option<u32>,
        pub phi_min: bool,
        pub hostcall: bool,
        pub handle_debug: bool,
        pub native_f64: bool,
        pub native_bool: bool,
        pub native_bool_abi: bool,
    }

    impl JitConfig {
        pub fn from_env() -> Self {
            Self::default()
        }
        
        pub fn apply_env(&self) {
            // No-op
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct Capabilities {
        pub supports_b1_sig: bool,
    }

    pub fn probe_capabilities() -> Capabilities {
        Capabilities::default()
    }

    pub fn apply_runtime_caps(config: JitConfig, _caps: Capabilities) -> JitConfig {
        config
    }

    pub fn set_current(_config: JitConfig) {
        // No-op
    }

    pub fn current() -> JitConfig {
        JitConfig::default()
    }
}

pub mod policy {
    pub fn current() -> () {
        // Return empty policy
    }
}

pub mod engine {
    pub struct JitEngine;

    impl JitEngine {
        pub fn new() -> Self {
            Self
        }

        pub fn last_lower_stats(&self) -> (u64, u64, u64) {
            (0, 0, 0) // Return default stats: phi_t, phi_b1, ret_b
        }
    }
}

pub mod events {
    pub fn emit(_category: &str, _name: &str, _h: Option<i64>, _none: Option<()>, _data: serde_json::Value) {
        // No-op - JIT events disabled for Phase 15
    }

    pub fn emit_lower(_data: serde_json::Value, _category: &str, _source: &str) {
        // No-op - JIT events disabled for Phase 15
    }
}

pub mod abi {
    #[derive(Debug, Clone)]
    pub enum JitValue {
        I64(i64),
        F64(f64),
        Bool(bool),
        Handle(i64),
    }

    pub mod adapter {
        use crate::NyashValue;
        use super::JitValue;

        pub fn from_jit_value(_value: &JitValue) -> NyashValue {
            NyashValue::Void
        }
    }
}

pub mod rt {
    use super::abi::JitValue;

    pub fn set_current_jit_args(_args: &[JitValue]) {
        // No-op
    }

    pub fn b1_norm_get() -> u64 {
        0
    }

    pub fn ret_bool_hint_get() -> u64 {
        0
    }

    pub mod handles {
        use std::sync::Arc;
        use crate::box_trait::NyashBox;

        pub fn to_handle(_boxref: Arc<dyn NyashBox>) -> i64 {
            0
        }

        pub fn get(_handle: i64) -> Option<Arc<dyn NyashBox>> {
            None
        }

        pub fn snapshot_arcs() -> Vec<Arc<dyn NyashBox>> {
            Vec::new()
        }
    }
}