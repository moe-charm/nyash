/*!
 * WASM Backend - Phase 8 Implementation
 * 
 * Converts MIR instructions to WebAssembly for sandboxed execution
 * Targets browser execution and wasmtime runtime
 */

mod codegen;
mod memory;
mod runtime;
mod host;
mod executor;

pub use codegen::{WasmCodegen, WasmModule};
pub use memory::{MemoryManager, BoxLayout};
pub use runtime::RuntimeImports;
pub use executor::WasmExecutor;

use crate::mir::MirModule;

/// WASM compilation error
#[derive(Debug)]
pub enum WasmError {
    CodegenError(String),
    MemoryError(String),
    UnsupportedInstruction(String),
    WasmValidationError(String),
    IOError(String),
    RuntimeError(String),
    CompilationError(String),
}

impl std::fmt::Display for WasmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WasmError::CodegenError(msg) => write!(f, "Codegen error: {}", msg),
            WasmError::MemoryError(msg) => write!(f, "Memory error: {}", msg),
            WasmError::UnsupportedInstruction(msg) => write!(f, "Unsupported instruction: {}", msg),
            WasmError::WasmValidationError(msg) => write!(f, "WASM validation error: {}", msg),
            WasmError::IOError(msg) => write!(f, "IO error: {}", msg),
            WasmError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            WasmError::CompilationError(msg) => write!(f, "Compilation error: {}", msg),
        }
    }
}

impl std::error::Error for WasmError {}

/// Main WASM backend compiler
pub struct WasmBackend {
    codegen: WasmCodegen,
    memory_manager: MemoryManager,
    runtime: RuntimeImports,
}

impl WasmBackend {
    /// Create a new WASM backend
    pub fn new() -> Self {
        Self {
            codegen: WasmCodegen::new(),
            memory_manager: MemoryManager::new(),
            runtime: RuntimeImports::new(),
        }
    }
    
    /// Compile MIR module to WASM bytes
    pub fn compile_module(&mut self, mir_module: MirModule) -> Result<Vec<u8>, WasmError> {
        // Generate WAT (WebAssembly Text) first for debugging
        let wat_text = self.compile_to_wat(mir_module)?;
        
        // Phase 9.77 Task 1.3: Fix UTF-8 encoding error in WAT→WASM conversion
        self.wat_to_wasm(&wat_text)
    }
    
    /// Convert WAT text to WASM binary with proper UTF-8 handling
    fn wat_to_wasm(&self, wat_source: &str) -> Result<Vec<u8>, WasmError> {
        // Debug: Print WAT source for analysis
        eprintln!("🔍 WAT Source Debug (length: {}):", wat_source.len());
        eprintln!("WAT Content:\n{}", wat_source);
        
        // UTF-8 validation to prevent encoding errors
        if !wat_source.is_ascii() {
            eprintln!("❌ WAT source contains non-ASCII characters");
            return Err(WasmError::WasmValidationError(
                "WAT source contains non-ASCII characters".to_string()
            ));
        }
        
        eprintln!("✅ WAT source is ASCII-compatible");
        
        // Convert to bytes as required by wabt::wat2wasm
        eprintln!("🔄 Converting WAT to WASM bytes...");
        let wasm_bytes = wabt::wat2wasm(wat_source.as_bytes())
            .map_err(|e| {
                eprintln!("❌ wabt::wat2wasm failed: {}", e);
                WasmError::WasmValidationError(format!("WAT to WASM conversion failed: {}", e))
            })?;
        
        eprintln!("✅ WASM conversion successful, {} bytes generated", wasm_bytes.len());
        Ok(wasm_bytes)
    }
    
    /// Compile MIR module to WAT text format (for debugging)
    pub fn compile_to_wat(&mut self, mir_module: MirModule) -> Result<String, WasmError> {
        let wasm_module = self.codegen.generate_module(mir_module, &self.memory_manager, &self.runtime)?;
        Ok(wasm_module.to_wat())
    }
    
    /// Execute WASM bytes using wasmtime (for testing)
    pub fn execute_wasm(&self, wasm_bytes: &[u8]) -> Result<i32, WasmError> {
        let engine = wasmtime::Engine::default();
        let module = wasmtime::Module::new(&engine, wasm_bytes)
            .map_err(|e| WasmError::WasmValidationError(format!("Module creation failed: {}", e)))?;
        
        let mut store = wasmtime::Store::new(&engine, ());
        
        // Create print function import
        let print_func = wasmtime::Func::wrap(&mut store, |value: i32| {
            println!("{}", value);
        });
        
        // Create print_str function import for string debugging
        let print_str_func = wasmtime::Func::wrap(&mut store, |mut caller: wasmtime::Caller<'_, ()>, ptr: i32, len: i32| -> Result<(), wasmtime::Error> {
            let memory = caller.get_export("memory")
                .and_then(|export| export.into_memory())
                .ok_or_else(|| wasmtime::Error::msg("Memory export not found"))?;
            
            let data = memory.data(&caller);
            let start = ptr as usize;
            let end = start + len as usize;
            
            if end <= data.len() {
                let bytes = &data[start..end];
                if let Ok(s) = std::str::from_utf8(bytes) {
                    println!("String: {}", s);
                } else {
                    println!("Invalid UTF-8 bytes: {:?}", bytes);
                }
            } else {
                println!("String out of bounds: ptr={}, len={}, memory_size={}", ptr, len, data.len());
            }
            
            Ok(())
        });
        
        let imports = [print_func.into(), print_str_func.into()];
        let instance = wasmtime::Instance::new(&mut store, &module, &imports)
            .map_err(|e| WasmError::WasmValidationError(format!("Instance creation failed: {}", e)))?;
        
        // Call main function
        let main_func = instance.get_typed_func::<(), i32>(&mut store, "main")
            .map_err(|e| WasmError::WasmValidationError(format!("Main function not found: {}", e)))?;
        
        let result = main_func.call(&mut store, ())
            .map_err(|e| WasmError::WasmValidationError(format!("Execution failed: {}", e)))?;
        
        Ok(result)
    }
}

impl Default for WasmBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::MirModule;
    
    #[test]
    fn test_backend_creation() {
        let _backend = WasmBackend::new();
        // Should not panic
        assert!(true);
    }
    
    #[test]
    fn test_empty_module_compilation() {
        let mut backend = WasmBackend::new();
        let module = MirModule::new("test".to_string());
        
        // Should handle empty module gracefully
        let result = backend.compile_to_wat(module);
        assert!(result.is_ok());
    }
}