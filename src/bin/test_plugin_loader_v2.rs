//! Test program for multi-box plugin loader

use nyash_rust::config::NyashConfigV2;
use nyash_rust::runtime::get_global_loader;

fn main() {
    env_logger::init();
    
    println!("=== Multi-Box Plugin Loader Test ===\n");
    
    // Load configuration
    let config_path = "test_nyash_v2.toml";
    println!("Loading configuration from: {}", config_path);
    
    let config = match NyashConfigV2::from_file(config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            return;
        }
    };
    
    println!("Configuration loaded successfully!");
    println!("Is v2 format: {}", config.is_v2_format());
    
    if let Some(libs) = &config.plugins.libraries {
        println!("\nLibraries found:");
        for (name, lib) in libs {
            println!("  {} -> {}", name, lib.plugin_path);
            println!("    Provides: {:?}", lib.provides);
        }
    }
    
    // Load plugins
    println!("\nLoading plugins...");
    let loader = get_global_loader();
    
    match loader.load_from_config(&config) {
        Ok(_) => println!("Plugins loaded successfully!"),
        Err(e) => {
            eprintln!("Failed to load plugins: {}", e);
            return;
        }
    }
    
    // Test box type resolution
    println!("\nTesting box type resolution:");
    for box_type in ["TestBoxA", "TestBoxB", "UnknownBox"] {
        if let Some(lib_name) = loader.get_library_for_box(box_type) {
            println!("  {} -> library: {}", box_type, lib_name);
        } else {
            println!("  {} -> not found", box_type);
        }
    }
    
    println!("\nTest completed!");
}