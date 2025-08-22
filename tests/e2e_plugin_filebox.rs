#![cfg(all(feature = "plugins", not(target_arch = "wasm32")))]

use nyash_rust::parser::NyashParser;
use nyash_rust::runtime::plugin_loader_v2::{init_global_loader_v2, get_global_loader_v2};
use nyash_rust::runtime::box_registry::get_global_registry;
use nyash_rust::runtime::PluginConfig;
use nyash_rust::runtime::NyashRuntime;
use nyash_rust::backend::VM;

fn try_init_plugins() -> bool {
    if !std::path::Path::new("nyash.toml").exists() {
        eprintln!("[e2e] nyash.toml not found; skipping plugin test");
        return false;
    }
    if let Err(e) = init_global_loader_v2("nyash.toml") {
        eprintln!("[e2e] init_global_loader_v2 failed: {:?}", e);
        return false;
    }
    // Apply mapping: Box name -> library name into legacy registry
    let loader = get_global_loader_v2();
    let loader = loader.read().unwrap();
    if let Some(conf) = &loader.config {
        let mut map = std::collections::HashMap::new();
        for (lib_name, lib_def) in &conf.libraries {
            for b in &lib_def.boxes {
                map.insert(b.clone(), lib_name.clone());
            }
        }
        let reg = get_global_registry();
        reg.apply_plugin_config(&PluginConfig { plugins: map });
        true
    } else {
        eprintln!("[e2e] loader has no config; skipping");
        false
    }
}

#[test]
fn e2e_interpreter_plugin_filebox_close_void() {
    if !try_init_plugins() { return; }

    let code = r#"
local f
f = new FileBox()
f.close()
"#;
    
    // Test through interpreter path first  
    let ast = NyashParser::parse_from_string(code).expect("parse failed");
    let mut interpreter = nyash_rust::interpreter::NyashInterpreter::new();
    
    match interpreter.execute(ast) {
        Ok(result) => {
            // close() returns void (BID-1 tag=9)
            let result_str = result.to_string_box().value;
            assert_eq!(result_str, "void", "Expected 'void' result from close()");
            println!("✅ E2E Plugin FileBox Interpreter test passed!");
        }
        Err(e) => {
            panic!("Failed to execute Nyash code: {:?}", e);
        }
    }
}

#[test]
fn e2e_interpreter_plugin_filebox_delegation() {
    if !try_init_plugins() { return; }

    let code = r#"
box LoggingFileBox from FileBox {
    init { log_count }
    
    birth() {
        from FileBox.birth()
        me.log_count = 0
    }
    
    override close() {
        me.log_count = me.log_count + 1
        print("Closing file, log count: " + me.log_count.toString())
        from FileBox.close()
    }
}

local lf
lf = new LoggingFileBox()
lf.close()
"#;
    
    // Test delegation through interpreter  
    let ast = NyashParser::parse_from_string(code).expect("parse failed");
    let mut interpreter = nyash_rust::interpreter::NyashInterpreter::new();
    
    match interpreter.execute(ast) {
        Ok(_) => {
            println!("✅ E2E Plugin FileBox delegation test passed!");
        }
        Err(e) => {
            panic!("Failed to execute delegation code: {:?}", e);
        }
    }
}

#[test]
fn e2e_vm_plugin_filebox_close_void() {
    if !try_init_plugins() { return; }

    let code = r#"
local f
f = new FileBox()
f.close()
"#;
    let ast = NyashParser::parse_from_string(code).expect("parse failed");

    let runtime = NyashRuntime::new();
    let mut compiler = nyash_rust::mir::MirCompiler::new();
    let compile_result = compiler.compile(ast).expect("mir compile failed");
    let mut vm = VM::with_runtime(runtime);
    let result = vm.execute_module(&compile_result.module).expect("vm exec failed");
    // close() is void; ensure result is void
    assert_eq!(result.to_string_box().value, "void");
}

#[test]
fn e2e_vm_plugin_filebox_open_rw() {
    if !try_init_plugins() { return; }

    // Open, write, read via VM backend
    let code = r#"
local f, data
f = new FileBox()
f.open("./test_write.txt", "rw")
f.write("HELLO")
data = f.read()
data
"#;

    let ast = NyashParser::parse_from_string(code).expect("parse failed");
    let runtime = NyashRuntime::new();
    let mut compiler = nyash_rust::mir::MirCompiler::new();
    let compile_result = compiler.compile(ast).expect("mir compile failed");
    let mut vm = VM::with_runtime(runtime);
    let result = vm.execute_module(&compile_result.module).expect("vm exec failed");
    assert_eq!(result.to_string_box().value, "HELLO");
}

#[test]
fn e2e_vm_plugin_filebox_copy_from_handle() {
    if !try_init_plugins() { return; }

    let p1 = "./test_out_src.txt";
    let p2 = "./test_out_dst.txt";

    let code = format!(r#"
local a, b, data
a = new FileBox()
b = new FileBox()
a.open("{}", "w")
b.open("{}", "rw")
a.write("HELLO")
b.copyFrom(a)
data = b.read()
data
"#, p1, p2);

    let ast = NyashParser::parse_from_string(&code).expect("parse failed");
    let runtime = NyashRuntime::new();
    let mut compiler = nyash_rust::mir::MirCompiler::new();
    let compile_result = compiler.compile(ast).expect("mir compile failed");
    let mut vm = VM::with_runtime(runtime);
    let result = vm.execute_module(&compile_result.module).expect("vm exec failed");
    assert_eq!(result.to_string_box().value, "HELLO");
}

#[test]
fn e2e_interpreter_plugin_filebox_copy_from_handle() {
    if !try_init_plugins() { return; }

    // Prepare two files and copy contents via plugin Handle argument
    let p1 = "./test_out_src.txt";
    let p2 = "./test_out_dst.txt";

    // Nyash program: open two FileBox, write to src, copy to dst via copyFrom, then read dst
    let code = format!(r#"
local a, b, data
a = new FileBox()
b = new FileBox()
a.open("{}", "w")
b.open("{}", "rw")
a.write("HELLO")
b.copyFrom(a)
data = b.read()
data
"#, p1, p2);

    let ast = NyashParser::parse_from_string(&code).expect("parse failed");
    let mut interpreter = nyash_rust::interpreter::NyashInterpreter::new();

    match interpreter.execute(ast) {
        Ok(result) => {
            assert_eq!(result.to_string_box().value, "HELLO");
        }
        Err(e) => panic!("Failed to execute copyFrom test: {:?}", e),
    }
}
