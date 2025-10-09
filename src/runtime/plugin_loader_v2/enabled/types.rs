#![allow(dead_code)]
use super::host_bridge::InvokeFn;
use crate::box_trait::{BoxCore, NyashBox, StringBox};
use std::any::Any;
use std::sync::{Arc, RwLock, Weak};
use once_cell::sync::OnceCell;

fn dbg_on() -> bool {
    std::env::var("NYASH_DEBUG_PLUGIN").unwrap_or_default() == "1"
}

/// Loaded plugin information (library handle + exported addresses)
pub struct LoadedPluginV2 {
    pub(super) _lib: Arc<libloading::Library>,
    pub(super) box_types: Vec<String>,
    pub(super) typeboxes: std::collections::HashMap<String, usize>,
    pub(super) init_fn: Option<unsafe extern "C" fn() -> i32>,
}

#[derive(Clone)]
pub struct PluginBoxMetadata {
    pub lib_name: String,
    pub box_type: String,
    pub type_id: u32,
    pub invoke_fn: InvokeFn,
    pub fini_method_id: Option<u32>,
}

/// v2 Plugin Box handle core
#[derive(Debug)]
pub struct PluginHandleInner {
    pub type_id: u32,
    pub invoke_fn: InvokeFn,
    pub instance_id: u32,
    pub fini_method_id: Option<u32>,
    pub(super) finalized: std::sync::atomic::AtomicBool,
}

impl Drop for PluginHandleInner {
    fn drop(&mut self) {
        if let Some(fini_id) = self.fini_method_id {
            if !self
                .finalized
                .swap(true, std::sync::atomic::Ordering::SeqCst)
            {
                let tlv_args: [u8; 4] = [1, 0, 0, 0];
                let _ = super::host_bridge::invoke_alloc(
                    self.invoke_fn,
                    self.type_id,
                    fini_id,
                    self.instance_id,
                    &tlv_args,
                );
            }
        }
    }
}

impl PluginHandleInner {
    pub fn finalize_now(&self) {
        if let Some(fini_id) = self.fini_method_id {
            if !self
                .finalized
                .swap(true, std::sync::atomic::Ordering::SeqCst)
            {
                if crate::config::env::release_trace() {
                    eprintln!(r#"{{"kind":"release","class":"PluginBox","id":{}}}"#, self.instance_id);
                }
                crate::runtime::leak_tracker::finalize_plugin("PluginBox", self.instance_id);
                let tlv_args: [u8; 4] = [1, 0, 0, 0];
                let _ = super::host_bridge::invoke_alloc(
                    self.invoke_fn,
                    self.type_id,
                    fini_id,
                    self.instance_id,
                    &tlv_args,
                );
            }
        }
    }
}

/// Hako ABI TypeBox FFI (a.k.a. Nyash TypeBox FFI; minimal PoC)
use std::os::raw::c_char;
#[repr(C)]
pub struct NyashTypeBoxFfi {
    pub abi_tag: u32,
    pub version: u16,
    pub struct_size: u16,
    pub name: *const c_char,
    pub resolve: Option<extern "C" fn(*const c_char) -> u32>,
    pub invoke_id: Option<extern "C" fn(u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32>,
    pub capabilities: u64,
}

// ---- Final ABI (Phase A minimal structs) ----
// These types provide a minimal FFI surface to probe newer plugins that
// expose NyValue/NyResult based call interfaces. Phase A uses them only
// for optional probing/logging; behavior remains unchanged.

#[repr(C)]
#[derive(Copy, Clone)]
pub struct NyValueFfi {
    pub tag: u32,        // kind discriminator (implementation dependent)
    pub reserved: u32,   // alignment/padding
    pub data0: u64,      // integer/flags or size
    pub data1: u64,      // extra integer payload
    pub ptr: *const u8,  // bytes/string pointer (when applicable)
    pub len: usize,      // length for ptr
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct NyResultFfi {
    pub status: i32,     // 0=ok, <0=error
    pub tag: u32,        // result kind (when ok)
    pub ptr: *const u8,  // optional payload pointer
    pub len: usize,      // payload length
}

#[repr(C)]
pub struct NyashTypeBoxFinalFfi {
    pub abi_tag: u32,    // e.g., 'TFIN'
    pub version: u16,    // minor version for compatibility
    pub struct_size: u16,
    pub invoke_final: Option<extern "C" fn(
        u32, /* type_id */
        u32, /* method_id */
        u32, /* instance_id */
        *const NyValueFfi,
        usize,
        *mut NyResultFfi,
    ) -> i32>,
    // Optional meta (future); Phase A ignores when absent
    pub get_method_meta: Option<extern "C" fn()>,
    pub get_all_methods: Option<extern "C" fn()>,
    pub get_type_info: Option<extern "C" fn()>,
}

#[derive(Debug, Clone)]
pub struct PluginBoxV2 {
    pub box_type: String,
    pub inner: Arc<PluginHandleInner>,
}

impl BoxCore for PluginBoxV2 {
    fn box_id(&self) -> u64 {
        self.inner.instance_id as u64
    }
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        None
    }
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}({})", self.box_type, self.inner.instance_id)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for PluginBoxV2 {
    fn is_identity(&self) -> bool {
        true
    }
    fn type_name(&self) -> &'static str {
        match self.box_type.as_str() {
            "FileBox" => "FileBox",
            _ => "PluginBoxV2",
        }
    }
    fn clone_box(&self) -> Box<dyn NyashBox> {
        if dbg_on() {
            eprintln!(
                "[PluginBoxV2] clone_box {}({})",
                self.box_type, self.inner.instance_id
            );
        }
        let tlv_args = [1u8, 0, 0, 0];
        let (result, out_len, out_buf) = super::host_bridge::invoke_alloc(
            self.inner.invoke_fn,
            self.inner.type_id,
            0,
            0,
            &tlv_args,
        );
        if result == 0 && out_len >= 4 {
            let new_instance_id =
                u32::from_le_bytes([out_buf[0], out_buf[1], out_buf[2], out_buf[3]]);
            Box::new(PluginBoxV2 {
                box_type: self.box_type.clone(),
                inner: Arc::new(PluginHandleInner {
                    type_id: self.inner.type_id,
                    invoke_fn: self.inner.invoke_fn,
                    instance_id: new_instance_id,
                    fini_method_id: self.inner.fini_method_id,
                    finalized: std::sync::atomic::AtomicBool::new(false),
                }),
            })
        } else {
            Box::new(StringBox::new(format!(
                "Clone failed for {}",
                self.box_type
            )))
        }
    }
    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("{}({})", self.box_type, self.inner.instance_id))
    }
    fn equals(&self, _other: &dyn NyashBox) -> crate::box_trait::BoolBox {
        crate::box_trait::BoolBox::new(false)
    }
    fn share_box(&self) -> Box<dyn NyashBox> {
        Box::new(PluginBoxV2 {
            box_type: self.box_type.clone(),
            inner: self.inner.clone(),
        })
    }
}

