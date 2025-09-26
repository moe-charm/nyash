//! JSON provider abstraction layer

use crate::ffi;
use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;
use std::ffi::CString;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
};

// Shared global state
pub static DOCS: Lazy<Mutex<HashMap<u32, DocInst>>> = Lazy::new(|| Mutex::new(HashMap::new()));
pub static NODES: Lazy<Mutex<HashMap<u32, NodeRep>>> = Lazy::new(|| Mutex::new(HashMap::new()));
pub static NEXT_ID: AtomicU32 = AtomicU32::new(1);

// Provider selection
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProviderKind {
    Serde,
    Yyjson,
}

// Node representation for both providers
#[derive(Clone)]
pub enum NodeRep {
    Serde(Arc<Value>),
    Yy { doc_id: u32, ptr: usize },
}

// Document instance
pub struct DocInst {
    pub root: Option<Arc<Value>>, // Serde provider
    pub doc_ptr: Option<usize>,   // Yyjson provider (opaque pointer value)
    pub last_err: Option<String>,
}

impl DocInst {
    pub fn new() -> Self {
        Self {
            root: None,
            doc_ptr: None,
            last_err: None,
        }
    }
}

pub fn provider_kind() -> ProviderKind {
    match std::env::var("NYASH_JSON_PROVIDER").ok().as_deref() {
        Some("yyjson") | Some("YYJSON") => ProviderKind::Yyjson,
        _ => ProviderKind::Serde,
    }
}

pub fn provider_parse(text: &str) -> Result<Value, String> {
    match provider_kind() {
        ProviderKind::Serde => serde_json::from_str::<Value>(text).map_err(|e| e.to_string()),
        ProviderKind::Yyjson => {
            // Skeleton phase: call into C shim to validate linkage, then fallback to serde_json
            unsafe {
                if let Ok(c) = CString::new(text.as_bytes()) {
                    let _ = ffi::nyash_json_shim_parse(c.as_ptr(), text.len());
                }
            }
            serde_json::from_str::<Value>(text).map_err(|e| e.to_string())
        }
    }
}
