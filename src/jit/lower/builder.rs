//! IR builder abstraction for JIT lowering
//!
//! This trait lets LowerCore target an abstract IR so we can plug Cranelift later
//! behind a feature flag. For now, we provide a NoopBuilder that counts calls.

#[derive(Debug, Clone, Copy)]
pub enum BinOpKind { Add, Sub, Mul, Div, Mod }

#[derive(Debug, Clone, Copy)]
pub enum CmpKind { Eq, Ne, Lt, Le, Gt, Ge }

pub trait IRBuilder {
    fn begin_function(&mut self, name: &str);
    fn end_function(&mut self);
    fn emit_const_i64(&mut self, _val: i64);
    fn emit_const_f64(&mut self, _val: f64);
    fn emit_binop(&mut self, _op: BinOpKind);
    fn emit_compare(&mut self, _op: CmpKind);
    fn emit_jump(&mut self);
    fn emit_branch(&mut self);
    fn emit_return(&mut self);
    /// Phase 10_d scaffolding: host-call emission (symbolic)
    fn emit_host_call(&mut self, _symbol: &str, _argc: usize, _has_ret: bool) { }
}

pub struct NoopBuilder {
    pub consts: usize,
    pub binops: usize,
    pub cmps: usize,
    pub branches: usize,
    pub rets: usize,
}

impl NoopBuilder {
    pub fn new() -> Self { Self { consts: 0, binops: 0, cmps: 0, branches: 0, rets: 0 } }
}

impl IRBuilder for NoopBuilder {
    fn begin_function(&mut self, _name: &str) {}
    fn end_function(&mut self) {}
    fn emit_const_i64(&mut self, _val: i64) { self.consts += 1; }
    fn emit_const_f64(&mut self, _val: f64) { self.consts += 1; }
    fn emit_binop(&mut self, _op: BinOpKind) { self.binops += 1; }
    fn emit_compare(&mut self, _op: CmpKind) { self.cmps += 1; }
    fn emit_jump(&mut self) { self.branches += 1; }
    fn emit_branch(&mut self) { self.branches += 1; }
    fn emit_return(&mut self) { self.rets += 1; }
}

#[cfg(feature = "cranelift-jit")]
pub struct CraneliftBuilder {
    pub module: cranelift_jit::JITModule,
    pub ctx: cranelift_codegen::Context,
    pub fbc: cranelift_frontend::FunctionBuilderContext,
    pub stats: (usize, usize, usize, usize, usize), // (consts, binops, cmps, branches, rets)
    // Build-state (minimal stack machine for Core-1)
    current_name: Option<String>,
    value_stack: Vec<cranelift_codegen::ir::Value>,
    entry_block: Option<cranelift_codegen::ir::Block>,
    // Finalized function pointer (if any)
    compiled_closure: Option<std::sync::Arc<dyn Fn(&[crate::backend::vm::VMValue]) -> crate::backend::vm::VMValue + Send + Sync>>, 
}

#[cfg(feature = "cranelift-jit")]
use cranelift_module::Module;
#[cfg(feature = "cranelift-jit")]
use cranelift_codegen::ir::InstBuilder;

#[cfg(feature = "cranelift-jit")]
impl IRBuilder for CraneliftBuilder {
    fn begin_function(&mut self, name: &str) {
        use cranelift_codegen::ir::{AbiParam, Signature, types};
        use cranelift_frontend::FunctionBuilder;

        self.current_name = Some(name.to_string());
        self.value_stack.clear();
        self.entry_block = None;

        // Minimal signature: () -> i64  (Core-1 integer path)
        let call_conv = self.module.isa().default_call_conv();
        let mut sig = Signature::new(call_conv);
        sig.returns.push(AbiParam::new(types::I64));
        self.ctx.func.signature = sig;
        self.ctx.func.name = cranelift_codegen::ir::UserFuncName::user(0, 0);

        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        let block = fb.create_block();
        fb.append_block_params_for_function_params(block);
        fb.switch_to_block(block);
        fb.seal_block(block);
        self.entry_block = Some(block);
        // Store builder back (drop at end_function)
        fb.finalize();
    }

