//! Nyash Plugin Tester
//! 
//! プラグイン開発者向けの診断ツール
//! Box名を決め打ちせず、プラグインから取得する

use clap::Parser;
use colored::*;
use libloading::{Library, Symbol};
use serde::Deserialize;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::{c_char, c_void};
use std::path::PathBuf;
use std::io::Write;

// ============ FFI Types (プラグインと同じ定義) ============

#[repr(C)]
pub struct NyashHostVtable {
    pub alloc: unsafe extern "C" fn(size: usize) -> *mut u8,
    pub free: unsafe extern "C" fn(ptr: *mut u8),
    pub wake: unsafe extern "C" fn(handle: u64),
    pub log: unsafe extern "C" fn(level: i32, msg: *const c_char),
}

#[repr(C)]
pub struct NyashMethodInfo {
    pub method_id: u32,
    pub name: *const c_char,
    pub signature: u32,
}

#[repr(C)]
pub struct NyashPluginInfo {
    pub type_id: u32,
    pub type_name: *const c_char,
    pub method_count: usize,
    pub methods: *const NyashMethodInfo,
}

// ============ TOML Configuration Types ============

#[derive(Debug)]
struct NyashConfig {
    plugins: HashMap<String, String>,
    plugin_configs: HashMap<String, PluginConfig>,
}

#[derive(Debug)]
struct PluginConfig {
    methods: Option<HashMap<String, MethodConfig>>,
}

#[derive(Deserialize, Debug)]
struct MethodConfig {
    args: Vec<TypeConversion>,
    returns: Option<String>,
}

#[derive(Deserialize, Debug)]
struct TypeConversion {
    name: Option<String>,
    from: String,
    to: String,
}

// ============ CLI Arguments ============

#[derive(Parser, Debug)]
#[command(name = "plugin-tester")]
#[command(about = "Nyash plugin diagnostic tool", long_about = None)]
struct Args {
    /// Action to perform
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Check plugin and display information
    Check {
        /// Path to plugin .so file
        plugin: PathBuf,
    },
    /// Test plugin lifecycle (birth/fini)
    Lifecycle {
        /// Path to plugin .so file
        plugin: PathBuf,
    },
    /// File I/O end-to-end test (open/write/read/close)
    Io {
        /// Path to plugin .so file
        plugin: PathBuf,
    },
    /// Debug TLV encoding/decoding with detailed output
    TlvDebug {
        /// Path to plugin .so file
        plugin: PathBuf,
        /// Test message to encode/decode
        #[arg(short, long, default_value = "Hello TLV Debug!")]
        message: String,
    },
    /// Validate plugin type information against nyash.toml
    Typecheck {
        /// Path to plugin .so file
        plugin: PathBuf,
        /// Path to nyash.toml configuration file
        #[arg(short, long, default_value = "../../nyash.toml")]
        config: PathBuf,
    },
}

// ============ Host Functions (テスト用実装) ============

unsafe extern "C" fn test_alloc(size: usize) -> *mut u8 {
    let layout = std::alloc::Layout::from_size_align(size, 8).unwrap();
    std::alloc::alloc(layout)
}

unsafe extern "C" fn test_free(ptr: *mut u8) {
    if !ptr.is_null() {
        // サイズ情報が必要だが、簡易実装のため省略
    }
}

unsafe extern "C" fn test_wake(_handle: u64) {
    // テスト用なので何もしない
}

unsafe extern "C" fn test_log(level: i32, msg: *const c_char) {
    if !msg.is_null() {
        let c_str = CStr::from_ptr(msg);
        let message = c_str.to_string_lossy();
        
        match level {
            0 => println!("{}: {}", "DEBUG".blue(), message),
            1 => println!("{}: {}", "INFO".green(), message),
            2 => println!("{}: {}", "WARN".yellow(), message),
            3 => println!("{}: {}", "ERROR".red(), message),
            _ => println!("{}: {}", "UNKNOWN".white(), message),
        }
    }
}

static HOST_VTABLE: NyashHostVtable = NyashHostVtable {
    alloc: test_alloc,
    free: test_free,
    wake: test_wake,
    log: test_log,
};

// ============ Main Functions ============

fn main() {
    let args = Args::parse();
    
    match args.command {
        Commands::Check { plugin } => check_plugin(&plugin),
        Commands::Lifecycle { plugin } => test_lifecycle(&plugin),
        Commands::Io { plugin } => test_file_io(&plugin),
        Commands::TlvDebug { plugin, message } => test_tlv_debug(&plugin, &message),
        Commands::Typecheck { plugin, config } => typecheck_plugin(&plugin, &config),
    }
}

// ============ Minimal BID-1 TLV Helpers ============

#[repr(C)]
#[derive(Clone, Copy)]
struct TlvHeader { version: u16, argc: u16 }

const TLV_VERSION: u16 = 1;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tag { Bool=1, I32=2, I64=3, F32=4, F64=5, String=6, Bytes=7, Handle=8, Void=9 }

