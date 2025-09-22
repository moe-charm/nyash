#![cfg(feature = "cranelift-jit")]

use cranelift_codegen::ir::InstBuilder;
use cranelift_module::Module;

// TLS: 単一関数あたり1つの FunctionBuilder を保持（jit-direct 専用）
pub(crate) mod clif_tls {
    use super::*;
    thread_local! {
        pub static FB: std::cell::RefCell<Option<TlsCtx>> = std::cell::RefCell::new(None);
    }
    pub struct TlsCtx {
        pub ctx: Box<cranelift_codegen::Context>,
        pub fbc: Box<cranelift_frontend::FunctionBuilderContext>,
        pub(crate) fb: *mut cranelift_frontend::FunctionBuilder<'static>,
    }
    impl TlsCtx {
        pub fn new() -> Self {
            Self {
                ctx: Box::new(cranelift_codegen::Context::new()),
                fbc: Box::new(cranelift_frontend::FunctionBuilderContext::new()),
                fb: core::ptr::null_mut(),
            }
        }
        pub unsafe fn create(&mut self) {
            let func_ptr: *mut cranelift_codegen::ir::Function = &mut self.ctx.func;
            let fbc_ptr: *mut cranelift_frontend::FunctionBuilderContext = &mut *self.fbc;
            let fb = Box::new(cranelift_frontend::FunctionBuilder::new(
                &mut *func_ptr,
                &mut *fbc_ptr,
            ));
            self.fb = Box::into_raw(fb);
        }
        pub fn with<R>(
            &mut self,
            f: impl FnOnce(&mut cranelift_frontend::FunctionBuilder<'static>) -> R,
        ) -> R {
            unsafe { f(&mut *self.fb) }
        }
        pub unsafe fn finalize_drop(&mut self) {
            if !self.fb.is_null() {
                let fb = Box::from_raw(self.fb);
                fb.finalize();
                self.fb = core::ptr::null_mut();
            }
        }
        /// Finalize the current FunctionBuilder and take ownership of the underlying Context.
        pub fn take_context(&mut self) -> cranelift_codegen::Context {
            unsafe {
                self.finalize_drop();
            }
            // Move the current context out and replace with a fresh one
            let old = std::mem::replace(&mut self.ctx, Box::new(cranelift_codegen::Context::new()));
            *old
        }
    }
}

// Small TLS helpers to call imported functions via the single FunctionBuilder
pub(crate) fn tls_call_import_ret(
    module: &mut cranelift_jit::JITModule,
    func_id: cranelift_module::FuncId,
    args: &[cranelift_codegen::ir::Value],
    has_ret: bool,
) -> Option<cranelift_codegen::ir::Value> {
    clif_tls::FB.with(|cell| {
        let mut opt = cell.borrow_mut();
        let tls = opt.as_mut().expect("FunctionBuilder TLS not initialized");
        tls.with(|fb| {
            // Guard: avoid emitting a verifier-invalid call when args are unexpectedly empty.
            // Some early shims (e.g., instrumentation) may have declared a 1-arity import;
            // if lowering produced no arguments, synthesize a zero literal when a return is expected,
            // and skip the call entirely to keep the IR valid.
            if args.is_empty() {
                if has_ret {
                    use cranelift_codegen::ir::types;
                    return Some(fb.ins().iconst(types::I64, 0));
                } else {
                    return None;
                }
            }
            let fref = module.declare_func_in_func(func_id, fb.func);
            let call_inst = fb.ins().call(fref, args);
            if has_ret {
                fb.inst_results(call_inst).get(0).copied()
            } else {
                None
            }
        })
    })
}

pub(crate) fn tls_call_import_with_iconsts(
    module: &mut cranelift_jit::JITModule,
    func_id: cranelift_module::FuncId,
    iconsts: &[i64],
    tail_args: &[cranelift_codegen::ir::Value],
    has_ret: bool,
) -> Option<cranelift_codegen::ir::Value> {
    use cranelift_codegen::ir::types;
    clif_tls::FB.with(|cell| {
        let mut opt = cell.borrow_mut();
        let tls = opt.as_mut().expect("FunctionBuilder TLS not initialized");
        tls.with(|fb| {
            let mut all_args: Vec<cranelift_codegen::ir::Value> = Vec::new();
            for &c in iconsts {
                all_args.push(fb.ins().iconst(types::I64, c));
            }
            all_args.extend_from_slice(tail_args);
            let fref = module.declare_func_in_func(func_id, fb.func);
            let call_inst = fb.ins().call(fref, &all_args);
            if has_ret {
                fb.inst_results(call_inst).get(0).copied()
            } else {
                None
            }
        })
    })
}
