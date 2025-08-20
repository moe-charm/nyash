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
use std::path::{Path, PathBuf};

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

fn tlv_encode_one_handle(type_id: u32, instance_id: u32) -> Vec<u8> {
    // BID-1 TLV header: u16 ver=1, u16 argc=1
    // Entry: tag=8(Handle), rsv=0, size=u16(8), payload=[type_id(4), instance_id(4)]
    let mut buf = Vec::with_capacity(4 + 4 + 8);
    buf.extend_from_slice(&1u16.to_le_bytes()); // ver
    buf.extend_from_slice(&1u16.to_le_bytes()); // argc
    buf.push(8u8); // tag=Handle
    buf.push(0u8); // reserved
    buf.extend_from_slice(&(8u16).to_le_bytes()); // size
    buf.extend_from_slice(&type_id.to_le_bytes());
    buf.extend_from_slice(&instance_id.to_le_bytes());
    buf
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
    
    // Base dir for relative plugin paths
    let config_base = config_path.parent().unwrap_or(Path::new("."));

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
        let lib_path = if Path::new(&lib_def.path).is_absolute() {
            PathBuf::from(&lib_def.path)
        } else {
            config_base.join(&lib_def.path)
        };
        let library = match unsafe { Library::new(&lib_path) } {
            Ok(lib) => lib,
            Err(e) => {
                eprintln!("  {}: Failed to load: {} (path: {})", "ERROR".red(), e, lib_path.display());
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
    
    // Resolve plugin path relative to config dir
    let config_base = config_path.parent().unwrap_or(Path::new("."));
    let lib_path = if Path::new(&lib_def.path).is_absolute() {
        PathBuf::from(&lib_def.path)
    } else { config_base.join(&lib_def.path) };

    // Load plugin
    let library = match unsafe { Library::new(&lib_path) } {
        Ok(lib) => lib,
        Err(e) => {
            eprintln!("{}: Failed to load plugin: {} (path: {})", "ERROR".red(), e, lib_path.display());
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

        // Optional: If method 'copyFrom' exists, create another instance and pass it as Box arg
        if box_config.methods.contains_key("copyFrom") {
            println!("\n{}", "1b. Testing method with Box arg: copyFrom(other) ...".cyan());

            // Birth another instance to serve as argument handle
            let args2 = tlv_encode_empty();
            let mut out2 = vec![0u8; 1024];
            let mut out2_len = out2.len();
            let rc2 = invoke_fn(
                box_config.type_id,
                0,
                0,
                args2.as_ptr(),
                args2.len(),
                out2.as_mut_ptr(),
                &mut out2_len,
            );
            if rc2 == 0 {
                if let Ok(other_id) = tlv_decode_u32(&out2[..out2_len]) {
                    // Encode one Box handle as argument
                    let arg_buf = tlv_encode_one_handle(box_config.type_id, other_id);
                    let mut ret = vec![0u8; 1024];
                    let mut ret_len = ret.len();
                    let method_id = box_config.methods.get("copyFrom").unwrap().method_id;
                    let rc_call = invoke_fn(
                        box_config.type_id,
                        method_id,
                        instance_id,
                        arg_buf.as_ptr(),
                        arg_buf.len(),
                        ret.as_mut_ptr(),
                        &mut ret_len,
                    );
                    if rc_call == 0 {
                        println!("{}: copyFrom call succeeded (arg=BoxRef)", "✓".green());
                    } else {
                        eprintln!("{}: copyFrom call failed (rc={})", "WARN".yellow(), rc_call);
                    }
                } else {
                    eprintln!("{}: Failed to decode other instance_id", "WARN".yellow());
                }
            } else {
                eprintln!("{}: Failed to create other instance for copyFrom (rc={})", "WARN".yellow(), rc2);
            }
        }

        // Optional: If method 'cloneSelf' exists, call it and verify Handle return
        if box_config.methods.contains_key("cloneSelf") {
            println!("\n{}", "1c. Testing method returning Box: cloneSelf() ...".cyan());
            let args0 = tlv_encode_empty();
            let mut out = vec![0u8; 1024];
            let mut out_len = out.len();
            let method_id = box_config.methods.get("cloneSelf").unwrap().method_id;
            let rc = invoke_fn(
                box_config.type_id,
                method_id,
                instance_id,
                args0.as_ptr(),
                args0.len(),
                out.as_mut_ptr(),
                &mut out_len,
            );
            if rc == 0 {
                // Parse TLV header + entry, expecting tag=8 size=8
                if out_len >= 12 && out[4] == 8 && out[7] as usize == 8 { // simplistic check
                    println!("{}: cloneSelf returned a Handle (tag=8)", "✓".green());
                } else {
                    eprintln!("{}: cloneSelf returned unexpected format", "WARN".yellow());
                }
            } else {
                eprintln!("{}: cloneSelf call failed (rc={})", "WARN".yellow(), rc);
            }
        }
        
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
