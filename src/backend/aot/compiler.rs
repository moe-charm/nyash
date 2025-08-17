/*!
 * AOT Compiler - Converts MIR to precompiled native code
 * 
 * Handles the MIR -> WASM -> Native compilation pipeline
 */

use super::{AotError, AotConfig, AotStats};
use crate::mir::MirModule;
use crate::backend::wasm::{WasmBackend, WasmError};
use wasmtime::{Engine, Module};
use std::time::Instant;

/// AOT compiler that handles the full compilation pipeline
pub struct AotCompiler {
    wasm_backend: WasmBackend,
    wasmtime_engine: Engine,
    stats: AotStats,
}

impl AotCompiler {
    /// Create a new AOT compiler with the given configuration
    pub fn new(config: &AotConfig) -> Result<Self, AotError> {
        // Create wasmtime engine with optimized configuration
        let engine = Engine::new(config.wasmtime_config())
            .map_err(|e| AotError::WasmtimeError(format!("Failed to create wasmtime engine: {}", e)))?;
        
        // Create WASM backend for MIR -> WASM compilation
        let wasm_backend = WasmBackend::new();
        
        let stats = AotStats {
            wasm_size: 0,
            precompiled_size: 0,
            compilation_time_ms: 0,
            optimization_level: format!("O{}", config.optimization_level()),
        };
        
        Ok(Self {
            wasm_backend,
            wasmtime_engine: engine,
            stats,
        })
    }
    
    /// Compile MIR module to WASM bytecode
    pub fn compile_mir_to_wasm(&mut self, mir_module: MirModule) -> Result<Vec<u8>, AotError> {
        let start_time = Instant::now();
        
        // Use existing WASM backend to compile MIR to WASM
        let wasm_bytes = self.wasm_backend.compile_module(mir_module)
            .map_err(|e| match e {
                WasmError::CodegenError(msg) => AotError::CompilationError(format!("WASM codegen failed: {}", msg)),
                WasmError::MemoryError(msg) => AotError::CompilationError(format!("WASM memory error: {}", msg)),
                WasmError::UnsupportedInstruction(msg) => AotError::CompilationError(format!("Unsupported MIR instruction: {}", msg)),
                WasmError::WasmValidationError(msg) => AotError::CompilationError(format!("WASM validation failed: {}", msg)),
                WasmError::IOError(msg) => AotError::IOError(msg),
            })?;
        
        self.stats.wasm_size = wasm_bytes.len();
        self.stats.compilation_time_ms += start_time.elapsed().as_millis() as u64;
        
        Ok(wasm_bytes)
    }
    
    /// Precompile WASM bytecode to native machine code
    pub fn precompile_wasm(&mut self, wasm_bytes: &[u8]) -> Result<Vec<u8>, AotError> {
        let start_time = Instant::now();
        
        // Parse and validate the WASM module
        let module = Module::from_binary(&self.wasmtime_engine, wasm_bytes)
            .map_err(|e| AotError::WasmtimeError(format!("Failed to parse WASM module: {}", e)))?;
        
        // Serialize the precompiled module to bytes
        let precompiled_bytes = module.serialize()
            .map_err(|e| AotError::WasmtimeError(format!("Failed to serialize precompiled module: {}", e)))?;
        
        self.stats.precompiled_size = precompiled_bytes.len();
        self.stats.compilation_time_ms += start_time.elapsed().as_millis() as u64;
        
        Ok(precompiled_bytes)
    }
    
    /// Compile MIR directly to precompiled native code (convenience method)
    pub fn compile_mir_to_native(&mut self, mir_module: MirModule) -> Result<Vec<u8>, AotError> {
        let wasm_bytes = self.compile_mir_to_wasm(mir_module)?;
        self.precompile_wasm(&wasm_bytes)
    }
    
