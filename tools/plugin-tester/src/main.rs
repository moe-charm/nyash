//! Nyash Plugin Tester - Multi-Box Type Support (v2)
//! 
//! プラグイン開発者向けの診断ツール
//! 単一Box型・複数Box型の両方をサポート

use clap::{Parser, Subcommand};
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
    methods: HashMap<String, MethodDef>,
}

#[derive(Debug, Deserialize)]
struct MethodDef {
    args: Vec<ArgDef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    returns: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ArgDef {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    from: String,
    to: String,
}

// ============ CLI ============

#[derive(Parser)]
#[command(name = "plugin-tester")]
#[command(about = "Nyash plugin testing tool", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check plugin exports and basic functionality
    Check {
        /// Path to plugin .so file
        plugin: PathBuf,
        
        /// Check for multiple Box types (v2 plugin)
        #[arg(short = 'm', long)]
        multi: bool,
    },
    /// Test Box lifecycle (birth/fini)
    Lifecycle {
        /// Path to plugin .so file
        plugin: PathBuf,
        
        /// Specify Box type name (for multi-box plugins)
        #[arg(short = 'b', long)]
        box_type: Option<String>,
    },
    /// Test file I/O operations
    Io {
        /// Path to plugin .so file
        plugin: PathBuf,
    },
    /// Debug TLV encoding/decoding
    TlvDebug {
        /// Path to plugin .so file (optional)
        #[arg(short, long)]
        plugin: Option<PathBuf>,
        
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
        Commands::Check { plugin, multi } => {
            if multi {
                check_multi_box_plugin(&plugin)
            } else {
                check_plugin(&plugin)
            }
        },
        Commands::Lifecycle { plugin, box_type } => test_lifecycle(&plugin, box_type),
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

fn tlv_decode_i32(data: &[u8]) -> Result<i32, String> {
    if data.len() < 12 {
        return Err("Buffer too short for I32 TLV".to_string());
    }
    let version = u16::from_le_bytes([data[0], data[1]]);
    let argc = u16::from_le_bytes([data[2], data[3]]);
    if version != TLV_VERSION || argc != 1 {
        return Err(format!("Invalid TLV header: v{} argc={}", version, argc));
    }
    let tag = data[4];
    if tag != Tag::I32 as u8 {
        return Err(format!("Expected I32 tag, got {}", tag));
    }
    let len = u16::from_le_bytes([data[6], data[7]]);
    if len != 4 {
        return Err(format!("Invalid I32 length: {}", len));
    }
    Ok(i32::from_le_bytes([data[8], data[9], data[10], data[11]]))
}

// ============ Plugin Check Functions ============

fn check_plugin(path: &PathBuf) {
    println!("{}", "=== Plugin Check (Single Box Type) ===".bold());
    println!("Plugin: {}", path.display());
    
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

// ============ Multi-Box Plugin Support (v2) ============

fn check_multi_box_plugin(path: &PathBuf) {
    println!("{}", "=== Plugin Check (Multi-Box Type v2) ===".bold());
    println!("Plugin: {}", path.display());
    
    let library = match unsafe { Library::new(path) } {
        Ok(lib) => lib,
        Err(e) => {
            eprintln!("{}: Failed to load plugin: {}", "ERROR".red(), e);
            return;
        }
    };
    
    println!("{}: Plugin loaded successfully", "✓".green());
    
    // Check for v2 functions
    unsafe {
        // Check if this is a v2 plugin
        let has_v2 = library.get::<Symbol<unsafe extern "C" fn() -> u32>>(b"nyash_plugin_get_box_count").is_ok();
        
        if !has_v2 {
            println!("{}: This is not a v2 multi-box plugin", "INFO".yellow());
            println!("    Falling back to single-box check...\n");
            drop(library);
            check_plugin(path);
            return;
        }
        
        // Get box count
        let get_count_fn: Symbol<unsafe extern "C" fn() -> u32> = 
            library.get(b"nyash_plugin_get_box_count").unwrap();
        
        let box_count = get_count_fn();
        println!("{}: Plugin provides {} Box types", "✓".green(), box_count);
        
        // Get box info function
        let get_info_fn: Symbol<unsafe extern "C" fn(u32) -> *const NyashPluginInfo> = 
            match library.get(b"nyash_plugin_get_box_info") {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("{}: nyash_plugin_get_box_info not found: {}", "ERROR".red(), e);
                    return;
                }
            };
        
        // Initialize plugin
        let init_fn: Symbol<unsafe extern "C" fn(*const NyashHostVtable, *mut c_void) -> i32> = 
            match library.get(b"nyash_plugin_init") {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("{}: nyash_plugin_init not found: {}", "ERROR".red(), e);
                    return;
                }
            };
        
        let result = init_fn(&HOST_VTABLE, std::ptr::null_mut());
        if result != 0 {
            eprintln!("{}: Plugin initialization failed", "ERROR".red());
            return;
        }
        
        println!("\n{}", "Box Types:".bold());
        
        // Display info for each Box type
        for i in 0..box_count {
            let info_ptr = get_info_fn(i);
            if info_ptr.is_null() {
                eprintln!("{}: Failed to get info for box index {}", "ERROR".red(), i);
                continue;
            }
            
            let info = &*info_ptr;
            let box_name = if info.type_name.is_null() {
                "<unknown>".to_string()
            } else {
                CStr::from_ptr(info.type_name).to_string_lossy().to_string()
            };
            
            println!("\n  {}. {} (ID: {})", i + 1, box_name.cyan(), info.type_id);
            println!("     Methods: {}", info.method_count);
            
            // Display methods
            if info.method_count > 0 && !info.methods.is_null() {
                let methods = std::slice::from_raw_parts(info.methods, info.method_count);
                
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
                    
                    println!("     - {} [ID: {}]{}",
                        method_name,
                        method.method_id,
                        method_type
                    );
                }
            }
        }
        
        // Check for get_type_id function
        if let Ok(get_type_id_fn) = library.get::<Symbol<unsafe extern "C" fn(*const c_char) -> u32>>(b"nyash_plugin_get_type_id") {
            println!("\n{}: Plugin supports type name resolution", "✓".green());
            
            // Test type name resolution
            for test_name in ["TestBoxA", "TestBoxB", "UnknownBox"] {
                let c_name = CString::new(test_name).unwrap();
                let type_id = get_type_id_fn(c_name.as_ptr());
                if type_id != 0 {
                    println!("  {} -> type_id: {}", test_name, type_id);
                } else {
                    println!("  {} -> not found", test_name.dimmed());
                }
            }
        }
    }
    
