//! Nyash MapBox Plugin — TypeBox v2 (minimal)
//! Methods: birth(0), size(1), get(2), has(3), set(4), fini(u32::MAX)
//! Extension: support both i64 and UTF-8 string keys; values remain i64.
//!
//! ## Phase 2-1: Instance Manager Macros Applied

mod tlv_codec;

use std::ffi::CStr;
use std::os::raw::c_char;

// Import shared TLV codec from hako_abi_impl
use hako_abi_impl::tlv::{
    read_arg_bool, read_arg_handle, read_arg_host_handle, read_arg_i64, read_arg_string,
    write_tlv_bool, write_tlv_handle, write_tlv_host_handle, write_tlv_i64, write_tlv_string,
};

// Import instance manager macros (note: macros generate helper functions locally)
use hako_abi_impl::{define_instance_storage, with_instance, with_instance_mut};

// Import plugin-specific helpers from local tlv_codec
use tlv_codec::{
    build_tlv_i64_handle, build_tlv_i64_host_handle, build_tlv_i64_i64, build_tlv_i64_string,
    preflight, v_to_string, write_mapval_tlv,
};

extern "C" {
    fn nyrt_host_call_name(
        handle: u64,
        method_ptr: *const u8,
        method_len: usize,
        args_ptr: *const u8,
        args_len: usize,
        out_ptr: *mut u8,
        out_len: *mut usize,
    ) -> i32;
}

// Error codes
const NYB_SUCCESS: i32 = 0;
const NYB_E_SHORT_BUFFER: i32 = -1;
const NYB_E_INVALID_TYPE: i32 = -2;
const NYB_E_INVALID_METHOD: i32 = -3;
const NYB_E_INVALID_ARGS: i32 = -4;
const NYB_E_PLUGIN_ERROR: i32 = -5;
const NYB_E_INVALID_HANDLE: i32 = -8;

// Methods
const METHOD_BIRTH: u32 = 0;
const METHOD_SIZE: u32 = 1;
const METHOD_GET: u32 = 2; // args: i64 key -> TLV i64
const METHOD_HAS: u32 = 3; // args: i64 key -> TLV bool
const METHOD_SET: u32 = 4; // args: key(int|string), value(i64) -> TLV i64 (size)
const METHOD_REMOVE: u32 = 6; // args: key(int|string) -> TLV bool (removed)
const METHOD_CLEAR: u32 = 7; // args: () -> TLV i64 (size after clear=0)
const METHOD_KEYS_S: u32 = 8; // args: () -> TLV string (newline-joined keys)
const METHOD_GET_OR: u32 = 9; // args: key(int|string), default(i64) -> TLV i64
const METHOD_FINI: u32 = u32::MAX;
// Extended string-key methods
const METHOD_SET_STR: u32 = 10; // setS(name: string, val: i64) -> i64(size)
const METHOD_GET_STR: u32 = 11; // getS(name: string) -> i64
const METHOD_HAS_STR: u32 = 12; // hasS(name: string) -> bool
const METHOD_VALUES_S: u32 = 13; // valuesStr() -> string (newline-joined)
const METHOD_TO_JSON: u32 = 14; // toJson() -> string
                                // Stage-2: real arrays via HostHandle (opt-in)
const METHOD_KEYS: u32 = 15; // keys() -> HostHandle(ArrayBox)
const METHOD_VALUES: u32 = 16; // values() -> HostHandle(ArrayBox)

// Type id (nyash.toml に合わせる)
const TYPE_ID_MAP: u32 = 11;
const TYPE_ID_ARRAY: u32 = 12; // must match Array plugin spec

#[derive(Clone)]
enum MapVal {
    I64(i64),
    Str(String),
    Handle(u32, u32),
    Host(u64),
}

struct MapInstance {
    data_i64: std::collections::HashMap<i64, MapVal>,
    data_str: std::collections::HashMap<String, MapVal>,
}

define_instance_storage!(MapInstance);