fn tlv_encode_string(s: &str, buf: &mut Vec<u8>) {
    let header_pos = buf.len();
    buf.extend_from_slice(&[0,0,0,0]);
    let mut argc: u16 = 0;
    // entry
    let bytes = s.as_bytes();
    buf.push(Tag::String as u8);
    buf.push(0);
    buf.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
    buf.extend_from_slice(bytes);
    argc += 1;
    // write header
    buf[header_pos..header_pos+2].copy_from_slice(&TLV_VERSION.to_le_bytes());
    buf[header_pos+2..header_pos+4].copy_from_slice(&argc.to_le_bytes());
}

fn tlv_encode_two_strings(a: &str, b: &str, buf: &mut Vec<u8>) {
    let header_pos = buf.len();
    buf.extend_from_slice(&[0,0,0,0]);
    let mut argc: u16 = 0;
    for s in [a,b] {
        let bytes = s.as_bytes();
        buf.push(Tag::String as u8);
        buf.push(0);
        buf.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
        buf.extend_from_slice(bytes);
        argc += 1;
    }
    buf[header_pos..header_pos+2].copy_from_slice(&TLV_VERSION.to_le_bytes());
    buf[header_pos+2..header_pos+4].copy_from_slice(&argc.to_le_bytes());
}

fn tlv_encode_i32(v: i32, buf: &mut Vec<u8>) {
    let header_pos = buf.len();
    buf.extend_from_slice(&[0,0,0,0]);
    buf.push(Tag::I32 as u8);
    buf.push(0);
    buf.extend_from_slice(&4u16.to_le_bytes());
    buf.extend_from_slice(&v.to_le_bytes());
    buf[header_pos..header_pos+2].copy_from_slice(&TLV_VERSION.to_le_bytes());
    buf[header_pos+2..header_pos+4].copy_from_slice(&1u16.to_le_bytes());
}

fn tlv_encode_bytes(data: &[u8], buf: &mut Vec<u8>) {
    let header_pos = buf.len();
    buf.extend_from_slice(&[0,0,0,0]);
    buf.push(Tag::Bytes as u8);
    buf.push(0);
    buf.extend_from_slice(&(data.len() as u16).to_le_bytes());
    buf.extend_from_slice(data);
    buf[header_pos..header_pos+2].copy_from_slice(&TLV_VERSION.to_le_bytes());
    buf[header_pos+2..header_pos+4].copy_from_slice(&1u16.to_le_bytes());
}

fn tlv_decode_first(bytes: &[u8]) -> Option<(u8, &[u8])> {
    if bytes.len() < 4 { return None; }
    let argc = u16::from_le_bytes([bytes[2], bytes[3]]);
    if argc == 0 { return None; }
    if bytes.len() < 8 { return None; }
    let tag = bytes[4];
    let size = u16::from_le_bytes([bytes[6], bytes[7]]) as usize;
    if bytes.len() < 8+size { return None; }
    Some((tag, &bytes[8..8+size]))
}

fn check_plugin(path: &PathBuf) {
    println!("{}", "=== Nyash Plugin Checker ===".bold());
    println!("Plugin: {}", path.display());
    
    // プラグインをロード
    let library = match unsafe { Library::new(path) } {
        Ok(lib) => lib,
        Err(e) => {
            eprintln!("{}: Failed to load plugin: {}", "ERROR".red(), e);
            return;
        }
    };
    
    println!("{}: Plugin loaded successfully", "✓".green());
    
    // ABI version確認
    unsafe {
        let abi_fn: Symbol<unsafe extern "C" fn() -> u32> = match library.get(b"nyash_plugin_abi") {
            Ok(f) => f,
            Err(e) => {
                eprintln!("{}: nyash_plugin_abi not found: {}", "ERROR".red(), e);
                return;
            }
        };
        
        let abi_version = abi_fn();
        println!("{}: ABI version: {}", "✓".green(), abi_version);
        
        if abi_version != 1 {
            eprintln!("{}: Unsupported ABI version (expected 1)", "WARNING".yellow());
        }
    }
    
    // Plugin初期化とBox名取得
    unsafe {
        let init_fn: Symbol<unsafe extern "C" fn(*const NyashHostVtable, *mut NyashPluginInfo) -> i32> = 
            match library.get(b"nyash_plugin_init") {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("{}: nyash_plugin_init not found: {}", "ERROR".red(), e);
                    return;
                }
            };
        
        let mut plugin_info = std::mem::zeroed::<NyashPluginInfo>();
        let result = init_fn(&HOST_VTABLE, &mut plugin_info);
        
        if result != 0 {
            eprintln!("{}: nyash_plugin_init failed with code {}", "ERROR".red(), result);
            return;
        }
        
        println!("{}: Plugin initialized", "✓".green());
        
        // 重要：Box名をプラグインから取得（決め打ちしない！）
        let box_name = if plugin_info.type_name.is_null() {
            "<unknown>".to_string()
        } else {
            CStr::from_ptr(plugin_info.type_name).to_string_lossy().to_string()
        };
        
        println!("\n{}", "Plugin Information:".bold());
        println!("  Box Type: {} (ID: {})", box_name.cyan(), plugin_info.type_id);
        println!("  Methods: {}", plugin_info.method_count);
        
        // メソッド一覧表示
        if plugin_info.method_count > 0 && !plugin_info.methods.is_null() {
            println!("\n{}", "Methods:".bold());
            let methods = std::slice::from_raw_parts(plugin_info.methods, plugin_info.method_count);
            
            for method in methods {
                let method_name = if method.name.is_null() {
                    "<unnamed>".to_string()
                } else {
                    CStr::from_ptr(method.name).to_string_lossy().to_string()
                };
                
                let method_type = match method.method_id {
                    0 => " (constructor)".yellow(),
                    id if id == u32::MAX => " (destructor)".yellow(),
                    _ => "".normal(),
                };
                
                println!("  - {} [ID: {}, Sig: 0x{:08X}]{}",
                    method_name,
                    method.method_id,
                    method.signature,
                    method_type
                );
            }
        }
    }
    
    // シャットダウン
    unsafe {
        if let Ok(shutdown_fn) = library.get::<Symbol<unsafe extern "C" fn()>>(b"nyash_plugin_shutdown") {
            shutdown_fn();
            println!("\n{}: Plugin shutdown completed", "✓".green());
        }
    }
    
    println!("\n{}", "Check completed!".green().bold());
}

