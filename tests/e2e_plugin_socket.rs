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
fn e2e_socket_ping_pong() {
    if !try_init_plugins() { return; }

    // Start server, client connect, ping/pong
    let code = r#"
local ss, sc, c, s, r
ss = new SocketServerBox()
ss.start(9100)

sc = new SocketClientBox()
c = sc.connect("127.0.0.1", 9100)

s = ss.accept()

c.send("ping")
r = s.recv()
// echo back
s.send("pong")
r = c.recv()
r
"#;

    let ast = NyashParser::parse_from_string(code).expect("parse failed");
    let mut interpreter = nyash_rust::interpreter::NyashInterpreter::new();
    let result = interpreter.execute(ast).expect("exec failed");
    assert_eq!(result.to_string_box().value, "pong");
}

#[test]
fn e2e_socket_accept_timeout_and_recv_timeout() {
    if !try_init_plugins() { return; }

    let code = r#"
local ss, sc, c, s, r
ss = new SocketServerBox()
ss.start(9101)

// before any client, acceptTimeout returns void
r = ss.acceptTimeout(50)
// now connect
sc = new SocketClientBox()
c = sc.connect("127.0.0.1", 9101)
s = ss.acceptTimeout(500)

// recvTimeout with no data should be empty
r = s.recvTimeout(50)

// send then recvTimeout should get data
c.send("hello")
r = s.recvTimeout(200)
r
"#;

    let ast = NyashParser::parse_from_string(code).expect("parse failed");
    let mut interpreter = nyash_rust::interpreter::NyashInterpreter::new();
    let result = interpreter.execute(ast).expect("exec failed");
    assert_eq!(result.to_string_box().value, "hello");
}

