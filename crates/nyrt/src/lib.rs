// Minimal NyRT static shim library (libnyrt.a)
// Exposes C ABI entry points used by AOT/JIT-emitted objects.

mod encode;
mod plugin;

pub use plugin::*;

// --- AOT ObjectModule dotted-name exports (String/Any helpers) ---
// String.len_h(handle) -> i64
#[export_name = "nyash.string.len_h"]
pub extern "C" fn nyash_string_len_h(handle: i64) -> i64 {
    use nyash_rust::jit::rt::handles;
    if std::env::var("NYASH_JIT_TRACE_LEN").ok().as_deref() == Some("1") {
        let present = if handle > 0 {
            handles::get(handle as u64).is_some()
        } else {
            false
        };
        eprintln!(
            "[AOT-LEN_H] string.len_h handle={} present={}",
            handle, present
        );
    }
    if handle <= 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(sb) = obj
            .as_any()
            .downcast_ref::<nyash_rust::box_trait::StringBox>()
        {
            return sb.value.len() as i64;
        }
    }
    0
}

// String.charCodeAt_h(handle, idx) -> i64 (byte-based; -1 if OOB)
#[export_name = "nyash.string.charCodeAt_h"]
pub extern "C" fn nyash_string_charcode_at_h_export(handle: i64, idx: i64) -> i64 {
    use nyash_rust::jit::rt::handles;
    if idx < 0 {
        return -1;
    }
    if handle <= 0 {
        return -1;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(sb) = obj
            .as_any()
            .downcast_ref::<nyash_rust::box_trait::StringBox>()
        {
            let s = &sb.value;
            let i = idx as usize;
            if i < s.len() {
                return s.as_bytes()[i] as i64;
            }
            return -1;
        }
    }
    -1
}

// String.concat_hh(lhs_h, rhs_h) -> handle
#[export_name = "nyash.string.concat_hh"]
pub extern "C" fn nyash_string_concat_hh_export(a_h: i64, b_h: i64) -> i64 {
    use nyash_rust::{
        box_trait::{NyashBox, StringBox},
        jit::rt::handles,
    };
    let to_s = |h: i64| -> String {
        if h > 0 {
            if let Some(o) = handles::get(h as u64) {
                return o.to_string_box().value;
            }
        }
        String::new()
    };
    let s = format!("{}{}", to_s(a_h), to_s(b_h));
    let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::new(StringBox::new(s));
    handles::to_handle(arc) as i64
}

// String.eq_hh(lhs_h, rhs_h) -> i64 (0/1)
#[export_name = "nyash.string.eq_hh"]
pub extern "C" fn nyash_string_eq_hh_export(a_h: i64, b_h: i64) -> i64 {
    use nyash_rust::jit::rt::handles;
    let to_s = |h: i64| -> String {
        if h > 0 {
            if let Some(o) = handles::get(h as u64) {
                return o.to_string_box().value;
            }
        }
        String::new()
    };
    if to_s(a_h) == to_s(b_h) {
        1
    } else {
        0
    }
}

// box.from_i8_string(ptr) -> handle
// Helper: build a StringBox from i8* and return a handle for AOT marshalling
#[export_name = "nyash.box.from_i8_string"]
pub extern "C" fn nyash_box_from_i8_string(ptr: *const i8) -> i64 {
    use nyash_rust::{
        box_trait::{NyashBox, StringBox},
        jit::rt::handles,
    };
    use std::ffi::CStr;
    if ptr.is_null() {
        return 0;
    }
    let c = unsafe { CStr::from_ptr(ptr) };
    let s = match c.to_str() {
        Ok(v) => v.to_string(),
        Err(_) => return 0,
    };
    let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::new(StringBox::new(s));
    handles::to_handle(arc) as i64
}

// box.from_f64(val) -> handle
// Helper: build a FloatBox and return a handle
#[export_name = "nyash.box.from_f64"]
pub extern "C" fn nyash_box_from_f64(val: f64) -> i64 {
    use nyash_rust::{box_trait::NyashBox, boxes::FloatBox, jit::rt::handles};
    let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::new(FloatBox::new(val));
    handles::to_handle(arc) as i64
}