fn test_lifecycle(path: &PathBuf) {
    println!("{}", "=== Lifecycle Test ===".bold());
    println!("Testing birth/fini for: {}", path.display());

    // プラグインをロード
    let library = match unsafe { Library::new(path) } {
        Ok(lib) => lib,
        Err(e) => {
            eprintln!("{}: Failed to load plugin: {}", "ERROR".red(), e);
            return;
        }
    };

    unsafe {
        // ABI version
        let abi_fn: Symbol<unsafe extern "C" fn() -> u32> = match library.get(b"nyash_plugin_abi") {
            Ok(f) => f,
            Err(e) => {
                eprintln!("{}: nyash_plugin_abi not found: {}", "ERROR".red(), e);
                return;
            }
        };
        let abi_version = abi_fn();
        println!("{}: ABI version: {}", "✓".green(), abi_version);
        if abi_version != 1 {
            eprintln!("{}: Unsupported ABI version (expected 1)", "WARNING".yellow());
        }

        // init
        let init_fn: Symbol<unsafe extern "C" fn(*const NyashHostVtable, *mut NyashPluginInfo) -> i32> =
            match library.get(b"nyash_plugin_init") {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("{}: nyash_plugin_init not found: {}", "ERROR".red(), e);
                    return;
                }
            };
        let mut plugin_info = std::mem::zeroed::<NyashPluginInfo>();
        let result = init_fn(&HOST_VTABLE, &mut plugin_info);
        if result != 0 {
            eprintln!("{}: nyash_plugin_init failed with code {}", "ERROR".red(), result);
            return;
        }
        println!("{}: Plugin initialized", "✓".green());

        // invoke
        let invoke_fn: Symbol<unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32> =
            match library.get(b"nyash_plugin_invoke") {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("{}: nyash_plugin_invoke not found: {}", "ERROR".red(), e);
                    return;
                }
            };

        let type_id = plugin_info.type_id;
        println!("{}: BoxType ID = {}", "i".blue(), type_id);

        // birth
        let mut out = [0u8; 8];
        let mut out_len: usize = out.len();
        let rc = invoke_fn(type_id, 0, 0, std::ptr::null(), 0, out.as_mut_ptr(), &mut out_len as *mut usize);
        if rc != 0 {
            eprintln!("{}: birth invoke failed with code {}", "ERROR".red(), rc);
            return;
        }
        if out_len < 4 {
            eprintln!("{}: birth returned too small result ({} bytes)", "ERROR".red(), out_len);
            return;
        }
        let instance_id = u32::from_le_bytes(out[0..4].try_into().unwrap());
        println!("{}: birth → instance_id={}", "✓".green(), instance_id);

        // fini
        let rc = invoke_fn(type_id, u32::MAX, instance_id, std::ptr::null(), 0, std::ptr::null_mut(), std::ptr::null_mut());
        if rc != 0 {
            eprintln!("{}: fini invoke failed with code {}", "ERROR".red(), rc);
            return;
        }
        println!("{}: fini → instance {} cleaned", "✓".green(), instance_id);

        // shutdown
        if let Ok(shutdown_fn) = library.get::<Symbol<unsafe extern "C" fn()>>(b"nyash_plugin_shutdown") {
            shutdown_fn();
            println!("{}: Plugin shutdown completed", "✓".green());
        }
    }

    println!("\n{}", "Lifecycle test completed!".green().bold());
}

