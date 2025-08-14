/*!
 * WASM Runtime Imports - External function imports for WASM modules
 * 
 * Phase 8.2 PoC1: Implements env.print import for debugging
 * Future: Additional runtime functions (file I/O, network, etc.)
 */

use super::WasmError;

/// Runtime import definitions for WASM modules
pub struct RuntimeImports {
    /// Available imports
    imports: Vec<ImportFunction>,
}

/// External function import definition
#[derive(Debug, Clone)]
pub struct ImportFunction {
    pub module: String,
    pub name: String,
    pub params: Vec<String>,
    pub result: Option<String>,
}

impl RuntimeImports {
    pub fn new() -> Self {
        let mut runtime = Self {
            imports: Vec::new(),
        };
        
        // Add standard runtime imports
        runtime.add_standard_imports();
        runtime
    }
    
    /// Add standard runtime function imports
    fn add_standard_imports(&mut self) {
        // env.print for debugging output
        self.imports.push(ImportFunction {
            module: "env".to_string(),
            name: "print".to_string(),
            params: vec!["i32".to_string()],
            result: None,
        });
        
        // env.print_str for string debugging (ptr, len)
        self.imports.push(ImportFunction {
            module: "env".to_string(),
            name: "print_str".to_string(),
            params: vec!["i32".to_string(), "i32".to_string()],
            result: None,
        });
        
        // Phase 9.7: Box FFI/ABI imports per BID specifications
        
        // env.console_log for console.log(message) - (string_ptr, string_len)
        self.imports.push(ImportFunction {
            module: "env".to_string(),
            name: "console_log".to_string(),
            params: vec!["i32".to_string(), "i32".to_string()],
            result: None,
        });
        
        // env.canvas_fillRect for canvas.fillRect(canvas_id, x, y, w, h, color)
        // Parameters: (canvas_id_ptr, canvas_id_len, x, y, w, h, color_ptr, color_len)
        self.imports.push(ImportFunction {
            module: "env".to_string(),
            name: "canvas_fillRect".to_string(),
            params: vec![
                "i32".to_string(), "i32".to_string(), // canvas_id (ptr, len)
                "i32".to_string(), "i32".to_string(), "i32".to_string(), "i32".to_string(), // x, y, w, h
                "i32".to_string(), "i32".to_string(), // color (ptr, len)
            ],
            result: None,
        });
        
        // env.canvas_fillText for canvas.fillText(canvas_id, text, x, y, font, color)
        // Parameters: (canvas_id_ptr, canvas_id_len, text_ptr, text_len, x, y, font_ptr, font_len, color_ptr, color_len)
        self.imports.push(ImportFunction {
            module: "env".to_string(),
            name: "canvas_fillText".to_string(),
            params: vec![
                "i32".to_string(), "i32".to_string(), // canvas_id (ptr, len)
                "i32".to_string(), "i32".to_string(), // text (ptr, len)
                "i32".to_string(), "i32".to_string(), // x, y
                "i32".to_string(), "i32".to_string(), // font (ptr, len)
                "i32".to_string(), "i32".to_string(), // color (ptr, len)
            ],
            result: None,
        });
        
        // Future: env.file_read, env.file_write for file I/O
        // Future: env.http_request for network access
    }
    
    /// Get all import declarations in WAT format
    pub fn get_imports(&self) -> Vec<String> {
        self.imports.iter().map(|import| {
            let params = if import.params.is_empty() {
                String::new()
            } else {
                format!("(param {})", import.params.join(" "))
            };
            
            let result = if let Some(ref result_type) = import.result {
                format!("(result {})", result_type)
            } else {
                String::new()
            };
            
            format!(
                "(import \"{}\" \"{}\" (func ${} {} {}))",
                import.module,
                import.name,
                import.name,
                params,
                result
            )
        }).collect()
    }
    
    /// Add custom import function
    pub fn add_import(&mut self, module: String, name: String, params: Vec<String>, result: Option<String>) {
        self.imports.push(ImportFunction {
            module,
            name,
            params,
            result,
        });
    }
    
    /// Check if an import is available
    pub fn has_import(&self, name: &str) -> bool {
        self.imports.iter().any(|import| import.name == name)
    }
    
    /// Get import function by name
    pub fn get_import(&self, name: &str) -> Option<&ImportFunction> {
        self.imports.iter().find(|import| import.name == name)
    }
    