// env.box.new(type_name: *const i8) -> handle (i64)
// Minimal shim for Core-13 pure AOT: constructs Box via registry by name (no args)
#[export_name = "nyash.env.box.new"]
pub extern "C" fn nyash_env_box_new(type_name: *const i8) -> i64 {
    use nyash_rust::{
        box_trait::NyashBox, jit::rt::handles, runtime::box_registry::get_global_registry,
    };
    use std::ffi::CStr;
    if type_name.is_null() {
        return 0;
    }
    let cstr = unsafe { CStr::from_ptr(type_name) };
    let ty = match cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    // Core-first special cases: construct built-in boxes directly
    if ty == "MapBox" {
        use nyash_rust::boxes::map_box::MapBox;
        let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::new(MapBox::new());
        return handles::to_handle(arc) as i64;
    }
    let reg = get_global_registry();
    match reg.create_box(ty, &[]) {
        Ok(b) => {
            let arc: std::sync::Arc<dyn NyashBox> = b.into();
            handles::to_handle(arc) as i64
        }
        Err(_) => 0,
    }
}

// env.box.new_i64x(type_name: *const i8, argc: i64, a1: i64, a2: i64, a3: i64, a4: i64) -> handle (i64)
// Minimal shim: construct args from handles or wrap i64 as IntegerBox
#[export_name = "nyash.env.box.new_i64x"]
pub extern "C" fn nyash_env_box_new_i64x(
    type_name: *const i8,
    argc: i64,
    a1: i64,
    a2: i64,
    a3: i64,
    a4: i64,
) -> i64 {
    use nyash_rust::{
        box_trait::{IntegerBox, NyashBox},
        jit::rt::handles,
        runtime::box_registry::get_global_registry,
    };
    use std::ffi::CStr;
    if type_name.is_null() {
        return 0;
    }
    let cstr = unsafe { CStr::from_ptr(type_name) };
    let ty = match cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    // Build args vec from provided i64 words
    let mut argv: Vec<Box<dyn NyashBox>> = Vec::new();
    let push_val = |dst: &mut Vec<Box<dyn NyashBox>>, v: i64| {
        if v > 0 {
            if let Some(obj) = handles::get(v as u64) {
                dst.push(obj.share_box());
                return;
            }
        }
        dst.push(Box::new(IntegerBox::new(v)));
    };
    if argc >= 1 {
        push_val(&mut argv, a1);
    }
    if argc >= 2 {
        push_val(&mut argv, a2);
    }
    if argc >= 3 {
        push_val(&mut argv, a3);
    }
    if argc >= 4 {
        push_val(&mut argv, a4);
    }

    let reg = get_global_registry();
    match reg.create_box(ty, &argv) {
        Ok(b) => {
            let arc: std::sync::Arc<dyn NyashBox> = b.into();
            handles::to_handle(arc) as i64
        }
        Err(_) => 0,
    }
}

// String.lt_hh(lhs_h, rhs_h) -> i64 (0/1)
#[export_name = "nyash.string.lt_hh"]
pub extern "C" fn nyash_string_lt_hh_export(a_h: i64, b_h: i64) -> i64 {
    use nyash_rust::jit::rt::handles;
    let to_s = |h: i64| -> String {
        if h > 0 {
            if let Some(o) = handles::get(h as u64) {
                return o.to_string_box().value;
            }
        }
        String::new()
    };
    if to_s(a_h) < to_s(b_h) {
        1
    } else {
        0
    }
}

// Any.length_h(handle) -> i64 (Array/String/Map)
#[export_name = "nyash.any.length_h"]
pub extern "C" fn nyash_any_length_h_export(handle: i64) -> i64 {
    use nyash_rust::jit::rt::handles;
    if std::env::var("NYASH_JIT_TRACE_LEN").ok().as_deref() == Some("1") {
        let present = if handle > 0 {
            handles::get(handle as u64).is_some()
        } else {
            false
        };
        eprintln!(
            "[AOT-LEN_H] any.length_h handle={} present={}",
            handle, present
        );
    }
    if handle <= 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(arr) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::array::ArrayBox>()
        {
            if let Some(ib) = arr
                .length()
                .as_any()
                .downcast_ref::<nyash_rust::box_trait::IntegerBox>()
            {
                return ib.value;
            }
        }
        if let Some(sb) = obj
            .as_any()
            .downcast_ref::<nyash_rust::box_trait::StringBox>()
        {
            return sb.value.len() as i64;
        }
        if let Some(map) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::map_box::MapBox>()
        {
            if let Some(ib) = map
                .size()
                .as_any()
                .downcast_ref::<nyash_rust::box_trait::IntegerBox>()
            {
                return ib.value;
            }
        }
    }
    0
}

