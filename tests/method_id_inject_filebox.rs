#![cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
use nyash_rust::runtime::box_registry::get_global_registry;
use nyash_rust::runtime::plugin_loader_v2::{get_global_loader_v2, init_global_loader_v2};
use nyash_rust::runtime::PluginConfig;
use nyash_rust::{
    mir::{instruction::MirInstruction, passes::method_id_inject::inject_method_ids, MirCompiler},
    parser::NyashParser,
};

fn try_init_plugins() -> bool {
    if !std::path::Path::new("nyash.toml").exists() {
        return false;
    }
    if let Err(e) = init_global_loader_v2("nyash.toml") {
        eprintln!("init failed: {:?}", e);
        return false;
    }
    let loader = get_global_loader_v2();
    let loader = loader.read().unwrap();
    if let Some(conf) = &loader.config {
        let mut map = std::collections::HashMap::new();
        for (lib, def) in &conf.libraries {
            for b in &def.boxes {
                map.insert(b.clone(), lib.clone());
            }
        }
        get_global_registry().apply_plugin_config(&PluginConfig { plugins: map });
        true
    } else {
        false
    }
}

#[test]
fn injects_method_id_for_filebox_open() {
    if !try_init_plugins() {
        return;
    }
    let code = r#"
local f
f = new FileBox()
f.open("/tmp/test.txt", "r")
"#;
    let ast = NyashParser::parse_from_string(code).expect("parse failed");
    let mut compiler = MirCompiler::new();
    let module = compiler.compile(ast).expect("mir compile failed").module;
    let mut module2 = module.clone();
    let injected = inject_method_ids(&mut module2);
    assert!(injected > 0);
    for func in module2.functions.values() {
        for block in func.blocks.values() {
            for inst in &block.instructions {
                if let MirInstruction::BoxCall {
                    method, method_id, ..
                } = inst
                {
                    if method == "open" {
                        assert!(method_id.is_some(), "open missing method_id");
                        return;
                    }
                }
            }
        }
    }
    panic!("FileBox.open not found");
}
