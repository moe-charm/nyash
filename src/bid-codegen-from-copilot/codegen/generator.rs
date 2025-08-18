/*!
 * Code Generator - Main entry point for BID code generation
 */

use crate::bid::{BidDefinition, BidError, BidResult};
use std::path::{Path, PathBuf};
use std::fs;

/// Code generation target
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeGenTarget {
    Wasm,
    VM,
    LLVM,
    TypeScript,
    Python,
}

impl CodeGenTarget {
    /// Parse target from string
    pub fn from_str(target: &str) -> Result<Self, BidError> {
        match target.to_lowercase().as_str() {
            "wasm" => Ok(CodeGenTarget::Wasm),
            "vm" => Ok(CodeGenTarget::VM),
            "llvm" => Ok(CodeGenTarget::LLVM),
            "ts" | "typescript" => Ok(CodeGenTarget::TypeScript),
            "py" | "python" => Ok(CodeGenTarget::Python),
            _ => Err(BidError::UnsupportedTarget(target.to_string())),
        }
    }
    
    /// Get the target name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            CodeGenTarget::Wasm => "wasm",
            CodeGenTarget::VM => "vm",
            CodeGenTarget::LLVM => "llvm",
            CodeGenTarget::TypeScript => "ts",
            CodeGenTarget::Python => "py",
        }
    }
}

/// Code generation options
pub struct CodeGenOptions {
    pub target: CodeGenTarget,
    pub output_dir: PathBuf,
    pub force: bool,
    pub dry_run: bool,
}

impl CodeGenOptions {
    /// Create new options
    pub fn new(target: CodeGenTarget, output_dir: PathBuf) -> Self {
        Self {
            target,
            output_dir,
            force: false,
            dry_run: false,
        }
    }
    
    /// Set force overwrite
    pub fn with_force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }
    
    /// Set dry run mode
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }
}

/// Generated file
pub struct GeneratedFile {
    pub path: PathBuf,
    pub content: String,
}

impl GeneratedFile {
    pub fn new(path: PathBuf, content: String) -> Self {
        Self { path, content }
    }
}

/// Code generation result
pub struct CodeGenResult {
    pub files: Vec<GeneratedFile>,
    pub target: CodeGenTarget,
    pub bid_name: String,
}

/// Main code generator
pub struct CodeGenerator;

impl CodeGenerator {
    /// Generate code from BID definition
    pub fn generate(bid: &BidDefinition, options: &CodeGenOptions) -> BidResult<CodeGenResult> {
        let bid_name = bid.name();
        
        // Generate based on target
        let files = match options.target {
            CodeGenTarget::Wasm => Self::generate_wasm(bid, options)?,
            CodeGenTarget::VM => Self::generate_vm(bid, options)?,
            CodeGenTarget::LLVM => Self::generate_llvm(bid, options)?,
            CodeGenTarget::TypeScript => Self::generate_typescript(bid, options)?,
            CodeGenTarget::Python => Self::generate_python(bid, options)?,
        };
        
        // Write files unless dry run
        if !options.dry_run {
            Self::write_files(&files, options)?;
        }
        
        Ok(CodeGenResult {
            files,
            target: options.target.clone(),
            bid_name,
        })
    }
    
    /// Generate WASM import declarations
    fn generate_wasm(bid: &BidDefinition, options: &CodeGenOptions) -> BidResult<Vec<GeneratedFile>> {
        use super::targets::wasm::WasmGenerator;
        WasmGenerator::generate(bid, options)
    }
    
    /// Generate VM function table definitions
    fn generate_vm(bid: &BidDefinition, options: &CodeGenOptions) -> BidResult<Vec<GeneratedFile>> {
        use super::targets::vm::VmGenerator;
        VmGenerator::generate(bid, options)
    }
    
    /// Generate LLVM declare statements
    fn generate_llvm(bid: &BidDefinition, options: &CodeGenOptions) -> BidResult<Vec<GeneratedFile>> {
        use super::targets::llvm::LlvmGenerator;
        LlvmGenerator::generate(bid, options)
    }
    
    /// Generate TypeScript wrapper
    fn generate_typescript(bid: &BidDefinition, options: &CodeGenOptions) -> BidResult<Vec<GeneratedFile>> {
        use super::targets::typescript::TypeScriptGenerator;
        TypeScriptGenerator::generate(bid, options)
    }
    
    /// Generate Python wrapper
    fn generate_python(bid: &BidDefinition, options: &CodeGenOptions) -> BidResult<Vec<GeneratedFile>> {
        use super::targets::python::PythonGenerator;
        PythonGenerator::generate(bid, options)
    }
    
    /// Write generated files to disk
    fn write_files(files: &[GeneratedFile], options: &CodeGenOptions) -> BidResult<()> {
        for file in files {
            // Check if file exists and force is not set
            if file.path.exists() && !options.force {
                return Err(BidError::IoError(format!(
                    "File already exists: {} (use --force to overwrite)",
                    file.path.display()
                )));
            }
            
            // Create parent directories
            if let Some(parent) = file.path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| BidError::IoError(format!(
                        "Failed to create directory {}: {}",
                        parent.display(),
                        e
                    )))?;
            }
            
            // Write file
            fs::write(&file.path, &file.content)
                .map_err(|e| BidError::IoError(format!(
                    "Failed to write file {}: {}",
                    file.path.display(),
                    e
                )))?;
        }
        
        Ok(())
    }
    
    /// Preview generated files (for dry run)
    pub fn preview_files(result: &CodeGenResult) {
        println!("📁 Generated files for target '{}' (BID: {}):", 
                 result.target.as_str(), result.bid_name);
        println!();
        
        for file in &result.files {
            println!("📄 {}", file.path.display());
            println!("   {} lines, {} bytes", 
                     file.content.lines().count(),
                     file.content.len());
            
            // Show first few lines
            let lines: Vec<&str> = file.content.lines().take(5).collect();
            for line in lines {
                println!("   │ {}", line);
            }
            
            if file.content.lines().count() > 5 {
                println!("   │ ... ({} more lines)", file.content.lines().count() - 5);
            }
            
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_code_gen_target_from_str() {
        assert_eq!(CodeGenTarget::from_str("wasm").unwrap(), CodeGenTarget::Wasm);
        assert_eq!(CodeGenTarget::from_str("vm").unwrap(), CodeGenTarget::VM);
        assert_eq!(CodeGenTarget::from_str("llvm").unwrap(), CodeGenTarget::LLVM);
        assert_eq!(CodeGenTarget::from_str("ts").unwrap(), CodeGenTarget::TypeScript);
        assert_eq!(CodeGenTarget::from_str("typescript").unwrap(), CodeGenTarget::TypeScript);
        assert_eq!(CodeGenTarget::from_str("py").unwrap(), CodeGenTarget::Python);
        assert_eq!(CodeGenTarget::from_str("python").unwrap(), CodeGenTarget::Python);
        
        assert!(CodeGenTarget::from_str("invalid").is_err());
    }
    
    #[test]
    fn test_code_gen_target_as_str() {
        assert_eq!(CodeGenTarget::Wasm.as_str(), "wasm");
        assert_eq!(CodeGenTarget::VM.as_str(), "vm");
        assert_eq!(CodeGenTarget::LLVM.as_str(), "llvm");
        assert_eq!(CodeGenTarget::TypeScript.as_str(), "ts");
        assert_eq!(CodeGenTarget::Python.as_str(), "py");
    }
}