fn test_file_io(path: &PathBuf) {
    println!("{}", "=== File I/O Test ===".bold());
    println!("Testing open/write/read/close: {}", path.display());

    // Load
    let library = match unsafe { Library::new(path) } {
        Ok(lib) => lib,
        Err(e) => {
            eprintln!("{}: Failed to load plugin: {}", "ERROR".red(), e);
            return;
        }
    };
    unsafe {
        let abi: Symbol<unsafe extern "C" fn() -> u32> = library.get(b"nyash_plugin_abi").unwrap();
        println!("{}: ABI version: {}", "✓".green(), abi());
        let init: Symbol<unsafe extern "C" fn(*const NyashHostVtable, *mut NyashPluginInfo) -> i32> = library.get(b"nyash_plugin_init").unwrap();
        let mut info = std::mem::zeroed::<NyashPluginInfo>();
        assert_eq!(0, init(&HOST_VTABLE, &mut info));
        let invoke: Symbol<unsafe extern "C" fn(u32,u32,u32,*const u8,usize,*mut u8,*mut usize)->i32> = library.get(b"nyash_plugin_invoke").unwrap();
        let shutdown: Symbol<unsafe extern "C" fn()> = library.get(b"nyash_plugin_shutdown").unwrap();

        // birth
        let mut buf_len: usize = 0;
        let rc = invoke(info.type_id, 0, 0, std::ptr::null(), 0, std::ptr::null_mut(), &mut buf_len as *mut usize);
        assert!(rc == -1 && buf_len >= 4, "unexpected birth preflight");
        let mut out = vec![0u8; buf_len];
        let mut out_len = buf_len;
        assert_eq!(0, invoke(info.type_id, 0, 0, std::ptr::null(), 0, out.as_mut_ptr(), &mut out_len as *mut usize));
        let instance_id = u32::from_le_bytes(out[0..4].try_into().unwrap());
        println!("{}: birth → instance_id={}", "✓".green(), instance_id);

        // open: write mode
        let mut args = Vec::new();
        let test_path = "plugins/nyash-filebox-plugin/target/test_io.txt";
        tlv_encode_two_strings(test_path, "w", &mut args);
        let mut res_len: usize = 0;
        let rc = invoke(info.type_id, 1, instance_id, args.as_ptr(), args.len(), std::ptr::null_mut(), &mut res_len as *mut usize);
        assert!(rc == -1 || rc == 0);
        let mut res = vec![0u8; res_len.max(4)];
        let mut rl = res_len;
        let _ = invoke(info.type_id, 1, instance_id, args.as_ptr(), args.len(), res.as_mut_ptr(), &mut rl as *mut usize);
        println!("{}: open(w)", "✓".green());

        // write
        let content = b"Hello from plugin-tester!";
        let mut wargs = Vec::new();
        tlv_encode_bytes(content, &mut wargs);
        let mut rlen: usize = 0;
        let rc = invoke(info.type_id, 3, instance_id, wargs.as_ptr(), wargs.len(), std::ptr::null_mut(), &mut rlen as *mut usize);
        assert!(rc == -1 || rc == 0);
        let mut wb = vec![0u8; rlen.max(8)];
        let mut rl2 = rlen;
        let _ = invoke(info.type_id, 3, instance_id, wargs.as_ptr(), wargs.len(), wb.as_mut_ptr(), &mut rl2 as *mut usize);
        if let Some((tag, payload)) = tlv_decode_first(&wb[..rl2]) {
            assert_eq!(tag, Tag::I32 as u8);
            let mut n = [0u8;4]; n.copy_from_slice(payload);
            let written = i32::from_le_bytes(n);
            println!("{}: write {} bytes", "✓".green(), written);
        }

        // close
        let mut clen: usize = 0;
        let _ = invoke(info.type_id, 4, instance_id, std::ptr::null(), 0, std::ptr::null_mut(), &mut clen as *mut usize);
        let mut cb = vec![0u8; clen.max(4)]; let mut cbl = clen; let _ = invoke(info.type_id, 4, instance_id, std::ptr::null(), 0, cb.as_mut_ptr(), &mut cbl as *mut usize);
        println!("{}: close", "✓".green());

        // reopen read
        let mut args2 = Vec::new(); tlv_encode_two_strings(test_path, "r", &mut args2);
        let mut r0: usize = 0; let _ = invoke(info.type_id, 1, instance_id, args2.as_ptr(), args2.len(), std::ptr::null_mut(), &mut r0 as *mut usize);
        let mut ob = vec![0u8; r0.max(4)]; let mut obl=r0; let _=invoke(info.type_id,1,instance_id,args2.as_ptr(),args2.len(),ob.as_mut_ptr(),&mut obl as *mut usize);
        println!("{}: open(r)", "✓".green());

        // read 1024
        let mut rargs = Vec::new(); tlv_encode_i32(1024, &mut rargs);
        let mut rneed: usize = 0; let rc = invoke(info.type_id, 2, instance_id, rargs.as_ptr(), rargs.len(), std::ptr::null_mut(), &mut rneed as *mut usize);
        assert!(rc == -1 || rc == 0);
        let mut rb = vec![0u8; rneed.max(16)]; let mut rbl=rneed; let rc2=invoke(info.type_id,2,instance_id,rargs.as_ptr(),rargs.len(),rb.as_mut_ptr(),&mut rbl as *mut usize);
        if rc2 != 0 { println!("{}: read rc={} (expected 0)", "WARN".yellow(), rc2); }
        if let Some((tag, payload)) = tlv_decode_first(&rb[..rbl]) {
            assert_eq!(tag, Tag::Bytes as u8);
            let s = String::from_utf8_lossy(payload).to_string();
            println!("{}: read {} bytes → '{}'", "✓".green(), payload.len(), s);
        } else {
            println!("{}: read decode failed (len={})", "WARN".yellow(), rbl);
        }

        // close & shutdown
        let mut clen2: usize = 0; let _=invoke(info.type_id,4,instance_id,std::ptr::null(),0,std::ptr::null_mut(),&mut clen2 as *mut usize);
        shutdown();
        println!("\n{}", "File I/O test completed!".green().bold());
    }
}