// ---- Nyash TypeBox (FFI minimal PoC) ----
#[repr(C)]
pub struct NyashTypeBoxFfi {
    pub abi_tag: u32,        // 'TYBX'
    pub version: u16,        // 1
    pub struct_size: u16,    // sizeof(NyashTypeBoxFfi)
    pub name: *const c_char, // C string
    pub resolve: Option<extern "C" fn(*const c_char) -> u32>,
    pub invoke_id: Option<extern "C" fn(u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32>,
    pub capabilities: u64,
}
unsafe impl Sync for NyashTypeBoxFfi {}

extern "C" fn mapbox_resolve(name: *const c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match s.as_ref() {
        "birth" => METHOD_BIRTH,
        "size" | "len" => METHOD_SIZE,
        "get" => METHOD_GET,
        "has" => METHOD_HAS,
        "set" => METHOD_SET,
        "fini" => METHOD_FINI,
        "getS" => METHOD_GET_STR,
        "hasS" => METHOD_HAS_STR,
        // Stage-1 host shim: expose string-returning variants for keys/values
        // Host will convert keysS/valuesS newline string to ArrayBox for keys()/values()
        "keysS" => METHOD_KEYS_S,
        "valuesS" => METHOD_VALUES_S,
        // Stage-2 opt-in: real arrays via HostHandle
        "keys" => METHOD_KEYS,
        "values" => METHOD_VALUES,
        _ => 0,
    }
}

#[cfg(feature = "host-handle")]
fn map_keys_values_stage2(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe {
        extern "C" {
            fn nyash_array_new_h() -> i64;
            fn nyash_host_from_plugin_handle(type_id: u32, instance_id: u32) -> u64;
        }
        extern "C" {
            fn nyrt_host_call_slot(
                handle: u64,
                selector_id: u64,
                args_ptr: *const u8,
                args_len: usize,
                out_ptr: *mut u8,
                out_len: *mut usize,
            ) -> i32;
        }
        let arr_h = nyash_array_new_h();
        if arr_h <= 0 {
            return NYB_E_PLUGIN_ERROR;
        }
        // Lock scope separation: Extract data BEFORE calling host functions
        // (avoids deadlock when nyrt_host_call_slot -> ArrayBox -> array-plugin needs INSTANCES lock)
        let data_to_insert: Vec<(i64, MapVal)> = match with_instance!(instance_id, |inst: &MapInstance| {
            if method_id == METHOD_KEYS {
                // Extract keys as strings (sorted)
                let keys = hako_core_map::keys_sorted_from_maps_i64_str(
                    &inst.data_i64,
                    &inst.data_str,
                );
                keys.into_iter()
                    .enumerate()
                    .map(|(idx, k)| (idx as i64, MapVal::Str(k)))
                    .collect()
            } else {
                // Extract values (aligned to sorted keys)
                let keys = hako_core_map::keys_sorted_from_maps_i64_str(
                    &inst.data_i64,
                    &inst.data_str,
                );
                let aligned = hako_core_map::values_for_keys(&keys, &inst.data_i64, &inst.data_str);
                aligned
                    .into_iter()
                    .enumerate()
                    .map(|(idx, v)| {
                        let val = match v {
                            Some(mv) => mv.clone(),
                            None => MapVal::Str(String::new()),
                        };
                        (idx as i64, val)
                    })
                    .collect()
            }
        }) {
            Ok(data) => data,
            Err(_) => return NYB_E_INVALID_HANDLE,
        };  // ← Lock released here!

        // Now populate ArrayBox WITHOUT holding INSTANCES lock
        for (i, val) in data_to_insert.into_iter() {
            if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                eprintln!("[map-plugin] stage2 insert idx={} variant={}", i, tlv_codec::v_to_string(&val));
            }
            let tlv = match val {
                MapVal::I64(n) => build_tlv_i64_i64(i, n),
                MapVal::Str(s) => build_tlv_i64_string(i, &s),
                MapVal::Handle(t, inst_id) => {
                    if t == TYPE_ID_ARRAY {
                        let h = nyash_host_from_plugin_handle(t, inst_id);
                        build_tlv_i64_host_handle(i, h)
                    } else {
                        build_tlv_i64_handle(i, t, inst_id)
                    }
                },
                MapVal::Host(h) => build_tlv_i64_host_handle(i, h),
            };
            let mut cap: usize = 64;
            let mut out: Vec<u8> = vec![0u8; cap];
            let rc = loop {
                let mut out_len: usize = out.len();
                let rc = nyrt_host_call_slot(
                    arr_h as u64,
                    101u64,
                    tlv.as_ptr(),
                    tlv.len(),
                    out.as_mut_ptr(),
                    &mut out_len,
                );
                if rc == 0 {
                    break rc;
                }
                if rc == -3 {
                    if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                        eprintln!("[map-plugin] SHORT_BUFFER on Array.set: cap={} need?>={} rc={}", cap, out_len, rc);
                    }
                    cap = cap.saturating_mul(2);
                    if cap > 64 * 1024 {
                        return NYB_E_PLUGIN_ERROR;
                    }
                    out.resize(cap, 0);
                    continue;
                }
                if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                    eprintln!(
                        "[map-plugin] host_call_slot(Array.set) failed rc={} (len={})",
                        rc, out_len
                    );
                }
                return NYB_E_PLUGIN_ERROR;
            };
            if rc != 0 {
                return NYB_E_PLUGIN_ERROR;
            }
        }
        write_tlv_host_handle(arr_h as u64, result, result_len)
    }
}

