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
fn e2e_http_two_servers_parallel() {
    std::env::set_var("NYASH_NET_LOG", "1");
    std::env::set_var("NYASH_NET_LOG_FILE", "net_plugin.log");
    if !try_init_plugins() { return; }

    let code = r#"
local s1, s2, c, r1, r2, resp1, resp2, req1, req2, p1, p2, x, y
s1 = new HttpServerBox()
s2 = new HttpServerBox()
s1.start(8101)
s2.start(8102)

c = new HttpClientBox()
r1 = c.get("http://localhost:8101/a")
r2 = c.get("http://localhost:8102/b")

// accept once per pending request and keep handles
req1 = s1.accept().get_value()
req2 = s2.accept().get_value()
p1 = req1.path()
p2 = req2.path()

x = new HttpResponseBox()
x.write("X")
y = new HttpResponseBox()
y.write("Y")

// respond using kept request handles
req1.respond(x)
req2.respond(y)

// read results
resp1 = r1.get_value()
resp2 = r2.get_value()
x = resp1.readBody()
y = resp2.readBody()
x + ":" + y
"#;

    let ast = NyashParser::parse_from_string(code).expect("parse");
    let mut i = nyash_rust::interpreter::NyashInterpreter::new();
    let result = i.execute(ast).expect("exec");
    assert!(result.to_string_box().value.contains(":"));
}

#[test]
fn e2e_http_long_body_and_headers() {
    std::env::set_var("NYASH_NET_LOG", "1");
    std::env::set_var("NYASH_NET_LOG_FILE", "net_plugin.log");
    if !try_init_plugins() { return; }

    let code = r#"
local s, c, r, resp, q, body, hv
s = new HttpServerBox()
s.start(8103)

c = new HttpClientBox()
r = c.post("http://localhost:8103/long", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")

q = s.accept().get_value()
body = q.readBody()
resp = new HttpResponseBox()
resp.setStatus(202)
resp.setHeader("X-Alpha", "A")
resp.setHeader("X-Beta", "B")
resp.write("OK-LONG")
q.respond(resp)

resp = r.get_value()
body = resp.readBody()
hv = resp.getHeader("X-Alpha")
hv + ":" + body
"#;

    let ast = NyashParser::parse_from_string(code).expect("parse");
    let mut i = nyash_rust::interpreter::NyashInterpreter::new();
    let result = i.execute(ast).expect("exec");
    let s = result.to_string_box().value;
    assert!(s.starts_with("A:"));
    assert!(s.contains("OK-LONG"));
}
