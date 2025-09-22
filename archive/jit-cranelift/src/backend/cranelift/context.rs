#![cfg(feature = "cranelift-jit")]

use std::collections::HashMap;

use cranelift_codegen::ir::{types, AbiParam, Block, Signature};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module};

use crate::mir::{BasicBlockId, MirFunction, ValueId};

/// Simple block map (MIR BB -> CLIF Block)
pub struct BlockMap(pub HashMap<BasicBlockId, Block>);

impl BlockMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn get(&self, bb: &BasicBlockId) -> Option<&Block> {
        self.0.get(bb)
    }
    pub fn insert(&mut self, bb: BasicBlockId, blk: Block) {
        self.0.insert(bb, blk);
    }
    /// Create a CLIF block for each MIR block id
    pub fn create_for_function(func: &MirFunction, builder: &mut FunctionBuilder) -> Self {
        let mut m = HashMap::new();
        for (bb_id, _) in &func.blocks {
            m.insert(*bb_id, builder.create_block());
        }
        Self(m)
    }
}

/// Value environment for CLIF lowering: holds current SSA values and pseudo-memory for Load/Store
pub struct ValueEnv {
    vals: HashMap<ValueId, cranelift_codegen::ir::Value>,
    mem: HashMap<ValueId, cranelift_codegen::ir::Value>,
}

impl ValueEnv {
    pub fn new() -> Self {
        Self {
            vals: HashMap::new(),
            mem: HashMap::new(),
        }
    }
    pub fn get_val(&self, id: &ValueId) -> Result<cranelift_codegen::ir::Value, String> {
        self.vals
            .get(id)
            .cloned()
            .ok_or_else(|| format!("undef {:?}", id))
    }
    pub fn set_val(&mut self, id: ValueId, v: cranelift_codegen::ir::Value) {
        self.vals.insert(id, v);
    }
    pub fn get_mem_or(
        &self,
        id: &ValueId,
        default: cranelift_codegen::ir::Value,
    ) -> cranelift_codegen::ir::Value {
        *self.mem.get(id).unwrap_or(&default)
    }
    pub fn set_mem(&mut self, id: ValueId, v: cranelift_codegen::ir::Value) {
        self.mem.insert(id, v);
    }
}

/// Cranelift JIT module wrapper (context)
pub struct ClifContext {
    pub module: JITModule,
}

impl ClifContext {
    pub fn new() -> Result<Self, String> {
        let isa_builder = cranelift_native::builder().map_err(|e| e.to_string())?;
        let flag_builder = cranelift_codegen::settings::builder();
        let flags = cranelift_codegen::settings::Flags::new(flag_builder);
        let isa = isa_builder.finish(flags).map_err(|e| e.to_string())?;
        let jit_builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        Ok(Self {
            module: JITModule::new(jit_builder),
        })
    }

    /// Declare an exported i64-return function and return its id and Cranelift context/signature
    pub fn declare_i64_fn(
        &mut self,
        name: &str,
    ) -> Result<(FuncId, cranelift_codegen::Context, Signature), String> {
        let mut sig = Signature::new(self.module.target_config().default_call_conv);
        sig.returns.push(AbiParam::new(types::I64));
        let func_id = self
            .module
            .declare_function(name, Linkage::Export, &sig)
            .map_err(|e| e.to_string())?;
        let mut ctx = self.module.make_context();
        ctx.func.signature = sig.clone();
        Ok((func_id, ctx, sig))
    }

    pub fn finalize(
        &mut self,
        func_id: FuncId,
        ctx: &mut cranelift_codegen::Context,
    ) -> Result<*const u8, String> {
        self.module
            .define_function(func_id, ctx)
            .map_err(|e| e.to_string())?;
        self.module.clear_context(ctx);
        let _ = self.module.finalize_definitions();
        Ok(self.module.get_finalized_function(func_id))
    }
}