    /// Generate JavaScript import object for browser execution
    pub fn get_js_import_object(&self) -> String {
        let mut js = String::new();
        js.push_str("const importObject = {\n");
        
        // Group by module
        let mut modules: std::collections::HashMap<String, Vec<&ImportFunction>> = std::collections::HashMap::new();
        for import in &self.imports {
            modules.entry(import.module.clone()).or_default().push(import);
        }
        
        for (module_name, functions) in modules {
            js.push_str(&format!("  {}: {{\n", module_name));
            
            for function in functions {
                match function.name.as_str() {
                    "print" => {
                        js.push_str("    print: (value) => console.log(value),\n");
                    },
                    "print_str" => {
                        js.push_str("    print_str: (ptr, len) => {\n");
                        js.push_str("      const memory = instance.exports.memory;\n");
                        js.push_str("      const str = new TextDecoder().decode(new Uint8Array(memory.buffer, ptr, len));\n");
                        js.push_str("      console.log(str);\n");
                        js.push_str("    },\n");
                    },
                    "console_log" => {
                        js.push_str("    console_log: (ptr, len) => {\n");
                        js.push_str("      const memory = instance.exports.memory;\n");
                        js.push_str("      const str = new TextDecoder().decode(new Uint8Array(memory.buffer, ptr, len));\n");
                        js.push_str("      console.log(str);\n");
                        js.push_str("    },\n");
                    },
                    "canvas_fillRect" => {
                        js.push_str("    canvas_fillRect: (canvasIdPtr, canvasIdLen, x, y, w, h, colorPtr, colorLen) => {\n");
                        js.push_str("      const memory = instance.exports.memory;\n");
                        js.push_str("      const canvasId = new TextDecoder().decode(new Uint8Array(memory.buffer, canvasIdPtr, canvasIdLen));\n");
                        js.push_str("      const color = new TextDecoder().decode(new Uint8Array(memory.buffer, colorPtr, colorLen));\n");
                        js.push_str("      const canvas = document.getElementById(canvasId);\n");
                        js.push_str("      if (canvas) {\n");
                        js.push_str("        const ctx = canvas.getContext('2d');\n");
                        js.push_str("        ctx.fillStyle = color;\n");
                        js.push_str("        ctx.fillRect(x, y, w, h);\n");
                        js.push_str("      }\n");
                        js.push_str("    },\n");
                    },
                    "canvas_fillText" => {
                        js.push_str("    canvas_fillText: (canvasIdPtr, canvasIdLen, textPtr, textLen, x, y, fontPtr, fontLen, colorPtr, colorLen) => {\n");
                        js.push_str("      const memory = instance.exports.memory;\n");
                        js.push_str("      const canvasId = new TextDecoder().decode(new Uint8Array(memory.buffer, canvasIdPtr, canvasIdLen));\n");
                        js.push_str("      const text = new TextDecoder().decode(new Uint8Array(memory.buffer, textPtr, textLen));\n");
                        js.push_str("      const font = new TextDecoder().decode(new Uint8Array(memory.buffer, fontPtr, fontLen));\n");
                        js.push_str("      const color = new TextDecoder().decode(new Uint8Array(memory.buffer, colorPtr, colorLen));\n");
                        js.push_str("      const canvas = document.getElementById(canvasId);\n");
                        js.push_str("      if (canvas) {\n");
                        js.push_str("        const ctx = canvas.getContext('2d');\n");
                        js.push_str("        ctx.font = font;\n");
                        js.push_str("        ctx.fillStyle = color;\n");
                        js.push_str("        ctx.fillText(text, x, y);\n");
                        js.push_str("      }\n");
                        js.push_str("    },\n");
                    },
                    _ => {
                        js.push_str(&format!("    {}: () => {{ throw new Error('Not implemented: {}'); }},\n", 
                                           function.name, function.name));
                    }
                }
            }
            
            js.push_str("  },\n");
        }
        
        js.push_str("};\n");
        js
    }
    
    /// Generate Rust wasmtime import bindings
    pub fn get_wasmtime_imports(&self) -> Result<String, WasmError> {
        let mut rust_code = String::new();
        rust_code.push_str("// Wasmtime import bindings\n");
        rust_code.push_str("let mut imports = Vec::new();\n\n");
        
        for import in &self.imports {
            match import.name.as_str() {
                "print" => {
                    rust_code.push_str(r#"
let print_func = wasmtime::Func::wrap(&mut store, |value: i32| {
    println!("{}", value);
});
imports.push(print_func.into());
"#);
                },
                _ => {
                    rust_code.push_str(&format!(r#"
// TODO: Implement {} import
let {}_func = wasmtime::Func::wrap(&mut store, || {{
    panic!("Not implemented: {}")
}});
imports.push({}_func.into());
"#, import.name, import.name, import.name, import.name));
                }
            }
        }
        
        Ok(rust_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_runtime_imports_creation() {
        let runtime = RuntimeImports::new();
        assert!(!runtime.imports.is_empty());
        assert!(runtime.has_import("print"));
    }
    
    #[test]
    fn test_import_wat_generation() {
        let runtime = RuntimeImports::new();
        let imports = runtime.get_imports();
        
        assert!(!imports.is_empty());
        assert!(imports[0].contains("import"));
        assert!(imports[0].contains("env"));
        assert!(imports[0].contains("print"));
    }
    
    #[test]
    fn test_custom_import_addition() {
        let mut runtime = RuntimeImports::new();
        runtime.add_import(
            "custom".to_string(),
            "test_func".to_string(),
            vec!["i32".to_string(), "i32".to_string()],
            Some("i32".to_string())
        );
        
        assert!(runtime.has_import("test_func"));
        let import = runtime.get_import("test_func").unwrap();
        assert_eq!(import.module, "custom");
        assert_eq!(import.params.len(), 2);
        assert!(import.result.is_some());
    }
    
    #[test]
    fn test_js_import_object_generation() {
        let runtime = RuntimeImports::new();
        let js = runtime.get_js_import_object();
        
        assert!(js.contains("importObject"));
        assert!(js.contains("env"));
        assert!(js.contains("print"));
        assert!(js.contains("console.log"));
    }
    
    #[test]
    fn test_wasmtime_imports_generation() {
        let runtime = RuntimeImports::new();
        let rust_code = runtime.get_wasmtime_imports().unwrap();
        
        assert!(rust_code.contains("wasmtime::Func::wrap"));
        assert!(rust_code.contains("print_func"));
        assert!(rust_code.contains("println!"));
    }
}