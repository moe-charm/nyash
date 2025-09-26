//! Test program for v2 plugin loader (Phase 12 prep)

use nyash_rust::config::NyashConfigV2;
use nyash_rust::runtime::{get_global_loader_v2, init_global_loader_v2};

fn main() {
    env_logger::init();

    println!("=== v2 Plugin Loader Test (Phase 12 prep) ===\n");

    // Load configuration
    let config_path = "test_nyash_v2.toml";
    println!("Loading configuration from: {}", config_path);

    let config = match NyashConfigV2::from_file(config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            return;
        }
    };

    println!("Configuration loaded successfully!");
    println!("Is v2 format: {}", config.is_v2_format());

    if let Some(libs) = &config.plugins.libraries {
        println!("\nLibraries found:");
        for (name, lib) in libs {
            println!("  {} -> {}", name, lib.plugin_path);
            println!("    Provides: {:?}", lib.provides);
        }
    }

    // Initialize and load plugins
    println!("\nLoading plugins...");
    if let Err(e) = init_global_loader_v2(config_path) {
        eprintln!("Failed to init loader: {:?}", e);
        return;
    }
    let loader = get_global_loader_v2();
    let loader = loader.read().unwrap();

    // Test box type resolution
    println!("\nTesting box type resolution:");
    for box_type in ["StringBox", "FileBox", "MapBox"] {
        match config.find_library_for_box(box_type) {
            Some((name, lib)) => {
                println!("  {} -> library: {} (path={})", box_type, name, lib.path)
            }
            None => println!("  {} -> not found in config", box_type),
        }
    }

    // Optional: try creating a simple box via loader API (if present)
    if let Ok(bx) = loader.create_box("StringBox", &[]) {
        println!("Created box: {}", bx.to_string_box().value);
    } else {
        println!("create_box(StringBox) not available or failed (ok for stub)");
    }

    // Simple reverse host-call exercise (simulate plugin calling host via C ABI by-slot)
    println!("\nReverse host-call (by-slot) quick test:");
    // Create ArrayBox and obtain HostHandle
    let mut arr = nyash_rust::boxes::ArrayBox::new();
    arr.push(Box::new(nyash_rust::box_trait::StringBox::new("init"))
        as Box<dyn nyash_rust::box_trait::NyashBox>);
    let handle = nyash_rust::runtime::host_handles::to_handle_box(Box::new(arr));
    // Call Array.set(0, "hello") via slot=101
    let mut tlv = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(2);
    nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut tlv, 0);
    nyash_rust::runtime::plugin_ffi_common::encode::string(&mut tlv, "hello");
    let mut out = vec![0u8; 256];
    let mut out_len: usize = out.len();
    let code = unsafe {
        nyash_rust::runtime::host_api::nyrt_host_call_slot(
            handle,
            101,
            tlv.as_ptr(),
            tlv.len(),
            out.as_mut_ptr(),
            &mut out_len,
        )
    };
    println!("  set(slot=101) -> code={}, out_len={}", code, out_len);
    // Call Array.get(0) via slot=100 and decode
    let mut tlv2 = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(1);
    nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut tlv2, 0);
    let mut out2 = vec![0u8; 256];
    let mut out2_len: usize = out2.len();
    let code2 = unsafe {
        nyash_rust::runtime::host_api::nyrt_host_call_slot(
            handle,
            100,
            tlv2.as_ptr(),
            tlv2.len(),
            out2.as_mut_ptr(),
            &mut out2_len,
        )
    };
    if code2 == 0 {
        if let Some((tag, _sz, payload)) =
            nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(&out2[..out2_len])
        {
            if tag == 6 || tag == 7 {
                // string/bytes
                let s = nyash_rust::runtime::plugin_ffi_common::decode::string(payload);
                println!("  get(slot=100) -> tag={}, value='{}'", tag, s);
            } else if tag == 3 {
                // i64
                let v = nyash_rust::runtime::plugin_ffi_common::decode::i32(payload)
                    .unwrap_or_default();
                println!("  get(slot=100) -> tag={}, i32={}", tag, v);
            } else {
                println!("  get(slot=100) -> tag={}, size={}", tag, _sz);
            }
        }
    } else {
        println!("  get(slot=100) failed with code {}", code2);
    }

    // MapBox slots test: set/get/has/size
    println!("\nReverse host-call (by-slot) MapBox test:");
    let map = nyash_rust::boxes::map_box::MapBox::new();
    let map_h = nyash_rust::runtime::host_handles::to_handle_box(Box::new(map));
    // set("k","v") → slot=204
    let mut tlv_set = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(2);
    nyash_rust::runtime::plugin_ffi_common::encode::string(&mut tlv_set, "k");
    nyash_rust::runtime::plugin_ffi_common::encode::string(&mut tlv_set, "v");
    let mut out_s = vec![0u8; 256];
    let mut out_s_len: usize = out_s.len();
    let code_s = unsafe {
        nyash_rust::runtime::host_api::nyrt_host_call_slot(
            map_h,
            204,
            tlv_set.as_ptr(),
            tlv_set.len(),
            out_s.as_mut_ptr(),
            &mut out_s_len,
        )
    };
    println!("  set(slot=204) -> code={}, out_len={}", code_s, out_s_len);
    // get("k") → slot=203
    let mut tlv_get = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(1);
    nyash_rust::runtime::plugin_ffi_common::encode::string(&mut tlv_get, "k");
    let mut out_g = vec![0u8; 256];
    let mut out_g_len: usize = out_g.len();
    let code_g = unsafe {
        nyash_rust::runtime::host_api::nyrt_host_call_slot(
            map_h,
            203,
            tlv_get.as_ptr(),
            tlv_get.len(),
            out_g.as_mut_ptr(),
            &mut out_g_len,
        )
    };
    if code_g == 0 {
        if let Some((tag, _sz, payload)) =
            nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(&out_g[..out_g_len])
        {
            if tag == 6 || tag == 7 {
                let s = nyash_rust::runtime::plugin_ffi_common::decode::string(payload);
                println!("  get(slot=203) -> '{}'", s);
            } else {
                println!("  get(slot=203) -> tag={}, size={}", tag, _sz);
            }
        }
    }
    // has("k") → slot=202
    let mut out_hb = vec![0u8; 16];
    let mut out_hb_len: usize = out_hb.len();
    let code_hb = unsafe {
        nyash_rust::runtime::host_api::nyrt_host_call_slot(
            map_h,
            202,
            tlv_get.as_ptr(),
            tlv_get.len(),
            out_hb.as_mut_ptr(),
            &mut out_hb_len,
        )
    };
    println!(
        "  has(slot=202) -> code={}, out_len={}",
        code_hb, out_hb_len
    );
    // size() → slot=200
    let mut out_sz = vec![0u8; 32];
    let mut out_sz_len: usize = out_sz.len();
    let code_sz = unsafe {
        nyash_rust::runtime::host_api::nyrt_host_call_slot(
            map_h,
            200,
            std::ptr::null(),
            0,
            out_sz.as_mut_ptr(),
            &mut out_sz_len,
        )
    };
    println!(
        "  size(slot=200) -> code={}, out_len={}",
        code_sz, out_sz_len
    );

    println!("\nTest completed!");
}