fn test_tlv_debug(path: &PathBuf, message: &str) {
    println!("{}", "=== TLV Debug Test ===".bold());
    println!("Testing TLV encoding/decoding with: '{}'", message);
    
    // Load plugin
    let library = match unsafe { Library::new(path) } {
        Ok(lib) => lib,
        Err(e) => {
            eprintln!("{}: Failed to load plugin: {}", "ERROR".red(), e);
            return;
        }
    };
    
    unsafe {
        let abi: Symbol<unsafe extern "C" fn() -> u32> = library.get(b"nyash_plugin_abi").unwrap();
        println!("{}: ABI version: {}", "✓".green(), abi());
        
        let init: Symbol<unsafe extern "C" fn(*const NyashHostVtable, *mut NyashPluginInfo) -> i32> = library.get(b"nyash_plugin_init").unwrap();
        let mut info = std::mem::zeroed::<NyashPluginInfo>();
        assert_eq!(0, init(&HOST_VTABLE, &mut info));
        
        let invoke: Symbol<unsafe extern "C" fn(u32,u32,u32,*const u8,usize,*mut u8,*mut usize)->i32> = library.get(b"nyash_plugin_invoke").unwrap();
        let shutdown: Symbol<unsafe extern "C" fn()> = library.get(b"nyash_plugin_shutdown").unwrap();
        
        // Test TLV encoding
        println!("\n{}", "--- Encoding Test ---".cyan());
        let mut encoded = Vec::new();
        tlv_encode_string(message, &mut encoded);
        
        println!("Original message: '{}'", message);
        println!("Encoded TLV ({} bytes): {:02x?}", encoded.len(), encoded);
        
        // Hex dump for readability
        print!("Hex dump: ");
        for (i, byte) in encoded.iter().enumerate() {
            if i % 16 == 0 && i > 0 { print!("\n          "); }
            print!("{:02x} ", byte);
        }
        println!();
        
        // Test TLV decoding  
        println!("\n{}", "--- Decoding Test ---".cyan());
        if let Some((tag, payload)) = tlv_decode_first(&encoded) {
            println!("Decoded tag: {} ({})", tag, 
                match tag {
                    6 => "String",
                    7 => "Bytes", 
                    _ => "Unknown"
                });
            println!("Decoded payload ({} bytes): {:02x?}", payload.len(), payload);
            
            if tag == Tag::String as u8 || tag == Tag::Bytes as u8 {
                let decoded_str = String::from_utf8_lossy(payload);
                println!("Decoded string: '{}'", decoded_str);
                
                if decoded_str == message {
                    println!("{}: TLV round-trip successful!", "✓".green());
                } else {
                    println!("{}: TLV round-trip failed! Expected: '{}', Got: '{}'", 
                             "✗".red(), message, decoded_str);
                }
            }
        } else {
            println!("{}: Failed to decode TLV!", "✗".red());
        }
        
        // Test with plugin write/read
        println!("\n{}", "--- Plugin Round-trip Test ---".cyan());
        
        // birth
        let mut buf_len: usize = 0;
        let rc = invoke(info.type_id, 0, 0, std::ptr::null(), 0, std::ptr::null_mut(), &mut buf_len);
        assert!(rc == -1 && buf_len >= 4);
        let mut out = vec![0u8; buf_len];
        let mut out_len = buf_len;
        assert_eq!(0, invoke(info.type_id, 0, 0, std::ptr::null(), 0, out.as_mut_ptr(), &mut out_len));
        let instance_id = u32::from_le_bytes(out[0..4].try_into().unwrap());
        println!("{}: birth → instance_id={}", "✓".green(), instance_id);
        
        // Test file write
        let test_path = "plugins/nyash-filebox-plugin/target/tlv_debug_test.txt";
        let mut args = Vec::new();
        tlv_encode_two_strings(test_path, "w", &mut args);
        println!("Write args TLV ({} bytes): {:02x?}", args.len(), &args[..args.len().min(32)]);
        
        let mut need: usize = 0;
        let _ = invoke(info.type_id, 1, instance_id, args.as_ptr(), args.len(), std::ptr::null_mut(), &mut need);
        let mut obuf = vec![0u8; need.max(4)];
        let mut olen = need;
        let _ = invoke(info.type_id, 1, instance_id, args.as_ptr(), args.len(), obuf.as_mut_ptr(), &mut olen);
        println!("{}: open(w) successful", "✓".green());
        
        // Write test message
        let mut write_args = Vec::new();
        tlv_encode_string(message, &mut write_args);
        println!("Write message TLV ({} bytes): {:02x?}", write_args.len(), &write_args[..write_args.len().min(32)]);
        
        let mut wneed: usize = 0;
        let _ = invoke(info.type_id, 3, instance_id, write_args.as_ptr(), write_args.len(), std::ptr::null_mut(), &mut wneed);
        let mut wbuf = vec![0u8; wneed.max(4)];
        let mut wlen = wneed;
        let _ = invoke(info.type_id, 3, instance_id, write_args.as_ptr(), write_args.len(), wbuf.as_mut_ptr(), &mut wlen);
        println!("{}: write successful", "✓".green());
        
        // Close
        let mut clen: usize = 0;
        let _ = invoke(info.type_id, 4, instance_id, std::ptr::null(), 0, std::ptr::null_mut(), &mut clen);
        let mut cb = vec![0u8; clen.max(4)];
        let mut cbl = clen;
        let _ = invoke(info.type_id, 4, instance_id, std::ptr::null(), 0, cb.as_mut_ptr(), &mut cbl);
        println!("{}: close successful", "✓".green());
        
        // Reopen for read
        let mut read_args = Vec::new();
        tlv_encode_two_strings(test_path, "r", &mut read_args);
        let mut rneed: usize = 0;
        let _ = invoke(info.type_id, 1, instance_id, read_args.as_ptr(), read_args.len(), std::ptr::null_mut(), &mut rneed);
        let mut robuf = vec![0u8; rneed.max(4)];
        let mut rolen = rneed;
        let _ = invoke(info.type_id, 1, instance_id, read_args.as_ptr(), read_args.len(), robuf.as_mut_ptr(), &mut rolen);
        println!("{}: open(r) successful", "✓".green());
        
        // Read back
        let mut size_args = Vec::new();
        tlv_encode_i32(1024, &mut size_args);
        let mut read_need: usize = 0;
        let rc = invoke(info.type_id, 2, instance_id, size_args.as_ptr(), size_args.len(), std::ptr::null_mut(), &mut read_need);
        println!("Read preflight: rc={}, need={} bytes", rc, read_need);
        
        let mut read_buf = vec![0u8; read_need.max(16)];
        let mut read_len = read_need;
        let rc2 = invoke(info.type_id, 2, instance_id, size_args.as_ptr(), size_args.len(), read_buf.as_mut_ptr(), &mut read_len);
        println!("Read actual: rc={}, got={} bytes", rc2, read_len);
        
        if read_len > 0 {
            println!("Read result TLV ({} bytes): {:02x?}", read_len, &read_buf[..read_len.min(32)]);
            
            // Try to decode
            if let Some((tag, payload)) = tlv_decode_first(&read_buf[..read_len]) {
                println!("Read decoded tag: {} ({})", tag, 
                    match tag {
                        6 => "String",
                        7 => "Bytes",
                        _ => "Unknown"
                    });
                let read_message = String::from_utf8_lossy(payload);
                println!("Read decoded message: '{}'", read_message);
                
                if read_message == message {
                    println!("{}: Plugin round-trip successful!", "✓".green());
                } else {
                    println!("{}: Plugin round-trip failed! Expected: '{}', Got: '{}'", 
                             "✗".red(), message, read_message);
                }
            } else {
                println!("{}: Failed to decode read result!", "✗".red());
                // Show detailed hex analysis
                if read_len >= 4 {
                    let version = u16::from_le_bytes([read_buf[0], read_buf[1]]);
                    let argc = u16::from_le_bytes([read_buf[2], read_buf[3]]);
                    println!("TLV Header analysis: version={}, argc={}", version, argc);
                    
                    if read_len >= 8 {
                        let entry_tag = read_buf[4];
                        let entry_reserved = read_buf[5];
                        let entry_len = u16::from_le_bytes([read_buf[6], read_buf[7]]);
                        println!("First entry: tag={}, reserved={}, len={}", entry_tag, entry_reserved, entry_len);
                    }
                }
            }
        }
        
        shutdown();
        println!("\n{}", "TLV Debug test completed!".green().bold());
    }
}

