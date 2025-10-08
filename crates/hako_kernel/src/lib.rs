// Hako Kernel — minimal static runtime shim for AOT linking
// Goal: provide tiny C-ABI functions for symbols used by the llvmlite harness
// during codegen (e.g., nyash.box.from_i8_string, nyash.array.push_h, etc.).
//
// Default behavior (NYASH_HAKO_MIN_SEM != 1): return neutral values to keep
// linking lightweight. When NYASH_HAKO_MIN_SEM=1, a tiny in-process arena
// provides minimal semantics for String/Array/Map and basic boxing.

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::{
    atomic::{AtomicI64, Ordering},
    Mutex, OnceLock,
};


fn min_sem_enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("NYASH_HAKO_MIN_SEM").ok().as_deref() == Some("1"))
}

#[derive(Debug)]
enum Obj {
    Str(String),
    Int(i64),
    F64(f64),
    Arr(Vec<i64>),             // elements are handles (0 allowed as Null)
    Map(HashMap<String, i64>), // keys are strings; values are handles
}

struct Arena {
    next: AtomicI64,
    objs: Mutex<HashMap<i64, Obj>>,
    cstr: Mutex<HashMap<i64, CString>>, // cached string pointers per handle
}

impl Arena {
    fn new() -> Self {
        Self {
            next: AtomicI64::new(1),
            objs: Mutex::new(HashMap::new()),
            cstr: Mutex::new(HashMap::new()),
        }
    }
    fn alloc(&self, o: Obj) -> i64 {
        let h = self.next.fetch_add(1, Ordering::Relaxed);
        self.objs.lock().unwrap().insert(h, o);
        h
    }
    fn get(&self, h: i64) -> Option<ObjRef<'_>> {
        if h <= 0 {
            return None;
        }
        // SAFETY: we lock and immediately create a temporary ref wrapper
        let guard = self.objs.lock().unwrap();
        // Using raw pointer to extend borrow (read-only) within this function scope only
        let ptr: *const HashMap<i64, Obj> = &*guard;
        // Return a wrapper that holds the guard to keep map alive
        Some(ObjRef {
            _guard: guard,
            obj: unsafe { (*ptr).get(&h) },
        })
    }
    fn get_mut(&self, h: i64) -> Option<std::sync::MutexGuard<'_, HashMap<i64, Obj>>> {
        if h <= 0 {
            return None;
        }
        Some(self.objs.lock().unwrap())
    }
    fn str_ptr(&self, h: i64) -> *const i8 {
        if let Some(ObjRef {
            obj: Some(Obj::Str(s)),
            ..
        }) = self.get(h)
        {
            let mut cache = self.cstr.lock().unwrap();
            if let Some(cs) = cache.get(&h) {
                return cs.as_ptr();
            }
            let cs = CString::new(s.as_str()).unwrap_or_else(|_| CString::new("").unwrap());
            let p = cs.as_ptr();
            cache.insert(h, cs);
            p
        } else {
            ptr::null()
        }
    }
}

struct ObjRef<'a> {
    _guard: std::sync::MutexGuard<'a, HashMap<i64, Obj>>,
    obj: Option<&'a Obj>,
}


// Global buffer pool for functions that must return i8* (C string pointers)
// Lifetime: process-wide (we keep CStrings to keep pointers valid)
fn cpool() -> &'static std::sync::Mutex<Vec<CString>> {
    static POOL: OnceLock<std::sync::Mutex<Vec<CString>>> = OnceLock::new();
    POOL.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}

fn arena() -> &'static Arena {
    static A: OnceLock<Arena> = OnceLock::new();
    A.get_or_init(Arena::new)
}

#[no_mangle]
pub extern "C" fn _hako_kernel_init_marker() {}

// --- String helpers (handles are opaque i64: 0 means null/none) ---

// nyash.box.from_i8_string(ptr) -> i64 (handle)
// Minimal: ignore pointer, return 0 (no handle). Suitable for tests that don't
// inspect the string contents at runtime.
#[export_name = "nyash.box.from_i8_string"]
pub extern "C" fn nyash_box_from_i8_string(ptr_in: *const i8) -> i64 {
    if !min_sem_enabled() {
        return 0;
    }
    if ptr_in.is_null() {
        return 0;
    }
    let s = unsafe { CStr::from_ptr(ptr_in) }
        .to_string_lossy()
        .to_string();
    arena().alloc(Obj::Str(s))
}

