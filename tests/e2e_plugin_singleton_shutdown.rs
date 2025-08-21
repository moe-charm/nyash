#![cfg(all(feature = "plugins", not(target_arch = "wasm32")))]

use nyash_rust::parser::NyashParser;
use nyash_rust::runtime::plugin_loader_v2::{init_global_loader_v2, get_global_loader_v2, shutdown_plugins_v2};
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

#[test]
fn e2e_singleton_shutdown_and_recreate() {
    if !try_init_plugins() { return; }

    // Use CounterBox singleton and bump to 1
    let code1 = r#"
local a
a = new CounterBox()
a.inc()
"#;
    let ast1 = NyashParser::parse_from_string(code1).expect("parse1");
    let mut interpreter = nyash_rust::interpreter::NyashInterpreter::new();
    interpreter.execute(ast1).expect("exec1");

    // Shutdown plugins (finalize singleton)
    shutdown_plugins_v2().expect("shutdown ok");

    // Re-init plugins and ensure singleton is recreated (count resets to 0)
    assert!(try_init_plugins());
    let code2 = r#"
local b, v
b = new CounterBox()
v = b.get()
v
"#;
    let ast2 = NyashParser::parse_from_string(code2).expect("parse2");
    let mut interpreter2 = nyash_rust::interpreter::NyashInterpreter::new();
    let result = interpreter2.execute(ast2).expect("exec2");
    assert_eq!(result.to_string_box().value, "0");
}