    println!("\n{}", "Multi-box check completed!".green().bold());
}

fn test_lifecycle(path: &PathBuf, box_type: Option<String>) {
    println!("{}", "=== Lifecycle Test ===".bold());
    
    // Load plugin
    let library = match unsafe { Library::new(path) } {
        Ok(lib) => lib,
        Err(e) => {
            eprintln!("{}: Failed to load plugin: {}", "ERROR".red(), e);
            return;
        }
    };
    
    unsafe {
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
            eprintln!("{}: Plugin initialization failed", "ERROR".red());
            return;
        }
        
        // Get invoke function
        let invoke_fn: Symbol<unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32> = 
            match library.get(b"nyash_plugin_invoke") {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("{}: nyash_plugin_invoke not found: {}", "ERROR".red(), e);
                    return;
                }
            };
        
        // Determine type_id
        let type_id = if let Some(ref box_name) = box_type {
            // For multi-box plugins, resolve type_id from name
            if let Ok(get_type_id_fn) = library.get::<Symbol<unsafe extern "C" fn(*const c_char) -> u32>>(b"nyash_plugin_get_type_id") {
                let c_name = CString::new(box_name.as_str()).unwrap();
                let id = get_type_id_fn(c_name.as_ptr());
                if id == 0 {
                    eprintln!("{}: Box type '{}' not found", "ERROR".red(), box_name);
                    return;
                }
                id
            } else {
                eprintln!("{}: Multi-box plugin doesn't support type name resolution", "ERROR".red());
                return;
            }
        } else {
            plugin_info.type_id
        };
        
        println!("Testing lifecycle for type_id: {}", type_id);
        
        // Test birth
        println!("\n{}", "1. Testing birth (constructor)...".cyan());
        
        let mut result_buf = vec![0u8; 1024];
        let mut result_len = result_buf.len();
        
        let result = invoke_fn(
            type_id,
            0, // METHOD_BIRTH
            0, // instance_id = 0 for birth
            std::ptr::null(),
            0,
            result_buf.as_mut_ptr(),
            &mut result_len
        );
        
        if result != 0 {
            eprintln!("{}: Birth failed with code {}", "ERROR".red(), result);
            return;
        }
        
        // Parse instance_id from result
        let instance_id = if result_len >= 4 {
            u32::from_le_bytes([result_buf[0], result_buf[1], result_buf[2], result_buf[3]])
        } else {
            eprintln!("{}: Invalid birth response", "ERROR".red());
            return;
        };
        
        println!("{}: Birth successful, instance_id = {}", "✓".green(), instance_id);
        
        // Test a method if FileBox
        if plugin_info.type_name != std::ptr::null() {
            let box_name = CStr::from_ptr(plugin_info.type_name).to_string_lossy();
            if box_name == "FileBox" {
                test_file_operations(&invoke_fn, type_id, instance_id);
            }
        }
        
        // Test fini
        println!("\n{}", "2. Testing fini (destructor)...".cyan());
        
        result_len = result_buf.len();
        let result = invoke_fn(
            type_id,
            u32::MAX, // METHOD_FINI
            instance_id,
            std::ptr::null(),
            0,
            result_buf.as_mut_ptr(),
            &mut result_len
        );
        
        if result != 0 {
            eprintln!("{}: Fini failed with code {}", "ERROR".red(), result);
        } else {
            println!("{}: Fini successful", "✓".green());
        }
    }
    
    println!("\n{}", "Lifecycle test completed!".green().bold());
}