// nyash.string.to_i8p_h(handle) -> i8* (debug/bridge)
// Minimal: return null pointer.
#[export_name = "nyash.string.to_i8p_h"]
pub extern "C" fn nyash_string_to_i8p_h(h: i64) -> *const i8 {
    if !min_sem_enabled() {
        return ptr::null();
    }
    arena().str_ptr(h)
}

// nyash.string.from_u64x2(lo_ptr, hi_unused, len) -> i64 (handle)
// Minimal: return 0 (unused by integer-only programs).
#[export_name = "nyash.string.from_u64x2"]
pub extern "C" fn nyash_string_from_u64x2(lo: u64, _hi: u64, len: u64) -> i64 {
    if !min_sem_enabled() {
        return 0;
    }
    if len == 0 {
        return arena().alloc(Obj::Str(String::new()));
    }
    // Interpret lo as pointer to bytes; best-effort
    let p = lo as *const u8;
    if p.is_null() {
        return 0;
    }
    let slice = unsafe { std::slice::from_raw_parts(p, len as usize) };
    let s = String::from_utf8_lossy(slice).to_string();
    arena().alloc(Obj::Str(s))
}

// nyash.string.birth_h() -> i64 (handle)
// Minimal: return 0 (no allocation).
#[export_name = "nyash.string.birth_h"]
pub extern "C" fn nyash_string_birth_h() -> i64 {
    if !min_sem_enabled() {
        return 0;
    }
    arena().alloc(Obj::Str(String::new()))
}

// nyrt.time.now_ms(): monotonic-ish millisecond counter
#[export_name = "nyrt.time.now_ms"]
pub extern "C" fn hako_time_now_ms() -> i64 {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_millis(0));
    let millis = duration.as_millis();
    if millis > i64::MAX as u128 {
        i64::MAX
    } else {
        millis as i64
    }
}

// nyash.string.len_h(handle) -> i64
#[export_name = "nyash.string.len_h"]
pub extern "C" fn nyash_string_len_h(h: i64) -> i64 {
    if !min_sem_enabled() {
        return 0;
    }
    match arena().get(h).and_then(|r| r.obj) {
        Some(Obj::Str(s)) => s.len() as i64,
        _ => 0,
    }
}

// nyash.string.charCodeAt_h(handle, idx) -> i64
#[export_name = "nyash.string.charCodeAt_h"]
pub extern "C" fn nyash_string_charcodeat_h(h: i64, idx: i64) -> i64 {
    if !min_sem_enabled() {
        return -1;
    }
    if idx < 0 {
        return -1;
    }
    match arena().get(h).and_then(|r| r.obj) {
        Some(Obj::Str(s)) => {
            let i = idx as usize;
            if i < s.len() {
                s.as_bytes()[i] as i64
            } else {
                -1
            }
        }
        _ => -1,
    }
}

// nyash.string.concat_hh(lhs_h, rhs_h) -> i64 (handle)
#[export_name = "nyash.string.concat_hh"]
pub extern "C" fn nyash_string_concat_hh(a: i64, b: i64) -> i64 {
    if !min_sem_enabled() {
        return 0;
    }
    let sa = match arena().get(a).and_then(|r| r.obj) {
        Some(Obj::Str(s)) => s.clone(),
        _ => String::new(),
    };
    let sb = match arena().get(b).and_then(|r| r.obj) {
        Some(Obj::Str(s)) => s.clone(),
        _ => String::new(),
    };
    arena().alloc(Obj::Str(format!("{}{}", sa, sb)))
}

// nyash.string.concat_ss(lhs_ptr, rhs_ptr) -> i8*
#[export_name = "nyash.string.concat_ss"]
pub extern "C" fn nyash_string_concat_ss(lp: *const i8, rp: *const i8) -> *const i8 {
    if lp.is_null() && rp.is_null() {
        return std::ptr::null();
    }
    let ls = if lp.is_null() { "".into() } else { unsafe { CStr::from_ptr(lp) }.to_string_lossy().to_string() };
    let rs = if rp.is_null() { "".into() } else { unsafe { CStr::from_ptr(rp) }.to_string_lossy().to_string() };
    let cs = CString::new(format!("{}{}", ls, rs)).unwrap_or_else(|_| CString::new("").unwrap());
    let p = cs.as_ptr();
    cpool().lock().unwrap().push(cs);
    p
}

