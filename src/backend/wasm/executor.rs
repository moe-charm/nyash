/*!
 * WASM Executor - Execute compiled WASM modules with host functions
 * 
 * Phase 4-3c: Provides wasmtime-based execution for Nyash WASM modules
 */

use wasmtime::*;
use std::path::Path;
use super::{WasmError, host::{HostState, create_host_functions}};

/// WASM module executor
pub struct WasmExecutor {
    engine: Engine,
}

impl WasmExecutor {
    /// Create new WASM executor
    pub fn new() -> Result<Self, WasmError> {
        let engine = Engine::default();
        Ok(Self { engine })
    }
    
    /// Execute a WAT file
    pub fn execute_wat_file<P: AsRef<Path>>(&self, wat_path: P) -> Result<String, WasmError> {
        // Read WAT file
        let wat_content = std::fs::read_to_string(&wat_path)
            .map_err(|e| WasmError::IOError(e.to_string()))?;
        
        self.execute_wat(&wat_content)
    }
    
    /// Execute WAT content
    pub fn execute_wat(&self, wat_content: &str) -> Result<String, WasmError> {
        // Create store with host state
        let mut store = Store::new(&self.engine, HostState::new());
        
        // Compile WAT to module
        let module = Module::new(&self.engine, wat_content)
            .map_err(|e| WasmError::CompilationError(format!("Failed to compile WAT: {}", e)))?;
        
        // Create host functions
        let host_functions = create_host_functions(&mut store)
            .map_err(|e| WasmError::RuntimeError(format!("Failed to create host functions: {}", e)))?;
        
        // Create imports list
        let mut imports = Vec::new();
        for (module_name, func_name, func) in host_functions {
            imports.push(func);
        }
        
        // Instantiate module with imports
        let instance = Instance::new(&mut store, &module, &imports)
            .map_err(|e| WasmError::RuntimeError(format!("Failed to instantiate module: {}", e)))?;
        
        // Get main function
        let main_func = instance
            .get_func(&mut store, "main")
            .ok_or_else(|| WasmError::RuntimeError("No main function found".to_string()))?;
        
        // Call main function
        let results = main_func
            .call(&mut store, &[], &mut [])
            .map_err(|e| WasmError::RuntimeError(format!("Failed to execute main: {}", e)))?;
        
        // Return success message
        Ok("WASM execution completed successfully".to_string())
    }
    
    /// Execute a WASM binary file
    pub fn execute_wasm_file<P: AsRef<Path>>(&self, wasm_path: P) -> Result<String, WasmError> {
        // Read WASM file
        let wasm_bytes = std::fs::read(&wasm_path)
            .map_err(|e| WasmError::IOError(e.to_string()))?;
        
        self.execute_wasm(&wasm_bytes)
    }
    
    /// Execute WASM bytes
    pub fn execute_wasm(&self, wasm_bytes: &[u8]) -> Result<String, WasmError> {
        // Create store with host state
        let mut store = Store::new(&self.engine, HostState::new());
        
        // Create module from bytes
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| WasmError::CompilationError(format!("Failed to load WASM: {}", e)))?;
        
        // Create host functions
        let host_functions = create_host_functions(&mut store)
            .map_err(|e| WasmError::RuntimeError(format!("Failed to create host functions: {}", e)))?;
        
        // Create imports list
        let mut imports = Vec::new();
        for (module_name, func_name, func) in host_functions {
            imports.push(func);
        }
        
        // Instantiate module with imports
        let instance = Instance::new(&mut store, &module, &imports)
            .map_err(|e| WasmError::RuntimeError(format!("Failed to instantiate module: {}", e)))?;
        
        // Get main function
        let main_func = instance
            .get_func(&mut store, "main")
            .ok_or_else(|| WasmError::RuntimeError("No main function found".to_string()))?;
        
        // Call main function
        let results = main_func
            .call(&mut store, &[], &mut [])
            .map_err(|e| WasmError::RuntimeError(format!("Failed to execute main: {}", e)))?;
        
        // Return success message
        Ok("WASM execution completed successfully".to_string())
    }
}