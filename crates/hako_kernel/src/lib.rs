// Hako Kernel — minimal static runtime shim for AOT linking
// Goal: provide tiny C-ABI stubs for symbols that llvmlite harness declares
// during codegen (e.g., nyash.box.from_i8_string, nyash.string.concat_hh, etc.).
// These stubs allow ny-llvmc to link an executable even when full NyKernel is
// not available yet. Semantics are intentionally minimal: functions return
// neutral defaults and do not allocate runtime-managed objects.

#[no_mangle]
pub extern "C" fn _hako_kernel_init_marker() {}

// --- String helpers (handles are opaque i64: 0 means null/none) ---

// nyash.box.from_i8_string(ptr) -> i64 (handle)
// Minimal: ignore pointer, return 0 (no handle). Suitable for tests that don't
// inspect the string contents at runtime.
#[export_name = "nyash.box.from_i8_string"]
pub extern "C" fn nyash_box_from_i8_string(_ptr: *const i8) -> i64 {
    0
}

// nyash.string.to_i8p_h(handle) -> i8* (debug/bridge)
// Minimal: return null pointer.
#[export_name = "nyash.string.to_i8p_h"]
pub extern "C" fn nyash_string_to_i8p_h(_h: i64) -> *const i8 {
    core::ptr::null()
}

// nyash.string.from_u64x2(lo_ptr, hi_unused, len) -> i64 (handle)
// Minimal: return 0 (unused by integer-only programs).
#[export_name = "nyash.string.from_u64x2"]
pub extern "C" fn nyash_string_from_u64x2(_lo: u64, _hi: u64, _len: u64) -> i64 {
    0
}

// nyash.string.birth_h() -> i64 (handle)
// Minimal: return 0 (no allocation).
#[export_name = "nyash.string.birth_h"]
pub extern "C" fn nyash_string_birth_h() -> i64 {
    0
}

// nyash.string.len_h(handle) -> i64
#[export_name = "nyash.string.len_h"]
pub extern "C" fn nyash_string_len_h(_h: i64) -> i64 {
    0
}

// nyash.string.charCodeAt_h(handle, idx) -> i64
#[export_name = "nyash.string.charCodeAt_h"]
pub extern "C" fn nyash_string_charcodeat_h(_h: i64, _idx: i64) -> i64 {
    -1
}

// nyash.string.concat_hh(lhs_h, rhs_h) -> i64 (handle)
#[export_name = "nyash.string.concat_hh"]
pub extern "C" fn nyash_string_concat_hh(_a: i64, _b: i64) -> i64 {
    0
}

// nyash.string.eq_hh(lhs_h, rhs_h) -> i64 (0/1)
#[export_name = "nyash.string.eq_hh"]
pub extern "C" fn nyash_string_eq_hh(_a: i64, _b: i64) -> i64 {
    0
}

// nyash.string.substring_hii(h, start, end) -> i64 (handle)
#[export_name = "nyash.string.substring_hii"]
pub extern "C" fn nyash_string_substring_hii(_h: i64, _s: i64, _e: i64) -> i64 {
    0
}

// nyash.string.lastIndexOf_hh(h, needle_h) -> i64
#[export_name = "nyash.string.lastIndexOf_hh"]
pub extern "C" fn nyash_string_lastindexof_hh(_h: i64, _n: i64) -> i64 {
    -1
}

// --- Any helpers ---

// nyash.any.length_h(handle) -> i64
#[export_name = "nyash.any.length_h"]
pub extern "C" fn nyash_any_length_h(_h: i64) -> i64 {
    0
}

// --- Boxing helpers (numeric) ---

// nyash.box.from_i64(v) -> i64 (handle)
#[export_name = "nyash.box.from_i64"]
pub extern "C" fn nyash_box_from_i64(_v: i64) -> i64 {
    0
}

// nyash.box.from_f64(v) -> i64 (handle)
#[export_name = "nyash.box.from_f64"]
pub extern "C" fn nyash_box_from_f64(_v: f64) -> i64 {
    0
}

// --- Collections minimal bridge (Array/Map birth & size only) ---

// birth (no-arg constructors) return 0 to indicate null (unused in integer-only smokes)
#[export_name = "nyash.array.birth_h"]
pub extern "C" fn nyash_array_birth_h() -> i64 {
    0
}

#[export_name = "nyash.map.birth_h"]
pub extern "C" fn nyash_map_birth_h() -> i64 {
    0
}

// Optional env API used by some builder paths (new box via type string)
#[export_name = "nyash.env.box.new_i64x"]
pub extern "C" fn nyash_env_box_new_i64x(_typ: *const i8, _argc: i64, _a1: i64, _a2: i64, _a3: i64, _a4: i64) -> i64 {
    0
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
        // Do not print to keep AOT parity scripts simple; just propagate exit code.
        code as i32
    }
}
