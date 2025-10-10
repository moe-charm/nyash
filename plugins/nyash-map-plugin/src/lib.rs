//! Nyash MapBox Plugin — TypeBox v2 (minimal)
//! Methods: birth(0), size(1), get(2), has(3), set(4), fini(u32::MAX)
//! Extension: support both i64 and UTF-8 string keys; values remain i64.

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex,
};

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
const METHOD_KEYS: u32 = 15;   // keys() -> HostHandle(ArrayBox)
const METHOD_VALUES: u32 = 16; // values() -> HostHandle(ArrayBox)

// Type id (nyash.toml に合わせる)
const TYPE_ID_MAP: u32 = 11;
const TYPE_ID_ARRAY: u32 = 12; // must match Array plugin spec

enum MapVal { I64(i64), Handle(u32, u32), Host(u64) }

struct MapInstance {
    data_i64: HashMap<i64, MapVal>,
    data_str: HashMap<String, MapVal>,
}
static INSTANCES: Lazy<Mutex<HashMap<u32, MapInstance>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static INSTANCE_COUNTER: AtomicU32 = AtomicU32::new(1);

// legacy v1 abi/init removed

/* legacy v1 entry removed
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
    unsafe {
        match method_id {
            METHOD_BIRTH => {
                if result_len.is_null() {
                    return NYB_E_INVALID_ARGS;
                }
                if preflight(result, result_len, 4) {
                    return NYB_E_SHORT_BUFFER;
                }
                let id = INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut map) = INSTANCES.lock() {
                    map.insert(
                        id,
                        MapInstance {
                            data_i64: HashMap::new(),
                            data_str: HashMap::new(),
                        },
                    );
                } else {
                    return NYB_E_PLUGIN_ERROR;
                }
                std::ptr::copy_nonoverlapping(id.to_le_bytes().as_ptr(), result, 4);
                *result_len = 4;
                NYB_SUCCESS
            }
            METHOD_FINI => {
                if let Ok(mut map) = INSTANCES.lock() {
                    map.remove(&instance_id);
                    NYB_SUCCESS
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            METHOD_SIZE => {
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        let sz = inst.data_i64.len() + inst.data_str.len();
                        write_tlv_i64(sz as i64, result, result_len)
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            METHOD_GET => {
                if let Some(ik) = read_arg_i64(args, args_len, 0) {
                    if let Ok(map) = INSTANCES.lock() {
                        if let Some(inst) = map.get(&instance_id) {
                            match inst.data_i64.get(&ik).copied() {
                                Some(v) => write_tlv_i64(v, result, result_len),
                                None => NYB_E_INVALID_ARGS,
                            }
                        } else {
                            NYB_E_INVALID_HANDLE
                        }
                    } else {
                        NYB_E_PLUGIN_ERROR
                    }
                } else if let Some(sk) = read_arg_string(args, args_len, 0) {
                    if let Ok(map) = INSTANCES.lock() {
                        if let Some(inst) = map.get(&instance_id) {
                            match inst.data_str.get(&sk).copied() {
                                Some(v) => write_tlv_i64(v, result, result_len),
                                None => NYB_E_INVALID_ARGS,
                            }
                        } else {
                            NYB_E_INVALID_HANDLE
                        }
                    } else {
                        NYB_E_PLUGIN_ERROR
                    }
                } else {
                    NYB_E_INVALID_ARGS
                }
            }
            METHOD_HAS => {
                if let Some(ik) = read_arg_i64(args, args_len, 0) {
                    if let Ok(map) = INSTANCES.lock() {
                        if let Some(inst) = map.get(&instance_id) {
                            write_tlv_bool(inst.data_i64.contains_key(&ik), result, result_len)
                        } else {
                            NYB_E_INVALID_HANDLE
                        }
                    } else {
                        NYB_E_PLUGIN_ERROR
                    }
                } else if let Some(sk) = read_arg_string(args, args_len, 0) {
                    if let Ok(map) = INSTANCES.lock() {
                        if let Some(inst) = map.get(&instance_id) {
                            write_tlv_bool(inst.data_str.contains_key(&sk), result, result_len)
                        } else {
                            NYB_E_INVALID_HANDLE
                        }
                    } else {
                        NYB_E_PLUGIN_ERROR
                    }
                } else {
                    NYB_E_INVALID_ARGS
                }
            }
            METHOD_SET => {
                // value は i64 限定
                let val = match read_arg_i64(args, args_len, 1) {
                    Some(v) => v,
                    None => return NYB_E_INVALID_ARGS,
                };
                if let Some(ik) = read_arg_i64(args, args_len, 0) {
                    if let Ok(mut map) = INSTANCES.lock() {
                        if let Some(inst) = map.get_mut(&instance_id) {
                            inst.data_i64.insert(ik, val);
                            let sz = inst.data_i64.len() + inst.data_str.len();
                            return write_tlv_i64(sz as i64, result, result_len);
                        } else {
                            NYB_E_INVALID_HANDLE
                        }
                    } else {
                        NYB_E_PLUGIN_ERROR
                    }
                } else if let Some(sk) = read_arg_string(args, args_len, 0) {
                    if let Ok(mut map) = INSTANCES.lock() {
                        if let Some(inst) = map.get_mut(&instance_id) {
                            inst.data_str.insert(sk, val);
                            let sz = inst.data_i64.len() + inst.data_str.len();
                            return write_tlv_i64(sz as i64, result, result_len);
                        } else {
                            NYB_E_INVALID_HANDLE
                        }
                    } else {
                        NYB_E_PLUGIN_ERROR
                    }
                } else {
                    NYB_E_INVALID_ARGS
                }
            }
            METHOD_REMOVE => {
                if let Ok(mut map) = INSTANCES.lock() {
                    if let Some(inst) = map.get_mut(&instance_id) {
                        // try int key
                        if let Some(ik) = read_arg_i64(args, args_len, 0) {
                            return write_tlv_bool(
                                inst.data_i64.remove(&ik).is_some(),
                                result,
                                result_len,
                            );
                        }
                        // try string key
                        if let Some(sk) = read_arg_string(args, args_len, 0) {
                            return write_tlv_bool(
                                inst.data_str.remove(&sk).is_some(),
                                result,
                                result_len,
                            );
                        }
                        NYB_E_INVALID_ARGS
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            METHOD_CLEAR => {
                if let Ok(mut map) = INSTANCES.lock() {
                    if let Some(inst) = map.get_mut(&instance_id) {
                        inst.data_i64.clear();
                        inst.data_str.clear();
                        return write_tlv_i64(0, result, result_len);
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            METHOD_KEYS_S => {
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
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
                        return write_tlv_string(&out, result, result_len);
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            METHOD_GET_OR => {
                let defv = match read_arg_i64(args, args_len, 1) {
                    Some(v) => v,
                    None => return NYB_E_INVALID_ARGS,
                };
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        // prefer exact match, else default
                        if let Some(ik) = read_arg_i64(args, args_len, 0) {
                            let v = inst.data_i64.get(&ik).copied().unwrap_or(defv);
                            return write_tlv_i64(v, result, result_len);
                        }
                        if let Some(sk) = read_arg_string(args, args_len, 0) {
                            let v = inst.data_str.get(&sk).copied().unwrap_or(defv);
                            return write_tlv_i64(v, result, result_len);
                        }
                        NYB_E_INVALID_ARGS
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            METHOD_SET_STR => {
                let key = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return NYB_E_INVALID_ARGS,
                };
                let val = match read_arg_i64(args, args_len, 1) {
                    Some(v) => v,
                    None => return NYB_E_INVALID_ARGS,
                };
                if let Ok(mut map) = INSTANCES.lock() {
                    if let Some(inst) = map.get_mut(&instance_id) {
                        inst.data_str.insert(key, val);
                        let sz = inst.data_i64.len() + inst.data_str.len();
                        return write_tlv_i64(sz as i64, result, result_len);
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            METHOD_GET_STR => {
                let key = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return NYB_E_INVALID_ARGS,
                };
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        match inst.data_str.get(&key).copied() {
                            Some(v) => write_tlv_i64(v, result, result_len),
                            None => NYB_E_INVALID_ARGS,
                        }
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            METHOD_HAS_STR => {
                let key = match read_arg_string(args, args_len, 0) {
                    Some(s) => s,
                    None => return NYB_E_INVALID_ARGS,
                };
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        write_tlv_bool(hako_core_map::has_key_str(&inst.data_str, &key), result, result_len)
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            METHOD_VALUES_S => {
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        let mut vals: Vec<String> =
                            Vec::with_capacity(inst.data_i64.len() + inst.data_str.len());
                        for v in inst.data_i64.values() {
                            vals.push(v.to_string());
                        }
                        for v in inst.data_str.values() {
                            vals.push(v.to_string());
                        }
                        let out = vals.join("\n");
                        return write_tlv_string(&out, result, result_len);
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            METHOD_TO_JSON => {
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        let mut s = String::from("{");
                        let mut first = true;
                        for (k, v) in inst.data_str.iter() {
                            if !first {
                                s.push(',');
                            }
                            first = false;
                            // JSON string key
                            s.push('"');
                            s.push_str(&escape_json(k));
                            s.push_str("\": ");
                            s.push_str(&v.to_string());
                        }
                        for (k, v) in inst.data_i64.iter() {
                            if !first {
                                s.push(',');
                            }
                            first = false;
                            // numeric key as string per JSON
                            s.push('"');
                            s.push_str(&k.to_string());
                            s.push_str("\": ");
                            s.push_str(&v.to_string());
                        }
                        s.push('}');
                        return write_tlv_string(&s, result, result_len);
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            }
            _ => NYB_E_INVALID_METHOD,
        }
    }
}
*/

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
        "size" | "len" => METHOD_SIZE,
        "get" => METHOD_GET,
        "has" => METHOD_HAS,
        "set" => METHOD_SET,
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
            let enable = std::env::var("NYASH_PLUGIN_MAP_ARRAY_HANDLE").ok().as_deref() == Some("1");
            if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                eprintln!("[map-plugin] METHOD_KEYS/VALUES stage2 enable={}", enable);
            }
            if !enable { return NYB_E_INVALID_METHOD; }
            unsafe {
                extern "C" { fn nyash_array_new_h() -> i64; }
                extern "C" { fn nyrt_host_call_slot(handle: u64, selector_id: u64,
                                       args_ptr: *const u8, args_len: usize,
                                       out_ptr: *mut u8, out_len: *mut usize) -> i32; }
                let arr_h = nyash_array_new_h();
                if arr_h <= 0 { return NYB_E_PLUGIN_ERROR; }
                let mut i: i64 = 0;
                // Prepare pairs (index, string)
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        if method_id == METHOD_KEYS {
                            // keys as strings, sorted
                            let keys = hako_core_map::keys_sorted_from_maps_i64_str(&inst.data_i64, &inst.data_str);
                            for k in keys.into_iter() {
                                let tlv = build_tlv_i64_string(i, &k);
                                // Robust out buffer: start moderately sized and grow on SHORT_BUFFER
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
                                    // Host encode_out uses -3 for SHORT_BUFFER
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
                                        eprintln!("[map-plugin] host_call_slot(Array.set) failed rc={} (len={})", rc, out_len);
                                    }
                                    return NYB_E_PLUGIN_ERROR;
                                };
                                if rc != 0 { return NYB_E_PLUGIN_ERROR; }
                                i += 1;
                            }
                        } else {
                            // values aligned to sorted keys; numeric as i64, others stringified
                            let keys = hako_core_map::keys_sorted_from_maps_i64_str(&inst.data_i64, &inst.data_str);
                            let aligned = hako_core_map::values_for_keys(&keys, &inst.data_i64, &inst.data_str);
                            for v in aligned.into_iter() {
                                let rc = if let Some(MapVal::I64(n)) = v {
                                    let tlv = build_tlv_i64_i64(i, *n);
                                    let mut cap: usize = 64; let mut out: Vec<u8> = vec![0u8; cap];
                                    loop { let mut out_len: usize = out.len();
                                        let rc = nyrt_host_call_slot(arr_h as u64, 101u64, tlv.as_ptr(), tlv.len(), out.as_mut_ptr(), &mut out_len);
                                        if rc == 0 { break rc; }
                                        if rc == -3 { cap = cap.saturating_mul(2); if cap > 64*1024 { return NYB_E_PLUGIN_ERROR; } out.resize(cap,0); continue; }
                                        return NYB_E_PLUGIN_ERROR;
                                    }
                                } else {
                                    let s = v.map(|vv| v_to_string(vv)).unwrap_or_else(String::new);
                                    let tlv = build_tlv_i64_string(i, &s);
                                    let mut cap: usize = 64; let mut out: Vec<u8> = vec![0u8; cap];
                                    loop { let mut out_len: usize = out.len();
                                        let rc = nyrt_host_call_slot(arr_h as u64, 101u64, tlv.as_ptr(), tlv.len(), out.as_mut_ptr(), &mut out_len);
                                        if rc == 0 { break rc; }
                                        if rc == -3 { cap = cap.saturating_mul(2); if cap > 64*1024 { return NYB_E_PLUGIN_ERROR; } out.resize(cap,0); continue; }
                                        return NYB_E_PLUGIN_ERROR;
                                    }
                                };
                                if rc != 0 { return NYB_E_PLUGIN_ERROR; }
                                i += 1;
                            }
                        }
                        }
                    } else { return NYB_E_INVALID_HANDLE; }
                } else { return NYB_E_PLUGIN_ERROR; }
                return write_tlv_host_handle(arr_h as u64, result, result_len);
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
            let id = INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
            if let Ok(mut map) = INSTANCES.lock() {
                map.insert(
                    id,
                    MapInstance { data_i64: HashMap::new(), data_str: HashMap::new() },
                );
            } else {
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
                eprintln!("[map-plugin] METHOD_BIRTH: id={} rc={} out_cap={} need={} tag=8", id, rc, out_cap, need);
            }
            rc
        }
        METHOD_SIZE => {
            if let Ok(map) = INSTANCES.lock() {
                if let Some(inst) = map.get(&instance_id) {
                    let sz = inst.data_i64.len() + inst.data_str.len();
                    return write_tlv_i64(sz as i64, result, result_len);
                } else {
                    NYB_E_INVALID_HANDLE
                }
            } else {
                NYB_E_PLUGIN_ERROR
            }
        }
        METHOD_FINI => {
            if let Ok(mut map) = INSTANCES.lock() {
                map.remove(&instance_id);
                NYB_SUCCESS
            } else {
                NYB_E_PLUGIN_ERROR
            }
        }
        METHOD_GET => {
            if let Some(ik) = read_arg_i64(args, args_len, 0) {
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        match inst.data_i64.get(&ik) {
                            Some(MapVal::I64(v)) => write_tlv_i64(*v, result, result_len),
                            Some(MapVal::Handle(t,i)) => write_tlv_handle(*t, *i, result, result_len),
                            Some(MapVal::Host(h)) => write_tlv_host_handle(*h as u64, result, result_len),
                            None => NYB_E_INVALID_ARGS,
                        }
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            } else if let Some(sk) = read_arg_string(args, args_len, 0) {
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        match inst.data_str.get(&sk) {
                            Some(MapVal::I64(v)) => write_tlv_i64(*v, result, result_len),
                            Some(MapVal::Handle(t,i)) => write_tlv_handle(*t, *i, result, result_len),
                            Some(MapVal::Host(h)) => write_tlv_host_handle(*h as u64, result, result_len),
                            None => NYB_E_INVALID_ARGS,
                        }
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            } else {
                NYB_E_INVALID_ARGS
            }
        }
        METHOD_HAS => {
            if let Some(ik) = read_arg_i64(args, args_len, 0) {
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        write_tlv_bool(inst.data_i64.contains_key(&ik), result, result_len)
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            } else if let Some(sk) = read_arg_string(args, args_len, 0) {
                if let Ok(map) = INSTANCES.lock() {
                    if let Some(inst) = map.get(&instance_id) {
                        write_tlv_bool(inst.data_str.contains_key(&sk), result, result_len)
                    } else {
                        NYB_E_INVALID_HANDLE
                    }
                } else {
                    NYB_E_PLUGIN_ERROR
                }
            } else {
                NYB_E_INVALID_ARGS
            }
        }
        METHOD_SET => {
            // key: i64 or string; value: i64 or Handle(tag=8) or HostHandle(tag=9)
            let val = if let Some(v) = read_arg_i64(args, args_len, 1) {
                MapVal::I64(v)
            } else if let Some((t,i)) = read_arg_handle(args, args_len, 1) {
                MapVal::Handle(t,i)
            } else if let Some(h) = read_arg_host_handle(args, args_len, 1) {
                MapVal::Host(h)
            } else {
                return NYB_E_INVALID_ARGS;
            };
            if let Some(ik) = read_arg_i64(args, args_len, 0) {
                if let Ok(mut map) = INSTANCES.lock() {
                    if let Some(inst) = map.get_mut(&instance_id) {
                        inst.data_i64.insert(ik, val);
                        let sz = inst.data_i64.len() + inst.data_str.len();
                        return write_tlv_i64(sz as i64, result, result_len);
                    } else { return NYB_E_INVALID_HANDLE; }
                } else { return NYB_E_PLUGIN_ERROR; }
            } else if let Some(sk) = read_arg_string(args, args_len, 0) {
                if let Ok(mut map) = INSTANCES.lock() {
                    if let Some(inst) = map.get_mut(&instance_id) {
                        inst.data_str.insert(sk, val);
                        let sz = inst.data_i64.len() + inst.data_str.len();
                        return write_tlv_i64(sz as i64, result, result_len);
                    } else { return NYB_E_INVALID_HANDLE; }
                } else { return NYB_E_PLUGIN_ERROR; }
            } else {
                NYB_E_INVALID_ARGS
            }
        }
        METHOD_GET_STR => {
            let key = match read_arg_string(args, args_len, 0) {
                Some(s) => s,
                None => return NYB_E_INVALID_ARGS,
            };
            if let Ok(map) = INSTANCES.lock() {
                if let Some(inst) = map.get(&instance_id) {
                    match inst.data_str.get(&key) {
                        Some(MapVal::I64(v)) => write_tlv_i64(*v, result, result_len),
                        Some(MapVal::Handle(t,i)) => write_tlv_handle(*t, *i, result, result_len),
                        Some(MapVal::Host(h)) => write_tlv_host_handle(*h as u64, result, result_len),
                        None => NYB_E_INVALID_ARGS,
                    }
                } else {
                    NYB_E_INVALID_HANDLE
                }
            } else {
                NYB_E_PLUGIN_ERROR
            }
        }
        METHOD_HAS_STR => {
            let key = match read_arg_string(args, args_len, 0) {
                Some(s) => s,
                None => return NYB_E_INVALID_ARGS,
            };
            if let Ok(map) = INSTANCES.lock() {
                if let Some(inst) = map.get(&instance_id) {
                    write_tlv_bool(hako_core_map::has_key_str(&inst.data_str, &key), result, result_len)
                } else {
                    NYB_E_INVALID_HANDLE
                }
            } else {
                NYB_E_PLUGIN_ERROR
            }
        }
        METHOD_KEYS_S => {
            if let Ok(map) = INSTANCES.lock() {
                if let Some(inst) = map.get(&instance_id) {
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
                    return write_tlv_string(&out, result, result_len);
                } else {
                    NYB_E_INVALID_HANDLE
                }
            } else {
                NYB_E_PLUGIN_ERROR
            }
        }
        METHOD_VALUES_S => {
            if let Ok(map) = INSTANCES.lock() {
                if let Some(inst) = map.get(&instance_id) {
                    let keys = hako_core_map::keys_sorted_from_maps_i64_str(&inst.data_i64, &inst.data_str);
                    let aligned = hako_core_map::values_for_keys(&keys, &inst.data_i64, &inst.data_str);
                    let mut vals: Vec<String> = Vec::with_capacity(keys.len());
                    for v in aligned.into_iter() { vals.push(v.map(|vv| v_to_string(vv)).unwrap_or_default()); }
                    let out = vals.join("\n");
                    return write_tlv_string(&out, result, result_len);
                } else { NYB_E_INVALID_HANDLE }
            } else { NYB_E_PLUGIN_ERROR }
        }
        _ => NYB_E_INVALID_METHOD,
    }
}

