//! Nyash Plugin Tester v2 - nyash.toml中心設計対応版
//! 
//! 究極のシンプル設計:
//! - Host VTable廃止
//! - nyash_plugin_invokeのみ使用
//! - すべてのメタ情報はnyash.tomlから取得

use clap::{Parser, Subcommand};
use colored::*;
use libloading::{Library, Symbol};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// ============ nyash.toml v2 Types ============

#[derive(Debug, Deserialize)]
struct NyashConfigV2 {
    libraries: HashMap<String, LibraryDefinition>,
}

#[derive(Debug, Deserialize)]
struct LibraryDefinition {
    boxes: Vec<String>,
    path: String,
}

#[derive(Debug, Deserialize)]
struct BoxTypeConfig {
    type_id: u32,
    #[serde(default = "default_abi_version")]
    abi_version: u32,
    methods: HashMap<String, MethodDefinition>,
}

fn default_abi_version() -> u32 { 1 }

#[derive(Debug, Deserialize)]
struct MethodDefinition {
    method_id: u32,
}

// ============ CLI ============

#[derive(Parser)]
#[command(name = "plugin-tester-v2")]
#[command(about = "Nyash plugin testing tool v2 - nyash.toml centric", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check plugin with nyash.toml v2
    Check {
        /// Path to nyash.toml file
        #[arg(short, long, default_value = "../../nyash.toml")]
        config: PathBuf,
        
        /// Library name (e.g., "libnyash_filebox_plugin.so")
        #[arg(short, long)]
        library: Option<String>,
    },
    /// Test Box lifecycle with nyash.toml v2
    Lifecycle {
        /// Path to nyash.toml file
        #[arg(short, long, default_value = "../../nyash.toml")]
        config: PathBuf,
        
        /// Box type name (e.g., "FileBox")
        box_type: String,
    },
    /// Validate all plugins in nyash.toml
    ValidateAll {
        /// Path to nyash.toml file
        #[arg(short, long, default_value = "../../nyash.toml")]
        config: PathBuf,
    },
}

// ============ TLV Helpers ============

fn tlv_encode_empty() -> Vec<u8> {
    vec![1, 0, 0, 0]  // version=1, argc=0
}

fn tlv_decode_u32(data: &[u8]) -> Result<u32, String> {
    if data.len() >= 4 {
        Ok(u32::from_le_bytes([data[0], data[1], data[2], data[3]]))
    } else {
        Err("Buffer too short for u32".to_string())
    }
}

// ============ Main Functions ============

fn main() {
    let args = Args::parse();
    
    match args.command {
        Commands::Check { config, library } => check_v2(&config, library.as_deref()),
        Commands::Lifecycle { config, box_type } => test_lifecycle_v2(&config, &box_type),
        Commands::ValidateAll { config } => validate_all(&config),
    }
}

fn check_v2(config_path: &PathBuf, library_filter: Option<&str>) {
    println!("{}", "=== Plugin Check v2 (nyash.toml centric) ===".bold());
    
    // Load nyash.toml v2
    let config_content = match fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("{}: Failed to read config: {}", "ERROR".red(), e);
            return;
        }
    };
    
    let config: NyashConfigV2 = match toml::from_str(&config_content) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("{}: Failed to parse nyash.toml v2: {}", "ERROR".red(), e);
            return;
        }
    };
    
    println!("{}: Loaded {} libraries from nyash.toml", "✓".green(), config.libraries.len());
    
    // Also parse raw TOML for nested box configs
    let raw_config: toml::Value = match toml::from_str(&config_content) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("{}: Failed to parse TOML value: {}", "ERROR".red(), e);
            return;
        }
    };
    
    // Check each library
    for (lib_name, lib_def) in &config.libraries {
        if let Some(filter) = library_filter {
            if lib_name != filter {
                continue;
            }
        }
        
        println!("\n{}: {}", "Library".bold(), lib_name.cyan());
        println!("  Path: {}", lib_def.path);
        println!("  Box types: {:?}", lib_def.boxes);
        
        // Try to load the plugin
        let library = match unsafe { Library::new(&lib_def.path) } {
            Ok(lib) => lib,
            Err(e) => {
                eprintln!("  {}: Failed to load: {}", "ERROR".red(), e);
                continue;
            }
        };
        
        println!("  {}: Plugin loaded successfully", "✓".green());
        
        // Check for nyash_plugin_invoke (the only required function!)
        match unsafe { library.get::<Symbol<unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32>>(b"nyash_plugin_invoke") } {
            Ok(_) => println!("  {}: nyash_plugin_invoke found", "✓".green()),
            Err(_) => {
                eprintln!("  {}: nyash_plugin_invoke NOT FOUND - not a valid v2 plugin!", "ERROR".red());
                continue;
            }
        }
        
        // Check each box type from nyash.toml
        for box_name in &lib_def.boxes {
            println!("\n  {}: {}", "Box Type".bold(), box_name.cyan());
            
            // Get box config from nested TOML
            let box_config = get_box_config(&raw_config, lib_name, box_name);
            
            if let Some(config) = box_config {
                println!("    Type ID: {}", config.type_id);
                println!("    ABI Version: {}", config.abi_version);
                println!("    Methods: {}", config.methods.len());
                
                // List methods
                for (method_name, method_def) in &config.methods {
                    let method_type = match method_def.method_id {
                        0 => " (constructor)".yellow(),
                        4294967295 => " (destructor)".yellow(),
                        _ => "".normal(),
                    };
                    
                    println!("    - {}: method_id={}{}", 
                        method_name, 
                        method_def.method_id,
                        method_type
                    );
                }
            } else {
                eprintln!("    {}: No configuration found for this box type", "WARNING".yellow());
            }
        }
    }
    
    println!("\n{}", "Check completed!".green().bold());
}