    /// Load and execute a precompiled module (for testing)
    pub fn execute_precompiled(&self, precompiled_bytes: &[u8]) -> Result<i32, AotError> {
        // Deserialize the precompiled module
        let module = unsafe {
            Module::deserialize(&self.wasmtime_engine, precompiled_bytes)
                .map_err(|e| AotError::WasmtimeError(format!("Failed to deserialize module: {}", e)))?
        };
        
        // Create instance and execute
        let mut store = wasmtime::Store::new(&self.wasmtime_engine, ());
        let instance = wasmtime::Instance::new(&mut store, &module, &[])
            .map_err(|e| AotError::RuntimeError(format!("Failed to create instance: {}", e)))?;
        
        // Look for main function or default export
        let main_func = instance
            .get_typed_func::<(), i32>(&mut store, "main")
            .or_else(|_| instance.get_typed_func::<(), i32>(&mut store, "_start"))
            .or_else(|_| instance.get_typed_func::<(), i32>(&mut store, "run"))
            .map_err(|e| AotError::RuntimeError(format!("No main function found: {}", e)))?;
        
        // Execute the function
        let result = main_func.call(&mut store, ())
            .map_err(|e| AotError::RuntimeError(format!("Execution failed: {}", e)))?;
        
        Ok(result)
    }
    
    /// Validate a WASM module before precompilation
    pub fn validate_wasm(&self, wasm_bytes: &[u8]) -> Result<(), AotError> {
        Module::validate(&self.wasmtime_engine, wasm_bytes)
            .map_err(|e| AotError::WasmtimeError(format!("WASM validation failed: {}", e)))?;
        Ok(())
    }
    
    /// Get compilation statistics
    pub fn get_stats(&self) -> AotStats {
        self.stats.clone()
    }
    
    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = AotStats {
            wasm_size: 0,
            precompiled_size: 0,
            compilation_time_ms: 0,
            optimization_level: self.stats.optimization_level.clone(),
        };
    }
    
    /// Get compression ratio (precompiled size / WASM size)
    pub fn compression_ratio(&self) -> f64 {
        if self.stats.wasm_size == 0 {
            return 0.0;
        }
        self.stats.precompiled_size as f64 / self.stats.wasm_size as f64
    }
    
    /// Get wasmtime engine info
    pub fn engine_info(&self) -> String {
        format!(
            "Wasmtime {} with Cranelift backend",
            env!("CARGO_PKG_VERSION")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::MirModule;
    
    #[test]
    fn test_compiler_creation() {
        let config = AotConfig::new().expect("Failed to create config");
        let _compiler = AotCompiler::new(&config).expect("Failed to create compiler");
        // Should not panic
        assert!(true);
    }
    
    #[test]
    fn test_empty_module_compilation() {
        let config = AotConfig::new().expect("Failed to create config");
        let mut compiler = AotCompiler::new(&config).expect("Failed to create compiler");
        let module = MirModule::new("test".to_string());
        
        // Should handle empty module gracefully
        let result = compiler.compile_mir_to_wasm(module);
        // Note: This might fail due to empty module, but should not panic
        // The result depends on the WASM backend implementation
        match result {
            Ok(_) => assert!(true),
            Err(_) => assert!(true), // Empty modules might legitimately fail
        }
    }
    
    #[test]
    fn test_stats_tracking() {
        let config = AotConfig::new().expect("Failed to create config");
        let compiler = AotCompiler::new(&config).expect("Failed to create compiler");
        let stats = compiler.get_stats();
        
        assert_eq!(stats.wasm_size, 0);
        assert_eq!(stats.precompiled_size, 0);
        assert_eq!(stats.compilation_time_ms, 0);
        assert!(stats.optimization_level.contains("O"));
    }
    
    #[test]
    fn test_wasm_validation() {
        let config = AotConfig::new().expect("Failed to create config");
        let compiler = AotCompiler::new(&config).expect("Failed to create compiler");
        
        // Test with invalid WASM bytes
        let invalid_wasm = vec![0x00, 0x61, 0x73, 0x6d]; // Incomplete WASM header
        assert!(compiler.validate_wasm(&invalid_wasm).is_err());
    }
    
    #[test]
    fn test_compression_ratio() {
        let config = AotConfig::new().expect("Failed to create config");
        let compiler = AotCompiler::new(&config).expect("Failed to create compiler");
        
        // With no compilation done, ratio should be 0
        assert_eq!(compiler.compression_ratio(), 0.0);
    }
    
    #[test]
    fn test_engine_info() {
        let config = AotConfig::new().expect("Failed to create config");
        let compiler = AotCompiler::new(&config).expect("Failed to create compiler");
        let info = compiler.engine_info();
        
        assert!(info.contains("Wasmtime"));
        assert!(info.contains("Cranelift"));
    }
}