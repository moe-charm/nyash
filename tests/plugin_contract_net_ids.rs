#![cfg(all(feature = "plugins", not(target_arch = "wasm32")))]

use nyash_rust::parser::NyashParser;
use nyash_rust::runtime::plugin_loader_v2::{init_global_loader_v2, get_global_loader_v2};
use nyash_rust::runtime::box_registry::get_global_registry;
use nyash_rust::runtime::PluginConfig;

fn try_init_plugins() -> bool {
    if !std::path::Path::new("nyash.toml").exists() { return false; }
    if let Err(e) = init_global_loader_v2("nyash.toml") { eprintln!("init failed: {:?}", e); return false; }
    let loader = get_global_loader_v2();
    let loader = loader.read().unwrap();
    if let Some(conf) = &loader.config {
        let mut map = std::collections::HashMap::new();
        for (lib, def) in &conf.libraries { for b in &def.boxes { map.insert(b.clone(), lib.clone()); } }
        get_global_registry().apply_plugin_config(&PluginConfig { plugins: map });
        true
    } else { false }
}

/// Minimal ABI sanity check: HttpRequestBox.path=1, readBody=2
#[test]
fn plugin_contract_http_request_ids_sanity() {
    if !try_init_plugins() { return; }
    // Exercise HttpRequestBox.path/readBody on a birthed request (no server needed)
    let code = r#"
local req, p, b
req = new HttpRequestBox()
p = req.path()
b = req.readBody()
p + ":" + b
"#;
    let ast = NyashParser::parse_from_string(code).expect("parse failed");
    let mut interpreter = nyash_rust::interpreter::NyashInterpreter::new();
    let result = interpreter.execute(ast).expect("exec failed");
    // Default path="", body="" for birthed request
    assert_eq!(result.to_string_box().value, ":");
}

