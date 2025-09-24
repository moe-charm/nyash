/*!
 * LLVM Context Management - Handle LLVM context, module, and target setup (legacy)
 */

/// Mock implementation when legacy inkwell backend is disabled
#[cfg(not(feature = "llvm-inkwell-legacy"))]
pub struct CodegenContext {
    _phantom: std::marker::PhantomData<()>,
}

#[cfg(not(feature = "llvm-inkwell-legacy"))]
impl CodegenContext {
    pub fn new(_module_name: &str) -> Result<Self, String> {
        Ok(Self {
            _phantom: std::marker::PhantomData,
        })
    }
}

// Real implementation (compiled only when feature "llvm-inkwell-legacy" is enabled)
#[cfg(feature = "llvm-inkwell-legacy")]
use inkwell::builder::Builder;
#[cfg(feature = "llvm-inkwell-legacy")]
use inkwell::context::Context;
#[cfg(feature = "llvm-inkwell-legacy")]
use inkwell::module::Module;
#[cfg(feature = "llvm-inkwell-legacy")]
use inkwell::targets::{InitializationConfig, Target, TargetMachine};

#[cfg(feature = "llvm-inkwell-legacy")]
pub struct CodegenContext<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub target_machine: TargetMachine,
}

#[cfg(feature = "llvm-inkwell-legacy")]
impl<'ctx> CodegenContext<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Result<Self, String> {
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|e| format!("Failed to initialize native target: {}", e))?;
        let module = context.create_module(module_name);
        let triple = TargetMachine::get_default_triple();
        let target =
            Target::from_triple(&triple).map_err(|e| format!("Failed to get target: {}", e))?;
        let target_machine = target
            .create_target_machine(
                &triple,
                "generic",
                "",
                inkwell::OptimizationLevel::None,
                inkwell::targets::RelocMode::Default,
                inkwell::targets::CodeModel::Default,
            )
            .ok_or_else(|| "Failed to create target machine".to_string())?;
        let builder = context.create_builder();
        Ok(Self {
            context,
            module,
            builder,
            target_machine,
        })
    }
}