extern "C" fn mapbox_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    match method_id {
        // Stage-2: keys()/values() return HostHandle(ArrayBox) when enabled by env
        METHOD_KEYS | METHOD_VALUES => {
            #[cfg(feature = "host-handle")]
            {
                let enable = std::env::var("NYASH_PLUGIN_MAP_ARRAY_HANDLE")
                    .ok()
                    .as_deref()
                    == Some("1");
                if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                    eprintln!("[map-plugin] METHOD_KEYS/VALUES stage2 enable={}", enable);
                }
                if !enable {
                    return NYB_E_INVALID_METHOD;
                }
                return map_keys_values_stage2(
                    instance_id,
                    method_id,
                    args,
                    args_len,
                    result,
                    result_len,
                );
            }
            #[cfg(not(feature = "host-handle"))]
            {
                if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                    eprintln!("[map-plugin] METHOD_KEYS/VALUES host-handle disabled");
                }
                return NYB_E_INVALID_METHOD;
            }
        }
        METHOD_BIRTH => {
            // Create new MapBox instance. Return format is gated for migration:
            // - Default (compat): raw 4-byte instance_id
            // - Opt-in: TLV handle(tag=8) when NYASH_PLUGIN_BIRTH_TLV=1
            unsafe {
                if result_len.is_null() {
                    return NYB_E_INVALID_ARGS;
                }
            }
            let id = allocate_instance_id();
            let inst = MapInstance {
                data_i64: std::collections::HashMap::new(),
                data_str: std::collections::HashMap::new(),
            };
            if store_instance(id, inst).is_err() {
                if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                    eprintln!("[map-plugin] METHOD_BIRTH: mutex lock failed");
                }
                return NYB_E_PLUGIN_ERROR;
            }
            let use_tlv = std::env::var("NYASH_PLUGIN_BIRTH_TLV").ok().as_deref() == Some("1");
            let rc = if use_tlv {
                write_tlv_handle(TYPE_ID_MAP, id, result, result_len)
            } else {
                // Legacy raw 4-byte instance_id
                unsafe {
                    if preflight(result, result_len, 4) {
                        return NYB_E_SHORT_BUFFER;
                    }
                    std::ptr::copy_nonoverlapping(id.to_le_bytes().as_ptr(), result, 4);
                    *result_len = 4;
                }
                NYB_SUCCESS
            };
            if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                let need = 4 + 4 + 8;
                let out_cap = unsafe { *result_len };
                eprintln!(
                    "[map-plugin] METHOD_BIRTH: id={} rc={} out_cap={} need={} tag=8",
                    id, rc, out_cap, need
                );
            }
            rc
        }
        METHOD_SIZE => {
            match with_instance!(instance_id, |inst: &MapInstance| {
                let sz = inst.data_i64.len() + inst.data_str.len();
                write_tlv_i64(sz as i64, result, result_len)
            }) {
                Ok(rc) => rc,
                Err(_) => NYB_E_INVALID_HANDLE,
            }
        }
        METHOD_FINI => {
            match remove_instance(instance_id) {
                Some(_) => NYB_SUCCESS,
                None => NYB_E_PLUGIN_ERROR,
            }
        }
        METHOD_GET => {
            if let Some(ik) = read_arg_i64(args, args_len, 0) {
                match with_instance!(instance_id, |inst: &MapInstance| {
                    match inst.data_i64.get(&ik) {
                        Some(MapVal::I64(v)) => write_tlv_i64(*v, result, result_len),
                        Some(MapVal::Str(s)) => write_tlv_string(s, result, result_len),
                        Some(MapVal::Handle(t, i)) => {
                            if *t == TYPE_ID_ARRAY {
                                unsafe {
                                    extern "C" { fn nyash_host_from_plugin_handle(type_id: u32, instance_id: u32) -> u64; }
                                    let h = nyash_host_from_plugin_handle(*t, *i);
                                    write_tlv_host_handle(h as u64, result, result_len)
                                }
                            } else {
                                write_tlv_handle(*t, *i, result, result_len)
                            }
                        }
                        Some(MapVal::Host(h)) => {
                            write_tlv_host_handle(*h as u64, result, result_len)
                        }
                        None => NYB_E_INVALID_ARGS,
                    }
                }) {
                    Ok(rc) => rc,
                    Err(_) => NYB_E_INVALID_HANDLE,
                }
            } else if let Some(sk) = read_arg_string(args, args_len, 0) {
                match with_instance!(instance_id, |inst: &MapInstance| {
                    match inst.data_str.get(&sk) {
                        Some(MapVal::I64(v)) => write_tlv_i64(*v, result, result_len),
                        Some(MapVal::Str(s)) => write_tlv_string(s, result, result_len),
                        Some(MapVal::Handle(t, i)) => {
                            if *t == TYPE_ID_ARRAY {
                                unsafe {
                                    extern "C" { fn nyash_host_from_plugin_handle(type_id: u32, instance_id: u32) -> u64; }
                                    let h = nyash_host_from_plugin_handle(*t, *i);
                                    write_tlv_host_handle(h as u64, result, result_len)
                                }
                            } else {
                                write_tlv_handle(*t, *i, result, result_len)
                            }
                        }
                        Some(MapVal::Host(h)) => {
                            write_tlv_host_handle(*h as u64, result, result_len)
                        }
                        None => NYB_E_INVALID_ARGS,
                    }
                }) {
                    Ok(rc) => rc,
                    Err(_) => NYB_E_INVALID_HANDLE,
                }
            } else {
                NYB_E_INVALID_ARGS
            }
        }
        METHOD_HAS => {
            if let Some(ik) = read_arg_i64(args, args_len, 0) {
                match with_instance!(instance_id, |inst: &MapInstance| {
                    write_tlv_bool(inst.data_i64.contains_key(&ik), result, result_len)
                }) {
                    Ok(rc) => rc,
                    Err(_) => NYB_E_INVALID_HANDLE,
                }
            } else if let Some(sk) = read_arg_string(args, args_len, 0) {
                match with_instance!(instance_id, |inst: &MapInstance| {
                    write_tlv_bool(inst.data_str.contains_key(&sk), result, result_len)
                }) {
                    Ok(rc) => rc,
                    Err(_) => NYB_E_INVALID_HANDLE,
                }
            } else {
                NYB_E_INVALID_ARGS
            }
        }
        METHOD_SET => {
            let debug = true;
            // key: i64 or string; value: i64/String/Handle/HostHandle
            let val = if let Some(v) = read_arg_i64(args, args_len, 1) {
                MapVal::I64(v)
            } else if let Some((t, i)) = read_arg_handle(args, args_len, 1) {
                MapVal::Handle(t, i)
            } else if let Some(h) = read_arg_host_handle(args, args_len, 1) {
                MapVal::Host(h)
            } else if let Some(s) = read_arg_string(args, args_len, 1) {
                MapVal::Str(s)
            } else {
                return NYB_E_INVALID_ARGS;
            };
            if debug {
                eprintln!("[map-plugin] set value variant={}", tlv_codec::v_to_string(&val));
            }
            if let Some(ik) = read_arg_i64(args, args_len, 0) {
                match with_instance_mut!(instance_id, |inst: &mut MapInstance| {
                    if debug {
                        eprintln!("[map-plugin] set int key={}", ik);
                    }
                    inst.data_i64.insert(ik, val);
                    unsafe {
                        if !result_len.is_null() {
                            *result_len = 0;
                        }
                    }
                    NYB_SUCCESS
                }) {
                    Ok(rc) => return rc,
                    Err(_) => return NYB_E_INVALID_HANDLE,
                }
            } else if let Some(sk) = read_arg_string(args, args_len, 0) {
                match with_instance_mut!(instance_id, |inst: &mut MapInstance| {
                    if debug {
                        eprintln!("[map-plugin] set str key={}", sk);
                    }
                    inst.data_str.insert(sk, val);
                    unsafe {
                        if !result_len.is_null() {
                            *result_len = 0;
                        }
                    }
                    NYB_SUCCESS
                }) {
                    Ok(rc) => return rc,
                    Err(_) => return NYB_E_INVALID_HANDLE,
                }
            } else {
                NYB_E_INVALID_ARGS
            }
        }
        METHOD_REMOVE => {
            let debug = std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1");
            match with_instance_mut!(instance_id, |inst: &mut MapInstance| {
                if let Some(ik) = read_arg_i64(args, args_len, 0) {
                    if let Some(value) = inst.data_i64.remove(&ik) {
                        if debug {
                            eprintln!(
                                "[map-plugin] remove int key={} variant={}",
                                ik,
                                v_to_string(&value)
                            );
                        }
                        return write_mapval_tlv(&value, result, result_len);
                    }
                    unsafe {
                        if !result_len.is_null() {
                            *result_len = 0;
                        }
                    }
                    return NYB_SUCCESS;
                }
                if let Some(sk) = read_arg_string(args, args_len, 0) {
                    if let Some(value) = inst.data_str.remove(&sk) {
                        if debug {
                            eprintln!(
                                "[map-plugin] remove str key={} variant={}",
                                sk,
                                v_to_string(&value)
                            );
                        }
                        return write_mapval_tlv(&value, result, result_len);
                    }
                    unsafe {
                        if !result_len.is_null() {
                            *result_len = 0;
                        }
                    }
                    return NYB_SUCCESS;
                }
                NYB_E_INVALID_ARGS
            }) {
                Ok(rc) => rc,
                Err(_) => NYB_E_INVALID_HANDLE,
            }
        }
        METHOD_CLEAR => {
            match with_instance_mut!(instance_id, |inst: &mut MapInstance| {
                inst.data_i64.clear();
                inst.data_str.clear();
                if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                    eprintln!("[map-plugin] clear instance {}", instance_id);
                }
                unsafe {
                    if !result_len.is_null() {
                        *result_len = 0;
                    }
                }
                NYB_SUCCESS
            }) {
                Ok(rc) => rc,
                Err(_) => NYB_E_INVALID_HANDLE,
            }
        }
        METHOD_GET_STR => {
            let key = match read_arg_string(args, args_len, 0) {
                Some(s) => s,
                None => return NYB_E_INVALID_ARGS,
            };
            match with_instance!(instance_id, |inst: &MapInstance| {
                match inst.data_str.get(&key) {
                    Some(MapVal::I64(v)) => write_tlv_i64(*v, result, result_len),
                    Some(MapVal::Str(s)) => write_tlv_string(s, result, result_len),
                    Some(MapVal::Handle(t, i)) => {
                        if (*t == TYPE_ID_ARRAY) {
                            unsafe {
                                extern "C" { fn nyash_host_from_plugin_handle(type_id: u32, instance_id: u32) -> u64; }
                                let h = nyash_host_from_plugin_handle(*t, *i);
                                write_tlv_host_handle(h as u64, result, result_len)
                            }
                        } else {
                            write_tlv_handle(*t, *i, result, result_len)
                        }
                    },
                    Some(MapVal::Host(h)) => {
                        write_tlv_host_handle(*h as u64, result, result_len)
                    }
                    None => NYB_E_INVALID_ARGS,
                }
            }) {
                Ok(rc) => rc,
                Err(_) => NYB_E_INVALID_HANDLE,
            }
        }
        METHOD_HAS_STR => {
            let key = match read_arg_string(args, args_len, 0) {
                Some(s) => s,
                None => return NYB_E_INVALID_ARGS,
            };
            match with_instance!(instance_id, |inst: &MapInstance| {
                write_tlv_bool(
                    hako_core_map::has_key_str(&inst.data_str, &key),
                    result,
                    result_len,
                )
            }) {
                Ok(rc) => rc,
                Err(_) => NYB_E_INVALID_HANDLE,
            }
        }
        METHOD_KEYS_S => {
            match with_instance!(instance_id, |inst: &MapInstance| {
                let mut keys: Vec<String> =
                    Vec::with_capacity(inst.data_i64.len() + inst.data_str.len());
                for k in inst.data_i64.keys() {
                    keys.push(k.to_string());
                }
                for k in inst.data_str.keys() {
                    keys.push(k.clone());
                }
                keys.sort();
                let out = keys.join("\n");
                write_tlv_string(&out, result, result_len)
            }) {
                Ok(rc) => rc,
                Err(_) => NYB_E_INVALID_HANDLE,
            }
        }
        METHOD_VALUES_S => {
            match with_instance!(instance_id, |inst: &MapInstance| {
                let keys = hako_core_map::keys_sorted_from_maps_i64_str(
                    &inst.data_i64,
                    &inst.data_str,
                );
                let aligned =
                    hako_core_map::values_for_keys(&keys, &inst.data_i64, &inst.data_str);
                let mut vals: Vec<String> = Vec::with_capacity(keys.len());
                for v in aligned.into_iter() {
                    vals.push(v.map(|vv| v_to_string(vv)).unwrap_or_default());
                }
                let out = vals.join("\n");
                write_tlv_string(&out, result, result_len)
            }) {
                Ok(rc) => rc,
                Err(_) => NYB_E_INVALID_HANDLE,
            }
        }
        _ => NYB_E_INVALID_METHOD,
    }
}

#[no_mangle]
pub extern "C" fn nyash_plugin_invoke(
    type_id: u32,
    method_id: u32,
    instance_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    if type_id != TYPE_ID_MAP {
        return NYB_E_INVALID_TYPE;
    }
    mapbox_invoke_id(instance_id, method_id, args, args_len, result, result_len)
}

#[no_mangle]
#[used]
pub static nyash_typebox_MapBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258, // 'TYBX'
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"MapBox\0".as_ptr() as *const c_char,
    resolve: Some(mapbox_resolve),
    invoke_id: Some(mapbox_invoke_id),
    capabilities: 0,
};

// Static-link per-Box invoke symbol for host static registration
#[no_mangle]
pub extern "C" fn nyash_map_plugin_invoke_static(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    mapbox_invoke_id(instance_id, method_id, args, args_len, result, result_len)
}
