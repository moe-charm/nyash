//! Nyash Plugin Tester
//! 
//! プラグイン開発者向けの診断ツール
//! Box名を決め打ちせず、プラグインから取得する

use clap::Parser;
use colored::*;
use libloading::{Library, Symbol};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::path::PathBuf;

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
    }
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
    
    // TODO: birth/finiのテスト実装
    println!("{}: Lifecycle test not yet implemented", "TODO".yellow());
}