#[no_mangle]
pub static nyash_typebox_MapBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258, // 'TYBX'
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"MapBox\0".as_ptr() as *const c_char,
    resolve: Some(mapbox_resolve),
    invoke_id: Some(mapbox_invoke_id),
    capabilities: 0,
};

fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

// TLV helpers
fn preflight(result: *mut u8, result_len: *mut usize, needed: usize) -> bool {
    unsafe {
        if result_len.is_null() {
            return false;
        }
        if result.is_null() || *result_len < needed {
            *result_len = needed;
            return true;
        }
    }
    false
}

fn write_tlv_result(payloads: &[(u8, &[u8])], result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return NYB_E_INVALID_ARGS;
    }
    let mut buf: Vec<u8> =
        Vec::with_capacity(4 + payloads.iter().map(|(_, p)| 4 + p.len()).sum::<usize>());
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&(payloads.len() as u16).to_le_bytes());
    for (tag, payload) in payloads {
        buf.push(*tag);
        buf.push(0);
        buf.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        buf.extend_from_slice(payload);
    }
    unsafe {
        let needed = buf.len();
        if result.is_null() || *result_len < needed {
            *result_len = needed;
            return NYB_E_SHORT_BUFFER;
        }
        std::ptr::copy_nonoverlapping(buf.as_ptr(), result, needed);
        *result_len = needed;
    }
    NYB_SUCCESS
}