// nyash.string.concat_si(lhs_ptr, rhs_i64) -> i8*
#[export_name = "nyash.string.concat_si"]
pub extern "C" fn nyash_string_concat_si(lp: *const i8, ri: i64) -> *const i8 {
    let ls = if lp.is_null() { "".into() } else { unsafe { CStr::from_ptr(lp) }.to_string_lossy().to_string() };
    let cs = CString::new(format!("{}{}", ls, ri)).unwrap_or_else(|_| CString::new("").unwrap());
    let p = cs.as_ptr();
    cpool().lock().unwrap().push(cs);
    p
}

// nyash.string.concat_is(lhs_i64, rhs_ptr) -> i8*
#[export_name = "nyash.string.concat_is"]
pub extern "C" fn nyash_string_concat_is(li: i64, rp: *const i8) -> *const i8 {
    let rs = if rp.is_null() { "".into() } else { unsafe { CStr::from_ptr(rp) }.to_string_lossy().to_string() };
    let cs = CString::new(format!("{}{}", li, rs)).unwrap_or_else(|_| CString::new("").unwrap());
    let p = cs.as_ptr();
    cpool().lock().unwrap().push(cs);
    p
}

// nyash.string.eq_hh(lhs_h, rhs_h) -> i64 (0/1)
#[export_name = "nyash.string.eq_hh"]
pub extern "C" fn nyash_string_eq_hh(a: i64, b: i64) -> i64 {
    if !min_sem_enabled() {
        return 0;
    }
    let sa = match arena().get(a).and_then(|r| r.obj) {
        Some(Obj::Str(s)) => s,
        _ => return 0,
    };
    let sb = match arena().get(b).and_then(|r| r.obj) {
        Some(Obj::Str(s)) => s,
        _ => return 0,
    };
    if sa == sb {
        1
    } else {
        0
    }
}

// nyash.string.substring_hii(h, start, end) -> i64 (handle)
#[export_name = "nyash.string.substring_hii"]
pub extern "C" fn nyash_string_substring_hii(h: i64, s: i64, e: i64) -> i64 {
    if !min_sem_enabled() {
        return 0;
    }
    let (start, end) = (s.max(0) as usize, e.max(0) as usize);
    match arena().get(h).and_then(|r| r.obj) {
        Some(Obj::Str(src)) => {
            let start = start.min(src.len());
            let end = end.min(src.len()).max(start);
            arena().alloc(Obj::Str(src[start..end].to_string()))
        }
        _ => 0,
    }
}

// nyash.string.lastIndexOf_hh(h, needle_h) -> i64
#[export_name = "nyash.string.lastIndexOf_hh"]
pub extern "C" fn nyash_string_lastindexof_hh(h: i64, n: i64) -> i64 {
    if !min_sem_enabled() {
        return -1;
    }
    match (
        arena().get(h).and_then(|r| r.obj),
        arena().get(n).and_then(|r| r.obj),
    ) {
        (Some(Obj::Str(hs)), Some(Obj::Str(ns))) => {
            hs.rfind(ns.as_str()).map(|i| i as i64).unwrap_or(-1)
        }
        _ => -1,
    }
}

// --- Any helpers ---

// nyash.any.length_h(handle) -> i64
#[export_name = "nyash.any.length_h"]
pub extern "C" fn nyash_any_length_h(h: i64) -> i64 {
    if !min_sem_enabled() {
        return 0;
    }
    let len = match arena().get(h).and_then(|r| r.obj) {
        Some(Obj::Arr(v)) => v.len() as i64,
        Some(Obj::Str(s)) => s.len() as i64,
        Some(Obj::Map(m)) => m.len() as i64,
        _ => 0,
    };
    if std::env::var("HAKO_DEBUG").ok().as_deref() == Some("1") {
        eprintln!("[hako_kernel] any.length h={} -> {}", h, len);
    }
    len
}

// --- Boxing helpers (numeric) ---

// nyash.box.from_i64(v) -> i64 (handle)
#[export_name = "nyash.box.from_i64"]
pub extern "C" fn nyash_box_from_i64(v: i64) -> i64 {
    if !min_sem_enabled() {
        return 0;
    }
    arena().alloc(Obj::Int(v))
}