fn test_file_operations(
    invoke_fn: &Symbol<unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32>,
    type_id: u32,
    instance_id: u32
) {
    println!("\n{}", "Testing file operations...".cyan());
    
    // Test open
    let mut args = Vec::new();
    tlv_encode_two_strings("test_lifecycle.txt", "w", &mut args);
    
    let mut result_buf = vec![0u8; 1024];
    let mut result_len = result_buf.len();
    
    unsafe {
        let result = invoke_fn(
            type_id,
            1, // METHOD_OPEN
            instance_id,
            args.as_ptr(),
            args.len(),
            result_buf.as_mut_ptr(),
            &mut result_len
        );
        
        if result == 0 {
            println!("{}: Open successful", "✓".green());
        } else {
            eprintln!("{}: Open failed", "ERROR".red());
        }
    }
}

fn test_file_io(path: &PathBuf) {
    println!("{}", "=== File I/O Test ===".bold());
    println!("(Full I/O test implementation omitted for brevity)");
    println!("Use lifecycle test with FileBox for basic I/O testing");
}

fn test_tlv_debug(plugin: &Option<PathBuf>, message: &str) {
    println!("{}", "=== TLV Debug ===".bold());
    
    // Encode string
    let mut encoded = Vec::new();
    tlv_encode_string(message, &mut encoded);
    
    println!("Original message: {}", message.cyan());
    println!("Encoded bytes ({} bytes):", encoded.len());
    
    // Display hex dump
    for (i, chunk) in encoded.chunks(16).enumerate() {
        print!("{:04x}: ", i * 16);
        for byte in chunk {
            print!("{:02x} ", byte);
        }
        println!();
    }
    
    // Decode header
    if encoded.len() >= 4 {
        let version = u16::from_le_bytes([encoded[0], encoded[1]]);
        let argc = u16::from_le_bytes([encoded[2], encoded[3]]);
        println!("\nTLV Header:");
        println!("  Version: {}", version);
        println!("  Arg count: {}", argc);
    }
}

