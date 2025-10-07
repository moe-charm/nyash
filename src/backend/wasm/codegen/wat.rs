/*!
 * WAT (WebAssembly Text) Output Generation
 */

/// WASM module representation for WAT generation
pub struct WasmModule {
    pub imports: Vec<String>,
    pub memory: String,
    pub data_segments: Vec<String>,
    pub globals: Vec<String>,
    pub functions: Vec<String>,
    pub exports: Vec<String>,
}

impl WasmModule {
    pub fn new() -> Self {
        Self {
            imports: Vec::new(),
            memory: String::new(),
            data_segments: Vec::new(),
            globals: Vec::new(),
            functions: Vec::new(),
            exports: Vec::new(),
        }
    }

    /// Generate WAT text format
    pub fn to_wat(&self) -> String {
        let mut wat = String::new();
        wat.push_str("(module\n");

        // Add imports first (must come before other definitions in WASM)
        for import in &self.imports {
            wat.push_str(&format!("  {}\n", import));
        }

        // Add memory declaration
        if !self.memory.is_empty() {
            wat.push_str(&format!("  {}\n", self.memory));
        }

        // Add data segments (must come after memory)
        for data_segment in &self.data_segments {
            wat.push_str(&format!("  {}\n", data_segment));
        }

        // Add globals
        for global in &self.globals {
            wat.push_str(&format!("  {}\n", global));
        }

        // Add functions
        for function in &self.functions {
            wat.push_str(&format!("  {}\n", function));
        }

        // Add exports
        for export in &self.exports {
            wat.push_str(&format!("  {}\n", export));
        }

        wat.push_str(")\n");
        wat
    }
}
