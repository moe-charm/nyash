/*!
 * Executable Builder - Creates standalone native executables
 * 
 * Embeds precompiled WASM modules into self-contained executables
 */

use super::{AotError, AotConfig};
use std::path::Path;
use std::fs;

/// Builder for creating standalone executable files
pub struct ExecutableBuilder<'a> {
    config: &'a AotConfig,
    precompiled_module: Option<Vec<u8>>,
    runtime_template: &'static str,
}

impl<'a> ExecutableBuilder<'a> {
    /// Create a new executable builder
    pub fn new(config: &'a AotConfig) -> Self {
        Self {
            config,
            precompiled_module: None,
            runtime_template: RUNTIME_TEMPLATE,
        }
    }
    
    /// Embed precompiled module data
    pub fn embed_precompiled_module(&mut self, module_data: Vec<u8>) -> Result<(), AotError> {
        self.precompiled_module = Some(module_data);
        Ok(())
    }
    
    /// Create the standalone executable
    pub fn create_executable<P: AsRef<Path>>(&self, output_path: P) -> Result<(), AotError> {
        let module_data = self.precompiled_module.as_ref()
            .ok_or_else(|| AotError::CompilationError("No precompiled module embedded".to_string()))?;
        
        // Generate the runtime code with embedded module
        let runtime_code = self.generate_runtime_code(module_data)?;
        
        // Write to temporary Rust source file
        let temp_dir = std::env::temp_dir();
        let temp_main = temp_dir.join("nyash_aot_main.rs");
        let temp_cargo = temp_dir.join("Cargo.toml");
        
        fs::write(&temp_main, runtime_code)?;
        fs::write(&temp_cargo, self.generate_cargo_toml())?;
        
        // Compile with Rust compiler
        self.compile_rust_executable(&temp_dir, output_path)?;
        
        // Clean up temporary files
        let _ = fs::remove_file(&temp_main);
        let _ = fs::remove_file(&temp_cargo);
        
        Ok(())
    }
    
    /// Generate the runtime code with embedded module
    fn generate_runtime_code(&self, module_data: &[u8]) -> Result<String, AotError> {
        let module_bytes = self.format_module_bytes(module_data);
        let compatibility_key = self.config.compatibility_key();
        
        let runtime_code = self.runtime_template
            .replace("{{MODULE_BYTES}}", &module_bytes)
            .replace("{{COMPATIBILITY_KEY}}", &compatibility_key)
            .replace("{{OPTIMIZATION_LEVEL}}", &self.config.optimization_level().to_string())
            .replace("{{TARGET_ARCH}}", self.config.target_arch())
            .replace("{{WASMTIME_VERSION}}", "18.0");
        
        Ok(runtime_code)
    }
    
    /// Format module bytes as Rust byte array literal
    fn format_module_bytes(&self, data: &[u8]) -> String {
        let mut result = String::with_capacity(data.len() * 6);
        result.push_str("&[\n    ");
        
        for (i, byte) in data.iter().enumerate() {
            if i > 0 && i % 16 == 0 {
                result.push_str("\n    ");
            }
            result.push_str(&format!("0x{:02x}, ", byte));
        }
        
        result.push_str("\n]");
        result
    }
    
