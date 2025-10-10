//! Test program for v2 plugin loader (Phase 12 prep)

use nyash_rust::config::NyashConfigV2;
use nyash_rust::runtime::{get_global_loader_v2, init_global_loader_v2};
use nyash_rust::runtime::host_api_box;

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
    let code = host_api_box::call_slot_grow(handle, 101, &tlv);
    match code {
        Ok(buf) => println!("  set(slot=101) -> code=0, out_len={}", buf.len()),
        Err(rc) => println!("  set(slot=101) -> code={}, out_len=0", rc),
    }
    // Call Array.get(0) via slot=100 and decode
    let mut tlv2 = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(1);
    nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut tlv2, 0);
    if let Ok(out2) = host_api_box::call_slot_grow(handle, 100, &tlv2) {
        if let Some((tag, _sz, payload)) =
            nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(&out2)
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
        println!("  get(slot=100) failed (host_api_box rc)");
    }

    // MapBox slots test: set/get/has/size
    println!("\nReverse host-call (by-slot) MapBox test:");
    let map = nyash_rust::boxes::map_box::MapBox::new();
    let map_h = nyash_rust::runtime::host_handles::to_handle_box(Box::new(map));
    // set("k","v") → slot=204
    let mut tlv_set = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(2);
    nyash_rust::runtime::plugin_ffi_common::encode::string(&mut tlv_set, "k");
    nyash_rust::runtime::plugin_ffi_common::encode::string(&mut tlv_set, "v");
    let code_s = host_api_box::call_slot_grow(map_h, 204, &tlv_set);
    match code_s {
        Ok(buf) => println!("  set(slot=204) -> code=0, out_len={}", buf.len()),
        Err(rc) => println!("  set(slot=204) -> code={}, out_len=0", rc),
    }
    // get("k") → slot=203
    let mut tlv_get = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(1);
    nyash_rust::runtime::plugin_ffi_common::encode::string(&mut tlv_get, "k");
    if let Ok(out_g) = host_api_box::call_slot_grow(map_h, 203, &tlv_get) {
        if let Some((tag, _sz, payload)) =
            nyash_rust::runtime::plugin_ffi_common::decode::tlv_first(&out_g)
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
    let code_hb = host_api_box::call_slot_grow(map_h, 202, &tlv_get);
    match code_hb {
        Ok(buf) => println!("  has(slot=202) -> code=0, out_len={}", buf.len()),
        Err(rc) => println!("  has(slot=202) -> code={}, out_len=0", rc),
    }
    // size() → slot=200
    let code_sz = host_api_box::call_slot_grow(map_h, 200, &[]);
    match code_sz {
        Ok(buf) => println!("  size(slot=200) -> code=0, out_len={}", buf.len()),
        Err(rc) => println!("  size(slot=200) -> code={}, out_len=0", rc),
    }

    println!("\nTest completed!");
}