// Any.is_empty_h(handle) -> i64 (0/1)
#[export_name = "nyash.any.is_empty_h"]
pub extern "C" fn nyash_any_is_empty_h_export(handle: i64) -> i64 {
    use nyash_rust::jit::rt::handles;
    if handle <= 0 {
        return 1;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(arr) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::array::ArrayBox>()
        {
            if let Ok(items) = arr.items.read() {
                return if items.is_empty() { 1 } else { 0 };
            }
        }
        if let Some(sb) = obj
            .as_any()
            .downcast_ref::<nyash_rust::box_trait::StringBox>()
        {
            return if sb.value.is_empty() { 1 } else { 0 };
        }
        if let Some(map) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::map_box::MapBox>()
        {
            if let Some(ib) = map
                .size()
                .as_any()
                .downcast_ref::<nyash_rust::box_trait::IntegerBox>()
            {
                return if ib.value == 0 { 1 } else { 0 };
            }
        }
    }
    1
}

// Instance birth by name (packed u64x2 + len) -> handle
// export: nyash.instance.birth_name_u64x2(lo, hi, len) -> i64
#[export_name = "nyash.instance.birth_name_u64x2"]
pub extern "C" fn nyash_instance_birth_name_u64x2_export(lo: i64, hi: i64, len: i64) -> i64 {
    use nyash_rust::runtime::get_global_plugin_host;
    let mut bytes = Vec::with_capacity(len.max(0) as usize);
    let lo_u = lo as u64;
    let hi_u = hi as u64;
    let l = len.max(0) as usize;
    let take = core::cmp::min(16, l);
    for i in 0..take.min(8) {
        bytes.push(((lo_u >> (8 * i)) & 0xff) as u8);
    }
    for i in 0..take.saturating_sub(8) {
        bytes.push(((hi_u >> (8 * i)) & 0xff) as u8);
    }
    // If len > 16, remaining bytes are not represented in (lo,hi); assume names <=16 bytes for now.
    if bytes.len() != l {
        bytes.resize(l, 0);
    }
    let name = String::from_utf8_lossy(&bytes).to_string();
    if let Ok(host_g) = get_global_plugin_host().read() {
        if let Ok(b) = host_g.create_box(&name, &[]) {
            let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> = std::sync::Arc::from(b);
            let h = nyash_rust::jit::rt::handles::to_handle(arc);
            return h as i64;
        }
    }
    0
}

// Construct StringBox from two u64 words (little-endian) + length (<=16) and return handle
// export: nyash.string.from_u64x2(lo, hi, len) -> i64
#[export_name = "nyash.string.from_u64x2"]
pub extern "C" fn nyash_string_from_u64x2_export(lo: i64, hi: i64, len: i64) -> i64 {
    use nyash_rust::{
        box_trait::{NyashBox, StringBox},
        jit::rt::handles,
    };
    let l = if len < 0 {
        0
    } else {
        core::cmp::min(len as usize, 16)
    };
    let mut bytes: Vec<u8> = Vec::with_capacity(l);
    let lo_u = lo as u64;
    let hi_u = hi as u64;
    for i in 0..l.min(8) {
        bytes.push(((lo_u >> (8 * i)) & 0xff) as u8);
    }
    for i in 0..l.saturating_sub(8) {
        bytes.push(((hi_u >> (8 * i)) & 0xff) as u8);
    }
    let s = String::from_utf8_lossy(&bytes).to_string();
    let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::new(StringBox::new(s));
    handles::to_handle(arc) as i64
}