fn write_tlv_handle(type_id: u32, instance_id: u32, result: *mut u8, result_len: *mut usize) -> i32 {
    let mut payload = Vec::with_capacity(8);
    payload.extend_from_slice(&type_id.to_le_bytes());
    payload.extend_from_slice(&instance_id.to_le_bytes());
    write_tlv_result(&[(8u8, &payload)], result, result_len)
}

fn write_tlv_i64(v: i64, result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(3u8, &v.to_le_bytes())], result, result_len)
}
fn write_tlv_bool(bv: bool, result: *mut u8, result_len: *mut usize) -> i32 {
    let b = [if bv { 1u8 } else { 0u8 }];
    write_tlv_result(&[(1u8, &b)], result, result_len)
}
fn write_tlv_string(s: &str, result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(6u8, s.as_bytes())], result, result_len)
}

fn read_arg_i64(args: *const u8, args_len: usize, n: usize) -> Option<i64> {
    if args.is_null() || args_len < 4 {
        return None;
    }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize;
    for i in 0..=n {
        if buf.len() < off + 4 {
            return None;
        }
        let tag = buf[off];
        let size = u16::from_le_bytes([buf[off + 2], buf[off + 3]]) as usize;
        if buf.len() < off + 4 + size {
            return None;
        }
        if i == n {
            if tag == 3 && size == 8 {
                let mut b = [0u8; 8];
                b.copy_from_slice(&buf[off + 4..off + 12]);
                return Some(i64::from_le_bytes(b));
            } else if tag == 2 && size == 4 {
                let mut b = [0u8; 4];
                b.copy_from_slice(&buf[off + 4..off + 8]);
                return Some(i32::from_le_bytes(b) as i64);
            } else {
                return None;
            }
        }
        off += 4 + size;
    }
    None
}

