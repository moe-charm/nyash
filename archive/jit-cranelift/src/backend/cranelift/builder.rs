/*!
 * ClifBuilder - IRBuilder implementation for Cranelift (skeleton)
 *
 * This satisfies the IRBuilder trait so LowerCore can target it.
 * Actual CLIF emission will be added incrementally.
 */

#![cfg(feature = "cranelift-jit")]

use crate::jit::lower::builder::{BinOpKind, CmpKind, IRBuilder, ParamKind};
use cranelift_codegen::ir::InstBuilder;

// Minimal recorded opcodes for Const/Add/Return first
enum RecOp {
    ConstI64(i64),
    ConstF64(f64),
    BinOp(BinOpKind),
    Return,
}

pub struct ClifBuilder {
    pub consts: usize,
    pub binops: usize,
    pub cmps: usize,
    pub branches: usize,
    pub rets: usize,
    ops: Vec<RecOp>,
}

impl ClifBuilder {
    pub fn new() -> Self {
        Self {
            consts: 0,
            binops: 0,
            cmps: 0,
            branches: 0,
            rets: 0,
            ops: Vec::new(),
        }
    }

    /// Build and execute the recorded ops as a native function using Cranelift
    pub fn finish_and_execute(&self) -> Result<i64, String> {
        use cranelift_codegen::ir::{types, AbiParam, Signature};
        use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
        use cranelift_module::{Linkage, Module};
        // JIT setup
        let isa_builder = cranelift_native::builder().map_err(|e| e.to_string())?;
        let flag_builder = cranelift_codegen::settings::builder();
        let flags = cranelift_codegen::settings::Flags::new(flag_builder);
        let isa = isa_builder.finish(flags).map_err(|e| e.to_string())?;
        let jit_builder =
            cranelift_jit::JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let mut module = cranelift_jit::JITModule::new(jit_builder);
        // Signature ()->i64
        let mut sig = Signature::new(module.target_config().default_call_conv);
        sig.returns.push(AbiParam::new(types::I64));
        let func_id = module
            .declare_function("ny_lowercore_main", Linkage::Export, &sig)
            .map_err(|e| e.to_string())?;
        let mut ctx = module.make_context();
        ctx.func.signature = sig;
        let mut fbc = FunctionBuilderContext::new();
        let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fbc);
        let entry = fb.create_block();
        fb.switch_to_block(entry);

        // Interpret ops with a small value stack of CLIF Values
        let mut vs: Vec<cranelift_codegen::ir::Value> = Vec::new();
        let mut did_return = false;
        for op in &self.ops {
            match *op {
                RecOp::ConstI64(i) => {
                    vs.push(fb.ins().iconst(types::I64, i));
                }
                RecOp::ConstF64(f) => {
                    let fv = fb.ins().f64const(f);
                    let iv = fb.ins().fcvt_to_sint(types::I64, fv);
                    vs.push(iv);
                }
                RecOp::BinOp(BinOpKind::Add) => {
                    if vs.len() < 2 {
                        vs.clear();
                        vs.push(fb.ins().iconst(types::I64, 0));
                    } else {
                        let r = vs.pop().unwrap();
                        let l = vs.pop().unwrap();
                        vs.push(fb.ins().iadd(l, r));
                    }
                }
                RecOp::BinOp(_) => { /* ignore others for now */ }
                RecOp::Return => {
                    let retv = if let Some(v) = vs.last().copied() {
                        v
                    } else {
                        fb.ins().iconst(types::I64, 0)
                    };
                    fb.ins().return_(&[retv]);
                    did_return = true;
                }
            }
        }
        // Ensure function ends with return
        if !did_return {
            let retv = if let Some(v) = vs.last().copied() {
                v
            } else {
                fb.ins().iconst(types::I64, 0)
            };
            fb.ins().return_(&[retv]);
        }
        fb.seal_block(entry);
        fb.finalize();
        module
            .define_function(func_id, &mut ctx)
            .map_err(|e| e.to_string())?;
        module.clear_context(&mut ctx);
        let _ = module.finalize_definitions();
        let code = module.get_finalized_function(func_id);
        let func = unsafe { std::mem::transmute::<_, extern "C" fn() -> i64>(code) };
        Ok(func())
    }
}

impl IRBuilder for ClifBuilder {
    fn begin_function(&mut self, _name: &str) {}
    fn end_function(&mut self) {}
    fn prepare_signature_i64(&mut self, _argc: usize, _has_ret: bool) {}
    fn prepare_signature_typed(&mut self, _params: &[ParamKind], _ret_is_f64: bool) {}
    fn emit_param_i64(&mut self, _index: usize) {}
    fn emit_const_i64(&mut self, val: i64) {
        self.consts += 1;
        self.ops.push(RecOp::ConstI64(val));
    }
    fn emit_const_f64(&mut self, val: f64) {
        self.consts += 1;
        self.ops.push(RecOp::ConstF64(val));
    }
    fn emit_binop(&mut self, op: BinOpKind) {
        self.binops += 1;
        self.ops.push(RecOp::BinOp(op));
    }
    fn emit_compare(&mut self, _op: CmpKind) {
        self.cmps += 1;
    }
    fn emit_jump(&mut self) {}
    fn emit_branch(&mut self) {
        self.branches += 1;
    }
    fn emit_return(&mut self) {
        self.rets += 1;
        self.ops.push(RecOp::Return);
    }
    fn emit_host_call(&mut self, _symbol: &str, _argc: usize, _has_ret: bool) {}
    fn emit_host_call_typed(
        &mut self,
        _symbol: &str,
        _params: &[ParamKind],
        _has_ret: bool,
        _ret_is_f64: bool,
    ) {
    }
    fn emit_plugin_invoke(&mut self, _type_id: u32, _method_id: u32, _argc: usize, _has_ret: bool) {
    }
    fn emit_plugin_invoke_by_name(&mut self, _method: &str, _argc: usize, _has_ret: bool) {}
    fn prepare_blocks(&mut self, _count: usize) {}
    fn switch_to_block(&mut self, _index: usize) {}
    fn seal_block(&mut self, _index: usize) {}
    fn br_if_top_is_true(&mut self, _then_index: usize, _else_index: usize) {}
    fn jump_to(&mut self, _target_index: usize) {}
    fn ensure_block_params_i64(&mut self, _index: usize, _count: usize) {}
    fn ensure_block_params_b1(&mut self, index: usize, count: usize) {
        self.ensure_block_params_i64(index, count);
    }
    fn ensure_block_param_i64(&mut self, index: usize) {
        self.ensure_block_params_i64(index, 1);
    }
    fn push_block_param_i64_at(&mut self, _pos: usize) {}
    fn push_block_param_b1_at(&mut self, pos: usize) {
        self.push_block_param_i64_at(pos);
    }
    fn push_block_param_i64(&mut self) {
        self.push_block_param_i64_at(0);
    }
    fn br_if_with_args(
        &mut self,
        _then_index: usize,
        _else_index: usize,
        _then_n: usize,
        _else_n: usize,
    ) {
        self.emit_branch();
    }
    fn jump_with_args(&mut self, _target_index: usize, _n: usize) {
        self.emit_jump();
    }
    fn hint_ret_bool(&mut self, _is_b1: bool) {}
    fn ensure_local_i64(&mut self, _index: usize) {}
    fn store_local_i64(&mut self, _index: usize) {}
    fn load_local_i64(&mut self, _index: usize) {}
}