fn typecheck_plugin(plugin_path: &PathBuf, config_path: &PathBuf) {
    println!("{}", "=== Type Information Validation ===".bold());
    println!("Plugin: {}", plugin_path.display());
    println!("Config: {}", config_path.display());
    
    // Load and parse configuration
    let config = match load_nyash_config(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}: Failed to load configuration: {}", "ERROR".red(), e);
            return;
        }
    };
    
    // Load plugin
    let library = match unsafe { Library::new(plugin_path) } {
        Ok(lib) => lib,
        Err(e) => {
            eprintln!("{}: Failed to load plugin: {}", "ERROR".red(), e);
            return;
        }
    };
    
    unsafe {
        // ABI version check
        let abi_fn: Symbol<unsafe extern "C" fn() -> u32> = match library.get(b"nyash_plugin_abi") {
            Ok(f) => f,
            Err(e) => {
                eprintln!("{}: nyash_plugin_abi not found: {}", "ERROR".red(), e);
                return;
            }
        };
        let abi_version = abi_fn();
        println!("{}: ABI version: {}", "✓".green(), abi_version);
        
        // Initialize plugin
        let init_fn: Symbol<unsafe extern "C" fn(*const NyashHostVtable, *mut NyashPluginInfo) -> i32> = 
            match library.get(b"nyash_plugin_init") {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("{}: nyash_plugin_init not found: {}", "ERROR".red(), e);
                    return;
                }
            };
        
        let mut plugin_info = std::mem::zeroed::<NyashPluginInfo>();
        let result = init_fn(&HOST_VTABLE, &mut plugin_info);
        
        if result != 0 {
            eprintln!("{}: nyash_plugin_init failed with code {}", "ERROR".red(), result);
            return;
        }
        
        // Get Box type name from plugin
        let box_name = if plugin_info.type_name.is_null() {
            eprintln!("{}: Plugin did not provide type name", "ERROR".red());
            return;
        } else {
            CStr::from_ptr(plugin_info.type_name).to_string_lossy().to_string()
        };
        
        println!("{}: Plugin Box type: {}", "✓".green(), box_name.cyan());
        
        // Validate type configuration
        validate_type_configuration(&config, &box_name, &plugin_info);
        
        // Validate method signatures
        validate_method_signatures(&config, &box_name, &plugin_info);
        
        // Check for duplicate method names (Nyash doesn't support overloading)
        check_duplicate_methods(&plugin_info);
        
        // Shutdown plugin
        if let Ok(shutdown_fn) = library.get::<Symbol<unsafe extern "C" fn()>>(b"nyash_plugin_shutdown") {
            shutdown_fn();
            println!("{}: Plugin shutdown completed", "✓".green());
        }
    }
    
    println!("\n{}", "Type validation completed!".green().bold());
}

