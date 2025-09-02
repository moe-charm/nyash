#![cfg(feature = "cranelift-jit")]
use super::{IRBuilder, ParamKind};
use cranelift_module::Module;
use cranelift_codegen::ir::InstBuilder;

pub struct ObjectBuilder {
    pub(crate) module: cranelift_object::ObjectModule,
    pub(crate) ctx: cranelift_codegen::Context,
    pub(crate) fbc: cranelift_frontend::FunctionBuilderContext,
    pub(crate) current_name: Option<String>,
    pub(crate) entry_block: Option<cranelift_codegen::ir::Block>,
    pub(crate) blocks: Vec<cranelift_codegen::ir::Block>,
    pub(crate) current_block_index: Option<usize>,
    pub(crate) value_stack: Vec<cranelift_codegen::ir::Value>,
    pub(crate) typed_sig_prepared: bool,
    pub(crate) desired_argc: usize,
    pub(crate) desired_has_ret: bool,
    pub(crate) desired_ret_is_f64: bool,
    pub(crate) ret_hint_is_b1: bool,
    pub(crate) local_slots: std::collections::HashMap<usize, cranelift_codegen::ir::StackSlot>,
    pub(crate) block_param_counts: std::collections::HashMap<usize, usize>,
    pub stats: (u64,u64,u64,u64,u64),
    pub object_bytes: Option<Vec<u8>>,
}

impl ObjectBuilder {
    pub fn new() -> Self {
        use cranelift_codegen::settings;
        let isa = cranelift_native::builder().expect("host ISA").finish(settings::Flags::new(settings::builder())).expect("finish ISA");
        let obj_builder = cranelift_object::ObjectBuilder::new(isa, "nyash_aot".to_string(), cranelift_module::default_libcall_names()).expect("ObjectBuilder");
        let module = cranelift_object::ObjectModule::new(obj_builder);
        Self { module, ctx: cranelift_codegen::Context::new(), fbc: cranelift_frontend::FunctionBuilderContext::new(), current_name: None, entry_block: None, blocks: Vec::new(), current_block_index: None, value_stack: Vec::new(), typed_sig_prepared: false, desired_argc: 0, desired_has_ret: true, desired_ret_is_f64: false, ret_hint_is_b1: false, local_slots: std::collections::HashMap::new(), block_param_counts: std::collections::HashMap::new(), stats: (0,0,0,0,0), object_bytes: None }
    }

    fn fresh_module() -> cranelift_object::ObjectModule {
        use cranelift_codegen::settings;
        let isa = cranelift_native::builder().expect("host ISA").finish(settings::Flags::new(settings::builder())).expect("finish ISA");
        let obj_builder = cranelift_object::ObjectBuilder::new(isa, "nyash_aot".to_string(), cranelift_module::default_libcall_names()).expect("ObjectBuilder");
        cranelift_object::ObjectModule::new(obj_builder)
    }

    pub fn take_object_bytes(&mut self) -> Option<Vec<u8>> { self.object_bytes.take() }

    fn entry_param(&mut self, index: usize) -> Option<cranelift_codegen::ir::Value> {
        use cranelift_frontend::FunctionBuilder;
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(b) = self.entry_block {
            fb.switch_to_block(b);
            let params = fb.func.dfg.block_params(b).to_vec();
            if let Some(v) = params.get(index).copied() { return Some(v); }
        }
        None
    }
}

impl IRBuilder for ObjectBuilder {
    fn begin_function(&mut self, name: &str) {
        use cranelift_codegen::ir::{AbiParam, Signature, types};
        use cranelift_frontend::FunctionBuilder;
        self.current_name = Some(name.to_string());
        self.value_stack.clear();
        if !self.typed_sig_prepared {
            let call_conv = self.module.isa().default_call_conv();
            let mut sig = Signature::new(call_conv);
            for _ in 0..self.desired_argc { sig.params.push(AbiParam::new(types::I64)); }
            if self.desired_has_ret { if self.desired_ret_is_f64 { sig.returns.push(AbiParam::new(types::F64)); } else { sig.returns.push(AbiParam::new(types::I64)); } }
            self.ctx.func.signature = sig;
        }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if self.blocks.is_empty() { self.blocks.push(fb.create_block()); }
        let entry = self.blocks[0];
        fb.append_block_params_for_function_params(entry);
        fb.switch_to_block(entry);
        self.entry_block = Some(entry);
        self.current_block_index = Some(0);
    }
    fn end_function(&mut self) {
        use cranelift_codegen::ir::StackSlotData;
        use cranelift_frontend::FunctionBuilder;
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        fb.finalize();
        let obj_id = self.module.declare_function(self.current_name.as_deref().unwrap_or("jit_aot"), cranelift_module::Linkage::Local, &self.ctx.func.signature).expect("declare func");
        self.module.define_function(obj_id, &mut self.ctx).expect("define");
        self.module.clear_context(&mut self.ctx);
        let finished = std::mem::replace(&mut self.module, Self::fresh_module());
        let product = finished.finish();
        self.object_bytes = Some(product.emit().expect("emit object"));
    }
    fn prepare_signature_i64(&mut self, argc: usize, has_ret: bool) { self.desired_argc = argc; self.desired_has_ret = has_ret; }
    fn prepare_signature_typed(&mut self, _params: &[ParamKind], _ret_is_f64: bool) { self.typed_sig_prepared = true; }
    fn emit_param_i64(&mut self, index: usize) { if let Some(v) = self.entry_param(index) { self.value_stack.push(v); } }
    fn emit_const_i64(&mut self, val: i64) { use cranelift_codegen::ir::types; use cranelift_frontend::FunctionBuilder; let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc); let v = fb.ins().iconst(types::I64, val); self.value_stack.push(v); self.stats.0 += 1; }
    fn emit_const_f64(&mut self, val: f64) { use cranelift_frontend::FunctionBuilder; use cranelift_codegen::ir::types; let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc); let v = fb.ins().f64const(val); self.value_stack.push(v); }
    fn emit_binop(&mut self, _op: super::BinOpKind) { self.stats.1 += 1; }
    fn emit_compare(&mut self, _op: super::CmpKind) { self.stats.2 += 1; }
    fn emit_jump(&mut self) { self.stats.3 += 1; }
    fn emit_branch(&mut self) { self.stats.3 += 1; }
    fn emit_return(&mut self) { self.stats.4 += 1; }
    fn ensure_local_i64(&mut self, _index: usize) {}
    fn store_local_i64(&mut self, _index: usize) {}
    fn load_local_i64(&mut self, _index: usize) {}
    fn prepare_blocks(&mut self, count: usize) { use cranelift_frontend::FunctionBuilder; if count == 0 { return; } let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc); if self.blocks.len() < count { for _ in 0..(count - self.blocks.len()) { self.blocks.push(fb.create_block()); } } }
    fn switch_to_block(&mut self, index: usize) { use cranelift_frontend::FunctionBuilder; if index >= self.blocks.len() { return; } let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc); fb.switch_to_block(self.blocks[index]); self.current_block_index = Some(index); }
}
