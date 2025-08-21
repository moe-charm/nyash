#![cfg(all(feature = "plugins", not(target_arch = "wasm32")))]

use nyash_rust::parser::NyashParser;
use nyash_rust::runtime::plugin_loader_v2::{init_global_loader_v2, get_global_loader_v2};
use nyash_rust::runtime::box_registry::get_global_registry;
use nyash_rust::runtime::PluginConfig;

fn try_init_plugins() -> bool {
    if !std::path::Path::new("nyash.toml").exists() { return false; }
    if let Err(e) = init_global_loader_v2("nyash.toml") {
        eprintln!("[e2e] init_global_loader_v2 failed: {:?}", e);
        return false;
    }
    // Map all configured boxes to plugin providers for legacy registry
    let loader = get_global_loader_v2();
    let loader = loader.read().unwrap();
    if let Some(conf) = &loader.config {
        let mut map = std::collections::HashMap::new();
        for (lib_name, lib_def) in &conf.libraries {
            for b in &lib_def.boxes { map.insert(b.clone(), lib_name.clone()); }
        }
        let reg = get_global_registry();
        reg.apply_plugin_config(&PluginConfig { plugins: map });
        true
    } else { false }
}

#[test]
fn e2e_counter_basic_inc_get() {
    if !try_init_plugins() { return; }

    let code = r#"
local c, v1, v2
c = new CounterBox()
v1 = c.get()
c.inc()
v2 = c.get()
v2
"#;
    let ast = NyashParser::parse_from_string(code).expect("parse failed");
    let mut interpreter = nyash_rust::interpreter::NyashInterpreter::new();

    match interpreter.execute(ast) {
        Ok(result) => {
            // After one inc(), count should be 1
            assert_eq!(result.to_string_box().value, "1");
        }
        Err(e) => panic!("Counter basic test failed: {:?}", e),
    }
}

#[test]
fn e2e_counter_assignment_shares_handle() {
    if !try_init_plugins() { return; }

    let code = r#"
local c, x, v
c = new CounterBox()
x = c
x.inc()
v = c.get()
v
"#;
    let ast = NyashParser::parse_from_string(code).expect("parse failed");
    let mut interpreter = nyash_rust::interpreter::NyashInterpreter::new();

    match interpreter.execute(ast) {
        Ok(result) => {
            // New semantics: plugin handle assign shares, so c reflects x.inc()
            assert_eq!(result.to_string_box().value, "1");
        }
        Err(e) => panic!("Counter assignment test failed: {:?}", e),
    }
}