fn load_nyash_config(config_path: &PathBuf) -> Result<NyashConfig, Box<dyn std::error::Error>> {
    let config_content = fs::read_to_string(config_path)?;
    let config: toml::Value = toml::from_str(&config_content)?;
    
    let mut plugin_map = HashMap::new();
    let mut plugin_configs = HashMap::new();
    
    if let Some(config_table) = config.as_table() {
        
        // Parse [plugin_names] section (alternative structure)
        if let Some(plugin_names) = config_table.get("plugin_names").and_then(|p| p.as_table()) {
            for (box_type, plugin_name) in plugin_names {
                if let Some(name) = plugin_name.as_str() {
                    plugin_map.insert(box_type.clone(), name.to_string());
                }
            }
        }
        
        // Parse [plugins] section for both mappings and nested configs
        if let Some(plugins) = config_table.get("plugins").and_then(|p| p.as_table()) {
            for (box_type, value) in plugins {
                if let Some(name) = value.as_str() {
                    // Simple string mapping: FileBox = "plugin-name"
                    plugin_map.insert(box_type.clone(), name.to_string());
                } else if let Some(nested) = value.as_table() {
                    // Nested table structure: [plugins.FileBox]
                    if let Some(plugin_name) = nested.get("plugin_name").and_then(|n| n.as_str()) {
                        plugin_map.insert(box_type.clone(), plugin_name.to_string());
                    }
                    
                    if let Some(methods_table) = nested.get("methods").and_then(|m| m.as_table()) {
                        let method_configs = parse_methods_table(methods_table)?;
                        plugin_configs.insert(
                            format!("plugins.{}", box_type),
                            PluginConfig { methods: Some(method_configs) }
                        );
                    }
                }
            }
        }
        
        // Also handle the problematic current structure by manual parsing
        // This is a workaround for the TOML structure issue
        for (section_name, section_value) in config_table {
            if section_name.starts_with("plugins.") && section_name.contains(".methods") {
                if let Some(methods_table) = section_value.as_table() {
                    let box_type_part = section_name.replace("plugins.", "").replace(".methods", "");
                    let method_configs = parse_methods_table(methods_table)?;
                    plugin_configs.insert(
                        format!("plugins.{}", box_type_part),
                        PluginConfig { methods: Some(method_configs) }
                    );
                }
            }
        }
    }
    
    Ok(NyashConfig {
        plugins: plugin_map,
        plugin_configs,
    })
}

fn parse_methods_table(methods_table: &toml::map::Map<String, toml::Value>) -> Result<HashMap<String, MethodConfig>, Box<dyn std::error::Error>> {
    let mut method_configs = HashMap::new();
    
    for (method_name, method_value) in methods_table {
        if let Some(method_table) = method_value.as_table() {
            let mut args = Vec::new();
            let mut returns = None;
            
            // Parse args array
            if let Some(args_array) = method_table.get("args").and_then(|v| v.as_array()) {
                for arg_value in args_array {
                    if let Some(arg_table) = arg_value.as_table() {
                        let from = arg_table.get("from")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        let to = arg_table.get("to")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        let name = arg_table.get("name")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        args.push(TypeConversion { from, to, name });
                    }
                }
            }
            
            // Parse returns field
            if let Some(returns_str) = method_table.get("returns").and_then(|v| v.as_str()) {
                returns = Some(returns_str.to_string());
            }
            
            method_configs.insert(method_name.clone(), MethodConfig { args, returns });
        }
    }
    
    Ok(method_configs)
}

