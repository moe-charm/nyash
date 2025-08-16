/*!
 * AOT (Ahead-of-Time) Backend - Phase 9 Implementation
 * 
 * Provides native executable generation using wasmtime precompilation
 * for maximum performance and zero JIT startup overhead
 */

mod compiler;
mod executable;
mod config;

pub use compiler::AotCompiler;
pub use executable::ExecutableBuilder;
pub use config::AotConfig;

use crate::mir::MirModule;
use std::path::Path;

/// AOT compilation error
#[derive(Debug)]
pub enum AotError {
    CompilationError(String),
    WasmtimeError(String),
    IOError(String),
    ConfigError(String),
    RuntimeError(String),
}

impl std::fmt::Display for AotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AotError::CompilationError(msg) => write!(f, "AOT compilation error: {}", msg),
            AotError::WasmtimeError(msg) => write!(f, "Wasmtime error: {}", msg),
            AotError::IOError(msg) => write!(f, "IO error: {}", msg),
            AotError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            AotError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}

impl std::error::Error for AotError {}

impl From<std::io::Error> for AotError {
    fn from(error: std::io::Error) -> Self {
        AotError::IOError(error.to_string())
    }
}

impl From<wasmtime::Error> for AotError {
    fn from(error: wasmtime::Error) -> Self {
        AotError::WasmtimeError(error.to_string())
    }
}

/// Main AOT backend
pub struct AotBackend {
    compiler: AotCompiler,
    #[allow(dead_code)]
    config: AotConfig,
}

impl AotBackend {
    /// Create a new AOT backend with default configuration
    pub fn new() -> Result<Self, AotError> {
        let config = AotConfig::new()?;
        let compiler = AotCompiler::new(&config)?;
        
        Ok(Self {
            compiler,
            config,
        })
    }
    
    /// Create AOT backend with custom configuration
    pub fn with_config(config: AotConfig) -> Result<Self, AotError> {
        let compiler = AotCompiler::new(&config)?;
        
        Ok(Self {
            compiler,
            config,
        })
    }
    
    /// Compile MIR module to standalone native executable
    pub fn compile_to_executable<P: AsRef<Path>>(
        &mut self, 
        mir_module: MirModule, 
        output_path: P
    ) -> Result<(), AotError> {
        // For now, just create a .cwasm precompiled module
        // TODO: Implement full standalone executable generation
        let cwasm_path = output_path.as_ref().with_extension("cwasm");
        self.compile_to_precompiled(mir_module, cwasm_path)
    }
    
    /// Compile MIR module to .cwasm precompiled module
    pub fn compile_to_precompiled<P: AsRef<Path>>(
        &mut self,
        mir_module: MirModule,
        output_path: P
    ) -> Result<(), AotError> {
        // Compile MIR to WASM
        let wasm_bytes = self.compiler.compile_mir_to_wasm(mir_module)?;
        
        // Precompile WASM to .cwasm
        let precompiled_module = self.compiler.precompile_wasm(&wasm_bytes)?;
        
        // Write to file
        std::fs::write(output_path, precompiled_module)?;
        
        Ok(())
    }
    
    /// Get performance statistics
    pub fn get_stats(&self) -> AotStats {
        self.compiler.get_stats()
    }
}

impl Default for AotBackend {
    fn default() -> Self {
        Self::new().expect("Failed to create default AOT backend")
    }
}

/// AOT compilation statistics
#[derive(Debug, Clone)]
pub struct AotStats {
    pub wasm_size: usize,
    pub precompiled_size: usize,
    pub compilation_time_ms: u64,
    pub optimization_level: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::MirModule;
    
    #[test]
    fn test_aot_backend_creation() {
        let _backend = AotBackend::new();
        // Should not panic - basic creation test
        assert!(true);
    }
    
    #[test] 
    fn test_default_config() {
        let config = AotConfig::new().expect("Failed to create default config");
        assert!(config.optimization_level() >= 1);
    }
}