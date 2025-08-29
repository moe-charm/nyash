// Minimal NyRT static shim library (libnyrt.a)
// Exposes C ABI entry points used by AOT/JIT-emitted objects.

#[no_mangle]
pub extern "C" fn nyash_plugin_invoke3_i64(
    type_id: i64,
    method_id: i64,
    argc: i64,
    a0: i64,
    a1: i64,
    a2: i64,
) -> i64 {
    use nyash_rust::runtime::plugin_loader_v2::PluginBoxV2;
    // Resolve receiver instance from legacy VM args (param index)
    let mut instance_id: u32 = 0;
    let mut invoke: Option<unsafe extern "C" fn(u32,u32,u32,*const u8,usize,*mut u8,*mut usize)->i32> = None;
    if a0 >= 0 {
        nyash_rust::jit::rt::with_legacy_vm_args(|args| {
            let idx = a0 as usize;
            if let Some(nyash_rust::backend::vm::VMValue::BoxRef(b)) = args.get(idx) {
                if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                    instance_id = p.instance_id();
                    invoke = Some(p.inner.invoke_fn);
                }
            }
        });
    }
    if invoke.is_none() { return 0; }
    // Build TLV args from a1/a2 if present
    let mut buf = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header((argc.saturating_sub(1).max(0) as u16));
    let mut add_i64 = |v: i64| { nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut buf, v); };
    if argc >= 2 { add_i64(a1); }
    if argc >= 3 { add_i64(a2); }
    // Prepare output buffer
    let mut out: [u8; 32] = [0; 32];
    let mut out_len: usize = out.len();
    let rc = unsafe { invoke.unwrap()(type_id as u32, method_id as u32, instance_id, buf.as_ptr(), buf.len(), out.as_mut_ptr(), &mut out_len) };
    if rc != 0 { return 0; }
    if let Some((tag, _sz, payload)) = nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len]) {
        match tag {
            3 => { // I64
                if let Some(v) = nyash_rust::runtime::plugin_ffi_common::decode::i32(payload) { return v as i64; }
                if payload.len() == 8 { let mut b=[0u8;8]; b.copy_from_slice(payload); return i64::from_le_bytes(b); }
            }
            1 => { // Bool
                return if nyash_rust::runtime::plugin_ffi_common::decode::bool(payload).unwrap_or(false) { 1 } else { 0 };
            }
            _ => {}
        }
    }
    0
}

// ---- Handle-based birth shims for AOT/JIT object linkage ----
// These resolve symbols like "nyash.string.birth_h" referenced by ObjectModule.

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

// ---- Process entry (driver) ----
#[no_mangle]
pub extern "C" fn main() -> i32 {
    // Initialize plugin host: prefer nyash.toml next to the executable; fallback to CWD
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()));
    let mut inited = false;
    if let Some(dir) = &exe_dir {
        let candidate = dir.join("nyash.toml");
        if candidate.exists() {
            let _ = nyash_rust::runtime::init_global_plugin_host(candidate.to_string_lossy().as_ref());
            inited = true;
        }
    }
    if !inited {
        let _ = nyash_rust::runtime::init_global_plugin_host("nyash.toml");
    }
    // Optional verbosity
    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
        println!("🔌 nyrt: plugin host init attempted (exe_dir={}, cwd={})",
            exe_dir.as_ref().map(|p| p.display().to_string()).unwrap_or_else(||"?".into()),
            std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_else(|_|"?".into())
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
