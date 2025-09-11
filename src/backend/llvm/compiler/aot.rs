use super::codegen;
use super::LLVMCompiler;
use crate::box_trait::NyashBox;
use crate::mir::function::MirModule;

impl LLVMCompiler {
    pub fn compile_module(&self, mir_module: &MirModule, output_path: &str) -> Result<(), String> {
        codegen::compile_module(mir_module, output_path)
    }

    pub fn compile_and_execute(
        &mut self,
        mir_module: &MirModule,
        temp_path: &str,
    ) -> Result<Box<dyn NyashBox>, String> {
        let obj_path = format!("{}.o", temp_path);
        self.compile_module(mir_module, &obj_path)?;
        self.run_interpreter(mir_module)
    }
}