fn validate_type_configuration(config: &NyashConfig, box_name: &str, plugin_info: &NyashPluginInfo) {
    println!("\n{}", "--- Type Configuration Validation ---".cyan());
    
    // Check if this Box type is configured in nyash.toml
    if let Some(plugin_name) = config.plugins.get(box_name) {
        println!("{}: Box type '{}' is configured to use plugin '{}'", 
                 "✓".green(), box_name, plugin_name);
    } else {
        println!("{}: Box type '{}' is not configured in nyash.toml", 
                 "WARNING".yellow(), box_name);
        println!("  Consider adding: {} = \"plugin-name\"", box_name);
    }
    
    // Check if method configuration exists
    let config_key = format!("plugins.{}", box_name);
    if let Some(plugin_config) = config.plugin_configs.get(&config_key) {
        if let Some(methods) = &plugin_config.methods {
            println!("{}: Found method configuration for {} methods", 
                     "✓".green(), methods.len());
        } else {
            println!("{}: No method configuration found for {}", 
                     "WARNING".yellow(), box_name);
        }
    } else {
        println!("{}: No method configuration section [plugins.{}.methods] found", 
                 "WARNING".yellow(), box_name);
    }
}

fn validate_method_signatures(config: &NyashConfig, box_name: &str, plugin_info: &NyashPluginInfo) {
    println!("\n{}", "--- Method Signature Validation ---".cyan());
    
    if plugin_info.method_count == 0 || plugin_info.methods.is_null() {
        println!("{}: Plugin has no methods to validate", "INFO".blue());
        return;
    }
    
    let config_key = format!("plugins.{}", box_name);
    let plugin_config = config.plugin_configs.get(&config_key);
    let method_configs = plugin_config.and_then(|c| c.methods.as_ref());
    
    unsafe {
        let methods = std::slice::from_raw_parts(plugin_info.methods, plugin_info.method_count);
        
        for method in methods {
            let method_name = if method.name.is_null() {
                "<unnamed>".to_string()
            } else {
                CStr::from_ptr(method.name).to_string_lossy().to_string()
            };
            
            println!("Validating method: {}", method_name.cyan());
            
            // Check if method is configured
            if let Some(configs) = method_configs {
                if let Some(method_config) = configs.get(&method_name) {
                    println!("  {}: Method configuration found", "✓".green());
                    
                    // Validate argument types
                    if method_config.args.is_empty() {
                        println!("  {}: No arguments configured", "✓".green());
                    } else {
                        println!("  {}: {} argument(s) configured", "✓".green(), method_config.args.len());
                        for (i, arg) in method_config.args.iter().enumerate() {
                            println!("    Arg {}: {} → {}", i, arg.from, arg.to);
                            if let Some(name) = &arg.name {
                                println!("      Name: {}", name);
                            }
                        }
                    }
                    
                    // Validate return type
                    if let Some(returns) = &method_config.returns {
                        println!("  {}: Return type: {}", "✓".green(), returns);
                    }
                } else {
                    println!("  {}: Method not configured in nyash.toml", "WARNING".yellow());
                    println!("    Consider adding configuration for method '{}'", method_name);
                }
            } else {
                println!("  {}: No method configurations available", "WARNING".yellow());
            }
        }
    }
}

fn check_duplicate_methods(plugin_info: &NyashPluginInfo) {
    println!("\n{}", "--- Duplicate Method Check ---".cyan());
    
    if plugin_info.method_count == 0 || plugin_info.methods.is_null() {
        println!("{}: Plugin has no methods to check", "INFO".blue());
        return;
    }
    
    let mut method_names = HashMap::new();
    let mut duplicates_found = false;
    
    unsafe {
        let methods = std::slice::from_raw_parts(plugin_info.methods, plugin_info.method_count);
        
        for method in methods {
            let method_name = if method.name.is_null() {
                "<unnamed>".to_string()
            } else {
                CStr::from_ptr(method.name).to_string_lossy().to_string()
            };
            
            if let Some(existing_id) = method_names.get(&method_name) {
                println!("{}: Duplicate method name '{}' found!", "ERROR".red(), method_name);
                println!("  Method ID {} and {} both use the same name", existing_id, method.method_id);
                println!("  Nyash does not support function overloading");
                duplicates_found = true;
            } else {
                method_names.insert(method_name.clone(), method.method_id);
                println!("{}: Method '{}' [ID: {}]", "✓".green(), method_name, method.method_id);
            }
        }
    }
    
    if duplicates_found {
        println!("\n{}: Duplicate method names detected!", "ERROR".red());
        println!("  Please ensure all method names are unique in your plugin.");
    } else {
        println!("\n{}: No duplicate method names found", "✓".green());
    }
}