    fn end_function(&mut self) {
        // Define and finalize into the module, create an invocable closure
        use cranelift_module::{Linkage, Module};
        if self.entry_block.is_none() {
            return;
        }

        // Declare a unique function symbol for JIT
        let sym_name = self.current_name.clone().unwrap_or_else(|| "jit_fn".to_string());
        let func_id = self.module.declare_function(&sym_name, Linkage::Local, &self.ctx.func.signature)
            .expect("declare_function failed");

        // Define
        self.module.define_function(func_id, &mut self.ctx)
            .expect("define_function failed");

        // Clear context for next compilation and finalize definitions
        self.module.clear_context(&mut self.ctx);
        self.module.finalize_definitions();

        // Get finalized code pointer and wrap into a safe closure
        let code = self.module.get_finalized_function(func_id);

        // SAFETY: We compiled a function with signature () -> i64
        unsafe {
            let f: extern "C" fn() -> i64 = std::mem::transmute(code);
            let closure = std::sync::Arc::new(move |_args: &[crate::backend::vm::VMValue]| -> crate::backend::vm::VMValue {
                let v = f();
                crate::backend::vm::VMValue::Integer(v)
            });
            self.compiled_closure = Some(closure);
        }
    }

    fn emit_const_i64(&mut self, val: i64) {
        use cranelift_codegen::ir::types;
        use cranelift_frontend::FunctionBuilder;
        // Recreate FunctionBuilder each emit (lightweight wrapper around ctx+fbc)
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        let v = fb.ins().iconst(types::I64, val);
        self.value_stack.push(v);
        self.stats.0 += 1;
        fb.finalize();
    }

    fn emit_const_f64(&mut self, _val: f64) { self.stats.0 += 1; /* not yet supported in Core-1 */ }

    fn emit_binop(&mut self, op: BinOpKind) {
        use cranelift_frontend::FunctionBuilder;
        if self.value_stack.len() < 2 { return; }
        let rhs = self.value_stack.pop().unwrap();
        let lhs = self.value_stack.pop().unwrap();
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        let res = match op {
            BinOpKind::Add => fb.ins().iadd(lhs, rhs),
            BinOpKind::Sub => fb.ins().isub(lhs, rhs),
            BinOpKind::Mul => fb.ins().imul(lhs, rhs),
            BinOpKind::Div => fb.ins().sdiv(lhs, rhs),
            BinOpKind::Mod => fb.ins().srem(lhs, rhs),
        };
        self.value_stack.push(res);
        self.stats.1 += 1;
        fb.finalize();
    }

    fn emit_compare(&mut self, op: CmpKind) {
        use cranelift_codegen::ir::{types, condcodes::IntCC};
        use cranelift_frontend::FunctionBuilder;
        if self.value_stack.len() < 2 { return; }
        let rhs = self.value_stack.pop().unwrap();
        let lhs = self.value_stack.pop().unwrap();
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        let cc = match op {
            CmpKind::Eq => IntCC::Equal,
            CmpKind::Ne => IntCC::NotEqual,
            CmpKind::Lt => IntCC::SignedLessThan,
            CmpKind::Le => IntCC::SignedLessThanOrEqual,
            CmpKind::Gt => IntCC::SignedGreaterThan,
            CmpKind::Ge => IntCC::SignedGreaterThanOrEqual,
        };
        let b1 = fb.ins().icmp(cc, lhs, rhs);
        let as_i64 = fb.ins().uextend(types::I64, b1);
        self.value_stack.push(as_i64);
        self.stats.2 += 1;
        fb.finalize();
    }
    fn emit_jump(&mut self) { self.stats.3 += 1; }
    fn emit_branch(&mut self) { self.stats.3 += 1; }

    fn emit_return(&mut self) {
        use cranelift_frontend::FunctionBuilder;
        self.stats.4 += 1;
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        if let Some(v) = self.value_stack.pop() {
            fb.ins().return_(&[v]);
        } else {
            // Return 0 if empty stack (defensive)
            use cranelift_codegen::ir::types;
            let zero = fb.ins().iconst(types::I64, 0);
            fb.ins().return_(&[zero]);
        }
        fb.finalize();
    }
}

#[cfg(feature = "cranelift-jit")]
impl CraneliftBuilder {
    pub fn new() -> Self {
        // Initialize a minimal JITModule to validate linking; not used yet
        let builder = cranelift_jit::JITBuilder::new(cranelift_module::default_libcall_names())
            .expect("failed to create JITBuilder");
        let module = cranelift_jit::JITModule::new(builder);
        let ctx = cranelift_codegen::Context::new();
        let fbc = cranelift_frontend::FunctionBuilderContext::new();
        CraneliftBuilder { 
            module, ctx, fbc, 
            stats: (0,0,0,0,0),
            current_name: None,
            value_stack: Vec::new(),
            entry_block: None,
            compiled_closure: None,
        }
    }

    /// Take ownership of compiled closure if available
    pub fn take_compiled_closure(&mut self) -> Option<std::sync::Arc<dyn Fn(&[crate::backend::vm::VMValue]) -> crate::backend::vm::VMValue + Send + Sync>> {
        self.compiled_closure.take()
    }
}