fn typecheck_plugin(plugin_path: &PathBuf, config_path: &PathBuf) {
    println!("{}", "=== Type Check ===".bold());
    
    // Load nyash.toml
    let config_content = match fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("{}: Failed to read config: {}", "ERROR".red(), e);
            return;
        }
    };
    
    let config_value: toml::Value = match toml::from_str(&config_content) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("{}: Failed to parse TOML: {}", "ERROR".red(), e);
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
        // Get plugin info
        let init_fn: Symbol<unsafe extern "C" fn(*const NyashHostVtable, *mut NyashPluginInfo) -> i32> = 
            match library.get(b"nyash_plugin_init") {
                Ok(f) => f,
                Err(_) => {
                    eprintln!("{}: Plugin doesn't export nyash_plugin_init", "ERROR".red());
                    return;
                }
            };
        
        let mut plugin_info = std::mem::zeroed::<NyashPluginInfo>();
        let result = init_fn(&HOST_VTABLE, &mut plugin_info);
        
        if result != 0 {
            eprintln!("{}: Plugin initialization failed", "ERROR".red());
            return;
        }
        
        let box_name = if plugin_info.type_name.is_null() {
            eprintln!("{}: Plugin doesn't provide type name", "ERROR".red());
            return;
        } else {
            CStr::from_ptr(plugin_info.type_name).to_string_lossy().to_string()
        };
        
        println!("Plugin Box type: {}", box_name.cyan());
        
        // Check if box is configured in nyash.toml
        if let Some(plugins) = config_value.get("plugins").and_then(|v| v.as_table()) {
            if let Some(plugin_name) = plugins.get(&box_name).and_then(|v| v.as_str()) {
                println!("{}: {} is configured as '{}'", "✓".green(), box_name, plugin_name);
                
                // Check method definitions
                let methods_key = format!("plugins.{}.methods", box_name);
                if let Some(methods) = config_value.get("plugins")
                    .and_then(|v| v.get(&box_name))
                    .and_then(|v| v.get("methods"))
                    .and_then(|v| v.as_table()) {
                    
                    println!("\n{}", "Configured methods:".bold());
                    
                    // Get actual methods from plugin
                    let actual_methods = if plugin_info.method_count > 0 && !plugin_info.methods.is_null() {
                        let methods = std::slice::from_raw_parts(plugin_info.methods, plugin_info.method_count);
                        methods.iter()
                            .filter_map(|m| {
                                if m.name.is_null() {
                                    None
                                } else {
                                    Some(CStr::from_ptr(m.name).to_string_lossy().to_string())
                                }
                            })
                            .collect::<Vec<_>>()
                    } else {
                        vec![]
                    };
                    
                    for (method_name, _method_def) in methods {
                        let status = if actual_methods.contains(method_name) {
                            format!("{}", "✓".green())
                        } else {
                            format!("{}", "✗".red())
                        };
                        println!("  {} {}", status, method_name);
                    }
                    
                    // Check for duplicate method names
                    let mut seen = std::collections::HashSet::new();
                    for method in &actual_methods {
                        if !seen.insert(method) {
                            eprintln!("{}: Duplicate method name: {}", "WARNING".yellow(), method);
                            eprintln!("    Note: Nyash doesn't support function overloading");
                        }
                    }
                } else {
                    eprintln!("{}: No method definitions found for {}", "WARNING".yellow(), box_name);
                }
            } else {
                eprintln!("{}: {} is not configured in nyash.toml", "WARNING".yellow(), box_name);
            }
        }
    }
    
    println!("\n{}", "Type check completed!".green().bold());
}