fn test_lifecycle_v2(config_path: &PathBuf, box_type: &str) {
    println!("{}", "=== Lifecycle Test v2 ===".bold());
    println!("Box type: {}", box_type.cyan());
    
    // Load nyash.toml
    let config_content = match fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("{}: Failed to read config: {}", "ERROR".red(), e);
            return;
        }
    };
    
    let config: NyashConfigV2 = match toml::from_str(&config_content) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("{}: Failed to parse nyash.toml: {}", "ERROR".red(), e);
            return;
        }
    };
    
    let raw_config: toml::Value = toml::from_str(&config_content).unwrap();
    
    // Find library that provides this box type
    let (lib_name, lib_def) = match find_library_for_box(&config, box_type) {
        Some((name, def)) => (name, def),
        None => {
            eprintln!("{}: Box type '{}' not found in nyash.toml", "ERROR".red(), box_type);
            return;
        }
    };
    
    println!("Found in library: {}", lib_name.cyan());
    
    // Get box configuration
    let box_config = match get_box_config(&raw_config, lib_name, box_type) {
        Some(cfg) => cfg,
        None => {
            eprintln!("{}: No configuration for box type", "ERROR".red());
            return;
        }
    };
    
    println!("Type ID: {}", box_config.type_id);
    
    // Load plugin
    let library = match unsafe { Library::new(&lib_def.path) } {
        Ok(lib) => lib,
        Err(e) => {
            eprintln!("{}: Failed to load plugin: {}", "ERROR".red(), e);
            return;
        }
    };
    
    // Get invoke function
    let invoke_fn: Symbol<unsafe extern "C" fn(u32, u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32> = 
        match unsafe { library.get(b"nyash_plugin_invoke") } {
            Ok(f) => f,
            Err(_) => {
                eprintln!("{}: nyash_plugin_invoke not found", "ERROR".red());
                return;
            }
        };
    
    unsafe {
        // Test birth
        println!("\n{}", "1. Testing birth (constructor)...".cyan());
        
        let args = tlv_encode_empty();  // No arguments for FileBox birth
        let mut result_buf = vec![0u8; 1024];
        let mut result_len = result_buf.len();
        
        let result = invoke_fn(
            box_config.type_id,
            0, // method_id = 0 (birth)
            0, // instance_id = 0 (static/birth)
            args.as_ptr(),
            args.len(),
            result_buf.as_mut_ptr(),
            &mut result_len
        );
        
        if result != 0 {
            eprintln!("{}: Birth failed with code {}", "ERROR".red(), result);
            return;
        }
        
        // Parse instance_id from result
        let instance_id = match tlv_decode_u32(&result_buf[..result_len]) {
            Ok(id) => id,
            Err(e) => {
                eprintln!("{}: Failed to decode instance_id: {}", "ERROR".red(), e);
                return;
            }
        };
        
        println!("{}: Birth successful, instance_id = {}", "✓".green(), instance_id);
        
        // Test fini
        println!("\n{}", "2. Testing fini (destructor)...".cyan());
        
        result_len = result_buf.len();
        let result = invoke_fn(
            box_config.type_id,
            4294967295, // method_id = 0xFFFFFFFF (fini)
            instance_id,
            args.as_ptr(),
            args.len(),
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

fn validate_all(config_path: &PathBuf) {
    println!("{}", "=== Validate All Plugins ===".bold());
    check_v2(config_path, None);
}

// ============ Helper Functions ============

fn find_library_for_box<'a>(config: &'a NyashConfigV2, box_type: &str) -> Option<(&'a str, &'a LibraryDefinition)> {
    config.libraries.iter()
        .find(|(_, lib)| lib.boxes.contains(&box_type.to_string()))
        .map(|(name, lib)| (name.as_str(), lib))
}

fn get_box_config(raw_config: &toml::Value, lib_name: &str, box_name: &str) -> Option<BoxTypeConfig> {
    raw_config
        .get("libraries")
        .and_then(|v| v.get(lib_name))
        .and_then(|v| v.get(box_name))
        .and_then(|v| v.clone().try_into::<BoxTypeConfig>().ok())
}