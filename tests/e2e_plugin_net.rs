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

#[test]
fn e2e_http_stub_end_to_end() {
    if !try_init_plugins() { return; }

    let code = r#"
local srv, cli, r, req, resp, body
srv = new HttpServerBox()
srv.start(8080)

cli = new HttpClientBox()
r = cli.get("http://localhost/hello")

req = srv.accept()
resp = new HttpResponseBox()
resp.setStatus(200)
resp.write("OK")
req.respond(resp)

body = r.readBody()
body
"#;

    let ast = NyashParser::parse_from_string(code).expect("parse failed");
    let mut interpreter = nyash_rust::interpreter::NyashInterpreter::new();
    let result = interpreter.execute(ast).expect("exec failed");
    assert_eq!(result.to_string_box().value, "OK");
}