// Convert a VM argument (param index or existing handle) into a runtime handle
// Exported as: nyash.handle.of
#[export_name = "nyash.handle.of"]
pub extern "C" fn nyash_handle_of_export(v: i64) -> i64 {
    use nyash_rust::box_trait::NyashBox;
    use nyash_rust::jit::rt::{handles, with_legacy_vm_args};
    // If already a positive handle, pass through
    if v > 0 {
        return v;
    }
    // Otherwise treat as legacy param index and box-ref → handleize
    if v >= 0 {
        let idx = v as usize;
        let mut out: i64 = 0;
        with_legacy_vm_args(|args| {
            if let Some(nyash_rust::backend::vm::VMValue::BoxRef(b)) = args.get(idx) {
                // If it's a PluginBoxV2 or any NyashBox, register into handle registry
                // Note: store as NyashBox for uniform access
                let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::from(b.clone());
                out = handles::to_handle(arc) as i64;
            } else if let Some(nyash_rust::backend::vm::VMValue::BoxRef(b)) = args.get(idx) {
                let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::from(b.clone());
                out = handles::to_handle(arc) as i64;
            }
        });
        return out;
    }
    0
}

// ---- Reserved runtime/GC externs for AOT linking ----
// Exported as: nyash.rt.checkpoint
#[export_name = "nyash.rt.checkpoint"]
pub extern "C" fn nyash_rt_checkpoint_export() -> i64 {
    if std::env::var("NYASH_RUNTIME_CHECKPOINT_TRACE")
        .ok()
        .as_deref()
        == Some("1")
    {
        eprintln!("[nyrt] nyash.rt.checkpoint reached");
    }
    0
}

// Exported as: nyash.gc.barrier_write
#[export_name = "nyash.gc.barrier_write"]
pub extern "C" fn nyash_gc_barrier_write_export(handle_or_ptr: i64) -> i64 {
    let _ = handle_or_ptr;
    if std::env::var("NYASH_GC_BARRIER_TRACE").ok().as_deref() == Some("1") {
        eprintln!("[nyrt] nyash.gc.barrier_write h=0x{:x}", handle_or_ptr);
    }
    0
}

#[export_name = "nyash.string.birth_h"]
pub extern "C" fn nyash_string_birth_h_export() -> i64 {
    // Create a new StringBox via unified plugin host; return runtime handle as i64
    if let Ok(host_g) = nyash_rust::runtime::get_global_plugin_host().read() {
        if let Ok(b) = host_g.create_box("StringBox", &[]) {
            let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> = std::sync::Arc::from(b);
            let h = nyash_rust::jit::rt::handles::to_handle(arc);
            return h as i64;
        }
    }
    0
}

#[export_name = "nyash.integer.birth_h"]
pub extern "C" fn nyash_integer_birth_h_export() -> i64 {
    if let Ok(host_g) = nyash_rust::runtime::get_global_plugin_host().read() {
        if let Ok(b) = host_g.create_box("IntegerBox", &[]) {
            let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> = std::sync::Arc::from(b);
            let h = nyash_rust::jit::rt::handles::to_handle(arc);
            return h as i64;
        }
    }
    0
}
// ConsoleBox birth shim for AOT/JIT handle-based creation
#[export_name = "nyash.console.birth_h"]
pub extern "C" fn nyash_console_birth_h_export() -> i64 {
    if let Ok(host_g) = nyash_rust::runtime::get_global_plugin_host().read() {
        if let Ok(b) = host_g.create_box("ConsoleBox", &[]) {
            let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> = std::sync::Arc::from(b);
            let h = nyash_rust::jit::rt::handles::to_handle(arc);
            return h as i64;
        }
    }
    0
}

