//! IR builder abstraction (thin hub).
//!
//! Lowering targets the `IRBuilder` trait. Concrete backends live in
//! submodules and are enabled via feature flags.

#[derive(Debug, Clone, Copy)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, Clone, Copy)]
pub enum CmpKind {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    I64,
    F64,
    B1,
}

pub trait IRBuilder {
    fn begin_function(&mut self, name: &str);
    fn end_function(&mut self);
    fn prepare_signature_i64(&mut self, _argc: usize, _has_ret: bool) {}
    fn prepare_signature_typed(&mut self, _params: &[ParamKind], _ret_is_f64: bool) {}
    fn emit_param_i64(&mut self, _index: usize) {}
    fn emit_const_i64(&mut self, _val: i64);
    fn emit_const_f64(&mut self, _val: f64);
    fn emit_binop(&mut self, _op: BinOpKind);
    fn emit_compare(&mut self, _op: CmpKind);
    fn emit_jump(&mut self);
    fn emit_branch(&mut self);
    fn emit_return(&mut self);
    fn emit_select_i64(&mut self) {}
    fn emit_host_call(&mut self, _symbol: &str, _argc: usize, _has_ret: bool) {}
    fn emit_host_call_typed(
        &mut self,
        _symbol: &str,
        _params: &[ParamKind],
        _has_ret: bool,
        _ret_is_f64: bool,
    ) {
    }
    fn emit_host_call_fixed3(&mut self, _symbol: &str, _has_ret: bool) {}
    fn emit_plugin_invoke(&mut self, _type_id: u32, _method_id: u32, _argc: usize, _has_ret: bool) {
    }
    fn emit_plugin_invoke_by_name(&mut self, _method: &str, _argc: usize, _has_ret: bool) {}
    // Create a StringBox handle from a string literal and push its handle (i64) onto the stack.
    fn emit_string_handle_from_literal(&mut self, _s: &str) {}
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
    fn push_block_param_b1_at(&mut self, _pos: usize) {
        self.push_block_param_i64_at(_pos);
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
        self.br_if_top_is_true(_then_index, _else_index);
    }
    fn jump_with_args(&mut self, _target_index: usize, _n: usize) {
        self.jump_to(_target_index);
    }
    fn hint_ret_bool(&mut self, _is_b1: bool) {}
    fn ensure_local_i64(&mut self, _index: usize) {}
    fn store_local_i64(&mut self, _index: usize) {}
    fn load_local_i64(&mut self, _index: usize) {}
    // Optional debug hook: print a local i64 value with a tag (Cranelift JIT only)
    fn emit_debug_i64_local(&mut self, _tag: i64, _slot: usize) {}
}

mod noop;
pub use noop::NoopBuilder;

// Backend modules (feature-gated)
#[cfg(feature = "cranelift-jit")]
mod cranelift;
#[cfg(feature = "cranelift-jit")]
pub use cranelift::CraneliftBuilder;

#[cfg(feature = "cranelift-jit")]
mod object;
#[cfg(feature = "cranelift-jit")]
pub use object::ObjectBuilder;

// TLS and runtime shim submodules used by Cranelift backend
#[cfg(feature = "cranelift-jit")]
mod tls;
#[cfg(feature = "cranelift-jit")]
pub(crate) use tls::clif_tls;
#[cfg(feature = "cranelift-jit")]
mod rt_shims;