fn read_arg_host_handle(args: *const u8, args_len: usize, n: usize) -> Option<u64> {
    if args.is_null() || args_len < 4 { return None; }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize;
    for i in 0..=n {
        if buf.len() < off + 4 { return None; }
        let tag = buf[off];
        let size = u16::from_le_bytes([buf[off+2], buf[off+3]]) as usize;
        if buf.len() < off + 4 + size { return None; }
        if i == n {
            if tag == 9 && size == 8 {
                let mut h = [0u8;8]; h.copy_from_slice(&buf[off+4..off+12]);
                return Some(u64::from_le_bytes(h));
            } else { return None; }
        }
        off += 4 + size;
    }
    None
}

fn read_arg_string(args: *const u8, args_len: usize, n: usize) -> Option<String> {
    if args.is_null() || args_len < 4 {
        return None;
    }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize;
    for i in 0..=n {
        if buf.len() < off + 4 {
            return None;
        }
        let tag = buf[off];
        let size = u16::from_le_bytes([buf[off + 2], buf[off + 3]]) as usize;
        if buf.len() < off + 4 + size {
            return None;
        }
        if i == n {
            if tag == 6 || tag == 7 {
                let s = &buf[off + 4..off + 4 + size];
                return Some(String::from_utf8_lossy(s).to_string());
            } else {
                return None;
            }
        }
        off += 4 + size;
    }
    None
}