// ---- Process entry (driver) ----
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() -> i32 {
    // Initialize plugin host: prefer nyash.toml next to the executable; fallback to CWD
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));

    // Windows: assist DLL/plugin discovery by extending PATH and normalizing PYTHONHOME
    #[cfg(target_os = "windows")]
    if let Some(dir) = &exe_dir {
        use std::path::PathBuf;
        // Extend PATH with exe_dir and exe_dir\plugins if not already present
        let mut path_val = std::env::var("PATH").unwrap_or_default();
        let add_path = |pv: &mut String, p: &PathBuf| {
            let ps = p.display().to_string();
            if !pv.split(';').any(|seg| seg.eq_ignore_ascii_case(&ps)) {
                if !pv.is_empty() {
                    pv.push(';');
                }
                pv.push_str(&ps);
            }
        };
        add_path(&mut path_val, dir);
        let plug = dir.join("plugins");
        if plug.is_dir() {
            add_path(&mut path_val, &plug);
        }
        std::env::set_var("PATH", &path_val);

        // Normalize PYTHONHOME: if unset, point to exe_dir\python when present.
        match std::env::var("PYTHONHOME") {
            Ok(v) => {
                // If relative, make absolute under exe_dir
                let pb = PathBuf::from(&v);
                if pb.is_relative() {
                    let abs = dir.join(pb);
                    std::env::set_var("PYTHONHOME", abs.display().to_string());
                }
            }
            Err(_) => {
                let cand = dir.join("python");
                if cand.is_dir() {
                    std::env::set_var("PYTHONHOME", cand.display().to_string());
                }
            }
        }
    }
    let mut inited = false;
    if let Some(dir) = &exe_dir {
        let candidate = dir.join("nyash.toml");
        if candidate.exists() {
            let _ =
                nyash_rust::runtime::init_global_plugin_host(candidate.to_string_lossy().as_ref());
            inited = true;
        }
    }
    if !inited {
        let _ = nyash_rust::runtime::init_global_plugin_host("nyash.toml");
    }
    // Optional verbosity
    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
        println!(
            "🔌 nyrt: plugin host init attempted (exe_dir={}, cwd={})",
            exe_dir
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "?".into()),
            std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "?".into())
        );
    }
    // Call exported Nyash entry if linked: `ny_main` (i64 -> return code normalized)
    unsafe {
        extern "C" {
            fn ny_main() -> i64;
        }
        // SAFETY: if not linked, calling will be an unresolved symbol at link-time; we rely on link step to include ny_main.
        let v = ny_main();
        // Print standardized result line for golden comparisons
        println!("Result: {}", v);
        v as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nyash_rust::{
        box_trait::{NyashBox, StringBox},
        jit::rt::handles,
        runtime::plugin_loader_v2::make_plugin_box_v2,
    };
    use std::sync::Arc;

    unsafe extern "C" fn fake_i32(
        _t: u32,
        _m: u32,
        _i: u32,
        _a: *const u8,
        _al: usize,
        res: *mut u8,
        len: *mut usize,
    ) -> i32 {
        let mut buf = Vec::new();
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.push(2);
        buf.push(0);
        buf.extend_from_slice(&4u16.to_le_bytes());
        buf.extend_from_slice(&123i32.to_le_bytes());
        if res.is_null() || len.is_null() || unsafe { *len } < buf.len() {
            unsafe {
                if !len.is_null() {
                    *len = buf.len();
                }
            }
            return -1;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(buf.as_ptr(), res, buf.len());
            *len = buf.len();
        }
        0
    }

    unsafe extern "C" fn fake_str(
        _t: u32,
        _m: u32,
        _i: u32,
        _a: *const u8,
        _al: usize,
        res: *mut u8,
        len: *mut usize,
    ) -> i32 {
        let s = b"hi";
        let mut buf = Vec::new();
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.push(7);
        buf.push(0);
        buf.extend_from_slice(&(s.len() as u16).to_le_bytes());
        buf.extend_from_slice(s);
        if res.is_null() || len.is_null() || unsafe { *len } < buf.len() {
            unsafe {
                if !len.is_null() {
                    *len = buf.len();
                }
            }
            return -1;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(buf.as_ptr(), res, buf.len());
            *len = buf.len();
        }
        0
    }

    #[test]
    fn decode_i32_and_string_returns() {
        let pb = make_plugin_box_v2("Dummy".into(), 1, 1, fake_i32);
        let arc: Arc<dyn NyashBox> = Arc::new(pb);
        let handle = handles::to_handle(arc) as i64;
        let val = nyash_plugin_invoke3_tagged_i64(1, 0, 0, handle, 0, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!(val, 123);

        let pb = make_plugin_box_v2("Dummy".into(), 1, 2, fake_str);
        let arc: Arc<dyn NyashBox> = Arc::new(pb);
        let handle = handles::to_handle(arc) as i64;
        let h = nyash_plugin_invoke3_tagged_i64(1, 0, 0, handle, 0, 0, 0, 0, 0, 0, 0, 0);
        assert!(h > 0);
        let obj = handles::get(h as u64).unwrap();
        let sb = obj.as_any().downcast_ref::<StringBox>().unwrap();
        assert_eq!(sb.value, "hi");
    }
}
