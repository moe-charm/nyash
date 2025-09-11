use crate::backend::llvm::context::CodegenContext;
use crate::mir::function::MirModule;
use inkwell::context::Context;

pub(crate) fn compile_module(mir_module: &MirModule, output_path: &str) -> Result<(), String> {
    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
        eprintln!(
            "[LLVM] compile_module start: functions={}, out={}",
            mir_module.functions.len(),
            output_path
        );
    }
    let context = Context::create();
    let codegen = CodegenContext::new(&context, "nyash_module")?;
    codegen
        .module
        .write_object_file(output_path)
        .map_err(|e| e.to_string())
}