fn read_arg_handle(args: *const u8, args_len: usize, n: usize) -> Option<(u32, u32)> {
    if args.is_null() || args_len < 4 { return None; }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize;
    for i in 0..=n {
        if buf.len() < off + 4 { return None; }
        let tag = buf[off];
        let size = u16::from_le_bytes([buf[off+2], buf[off+3]]) as usize;
        if buf.len() < off + 4 + size { return None; }
        if i == n {
            if tag == 8 && size == 8 {
            let mut t = [0u8;4]; t.copy_from_slice(&buf[off+4..off+8]);
            let mut id = [0u8;4]; id.copy_from_slice(&buf[off+8..off+12]);
            return Some((u32::from_le_bytes(t), u32::from_le_bytes(id)));
        } else { return None; }
        }
        off += 4 + size;
    }
    None
}

// ---- TLV helpers (local) for Stage-2 ----
fn build_tlv_i64_string(idx: i64, s: &str) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(4 + 4 + 8 + 4 + s.len());
    // header: version=1, argc=2
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes());
    // arg0: i64 idx (tag=3,size=8)
    buf.push(3u8); buf.push(0u8);
    buf.extend_from_slice(&(8u16).to_le_bytes());
    buf.extend_from_slice(&idx.to_le_bytes());
    // arg1: string (tag=6)
    buf.push(6u8); buf.push(0u8);
    let len = core::cmp::min(s.as_bytes().len(), u16::MAX as usize) as u16;
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(&s.as_bytes()[..len as usize]);
    buf
}