impl PluginBoxV2 {
    pub fn instance_id(&self) -> u32 {
        self.inner.instance_id
    }
    pub fn finalize_now(&self) {
        self.inner.finalize_now()
    }
    pub fn is_finalized(&self) -> bool {
        self.inner
            .finalized
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}

// Global cache: (type_id, instance_id) → Weak<PluginHandleInner>
static HANDLE_CACHE: OnceCell<RwLock<std::collections::HashMap<(u32, u32), Weak<PluginHandleInner>>>> = OnceCell::new();

fn cache() -> &'static RwLock<std::collections::HashMap<(u32, u32), Weak<PluginHandleInner>>> {
    HANDLE_CACHE.get_or_init(|| RwLock::new(std::collections::HashMap::new()))
}

pub fn get_or_create_handle(
    type_id: u32,
    instance_id: u32,
    invoke_fn: super::host_bridge::InvokeFn,
    fini_method_id: Option<u32>,
) -> Arc<PluginHandleInner> {
    if let Ok(map) = cache().read() {
        if let Some(w) = map.get(&(type_id, instance_id)) {
            if let Some(a) = w.upgrade() {
                return a;
            }
        }
    }
    let arc = Arc::new(PluginHandleInner {
        type_id,
        invoke_fn,
        instance_id,
        fini_method_id,
        finalized: std::sync::atomic::AtomicBool::new(false),
    });
    if let Ok(mut map) = cache().write() {
        map.insert((type_id, instance_id), Arc::downgrade(&arc));
    }
    arc
}

/// Lookup an existing handle by instance_id (any type). Dev helper to avoid
/// relying on global type_id→invoke mapping when we already have a live handle.
pub fn find_handle_by_instance(instance_id: u32) -> Option<Arc<PluginHandleInner>> {
    if let Ok(map) = cache().read() {
        for ((_tid, iid), weak) in map.iter() {
            if *iid == instance_id {
                if let Some(a) = weak.upgrade() { return Some(a); }
            }
        }
    }
    None
}

/// Helper to construct a PluginBoxV2 from raw ids and invoke pointer safely
pub fn make_plugin_box_v2(
    box_type: String,
    type_id: u32,
    instance_id: u32,
    invoke_fn: InvokeFn,
) -> PluginBoxV2 {
    let inner = get_or_create_handle(type_id, instance_id, invoke_fn, None);
    PluginBoxV2 { box_type, inner }
}

/// Public helper to construct a PluginBoxV2 from raw parts (for VM/JIT integration)
pub fn construct_plugin_box(
    box_type: String,
    type_id: u32,
    invoke_fn: InvokeFn,
    instance_id: u32,
    fini_method_id: Option<u32>,
) -> PluginBoxV2 {
    let inner = get_or_create_handle(type_id, instance_id, invoke_fn, fini_method_id);
    PluginBoxV2 { box_type, inner }
}