    /// Generate Cargo.toml for the executable
    fn generate_cargo_toml(&self) -> String {
        format!(r#"[package]
name = "nyash-aot-executable"
version = "0.1.0"
edition = "2021"

[dependencies]
wasmtime = "18.0"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true

[[bin]]
name = "nyash-aot-executable"
path = "nyash_aot_main.rs"
"#)
    }
    
    /// Compile the Rust executable
    fn compile_rust_executable<P: AsRef<Path>, Q: AsRef<Path>>(&self, temp_dir: P, output_path: Q) -> Result<(), AotError> {
        let temp_dir = temp_dir.as_ref();
        let output_path = output_path.as_ref();
        
        // Use cargo to compile
        let mut cmd = std::process::Command::new("cargo");
        cmd.current_dir(temp_dir)
           .args(&["build", "--release", "--bin", "nyash-aot-executable"]);
        
        let output = cmd.output()
            .map_err(|e| AotError::CompilationError(format!("Failed to run cargo: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AotError::CompilationError(format!("Cargo build failed: {}", stderr)));
        }
        
        // Copy the compiled executable to the desired location
        let compiled_exe = temp_dir.join("target/release/nyash-aot-executable");
        let compiled_exe = if cfg!(windows) {
            compiled_exe.with_extension("exe")
        } else {
            compiled_exe
        };
        
        if !compiled_exe.exists() {
            return Err(AotError::CompilationError("Compiled executable not found".to_string()));
        }
        
        fs::copy(&compiled_exe, output_path)
            .map_err(|e| AotError::IOError(format!("Failed to copy executable: {}", e)))?;
        
        Ok(())
    }
}

/// Runtime template for generated executables
const RUNTIME_TEMPLATE: &str = r#"/*!
 * Nyash AOT Runtime - Generated executable
 * 
 * This file is automatically generated by the Nyash AOT compiler.
 * It contains a precompiled WebAssembly module and minimal runtime.
 */

use wasmtime::{Engine, Module, Instance, Store, Config, OptLevel, Strategy};
use std::process;

// Embedded precompiled module (generated by AOT compiler)
const MODULE_DATA: &[u8] = {{MODULE_BYTES}};

// Compilation metadata
const COMPATIBILITY_KEY: &str = "{{COMPATIBILITY_KEY}}";
const OPTIMIZATION_LEVEL: &str = "{{OPTIMIZATION_LEVEL}}";
const TARGET_ARCH: &str = "{{TARGET_ARCH}}";
const WASMTIME_VERSION: &str = "{{WASMTIME_VERSION}}";

fn main() {
    if let Err(e) = run_aot_module() {
        eprintln!("❌ AOT execution error: {}", e);
        process::exit(1);
    }
}

fn run_aot_module() -> Result<(), Box<dyn std::error::Error>> {
    // Create optimized wasmtime configuration
    let mut config = Config::new();
    config.strategy(Strategy::Cranelift);
    config.cranelift_opt_level(OptLevel::Speed);
    
    // Enable features used during compilation
    config.wasm_simd(true);
    config.wasm_bulk_memory(true);
    config.wasm_multi_memory(true);
    
    // Create engine with the configuration
    let engine = Engine::new(&config)?;
    
    // Deserialize the precompiled module
    let module = unsafe {
        Module::deserialize(&engine, MODULE_DATA)?
    };
    
    // Create store and instance
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[])?;
    
    // Look for the main function
    let main_func = instance
        .get_typed_func::<(), i32>(&mut store, "main")
        .or_else(|_| instance.get_typed_func::<(), i32>(&mut store, "_start"))
        .or_else(|_| instance.get_typed_func::<(), i32>(&mut store, "run"))
        .map_err(|_| "No main function found in module")?;
    
    // Execute the function
    let result = main_func.call(&mut store, ())?;
    
    println!("✅ AOT execution completed successfully!");
    println!("📊 Metadata:");
    println!("   Compatibility: {}", COMPATIBILITY_KEY);
    println!("   Optimization: {}", OPTIMIZATION_LEVEL);
    println!("   Target: {}", TARGET_ARCH);
    println!("   Wasmtime: {}", WASMTIME_VERSION);
    println!("   Result: {}", result);
    
    process::exit(result);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_executable_builder_creation() {
        let config = AotConfig::new().expect("Failed to create config");
        let _builder = ExecutableBuilder::new(&config);
        // Should not panic
        assert!(true);
    }
    
    #[test]
    fn test_embed_module() {
        let config = AotConfig::new().expect("Failed to create config");
        let mut builder = ExecutableBuilder::new(&config);
        let test_data = vec![1, 2, 3, 4, 5];
        
        builder.embed_precompiled_module(test_data).expect("Failed to embed module");
        assert!(builder.precompiled_module.is_some());
    }
    
    #[test]
    fn test_format_module_bytes() {
        let config = AotConfig::new().expect("Failed to create config");
        let builder = ExecutableBuilder::new(&config);
        let test_data = vec![0x00, 0x61, 0x73, 0x6d];
        
        let formatted = builder.format_module_bytes(&test_data);
        assert!(formatted.contains("0x00"));
        assert!(formatted.contains("0x61"));
        assert!(formatted.contains("0x73"));
        assert!(formatted.contains("0x6d"));
    }
    
    #[test]
    fn test_cargo_toml_generation() {
        let config = AotConfig::new().expect("Failed to create config");
        let builder = ExecutableBuilder::new(&config);
        let cargo_toml = builder.generate_cargo_toml();
        
        assert!(cargo_toml.contains("nyash-aot-executable"));
        assert!(cargo_toml.contains("wasmtime"));
        assert!(cargo_toml.contains("opt-level = 3"));
    }
    
    #[test]
    fn test_runtime_code_generation() {
        let config = AotConfig::new().expect("Failed to create config");
        let builder = ExecutableBuilder::new(&config);
        let test_data = vec![0x00, 0x61, 0x73, 0x6d];
        
        let runtime_code = builder.generate_runtime_code(&test_data).expect("Failed to generate runtime");
        assert!(runtime_code.contains("MODULE_DATA"));
        assert!(runtime_code.contains("0x00"));
        assert!(runtime_code.contains("18.0"));  // Wasmtime version
    }
}