// nyash.box.from_f64(v) -> i64 (handle)
#[export_name = "nyash.box.from_f64"]
pub extern "C" fn nyash_box_from_f64(v: f64) -> i64 {
    if !min_sem_enabled() {
        return 0;
    }
    arena().alloc(Obj::F64(v))
}

// --- Collections minimal bridge (Array/Map birth & size only) ---

// birth (no-arg constructors) return 0 to indicate null (unused in integer-only smokes)
#[export_name = "nyash.array.birth_h"]
pub extern "C" fn nyash_array_birth_h() -> i64 {
    if !min_sem_enabled() {
        return 0;
    }
    let h = arena().alloc(Obj::Arr(Vec::new()));
    if std::env::var("HAKO_DEBUG").ok().as_deref() == Some("1") {
        eprintln!("[hako_kernel] array.birth -> {}", h);
    }
    h
}

#[export_name = "nyash.map.birth_h"]
pub extern "C" fn nyash_map_birth_h() -> i64 {
    if !min_sem_enabled() {
        return 0;
    }
    arena().alloc(Obj::Map(HashMap::new()))
}

// Additional aliases sometimes used by MIR v1 calls
#[export_name = "nyash.array.new"]
pub extern "C" fn nyash_array_new() -> i64 {
    nyash_array_birth_h()
}
#[export_name = "nyash.map.new"]
pub extern "C" fn nyash_map_new() -> i64 {
    nyash_map_birth_h()
}

// Optional env API used by some builder paths (new box via type string)
#[export_name = "nyash.env.box.new_i64x"]
pub extern "C" fn nyash_env_box_new_i64x(
    typ: *const i8,
    _argc: i64,
    _a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
) -> i64 {
    if !min_sem_enabled() {
        return 0;
    }
    if typ.is_null() {
        return 0;
    }
    let ty = unsafe { CStr::from_ptr(typ) }.to_string_lossy();
    match ty.as_ref() {
        "ArrayBox" => nyash_array_birth_h(),
        "MapBox" => nyash_map_birth_h(),
        "StringBox" => nyash_string_birth_h(),
        _ => 0,
    }
}

// Array and Map operations
#[export_name = "nyash.array.push_h"]
pub extern "C" fn nyash_array_push_h(arr_h: i64, val_h: i64) -> i64 {
    if !min_sem_enabled() {
        return 0;
    }
    if let Some(mut guard) = arena().get_mut(arr_h) {
        if let Some(Obj::Arr(v)) = guard.get_mut(&arr_h) {
            v.push(val_h);
            if std::env::var("HAKO_DEBUG").ok().as_deref() == Some("1") {
                eprintln!(
                    "[hako_kernel] array.push h={} val={} -> len={}",
                    arr_h,
                    val_h,
                    v.len()
                );
            }
            return v.len() as i64;
        }
    }
    0
}

#[export_name = "nyash.map.set_hh"]
pub extern "C" fn nyash_map_set_hh(map_h: i64, key_h: i64, val_h: i64) -> i64 {
    if !min_sem_enabled() {
        return 0;
    }
    let key_s = match arena().get(key_h).and_then(|r| r.obj) {
        Some(Obj::Str(s)) => s.clone(),
        _ => format!("{}", key_h),
    };
    if let Some(mut guard) = arena().get_mut(map_h) {
        if let Some(Obj::Map(m)) = guard.get_mut(&map_h) {
            m.insert(key_s, val_h);
            return m.len() as i64;
        }
    }
    0
}

#[export_name = "nyash.map.has_hh"]
pub extern "C" fn nyash_map_has_hh(map_h: i64, key_h: i64) -> i64 {
    if !min_sem_enabled() {
        return 0;
    }
    let key_s = match arena().get(key_h).and_then(|r| r.obj) {
        Some(Obj::Str(s)) => s.clone(),
        _ => format!("{}", key_h),
    };
    if let Some(ObjRef {
        obj: Some(Obj::Map(m)),
        ..
    }) = arena().get(map_h)
    {
        return if m.contains_key(&key_s) { 1 } else { 0 };
    }
    0
}

