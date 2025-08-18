/*!
 * LLVM Context Management - Handle LLVM context, module, and target setup
 */

#[cfg(feature = "llvm")]
use inkwell::context::Context;
#[cfg(feature = "llvm")]
use inkwell::module::Module;
#[cfg(feature = "llvm")]
use inkwell::builder::Builder;
#[cfg(feature = "llvm")]
use inkwell::targets::{Target, TargetMachine, TargetTriple, InitializationConfig};

#[cfg(feature = "llvm")]
pub struct CodegenContext<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub target_machine: TargetMachine,
}

#[cfg(feature = "llvm")]
impl<'ctx> CodegenContext<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Result<Self, String> {
        // 1. ターゲット初期化
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|e| format!("Failed to initialize native target: {}", e))?;
        
        // 2. モジュール作成
        let module = context.create_module(module_name);
        
        // 3. ターゲットマシン作成
        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple)
            .map_err(|e| format!("Failed to get target: {}", e))?;
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
        
        // 4. データレイアウト設定
        module.set_triple(&triple);
        module.set_data_layout(&target_machine.get_target_data().get_data_layout());
        
        Ok(Self {
            context,
            module,
            builder: context.create_builder(),
            target_machine,
        })
    }
}

#[cfg(not(feature = "llvm"))]
pub struct CodegenContext<'ctx> {
    _phantom: std::marker::PhantomData<&'ctx ()>,
}

#[cfg(not(feature = "llvm"))]
impl<'ctx> CodegenContext<'ctx> {
    pub fn new(_context: &'ctx (), _module_name: &str) -> Result<Self, String> {
        Err("LLVM feature not enabled".to_string())
    }
}