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
    std::env::set_var("NYASH_NET_LOG", "1");
    std::env::set_var("NYASH_NET_LOG_FILE", "net_plugin.log");
    if !try_init_plugins() { return; }

    let code = r#"
local srv, cli, r, resp, req, body
srv = new HttpServerBox()
srv.start(8080)

cli = new HttpClientBox()
r = cli.get("http://localhost:8080/hello")

req = srv.accept().get_value()
resp = new HttpResponseBox()
resp.setStatus(200)
resp.write("OK")
req.respond(resp)

resp = r.get_value()
body = resp.readBody()
body
"#;

    let ast = NyashParser::parse_from_string(code).expect("parse failed");
    let mut interpreter = nyash_rust::interpreter::NyashInterpreter::new();
    let result = interpreter.execute(ast).expect("exec failed");
    assert_eq!(result.to_string_box().value, "OK");
}

#[test]
fn e2e_http_server_restart() {
    std::env::set_var("NYASH_NET_LOG", "1");
    std::env::set_var("NYASH_NET_LOG_FILE", "net_plugin.log");
    if !try_init_plugins() { return; }

    let code = r#"
local srv, cli, r, resp, req, body
srv = new HttpServerBox()
srv.start(8081)

cli = new HttpClientBox()
r = cli.get("http://localhost:8081/test1")
req = srv.accept().get_value()
resp = new HttpResponseBox()
resp.write("A")
req.respond(resp)

srv.stop()
srv.start(8081)

resp = r.get_value()
_ = resp.readBody()  # consume first response (optional)

r = cli.get("http://localhost:8081/test2")
req = srv.accept().get_value()
resp = new HttpResponseBox()
resp.write("B")
req.respond(resp)

resp = r.get_value()
body = resp.readBody()
body
"#;

    let ast = NyashParser::parse_from_string(code).expect("parse failed");
    let mut interpreter = nyash_rust::interpreter::NyashInterpreter::new();
    let result = interpreter.execute(ast).expect("exec failed");
    assert_eq!(result.to_string_box().value, "B");
}

#[test]
fn e2e_http_server_shutdown_and_restart() {
    std::env::set_var("NYASH_NET_LOG", "1");
    std::env::set_var("NYASH_NET_LOG_FILE", "net_plugin.log");
    if !try_init_plugins() { return; }

    // First run: start and respond
    let code1 = r#"
local srv, cli, r, resp, req
srv = new HttpServerBox()
srv.start(8082)
cli = new HttpClientBox()
r = cli.get("http://localhost:8082/first")
req = srv.accept().get_value()
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
r = cli.get("http://localhost:8083/second")
req = srv.accept().get_value()
resp = new HttpResponseBox()
resp.write("Y")
req.respond(resp)
resp = r.get_value()
body = resp.readBody()
body
"#;
    let ast2 = NyashParser::parse_from_string(code2).expect("parse2");
    let mut i2 = nyash_rust::interpreter::NyashInterpreter::new();
    let result = i2.execute(ast2).expect("exec2");
    assert_eq!(result.to_string_box().value, "Y");
}

#[test]
fn e2e_http_post_and_headers() {
    std::env::set_var("NYASH_NET_LOG", "1");
    std::env::set_var("NYASH_NET_LOG_FILE", "net_plugin.log");
    if !try_init_plugins() { return; }

    let code = r#"
local srv, cli, r, resp, req, body, st, hv
srv = new HttpServerBox()
srv.start(8090)

cli = new HttpClientBox()
r = cli.post("http://localhost:8090/api", "DATA")

req = srv.accept().get_value()
// check server saw body
body = req.readBody()
// prepare response
resp = new HttpResponseBox()
resp.setStatus(201)
resp.setHeader("X-Test", "V")
resp.write("R")
req.respond(resp)

// client reads status, header, body
resp = r.get_value()
st = resp.getStatus()
hv = resp.getHeader("X-Test")
body = resp.readBody()
st.toString() + ":" + hv + ":" + body
"#;

    let ast = NyashParser::parse_from_string(code).expect("parse failed");
    let mut interpreter = nyash_rust::interpreter::NyashInterpreter::new();
    let result = interpreter.execute(ast).expect("exec failed");
    assert_eq!(result.to_string_box().value, "201:V:R");
}

#[test]
fn e2e_http_multiple_requests_order() {
    std::env::set_var("NYASH_NET_LOG", "1");
    std::env::set_var("NYASH_NET_LOG_FILE", "net_plugin.log");
    if !try_init_plugins() { return; }

    let code = r#"
local srv, cli, r1, r2, r3, req1, req2, req3, q1, q2, q3
srv = new HttpServerBox()
srv.start(8091)

cli = new HttpClientBox()
r1 = cli.get("http://localhost:8091/a")
r2 = cli.get("http://localhost:8091/b")
r3 = cli.get("http://localhost:8091/c")

req1 = srv.accept().get_value()
q1 = req1.path()
req2 = srv.accept().get_value()
q2 = req2.path()
req3 = srv.accept().get_value()
q3 = req3.path()

q1 + "," + q2 + "," + q3
"#;

    let ast = NyashParser::parse_from_string(code).expect("parse failed");
    let mut interpreter = nyash_rust::interpreter::NyashInterpreter::new();
    let result = interpreter.execute(ast).expect("exec failed");
    assert_eq!(result.to_string_box().value, "/a,/b,/c");
}
