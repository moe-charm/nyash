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

#[test]
fn e2e_http_server_restart() {
    if !try_init_plugins() { return; }

    let code = r#"
local srv, cli, r, req, resp, body
srv = new HttpServerBox()
srv.start(8081)

cli = new HttpClientBox()
r = cli.get("http://localhost/test1")
req = srv.accept()
resp = new HttpResponseBox()
resp.write("A")
req.respond(resp)

srv.stop()
srv.start(8081)

r = cli.get("http://localhost/test2")
req = srv.accept()
resp = new HttpResponseBox()
resp.write("B")
req.respond(resp)

body = r.readBody()
body
"#;

    let ast = NyashParser::parse_from_string(code).expect("parse failed");
    let mut interpreter = nyash_rust::interpreter::NyashInterpreter::new();
    let result = interpreter.execute(ast).expect("exec failed");
    assert_eq!(result.to_string_box().value, "B");
}

#[test]
fn e2e_http_server_shutdown_and_restart() {
    if !try_init_plugins() { return; }

    // First run: start and respond
    let code1 = r#"
local srv, cli, r, req, resp
srv = new HttpServerBox()
srv.start(8082)
cli = new HttpClientBox()
r = cli.get("http://localhost/first")
req = srv.accept()
resp = new HttpResponseBox()
resp.write("X")
req.respond(resp)
"#;
    let ast1 = NyashParser::parse_from_string(code1).expect("parse1");
    let mut i1 = nyash_rust::interpreter::NyashInterpreter::new();
    i1.execute(ast1).expect("exec1");

    // Shutdown plugins (finalize singleton) and re-init
    nyash_rust::runtime::plugin_loader_v2::shutdown_plugins_v2().expect("shutdown ok");
    assert!(try_init_plugins());

    // Second run: ensure fresh instance works
    let code2 = r#"
local srv, cli, r, req, resp, body
srv = new HttpServerBox()
srv.start(8083)
cli = new HttpClientBox()
r = cli.get("http://localhost/second")
req = srv.accept()
resp = new HttpResponseBox()
resp.write("Y")
req.respond(resp)
body = r.readBody()
body
"#;
    let ast2 = NyashParser::parse_from_string(code2).expect("parse2");
    let mut i2 = nyash_rust::interpreter::NyashInterpreter::new();
    let result = i2.execute(ast2).expect("exec2");
    assert_eq!(result.to_string_box().value, "Y");
}

#[test]
fn e2e_http_post_and_headers() {
    if !try_init_plugins() { return; }

    let code = r#"
local srv, cli, r, req, resp, body, st, hv
srv = new HttpServerBox()
srv.start(8090)

cli = new HttpClientBox()
r = cli.post("http://localhost/api", "DATA")

req = srv.accept()
// check server saw body
body = req.readBody()
// prepare response
resp = new HttpResponseBox()
resp.setStatus(201)
resp.setHeader("X-Test", "V")
resp.write("R")
req.respond(resp)

// client reads status, header, body
st = r.getStatus()
hv = r.getHeader("X-Test")
body = r.readBody()
st.toString() + ":" + hv + ":" + body
"#;

    let ast = NyashParser::parse_from_string(code).expect("parse failed");
    let mut interpreter = nyash_rust::interpreter::NyashInterpreter::new();
    let result = interpreter.execute(ast).expect("exec failed");
    assert_eq!(result.to_string_box().value, "201:V:R");
}