#[export_name = "nyash.map.get_hh"]
pub extern "C" fn nyash_map_get_hh(map_h: i64, key_h: i64) -> i64 {
    if !min_sem_enabled() {
        return 0;
    }
    let key_s = match arena().get(key_h).and_then(|r| r.obj) {
        Some(Obj::Str(s)) => s.clone(),
        _ => format!("{}", key_h),
    };
    if let Some(ObjRef {
        obj: Some(Obj::Map(m)),
        ..
    }) = arena().get(map_h)
    {
        if let Some(vh) = m.get(&key_s) {
            return *vh;
        }
    }
    0
}

// nyash.string.concat_si(str_p, int_val) -> i8*
// String pointer + Integer concatenation (returns C string pointer)
#[export_name = "nyash.string.concat_si"]
pub extern "C" fn nyash_string_concat_si(str_p: *const i8, int_val: i64) -> *const i8 {
    if !min_sem_enabled() {
        return ptr::null();
    }
    unsafe {
        if str_p.is_null() {
            return ptr::null();
        }
        let s_rust = match CStr::from_ptr(str_p).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null(),
        };
        let result = format!("{}{}", s_rust, int_val);
        let h = arena().alloc(Obj::Str(result));
        // Get or create CString for this handle
        arena().str_ptr(h)
    }
}

// nyash.console.log(str_p) -> i64
// Print string to stdout (takes C string pointer)
#[export_name = "nyash.console.log"]
pub extern "C" fn nyash_console_log(str_p: *const i8) -> i64 {
    if !min_sem_enabled() {
        return 0;
    }
    unsafe {
        if str_p.is_null() {
            return 0;
        }
        match CStr::from_ptr(str_p).to_str() {
            Ok(s) => {
                println!("{}", s);
                use std::io::Write as _;
                let _ = std::io::stdout().flush();
                0
            }
            Err(_) => 0,
        }
    }
}

// --- Console (handle-based) ---
// nyash.console.log_handle(h) -> i64
#[export_name = "nyash.console.log_handle"]
pub extern "C" fn nyash_console_log_handle(h: i64) -> i64 {
    if min_sem_enabled() {
        if h > 0 {
            if let Some(ObjRef { obj: Some(Obj::Str(s)), .. }) = arena().get(h) {
                println!("{}", s);
            } else {
                println!("{}", h);
            }
        } else {
            println!("null");
        }
    }
    0
}

#[export_name = "nyash.console.warn_handle"]
pub extern "C" fn nyash_console_warn_handle(h: i64) -> i64 {
    if min_sem_enabled() {
        if h > 0 {
            if let Some(ObjRef { obj: Some(Obj::Str(s)), .. }) = arena().get(h) {
                eprintln!("[warn] {}", s);
            } else {
                eprintln!("[warn] {}", h);
            }
        } else {
            eprintln!("[warn] null");
        }
    }
    0
}

#[export_name = "nyash.console.error_handle"]
pub extern "C" fn nyash_console_error_handle(h: i64) -> i64 {
    if min_sem_enabled() {
        if h > 0 {
            if let Some(ObjRef { obj: Some(Obj::Str(s)), .. }) = arena().get(h) {
                eprintln!("[error] {}", s);
            } else {
                eprintln!("[error] {}", h);
            }
        } else {
            eprintln!("[error] null");
        }
    }
    0
}

// nyash.console.readline() -> i8*
#[export_name = "nyash.console.readline"]
pub extern "C" fn nyash_console_readline() -> *const i8 {
    // Minimal stub: return empty string
    let cs = CString::new("").unwrap();
    let p = cs.as_ptr();
    cpool().lock().unwrap().push(cs);
    p
}

// --- Safepoints (no-op stubs) ---
#[export_name = "ny_check_safepoint"]
pub extern "C" fn ny_check_safepoint() { /* no-op */
}

#[export_name = "ny_safepoint"]
pub extern "C" fn ny_safepoint(_live_count: i64, _live_values: *const i64) { /* no-op */
}

// --- Process entry (driver) ---
// Provide a minimal C entry point that calls ny_main() if present.
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() -> i32 {
    unsafe {
        extern "C" {
            fn ny_main() -> i64;
        }
        let code = ny_main();
        let quiet = std::env::var("NYASH_NYRT_SILENT_RESULT").ok().as_deref() == Some("1");
        if !quiet {
            println!("Result: {}", code as i32);
            use std::io::Write as _;
            let _ = std::io::stdout().flush();
        }
        (code as i32) & 0xFF
    }
}