fn write_tlv_host_handle(handle_id: u64, result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() { return NYB_E_INVALID_ARGS; }
    let mut buf: Vec<u8> = Vec::with_capacity(4 + 4 + 8);
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.push(9u8); buf.push(0u8);
    buf.extend_from_slice(&(8u16).to_le_bytes());
    buf.extend_from_slice(&handle_id.to_le_bytes());
    unsafe {
        let need = buf.len();
        if result.is_null() || *result_len < need { *result_len = need; return NYB_E_SHORT_BUFFER; }
        std::ptr::copy_nonoverlapping(buf.as_ptr(), result, need);
        *result_len = need;
    }
    NYB_SUCCESS
}

fn v_to_string(v: &MapVal) -> String { match v { MapVal::I64(n) => n.to_string(), MapVal::Handle(t,i) => format!("handle({},{})", t,i), MapVal::Host(_) => "host-handle".to_string() } }

/// Build TLV with two i64 arguments (for Array.set via host)
fn build_tlv_i64_i64(idx: i64, value: i64) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(4 + 4 + 8 + 4 + 8);
    // header: version=1, argc=2
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes());
    // arg0: i64 idx (tag=3, size=8)
    buf.push(3u8); buf.push(0u8);
    buf.extend_from_slice(&(8u16).to_le_bytes());
    buf.extend_from_slice(&idx.to_le_bytes());
    // arg1: i64 value (tag=3, size=8)
    buf.push(3u8); buf.push(0u8);
    buf.extend_from_slice(&(8u16).to_le_bytes());
    buf.extend_from_slice(&value.to_le_bytes());
    buf
}

