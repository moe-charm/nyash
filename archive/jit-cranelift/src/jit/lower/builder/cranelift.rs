#![cfg(feature = "cranelift-jit")]

// Cranelift-based IR builder moved out of builder.rs for readability and maintainability

use super::{BinOpKind, CmpKind, IRBuilder, ParamKind};
use cranelift_codegen::ir::InstBuilder;
use cranelift_module::Module;

// TLS utilities and runtime shims live next to this builder under the same module
use super::rt_shims::{
    nyash_host_stub0, nyash_jit_block_enter, nyash_jit_dbg_i64, nyash_plugin_invoke3_f64,
    nyash_plugin_invoke3_i64, nyash_plugin_invoke_name_call_i64,
    nyash_plugin_invoke_name_getattr_i64,
};
use super::tls::{self, clif_tls, tls_call_import_ret, tls_call_import_with_iconsts};

// Handle-based extern thunks used by lowering
use super::super::extern_thunks::{
    nyash_any_is_empty_h, nyash_any_length_h, nyash_array_get_h, nyash_array_last_h,
    nyash_array_len_h, nyash_array_push_h, nyash_array_set_h, nyash_box_birth_h,
    nyash_box_birth_i64, nyash_console_birth_h, nyash_gc_barrier_write, nyash_handle_of,
    nyash_integer_birth_h, nyash_map_get_h, nyash_map_get_hh, nyash_map_has_h, nyash_map_set_h,
    nyash_map_size_h, nyash_math_abs_f64, nyash_math_cos_f64, nyash_math_max_f64,
    nyash_math_min_f64, nyash_math_sin_f64, nyash_rt_checkpoint, nyash_string_birth_h,
    nyash_string_charcode_at_h, nyash_string_concat_hh, nyash_string_eq_hh, nyash_string_from_ptr,
    nyash_string_len_h, nyash_string_lt_hh,
};

use crate::jit::r#extern::r#async::nyash_future_await_h;
use crate::jit::r#extern::result::{nyash_result_err_h, nyash_result_ok_h};
use crate::{
    jit::events,
    mir::{Effect as OpEffect, MirFunction, MirType},
};

pub struct CraneliftBuilder {
    pub module: cranelift_jit::JITModule,
    pub ctx: cranelift_codegen::Context,
    pub fbc: cranelift_frontend::FunctionBuilderContext,
    pub stats: (usize, usize, usize, usize, usize), // (consts, binops, cmps, branches, rets)
    // Build-state (minimal stack machine for Core-1)
    current_name: Option<String>,
    value_stack: Vec<cranelift_codegen::ir::Value>,
    entry_block: Option<cranelift_codegen::ir::Block>,
    // Phase 10.7: basic block wiring state
    blocks: Vec<cranelift_codegen::ir::Block>,
    current_block_index: Option<usize>,
    block_param_counts: std::collections::HashMap<usize, usize>,
    // Local stack slots for minimal Load/Store lowering (i64 only)
    local_slots: std::collections::HashMap<usize, cranelift_codegen::ir::StackSlot>,
    // Finalized function pointer (if any)
    compiled_closure: Option<
        std::sync::Arc<
            dyn Fn(&[crate::jit::abi::JitValue]) -> crate::jit::abi::JitValue + Send + Sync,
        >,
    >,
    // Desired simple ABI (Phase 10_c minimal): i64 params count and i64 return
    desired_argc: usize,
    desired_has_ret: bool,
    desired_ret_is_f64: bool,
    typed_sig_prepared: bool,
    // Return-type hint: function returns boolean (footing only; ABI remains i64 for now)
    ret_hint_is_b1: bool,
    // Single-exit epilogue (jit-direct stability): ret block + i64 slot
    ret_block: Option<cranelift_codegen::ir::Block>,
    ret_slot: Option<cranelift_codegen::ir::StackSlot>,
    // Blocks requested before begin_function (to avoid TLS usage early)
    pending_blocks: usize,
    // Whether current block needs a terminator before switching away
    cur_needs_term: bool,
    // Track blocks sealed to avoid resealing
    sealed_blocks: std::collections::HashSet<usize>,
}

impl IRBuilder for CraneliftBuilder {
    fn prepare_signature_typed(&mut self, params: &[ParamKind], ret_is_f64: bool) {
        use cranelift_codegen::ir::{types, AbiParam, Signature};
        fn abi_param_for_kind(
            k: ParamKind,
            cfg: &crate::jit::config::JitConfig,
        ) -> cranelift_codegen::ir::AbiParam {
            use cranelift_codegen::ir::types;
            match k {
                ParamKind::I64 => cranelift_codegen::ir::AbiParam::new(types::I64),
                ParamKind::F64 => cranelift_codegen::ir::AbiParam::new(types::F64),
                ParamKind::B1 => {
                    let _ = cfg.native_bool_abi;
                    #[cfg(feature = "jit-b1-abi")]
                    {
                        if crate::jit::config::probe_capabilities().supports_b1_sig
                            && cfg.native_bool_abi
                        {
                            return cranelift_codegen::ir::AbiParam::new(types::B1);
                        }
                    }
                    cranelift_codegen::ir::AbiParam::new(types::I64)
                }
            }
        }
        self.desired_argc = params.len();
        self.desired_has_ret = true;
        self.desired_ret_is_f64 = ret_is_f64;
        let call_conv = self.module.isa().default_call_conv();
        let mut sig = Signature::new(call_conv);
        let cfg_now = crate::jit::config::current();
        for &k in params {
            sig.params.push(abi_param_for_kind(k, &cfg_now));
        }
        if self.desired_has_ret {
            if self.desired_ret_is_f64 {
                sig.returns.push(AbiParam::new(types::F64));
            } else {
                let mut used_b1 = false;
                #[cfg(feature = "jit-b1-abi")]
                {
                    let cfg_now = crate::jit::config::current();
                    if crate::jit::config::probe_capabilities().supports_b1_sig
                        && cfg_now.native_bool_abi
                        && self.ret_hint_is_b1
                    {
                        sig.returns.push(AbiParam::new(types::B1));
                        used_b1 = true;
                    }
                }
                if !used_b1 {
                    sig.returns.push(AbiParam::new(types::I64));
                }
            }
        }
        self.ctx.func.signature = sig;
        self.typed_sig_prepared = true;
    }
    fn emit_param_i64(&mut self, index: usize) {
        if let Some(v) = self.entry_param(index) {
            self.value_stack.push(v);
        }
    }
    fn prepare_signature_i64(&mut self, argc: usize, _has_ret: bool) {
        self.desired_argc = argc;
        // JIT-direct stability: always materialize an i64 return slot (VMValue Integer/Bool/Float can be coerced)
        self.desired_has_ret = true;
        // i64-only signature: return type must be i64 regardless of host f64 capability
        self.desired_ret_is_f64 = false;
    }
    fn begin_function(&mut self, name: &str) {
        use cranelift_codegen::ir::{types, AbiParam, Signature};
        self.current_name = Some(name.to_string());
        self.value_stack.clear();
        clif_tls::FB.with(|cell| {
            let mut tls = clif_tls::TlsCtx::new();
            let call_conv = self.module.isa().default_call_conv();
            let mut sig = Signature::new(call_conv);
            if std::env::var("NYASH_JIT_TRACE_SIG").ok().as_deref() == Some("1") {
                eprintln!(
                    "[SIG] begin desired: argc={} has_ret={} ret_is_f64={} typed_prepared={}",
                    self.desired_argc,
                    self.desired_has_ret,
                    self.desired_ret_is_f64,
                    self.typed_sig_prepared
                );
            }
            if !self.typed_sig_prepared {
                for _ in 0..self.desired_argc {
                    sig.params.push(AbiParam::new(types::I64));
                }
                if self.desired_has_ret {
                    if self.desired_ret_is_f64 {
                        sig.returns.push(AbiParam::new(types::F64));
                    } else {
                        sig.returns.push(AbiParam::new(types::I64));
                    }
                }
            } else {
                for _ in 0..self.desired_argc {
                    sig.params.push(AbiParam::new(types::I64));
                }
                if self.desired_has_ret {
                    let mut used_b1 = false;
                    #[cfg(feature = "jit-b1-abi")]
                    {
                        let cfg_now = crate::jit::config::current();
                        if crate::jit::config::probe_capabilities().supports_b1_sig
                            && cfg_now.native_bool_abi
                            && self.ret_hint_is_b1
                        {
                            sig.returns.push(AbiParam::new(types::B1));
                            used_b1 = true;
                        }
                    }
                    if !used_b1 {
                        if self.desired_ret_is_f64 {
                            sig.returns.push(AbiParam::new(types::F64));
                        } else {
                            sig.returns.push(AbiParam::new(types::I64));
                        }
                    }
                }
            }
            tls.ctx.func.signature = sig;
            tls.ctx.func.name = cranelift_codegen::ir::UserFuncName::user(0, 0);
            unsafe {
                tls.create();
            }
            tls.with(|fb| {
                if self.blocks.is_empty() {
                    let block = fb.create_block();
                    self.blocks.push(block);
                }
                if self.pending_blocks > self.blocks.len() {
                    let to_create = self.pending_blocks - self.blocks.len();
                    for _ in 0..to_create {
                        self.blocks.push(fb.create_block());
                    }
                }
                let entry = self.blocks[0];
                fb.append_block_params_for_function_params(entry);
                fb.switch_to_block(entry);
                self.entry_block = Some(entry);
                self.current_block_index = Some(0);
                self.cur_needs_term = true;
                // Force a dbg call at function entry to verify import linking works at runtime
                {
                    use cranelift_codegen::ir::{AbiParam, Signature};
                    let mut sig = Signature::new(self.module.isa().default_call_conv());
                    sig.params.push(AbiParam::new(types::I64));
                    sig.params.push(AbiParam::new(types::I64));
                    sig.returns.push(AbiParam::new(types::I64));
                    let fid = self
                        .module
                        .declare_function(
                            "nyash.jit.dbg_i64",
                            cranelift_module::Linkage::Import,
                            &sig,
                        )
                        .expect("declare dbg_i64 at entry");
                    let fref = self.module.declare_func_in_func(fid, fb.func);
                    let ttag = fb.ins().iconst(types::I64, 900);
                    let tval = fb.ins().iconst(types::I64, 123);
                    let _ = fb.ins().call(fref, &[ttag, tval]);
                }
                let rb = fb.create_block();
                self.ret_block = Some(rb);
                fb.append_block_param(rb, types::I64);
                self.blocks.push(rb);
                self.ret_slot = None;
            });
            cell.replace(Some(tls));
        });
    }
    fn end_function(&mut self) {
        use cranelift_module::Linkage;
        if self.entry_block.is_none() {
            return;
        }
        let mut ctx_opt: Option<cranelift_codegen::Context> = None;
        clif_tls::FB.with(|cell| {
            if let Some(mut tls) = cell.take() {
                tls.with(|fb| {
                    use cranelift_codegen::ir::types;
                    if let Some(rb) = self.ret_block {
                        if let Some(cur) = self.current_block_index {
                            if self.cur_needs_term && self.blocks[cur] != rb {
                                fb.ins().jump(rb, &[]);
                                self.cur_needs_term = false;
                            }
                        }
                        fb.switch_to_block(rb);
                        if fb.func.signature.returns.is_empty() {
                            fb.ins().return_(&[]);
                        } else {
                            // Prefer the persisted return slot if available; fallback to block param 0
                            let mut v = if let Some(ss) = self.ret_slot {
                                fb.ins().stack_load(types::I64, ss, 0)
                            } else {
                                let params = fb.func.dfg.block_params(rb).to_vec();
                                params
                                    .get(0)
                                    .copied()
                                    .unwrap_or_else(|| fb.ins().iconst(types::I64, 0))
                            };
                            // Unconditional runtime debug call to observe return value just before final return (feed result back)
                            {
                                use cranelift_codegen::ir::{AbiParam, Signature};
                                let mut sig = Signature::new(self.module.isa().default_call_conv());
                                sig.params.push(AbiParam::new(types::I64));
                                sig.params.push(AbiParam::new(types::I64));
                                sig.returns.push(AbiParam::new(types::I64));
                                let fid = self
                                    .module
                                    .declare_function("nyash.jit.dbg_i64", Linkage::Import, &sig)
                                    .expect("declare dbg_i64");
                                let fref = self.module.declare_func_in_func(fid, fb.func);
                                let tag = fb.ins().iconst(types::I64, 210);
                                let call_inst = fb.ins().call(fref, &[tag, v]);
                                if let Some(rv) = fb.inst_results(call_inst).get(0).copied() {
                                    v = rv;
                                }
                            }
                            let ret_ty = fb
                                .func
                                .signature
                                .returns
                                .get(0)
                                .map(|p| p.value_type)
                                .unwrap_or(types::I64);
                            if ret_ty == types::F64 {
                                v = fb.ins().fcvt_from_sint(types::F64, v);
                            }
                            fb.ins().return_(&[v]);
                        }
                    }
                    // Seal all blocks to satisfy CLIF verifier
                    for &b in &self.blocks {
                        fb.seal_block(b);
                    }
                });
                ctx_opt = Some(tls.take_context());
            }
        });
        if let Some(mut ctx) = ctx_opt.take() {
            let func_name = self.current_name.as_deref().unwrap_or("jit_func");
            let func_id = self
                .module
                .declare_function(func_name, Linkage::Local, &ctx.func.signature)
                .expect("declare function");
            if std::env::var("NYASH_JIT_TRACE_SIG").ok().as_deref() == Some("1") {
                eprintln!(
                    "[SIG] end returns={} params={}",
                    ctx.func.signature.returns.len(),
                    ctx.func.signature.params.len()
                );
            }
            if std::env::var("NYASH_JIT_DUMP_CLIF").ok().as_deref() == Some("1") {
                eprintln!("[CLIF] {}\n{}", func_name, ctx.func.display());
            }
            self.module
                .define_function(func_id, &mut ctx)
                .expect("define function");
            self.module.clear_context(&mut ctx);
            let _ = self.module.finalize_definitions();
            let code = self.module.get_finalized_function(func_id);
            // Build a callable closure capturing the code pointer
            let argc = self.desired_argc;
            let has_ret = self.desired_has_ret;
            let ret_is_f64 = self.desired_has_ret && self.desired_ret_is_f64;
            let code_usize = code as usize;
            unsafe {
                let closure = std::sync::Arc::new(
                    move |args: &[crate::jit::abi::JitValue]| -> crate::jit::abi::JitValue {
                        let mut a: [i64; 6] = [0; 6];
                        let take = core::cmp::min(core::cmp::min(argc, 6), args.len());
                        for i in 0..take {
                            a[i] = match args[i] {
                                crate::jit::abi::JitValue::I64(v) => v,
                                crate::jit::abi::JitValue::Bool(b) => {
                                    if b {
                                        1
                                    } else {
                                        0
                                    }
                                }
                                crate::jit::abi::JitValue::F64(f) => f as i64,
                                crate::jit::abi::JitValue::Handle(h) => h as i64,
                            };
                        }
                        let ret_i64 = if has_ret {
                            match argc {
                                0 => {
                                    let f: extern "C" fn() -> i64 = std::mem::transmute(code_usize);
                                    f()
                                }
                                1 => {
                                    let f: extern "C" fn(i64) -> i64 =
                                        std::mem::transmute(code_usize);
                                    f(a[0])
                                }
                                2 => {
                                    let f: extern "C" fn(i64, i64) -> i64 =
                                        std::mem::transmute(code_usize);
                                    f(a[0], a[1])
                                }
                                3 => {
                                    let f: extern "C" fn(i64, i64, i64) -> i64 =
                                        std::mem::transmute(code_usize);
                                    f(a[0], a[1], a[2])
                                }
                                4 => {
                                    let f: extern "C" fn(i64, i64, i64, i64) -> i64 =
                                        std::mem::transmute(code_usize);
                                    f(a[0], a[1], a[2], a[3])
                                }
                                5 => {
                                    let f: extern "C" fn(i64, i64, i64, i64, i64) -> i64 =
                                        std::mem::transmute(code_usize);
                                    f(a[0], a[1], a[2], a[3], a[4])
                                }
                                _ => {
                                    let f: extern "C" fn(i64, i64, i64, i64, i64, i64) -> i64 =
                                        std::mem::transmute(code_usize);
                                    f(a[0], a[1], a[2], a[3], a[4], a[5])
                                }
                            }
                        } else {
                            0
                        };
                        if has_ret && ret_is_f64 {
                            let ret_f64 = match argc {
                                0 => {
                                    let f: extern "C" fn() -> f64 = std::mem::transmute(code_usize);
                                    f()
                                }
                                1 => {
                                    let f: extern "C" fn(i64) -> f64 =
                                        std::mem::transmute(code_usize);
                                    f(a[0])
                                }
                                2 => {
                                    let f: extern "C" fn(i64, i64) -> f64 =
                                        std::mem::transmute(code_usize);
                                    f(a[0], a[1])
                                }
                                3 => {
                                    let f: extern "C" fn(i64, i64, i64) -> f64 =
                                        std::mem::transmute(code_usize);
                                    f(a[0], a[1], a[2])
                                }
                                4 => {
                                    let f: extern "C" fn(i64, i64, i64, i64) -> f64 =
                                        std::mem::transmute(code_usize);
                                    f(a[0], a[1], a[2], a[3])
                                }
                                5 => {
                                    let f: extern "C" fn(i64, i64, i64, i64, i64) -> f64 =
                                        std::mem::transmute(code_usize);
                                    f(a[0], a[1], a[2], a[3], a[4])
                                }
                                _ => {
                                    let f: extern "C" fn(i64, i64, i64, i64, i64, i64) -> f64 =
                                        std::mem::transmute(code_usize);
                                    f(a[0], a[1], a[2], a[3], a[4], a[5])
                                }
                            };
                            if std::env::var("NYASH_JIT_TRACE_CALL").ok().as_deref() == Some("1") {
                                eprintln!("[JIT-CALL] ret_f64={}", ret_f64);
                            }
                            return crate::jit::abi::JitValue::F64(ret_f64);
                        }
                        if std::env::var("NYASH_JIT_TRACE_CALL").ok().as_deref() == Some("1") {
                            eprintln!("[JIT-CALL] ret_i64={}", ret_i64);
                        }
                        crate::jit::abi::JitValue::I64(ret_i64)
                    },
                );
                self.compiled_closure = Some(closure);
            }
        }
    }
    fn emit_const_i64(&mut self, val: i64) {
        use cranelift_codegen::ir::types;
        let v = Self::with_fb(|fb| fb.ins().iconst(types::I64, val));
        self.value_stack.push(v);
        self.stats.0 += 1;
    }
    fn emit_const_f64(&mut self, val: f64) {
        use cranelift_codegen::ir::types;
        let v = Self::with_fb(|fb| fb.ins().f64const(val));
        self.value_stack.push(v);
        self.stats.0 += 1;
    }
    fn emit_binop(&mut self, op: BinOpKind) {
        use cranelift_codegen::ir::types;
        if self.value_stack.len() < 2 {
            return;
        }
        let mut rhs = self.value_stack.pop().unwrap();
        let mut lhs = self.value_stack.pop().unwrap();
        let res = Self::with_fb(|fb| {
            let lty = fb.func.dfg.value_type(lhs);
            let rty = fb.func.dfg.value_type(rhs);
            let native_f64 = crate::jit::config::current().native_f64;
            let use_f64 = native_f64 && (lty == types::F64 || rty == types::F64);
            if use_f64 {
                if lty != types::F64 {
                    lhs = fb.ins().fcvt_from_sint(types::F64, lhs);
                }
                if rty != types::F64 {
                    rhs = fb.ins().fcvt_from_sint(types::F64, rhs);
                }
                match op {
                    BinOpKind::Add => fb.ins().fadd(lhs, rhs),
                    BinOpKind::Sub => fb.ins().fsub(lhs, rhs),
                    BinOpKind::Mul => fb.ins().fmul(lhs, rhs),
                    BinOpKind::Div => fb.ins().fdiv(lhs, rhs),
                    // Cranelift does not have a native fmod; approximate by integer remainder on truncated values
                    BinOpKind::Mod => {
                        let li = fb
                            .ins()
                            .fcvt_to_sint(cranelift_codegen::ir::types::I64, lhs);
                        let ri = fb
                            .ins()
                            .fcvt_to_sint(cranelift_codegen::ir::types::I64, rhs);
                        fb.ins().srem(li, ri)
                    }
                }
            } else {
                match op {
                    BinOpKind::Add => fb.ins().iadd(lhs, rhs),
                    BinOpKind::Sub => fb.ins().isub(lhs, rhs),
                    BinOpKind::Mul => fb.ins().imul(lhs, rhs),
                    BinOpKind::Div => fb.ins().sdiv(lhs, rhs),
                    BinOpKind::Mod => fb.ins().srem(lhs, rhs),
                }
            }
        });
        self.value_stack.push(res);
        self.stats.1 += 1;
    }
    fn emit_compare(&mut self, op: CmpKind) {
        use cranelift_codegen::ir::{
            condcodes::{FloatCC, IntCC},
            types,
        };
        if self.value_stack.len() < 2 {
            return;
        }
        let mut rhs = self.value_stack.pop().unwrap();
        let mut lhs = self.value_stack.pop().unwrap();
        Self::with_fb(|fb| {
            let lty = fb.func.dfg.value_type(lhs);
            let rty = fb.func.dfg.value_type(rhs);
            let native_f64 = crate::jit::config::current().native_f64;
            let use_f64 = native_f64 && (lty == types::F64 || rty == types::F64);
            let b1 = if use_f64 {
                if lty != types::F64 {
                    lhs = fb.ins().fcvt_from_sint(types::F64, lhs);
                }
                if rty != types::F64 {
                    rhs = fb.ins().fcvt_from_sint(types::F64, rhs);
                }
                let cc = match op {
                    CmpKind::Eq => FloatCC::Equal,
                    CmpKind::Ne => FloatCC::NotEqual,
                    CmpKind::Lt => FloatCC::LessThan,
                    CmpKind::Le => FloatCC::LessThanOrEqual,
                    CmpKind::Gt => FloatCC::GreaterThan,
                    CmpKind::Ge => FloatCC::GreaterThanOrEqual,
                };
                fb.ins().fcmp(cc, lhs, rhs)
            } else {
                let cc = match op {
                    CmpKind::Eq => IntCC::Equal,
                    CmpKind::Ne => IntCC::NotEqual,
                    CmpKind::Lt => IntCC::SignedLessThan,
                    CmpKind::Le => IntCC::SignedLessThanOrEqual,
                    CmpKind::Gt => IntCC::SignedGreaterThan,
                    CmpKind::Ge => IntCC::SignedGreaterThanOrEqual,
                };
                fb.ins().icmp(cc, lhs, rhs)
            };
            self.value_stack.push(b1);
            self.stats.2 += 1;
        });
    }
    fn emit_select_i64(&mut self) {
        use cranelift_codegen::ir::{condcodes::IntCC, types};
        if self.value_stack.len() < 3 {
            return;
        }
        let mut else_v = self.value_stack.pop().unwrap();
        let mut then_v = self.value_stack.pop().unwrap();
        let mut cond_v = self.value_stack.pop().unwrap();
        let sel = Self::with_fb(|fb| {
            let cty = fb.func.dfg.value_type(cond_v);
            if cty == types::I64 {
                cond_v = fb.ins().icmp_imm(IntCC::NotEqual, cond_v, 0);
                crate::jit::rt::b1_norm_inc(1);
            }
            let tty = fb.func.dfg.value_type(then_v);
            if tty != types::I64 {
                then_v = fb.ins().fcvt_to_sint(types::I64, then_v);
            }
            let ety = fb.func.dfg.value_type(else_v);
            if ety != types::I64 {
                else_v = fb.ins().fcvt_to_sint(types::I64, else_v);
            }
            if std::env::var("NYASH_JIT_TRACE_SEL").ok().as_deref() == Some("1") {
                use cranelift_codegen::ir::{AbiParam, Signature};
                let mut sig = Signature::new(self.module.isa().default_call_conv());
                sig.params.push(AbiParam::new(types::I64));
                sig.params.push(AbiParam::new(types::I64));
                sig.returns.push(AbiParam::new(types::I64));
                let fid = self
                    .module
                    .declare_function("nyash.jit.dbg_i64", cranelift_module::Linkage::Import, &sig)
                    .expect("declare dbg_i64");
                let fref = self.module.declare_func_in_func(fid, fb.func);
                let t_cond = fb.ins().iconst(types::I64, 100);
                let one = fb.ins().iconst(types::I64, 1);
                let zero = fb.ins().iconst(types::I64, 0);
                let ci = fb.ins().select(cond_v, one, zero);
                let _ = fb.ins().call(fref, &[t_cond, ci]);
                let t_then = fb.ins().iconst(types::I64, 101);
                let _ = fb.ins().call(fref, &[t_then, then_v]);
                let t_else = fb.ins().iconst(types::I64, 102);
                let _ = fb.ins().call(fref, &[t_else, else_v]);
            }
            fb.ins().select(cond_v, then_v, else_v)
        });
        self.value_stack.push(sel);
    }
    fn emit_jump(&mut self) {
        self.stats.3 += 1;
    }
    fn emit_branch(&mut self) {
        self.stats.3 += 1;
    }
    fn emit_return(&mut self) {
        use cranelift_codegen::ir::types;
        self.stats.4 += 1;
        Self::with_fb(|fb| {
            if fb.func.signature.returns.is_empty() {
                fb.ins().return_(&[]);
                return;
            }
            let mut v = if let Some(x) = self.value_stack.pop() {
                x
            } else {
                fb.ins().iconst(types::I64, 0)
            };
            let v_ty = fb.func.dfg.value_type(v);
            if v_ty != types::I64 {
                v = if v_ty == types::F64 {
                    fb.ins().fcvt_to_sint(types::I64, v)
                } else {
                    let one = fb.ins().iconst(types::I64, 1);
                    let zero = fb.ins().iconst(types::I64, 0);
                    fb.ins().select(v, one, zero)
                }
            }
            if std::env::var("NYASH_JIT_TRACE_RET").ok().as_deref() == Some("1")
                || std::env::var("NYASH_JIT_FORCE_RET_DBG").ok().as_deref() == Some("1")
            {
                use cranelift_codegen::ir::{AbiParam, Signature};
                let mut sig = Signature::new(self.module.isa().default_call_conv());
                sig.params.push(AbiParam::new(types::I64));
                sig.params.push(AbiParam::new(types::I64));
                sig.returns.push(AbiParam::new(types::I64));
                let fid = self
                    .module
                    .declare_function("nyash.jit.dbg_i64", cranelift_module::Linkage::Import, &sig)
                    .expect("declare dbg_i64");
                let fref = self.module.declare_func_in_func(fid, fb.func);
                let tag = fb.ins().iconst(types::I64, 201);
                let _ = fb.ins().call(fref, &[tag, v]);
            }
            // Persist return value in a dedicated stack slot to avoid SSA arg mishaps on ret block
            if self.ret_slot.is_none() {
                use cranelift_codegen::ir::StackSlotData;
                let ss = fb.create_sized_stack_slot(StackSlotData::new(
                    cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
                    8,
                ));
                self.ret_slot = Some(ss);
            }
            if let Some(ss) = self.ret_slot {
                fb.ins().stack_store(v, ss, 0);
            }
            // Unconditional debug of return value just before ret block jump (feed result back to v)
            {
                use cranelift_codegen::ir::{AbiParam, Signature};
                let mut sig = Signature::new(self.module.isa().default_call_conv());
                sig.params.push(AbiParam::new(types::I64));
                sig.params.push(AbiParam::new(types::I64));
                sig.returns.push(AbiParam::new(types::I64));
                let fid = self
                    .module
                    .declare_function("nyash.jit.dbg_i64", cranelift_module::Linkage::Import, &sig)
                    .expect("declare dbg_i64");
                let fref = self.module.declare_func_in_func(fid, fb.func);
                let tag = fb.ins().iconst(types::I64, 211);
                let call_inst = fb.ins().call(fref, &[tag, v]);
                if let Some(rv) = fb.inst_results(call_inst).get(0).copied() {
                    v = rv;
                }
            }
            if let Some(rb) = self.ret_block {
                fb.ins().jump(rb, &[v]);
            }
        });
        self.cur_needs_term = false;
    }
    fn emit_host_call(&mut self, symbol: &str, _argc: usize, has_ret: bool) {
        use cranelift_codegen::ir::{types, AbiParam, Signature};
        // Structured lower event for import call
        {
            let mut arg_types: Vec<&'static str> = Vec::new();
            for _ in 0.._argc {
                arg_types.push("I64");
            }
            crate::jit::events::emit_lower(
                serde_json::json!({
                    "id": symbol,
                    "decision": "allow",
                    "reason": "import_call",
                    "argc": _argc,
                    "arg_types": arg_types,
                    "ret": if has_ret { "I64" } else { "Void" }
                }),
                "hostcall",
                "<jit>",
            );
        }
        let call_conv = self.module.isa().default_call_conv();
        let mut sig = Signature::new(call_conv);
        // Collect up to _argc i64 values from stack (right-to-left) and pad with zeros to match arity
        let mut args: Vec<cranelift_codegen::ir::Value> = Vec::new();
        let take_n = _argc.min(self.value_stack.len());
        for _ in 0..take_n {
            if let Some(v) = self.value_stack.pop() {
                args.push(v);
            }
        }
        args.reverse();
        Self::with_fb(|fb| {
            while args.len() < _argc {
                args.push(fb.ins().iconst(types::I64, 0));
            }
        });
        for _ in 0.._argc {
            sig.params.push(AbiParam::new(types::I64));
        }

        let func_id = self
            .module
            .declare_function(symbol, cranelift_module::Linkage::Import, &sig)
            .expect("declare import failed");
        if let Some(v) = tls_call_import_ret(&mut self.module, func_id, &args, has_ret) {
            self.value_stack.push(v);
        }
    }
    fn emit_host_call_typed(
        &mut self,
        symbol: &str,
        params: &[ParamKind],
        has_ret: bool,
        ret_is_f64: bool,
    ) {
        use cranelift_codegen::ir::{types, AbiParam, Signature};
        // Structured lower event for typed import call
        {
            let mut arg_types: Vec<&'static str> = Vec::new();
            for k in params {
                arg_types.push(match k {
                    ParamKind::I64 | ParamKind::B1 => "I64",
                    ParamKind::F64 => "F64",
                });
            }
            crate::jit::events::emit_lower(
                serde_json::json!({
                    "id": symbol,
                    "decision": "allow",
                    "reason": "import_call_typed",
                    "argc": params.len(),
                    "arg_types": arg_types,
                    "ret": if has_ret { if ret_is_f64 { "F64" } else { "I64" } } else { "Void" }
                }),
                "hostcall",
                "<jit>",
            );
        }
        let mut args: Vec<cranelift_codegen::ir::Value> = Vec::new();
        let take_n = params.len().min(self.value_stack.len());
        for _ in 0..take_n {
            if let Some(v) = self.value_stack.pop() {
                args.push(v);
            }
        }
        args.reverse();
        let call_conv = self.module.isa().default_call_conv();
        let mut sig = Signature::new(call_conv);
        let abi_param_for_kind = |k: &ParamKind| match k {
            ParamKind::I64 => AbiParam::new(types::I64),
            ParamKind::F64 => AbiParam::new(types::F64),
            ParamKind::B1 => AbiParam::new(types::I64),
        };
        for k in params {
            sig.params.push(abi_param_for_kind(k));
        }
        if has_ret {
            if ret_is_f64 {
                sig.returns.push(AbiParam::new(types::F64));
            } else {
                sig.returns.push(AbiParam::new(types::I64));
            }
        }
        let func_id = self
            .module
            .declare_function(symbol, cranelift_module::Linkage::Import, &sig)
            .expect("declare typed import failed");
        if let Some(v) = tls_call_import_ret(&mut self.module, func_id, &args, has_ret) {
            self.value_stack.push(v);
        }
    }
    fn emit_debug_i64_local(&mut self, tag: i64, slot: usize) {
        if std::env::var("NYASH_JIT_TRACE_LEN").ok().as_deref() != Some("1") {
            return;
        }
        use cranelift_codegen::ir::types;
        // Push tag and value
        let t = Self::with_fb(|fb| fb.ins().iconst(types::I64, tag));
        self.value_stack.push(t);
        self.load_local_i64(slot);
        // Use existing typed hostcall helper to pass two I64 args
        self.emit_host_call_typed(
            "nyash.jit.dbg_i64",
            &[ParamKind::I64, ParamKind::I64],
            true,
            false,
        );
        // Drop the returned value to keep stack balanced
        let _ = self.value_stack.pop();
    }
    fn emit_host_call_fixed3(&mut self, symbol: &str, has_ret: bool) {
        use cranelift_codegen::ir::{types, AbiParam, Signature};
        let mut args: Vec<cranelift_codegen::ir::Value> = Vec::new();
        // Pop up to 3 values; pad with zeros to reach exactly 3
        let take_n = core::cmp::min(3, self.value_stack.len());
        for _ in 0..take_n {
            if let Some(v) = self.value_stack.pop() {
                args.push(v);
            }
        }
        args.reverse();
        Self::with_fb(|fb| {
            while args.len() < 3 {
                args.push(fb.ins().iconst(types::I64, 0));
            }
        });
        let call_conv = self.module.isa().default_call_conv();
        let mut sig = Signature::new(call_conv);
        for _ in 0..3 {
            sig.params.push(AbiParam::new(types::I64));
        }
        // Always declare with I64 return to keep signature stable across call sites
        sig.returns.push(AbiParam::new(types::I64));
        let func_id = self
            .module
            .declare_function(symbol, cranelift_module::Linkage::Import, &sig)
            .expect("declare import fixed3 failed");
        if let Some(v) = tls_call_import_ret(&mut self.module, func_id, &args, true) {
            if has_ret {
                self.value_stack.push(v);
            }
        }
    }
    fn emit_plugin_invoke(&mut self, type_id: u32, method_id: u32, argc: usize, has_ret: bool) {
        use cranelift_codegen::ir::{types, AbiParam, Signature};
        // Pop argc values (right-to-left): receiver + up to 2 args
        let mut arg_vals: Vec<cranelift_codegen::ir::Value> = {
            let take_n = argc.min(self.value_stack.len());
            let mut tmp = Vec::new();
            for _ in 0..take_n {
                if let Some(v) = self.value_stack.pop() {
                    tmp.push(v);
                }
            }
            tmp.reverse();
            tmp
        };
        // Ensure receiver (a0) is a runtime handle via nyash.handle.of
        let a0_handle = {
            use crate::jit::r#extern::handles as h;
            let call_conv_h = self.module.isa().default_call_conv();
            let mut sig_h = Signature::new(call_conv_h);
            sig_h.params.push(AbiParam::new(types::I64));
            sig_h.returns.push(AbiParam::new(types::I64));
            let func_id_h = self
                .module
                .declare_function(h::SYM_HANDLE_OF, cranelift_module::Linkage::Import, &sig_h)
                .expect("declare handle.of failed");
            tls_call_import_ret(&mut self.module, func_id_h, &arg_vals[0..1], true)
                .expect("handle.of ret")
        };
        arg_vals[0] = a0_handle;
        // f64 shim allowed by env allowlist
        let use_f64 = if has_ret {
            if let Ok(list) = std::env::var("NYASH_JIT_PLUGIN_F64") {
                list.split(',').any(|e| { let mut it = e.split(':'); matches!((it.next(), it.next()), (Some(t), Some(m)) if t.parse::<u32>().ok()==Some(type_id) && m.parse::<u32>().ok()==Some(method_id)) })
            } else {
                false
            }
        } else {
            false
        };
        let call_conv = self.module.isa().default_call_conv();
        let mut sig = Signature::new(call_conv);
        for _ in 0..6 {
            sig.params.push(AbiParam::new(types::I64));
        }
        if has_ret {
            sig.returns
                .push(AbiParam::new(if use_f64 { types::F64 } else { types::I64 }));
        }
        let symbol = if use_f64 {
            "nyash_plugin_invoke3_f64"
        } else {
            "nyash_plugin_invoke3_i64"
        };
        let func_id = self
            .module
            .declare_function(symbol, cranelift_module::Linkage::Import, &sig)
            .expect("declare plugin shim failed");
        let ret_val = Self::with_fb(|fb| {
            if let Some(idx) = self.current_block_index {
                fb.switch_to_block(self.blocks[idx]);
            } else if let Some(b) = self.entry_block {
                fb.switch_to_block(b);
            }
            while arg_vals.len() < 3 {
                let z = fb.ins().iconst(types::I64, 0);
                arg_vals.push(z);
            }
            // handle.of on receiver (redundant-safe)
            let call_conv_h = self.module.isa().default_call_conv();
            let mut sig_h = Signature::new(call_conv_h);
            sig_h.params.push(AbiParam::new(types::I64));
            sig_h.returns.push(AbiParam::new(types::I64));
            let func_id_h = self
                .module
                .declare_function(
                    crate::jit::r#extern::handles::SYM_HANDLE_OF,
                    cranelift_module::Linkage::Import,
                    &sig_h,
                )
                .expect("declare handle.of failed");
            let fref_h = self.module.declare_func_in_func(func_id_h, fb.func);
            let call_h = fb.ins().call(fref_h, &[arg_vals[0]]);
            if let Some(rv) = fb.inst_results(call_h).get(0).copied() {
                arg_vals[0] = rv;
            }
            let fref = self.module.declare_func_in_func(func_id, fb.func);
            let c_type = fb.ins().iconst(types::I64, type_id as i64);
            let c_meth = fb.ins().iconst(types::I64, method_id as i64);
            let c_argc = fb.ins().iconst(types::I64, argc as i64);
            let call_inst = fb.ins().call(
                fref,
                &[
                    c_type,
                    c_meth,
                    c_argc,
                    arg_vals[0],
                    arg_vals[1],
                    arg_vals[2],
                ],
            );
            if has_ret {
                fb.inst_results(call_inst).get(0).copied()
            } else {
                None
            }
        });
        if let Some(v) = ret_val {
            self.value_stack.push(v);
        }
    }
    fn emit_plugin_invoke_by_name(&mut self, method: &str, argc: usize, has_ret: bool) {
        use cranelift_codegen::ir::{types, AbiParam, Signature};
        // Collect call args
        let mut arg_vals: Vec<cranelift_codegen::ir::Value> = {
            let take_n = argc.min(self.value_stack.len());
            let mut tmp = Vec::new();
            for _ in 0..take_n {
                if let Some(v) = self.value_stack.pop() {
                    tmp.push(v);
                }
            }
            tmp.reverse();
            tmp
        };
        // Signature: nyash_plugin_invoke_name_*(argc, a0, a1, a2)
        let mut sig = Signature::new(self.module.isa().default_call_conv());
        sig.params.push(AbiParam::new(types::I64));
        sig.params.push(AbiParam::new(types::I64));
        sig.params.push(AbiParam::new(types::I64));
        sig.params.push(AbiParam::new(types::I64));
        sig.returns.push(AbiParam::new(types::I64));

        let sym = match method {
            "getattr" => "nyash_plugin_invoke_name_getattr_i64",
            _ => "nyash_plugin_invoke_name_call_i64",
        };
        let func_id = self
            .module
            .declare_function(sym, cranelift_module::Linkage::Import, &sig)
            .expect("declare name shim failed");
        let ret_val = Self::with_fb(|fb| {
            while arg_vals.len() < 3 {
                let z = fb.ins().iconst(types::I64, 0);
                arg_vals.push(z);
            }
            let fref = self.module.declare_func_in_func(func_id, fb.func);
            let cargc = fb.ins().iconst(types::I64, argc as i64);
            let call_inst = fb
                .ins()
                .call(fref, &[cargc, arg_vals[0], arg_vals[1], arg_vals[2]]);
            if has_ret {
                fb.inst_results(call_inst).get(0).copied()
            } else {
                None
            }
        });
        if let Some(v) = ret_val {
            self.value_stack.push(v);
        }
    }
    fn emit_string_handle_from_literal(&mut self, s: &str) {
        use cranelift_codegen::ir::{types, AbiParam, Signature};
        // Pack up to 16 bytes into two u64 words (little-endian)
        let bytes = s.as_bytes();
        let mut lo: u64 = 0;
        let mut hi: u64 = 0;
        let take = core::cmp::min(16, bytes.len());
        for i in 0..take.min(8) {
            lo |= (bytes[i] as u64) << (8 * i as u32);
        }
        for i in 8..take {
            hi |= (bytes[i] as u64) << (8 * (i - 8) as u32);
        }
        // Call thunk: nyash.string.from_u64x2(lo, hi, len) -> handle(i64)
        let call_conv = self.module.isa().default_call_conv();
        let mut sig = Signature::new(call_conv);
        sig.params.push(AbiParam::new(types::I64)); // lo
        sig.params.push(AbiParam::new(types::I64)); // hi
        sig.params.push(AbiParam::new(types::I64)); // len
        sig.returns.push(AbiParam::new(types::I64));
        let func_id = self
            .module
            .declare_function(
                crate::jit::r#extern::collections::SYM_STRING_FROM_U64X2,
                cranelift_module::Linkage::Import,
                &sig,
            )
            .expect("declare string.from_u64x2");
        let v = Self::with_fb(|fb| {
            let lo_v = fb.ins().iconst(types::I64, lo as i64);
            let hi_v = fb.ins().iconst(types::I64, hi as i64);
            let len_v = fb.ins().iconst(types::I64, bytes.len() as i64);
            let fref = self.module.declare_func_in_func(func_id, fb.func);
            let call_inst = fb.ins().call(fref, &[lo_v, hi_v, len_v]);
            fb.inst_results(call_inst)
                .get(0)
                .copied()
                .expect("str.from_ptr ret")
        });
        self.value_stack.push(v);
        self.stats.0 += 1;
    }
    fn prepare_blocks(&mut self, count: usize) {
        // Allow being called before begin_function; stash desired count
        let mut need_tls = false;
        clif_tls::FB.with(|cell| {
            need_tls = cell.borrow().is_none();
        });
        if need_tls {
            self.pending_blocks = self.pending_blocks.max(count);
            return;
        }
        Self::with_fb(|fb| {
            if count == 0 {
                return;
            }
            if self.blocks.len() < count {
                for _ in 0..(count - self.blocks.len()) {
                    self.blocks.push(fb.create_block());
                }
            }
        });
    }
    fn switch_to_block(&mut self, index: usize) {
        if index >= self.blocks.len() {
            return;
        }
        // Avoid redundant switch_to_block calls that can trip FunctionBuilder state
        if self.current_block_index == Some(index) {
            return;
        }
        Self::with_fb(|fb| {
            // If switching away from a non-terminated block, inject jump to keep CFG sane
            if let Some(cur) = self.current_block_index {
                if self.cur_needs_term && cur != index {
                    fb.ins().jump(self.blocks[index], &[]);
                    self.cur_needs_term = false;
                }
            }
            fb.switch_to_block(self.blocks[index]);
            self.current_block_index = Some(index);
            // New current block now requires a terminator before any further switch
            self.cur_needs_term = true;
        });
    }
    fn seal_block(&mut self, _index: usize) { /* final sealing handled in end_function */
    }
    fn br_if_top_is_true(&mut self, then_index: usize, else_index: usize) {
        use cranelift_codegen::ir::condcodes::IntCC;
        Self::with_fb(|fb| {
            if then_index >= self.blocks.len() || else_index >= self.blocks.len() {
                return;
            }
            let cond_val = if let Some(v) = self.value_stack.pop() {
                v
            } else {
                fb.ins().iconst(cranelift_codegen::ir::types::I64, 0)
            };
            let b1 = fb.ins().icmp_imm(IntCC::NotEqual, cond_val, 0);
            fb.ins().brif(
                b1,
                self.blocks[then_index],
                &[],
                self.blocks[else_index],
                &[],
            );
        });
        self.cur_needs_term = false;
        self.stats.3 += 1;
    }
    fn jump_to(&mut self, target_index: usize) {
        Self::with_fb(|fb| {
            if target_index < self.blocks.len() {
                fb.ins().jump(self.blocks[target_index], &[]);
            }
        });
        self.stats.3 += 1;
    }
    fn ensure_block_params_i64(&mut self, index: usize, count: usize) {
        self.block_param_counts.insert(index, count);
    }
    fn push_block_param_i64_at(&mut self, pos: usize) {
        let v = Self::with_fb(|fb| {
            let b = if let Some(i) = self.current_block_index {
                self.blocks[i]
            } else {
                self.entry_block.unwrap()
            };
            let params = fb.func.dfg.block_params(b).to_vec();
            params
                .get(pos)
                .copied()
                .unwrap_or_else(|| fb.ins().iconst(cranelift_codegen::ir::types::I64, 0))
        });
        self.value_stack.push(v);
    }
    fn br_if_with_args(
        &mut self,
        then_index: usize,
        else_index: usize,
        then_n: usize,
        else_n: usize,
    ) {
        use cranelift_codegen::ir::{condcodes::IntCC, types};
        if then_index >= self.blocks.len() || else_index >= self.blocks.len() {
            return;
        }
        let mut else_args: Vec<cranelift_codegen::ir::Value> = Vec::new();
        for _ in 0..else_n {
            if let Some(v) = self.value_stack.pop() {
                else_args.push(v);
            }
        }
        else_args.reverse();
        let mut then_args: Vec<cranelift_codegen::ir::Value> = Vec::new();
        for _ in 0..then_n {
            if let Some(v) = self.value_stack.pop() {
                then_args.push(v);
            }
        }
        then_args.reverse();
        Self::with_fb(|fb| {
            let then_has_inst = self.materialize_succ_params(fb, then_index);
            let else_has_inst = self.materialize_succ_params(fb, else_index);
            let cond_b1 = if let Some(v) = self.value_stack.pop() {
                let ty = fb.func.dfg.value_type(v);
                if ty == types::I64 {
                    let out = fb.ins().icmp_imm(IntCC::NotEqual, v, 0);
                    crate::jit::rt::b1_norm_inc(1);
                    out
                } else {
                    v
                }
            } else {
                let zero = fb.ins().iconst(types::I64, 0);
                let out = fb.ins().icmp_imm(IntCC::NotEqual, zero, 0);
                crate::jit::rt::b1_norm_inc(1);
                out
            };
            let targs = if then_has_inst { Vec::new() } else { then_args };
            let eargs = if else_has_inst { Vec::new() } else { else_args };
            fb.ins().brif(
                cond_b1,
                self.blocks[then_index],
                &targs,
                self.blocks[else_index],
                &eargs,
            );
        });
        self.cur_needs_term = false;
        self.stats.3 += 1;
    }
    fn jump_with_args(&mut self, target_index: usize, n: usize) {
        let mut args: Vec<cranelift_codegen::ir::Value> = Vec::new();
        for _ in 0..n {
            if let Some(v) = self.value_stack.pop() {
                args.push(v);
            }
        }
        args.reverse();
        Self::with_fb(|fb| {
            let has_inst = self.materialize_succ_params(fb, target_index);
            if has_inst {
                args.clear();
            }
            fb.ins().jump(self.blocks[target_index], &args);
        });
        self.cur_needs_term = false;
        self.stats.3 += 1;
    }
    fn hint_ret_bool(&mut self, is_b1: bool) {
        self.ret_hint_is_b1 = is_b1;
    }
    fn ensure_local_i64(&mut self, index: usize) {
        use cranelift_codegen::ir::{StackSlotData, StackSlotKind};
        if self.local_slots.contains_key(&index) {
            return;
        }
        Self::with_fb(|fb| {
            let slot =
                fb.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8));
            self.local_slots.insert(index, slot);
        });
    }
    fn store_local_i64(&mut self, index: usize) {
        use cranelift_codegen::ir::{condcodes::IntCC, types};
        if let Some(mut v) = self.value_stack.pop() {
            if !self.local_slots.contains_key(&index) {
                self.ensure_local_i64(index);
            }
            let slot = self.local_slots.get(&index).copied();
            Self::with_fb(|fb| {
                let ty = fb.func.dfg.value_type(v);
                if ty != types::I64 {
                    if ty == types::F64 {
                        v = fb.ins().fcvt_to_sint(types::I64, v);
                    } else {
                        let one = fb.ins().iconst(types::I64, 1);
                        let zero = fb.ins().iconst(types::I64, 0);
                        let b1 = fb.ins().icmp_imm(IntCC::NotEqual, v, 0);
                        v = fb.ins().select(b1, one, zero);
                    }
                }
                if let Some(slot) = slot {
                    fb.ins().stack_store(v, slot, 0);
                }
                if std::env::var("NYASH_JIT_TRACE_LOCAL").ok().as_deref() == Some("1") {
                    eprintln!(
                        "[JIT-LOCAL] store idx={} (tracked_slots={})",
                        index,
                        self.local_slots.len()
                    );
                }
            });
            if std::env::var("NYASH_JIT_TRACE_LOCAL").ok().as_deref() == Some("1") {
                // Also emit value via dbg hook: tag = 1000 + index
                let tag = Self::with_fb(|fb| fb.ins().iconst(types::I64, (1000 + index as i64)));
                self.value_stack.push(tag);
                self.value_stack.push(v);
                self.emit_host_call_typed(
                    "nyash.jit.dbg_i64",
                    &[ParamKind::I64, ParamKind::I64],
                    true,
                    false,
                );
                let _ = self.value_stack.pop();
            }
        }
    }
    fn load_local_i64(&mut self, index: usize) {
        use cranelift_codegen::ir::types;
        if !self.local_slots.contains_key(&index) {
            self.ensure_local_i64(index);
        }
        if let Some(&slot) = self.local_slots.get(&index) {
            let v = Self::with_fb(|fb| fb.ins().stack_load(types::I64, slot, 0));
            if std::env::var("NYASH_JIT_TRACE_LOCAL").ok().as_deref() == Some("1") {
                eprintln!(
                    "[JIT-LOCAL] load idx={} (tracked_slots={})",
                    index,
                    self.local_slots.len()
                );
            }
            self.value_stack.push(v);
            self.stats.0 += 1;
            if std::env::var("NYASH_JIT_TRACE_LOCAL").ok().as_deref() == Some("1") {
                // tag = 2000 + index
                let tag = Self::with_fb(|fb| fb.ins().iconst(types::I64, (2000 + index as i64)));
                self.value_stack.push(tag);
                self.value_stack.push(v);
                self.emit_host_call_typed(
                    "nyash.jit.dbg_i64",
                    &[ParamKind::I64, ParamKind::I64],
                    true,
                    false,
                );
                let _ = self.value_stack.pop();
            }
        }
    }
}

impl CraneliftBuilder {
    fn materialize_succ_params(
        &mut self,
        fb: &mut cranelift_frontend::FunctionBuilder<'static>,
        succ_index: usize,
    ) -> bool {
        use cranelift_codegen::ir::types;
        if succ_index >= self.blocks.len() {
            return false;
        }
        let b = self.blocks[succ_index];
        let has_inst = fb.func.layout.first_inst(b).is_some();
        if !has_inst {
            let desired = self
                .block_param_counts
                .get(&succ_index)
                .copied()
                .unwrap_or(0);
            let current = fb.func.dfg.block_params(b).len();
            if desired > current {
                for _ in current..desired {
                    let _ = fb.append_block_param(b, types::I64);
                }
            }
        }
        has_inst
    }
    fn entry_param(&mut self, index: usize) -> Option<cranelift_codegen::ir::Value> {
        if let Some(b) = self.entry_block {
            return Self::with_fb(|fb| fb.func.dfg.block_params(b).get(index).copied());
        }
        None
    }
    fn with_fb<R>(f: impl FnOnce(&mut cranelift_frontend::FunctionBuilder<'static>) -> R) -> R {
        clif_tls::FB.with(|cell| {
            let mut opt = cell.borrow_mut();
            let tls = opt.as_mut().expect("FunctionBuilder TLS not initialized");
            tls.with(f)
        })
    }
    pub fn new() -> Self {
        let mut builder = cranelift_jit::JITBuilder::new(cranelift_module::default_libcall_names())
            .expect("JITBuilder");
        // Hostcall symbols
        builder.symbol("nyash.host.stub0", nyash_host_stub0 as *const u8);
        builder.symbol("nyash.jit.dbg_i64", nyash_jit_dbg_i64 as *const u8);
        builder.symbol("nyash.jit.block_enter", nyash_jit_block_enter as *const u8);
        // Async/Result
        builder.symbol(
            crate::jit::r#extern::r#async::SYM_FUTURE_AWAIT_H,
            nyash_future_await_h as *const u8,
        );
        builder.symbol(
            crate::jit::r#extern::result::SYM_RESULT_OK_H,
            nyash_result_ok_h as *const u8,
        );
        builder.symbol(
            crate::jit::r#extern::result::SYM_RESULT_ERR_H,
            nyash_result_err_h as *const u8,
        );
        // Math
        builder.symbol("nyash.math.sin_f64", nyash_math_sin_f64 as *const u8);
        builder.symbol("nyash.math.cos_f64", nyash_math_cos_f64 as *const u8);
        builder.symbol("nyash.math.abs_f64", nyash_math_abs_f64 as *const u8);
        builder.symbol("nyash.math.min_f64", nyash_math_min_f64 as *const u8);
        builder.symbol("nyash.math.max_f64", nyash_math_max_f64 as *const u8);
        // Handle-based collection/string/runtime
        {
            use crate::jit::r#extern::{birth as b, collections as c, handles as h, runtime as r};
            builder.symbol(c::SYM_ARRAY_LEN_H, nyash_array_len_h as *const u8);
            builder.symbol(c::SYM_ARRAY_GET_H, nyash_array_get_h as *const u8);
            builder.symbol(c::SYM_ARRAY_SET_H, nyash_array_set_h as *const u8);
            builder.symbol(c::SYM_ARRAY_PUSH_H, nyash_array_push_h as *const u8);
            builder.symbol(c::SYM_ARRAY_LAST_H, nyash_array_last_h as *const u8);
            builder.symbol(c::SYM_MAP_SIZE_H, nyash_map_size_h as *const u8);
            builder.symbol(c::SYM_MAP_GET_H, nyash_map_get_h as *const u8);
            builder.symbol(c::SYM_MAP_GET_HH, nyash_map_get_hh as *const u8);
            builder.symbol(c::SYM_MAP_SET_H, nyash_map_set_h as *const u8);
            builder.symbol(c::SYM_MAP_HAS_H, nyash_map_has_h as *const u8);
            builder.symbol(c::SYM_ANY_LEN_H, nyash_any_length_h as *const u8);
            builder.symbol(c::SYM_STRING_LEN_H, nyash_string_len_h as *const u8);
            builder.symbol(c::SYM_ANY_IS_EMPTY_H, nyash_any_is_empty_h as *const u8);
            builder.symbol(
                c::SYM_STRING_CHARCODE_AT_H,
                nyash_string_charcode_at_h as *const u8,
            );
            builder.symbol(c::SYM_STRING_BIRTH_H, nyash_string_birth_h as *const u8);
            builder.symbol(c::SYM_INTEGER_BIRTH_H, nyash_integer_birth_h as *const u8);
            builder.symbol("nyash.console.birth_h", nyash_console_birth_h as *const u8);
            builder.symbol(c::SYM_STRING_CONCAT_HH, nyash_string_concat_hh as *const u8);
            builder.symbol(c::SYM_STRING_EQ_HH, nyash_string_eq_hh as *const u8);
            builder.symbol(c::SYM_STRING_LT_HH, nyash_string_lt_hh as *const u8);
            builder.symbol(b::SYM_BOX_BIRTH_H, nyash_box_birth_h as *const u8);
            builder.symbol("nyash.box.birth_i64", nyash_box_birth_i64 as *const u8);
            builder.symbol(
                crate::jit::r#extern::birth::SYM_INSTANCE_BIRTH_NAME_U64X2,
                super::super::extern_thunks::nyash_instance_birth_name_u64x2 as *const u8,
            );
            builder.symbol(h::SYM_HANDLE_OF, nyash_handle_of as *const u8);
            builder.symbol(r::SYM_RT_CHECKPOINT, nyash_rt_checkpoint as *const u8);
            builder.symbol(r::SYM_GC_BARRIER_WRITE, nyash_gc_barrier_write as *const u8);
        }
        // Plugin invoke shims
        builder.symbol(
            "nyash_plugin_invoke3_i64",
            nyash_plugin_invoke3_i64 as *const u8,
        );
        builder.symbol(
            "nyash_plugin_invoke3_f64",
            nyash_plugin_invoke3_f64 as *const u8,
        );
        builder.symbol(
            "nyash_plugin_invoke_name_getattr_i64",
            nyash_plugin_invoke_name_getattr_i64 as *const u8,
        );
        builder.symbol(
            "nyash_plugin_invoke_name_call_i64",
            nyash_plugin_invoke_name_call_i64 as *const u8,
        );
        builder.symbol(
            crate::jit::r#extern::collections::SYM_STRING_FROM_U64X2,
            super::super::extern_thunks::nyash_string_from_u64x2 as *const u8,
        );

        // Host-bridge (by-slot) imports (opt-in)
        if std::env::var("NYASH_JIT_HOST_BRIDGE").ok().as_deref() == Some("1") {
            use crate::jit::r#extern::host_bridge as hb;
            // Instance.getField/setField (recv_h, name_i[, val_i])
            // Use arity-stable import symbols to avoid signature collisions
            builder.symbol(
                hb::SYM_HOST_INSTANCE_FIELD3,
                super::super::extern_thunks::nyash_host_instance_field3 as *const u8,
            );
            // String.len (recv_h)
            builder.symbol(
                hb::SYM_HOST_STRING_LEN,
                super::super::extern_thunks::nyash_host_string_len as *const u8,
            );
            // Console.* (value)
            builder.symbol(
                hb::SYM_HOST_CONSOLE_LOG,
                super::super::extern_thunks::nyash_host_console_log_i64 as *const u8,
            );
            builder.symbol(
                hb::SYM_HOST_CONSOLE_WARN,
                super::super::extern_thunks::nyash_host_console_warn_i64 as *const u8,
            );
            builder.symbol(
                hb::SYM_HOST_CONSOLE_ERROR,
                super::super::extern_thunks::nyash_host_console_error_i64 as *const u8,
            );
        }

        let module = cranelift_jit::JITModule::new(builder);
        let ctx = cranelift_codegen::Context::new();
        let fbc = cranelift_frontend::FunctionBuilderContext::new();
        CraneliftBuilder {
            module,
            ctx,
            fbc,
            stats: (0, 0, 0, 0, 0),
            current_name: None,
            value_stack: Vec::new(),
            entry_block: None,
            blocks: Vec::new(),
            current_block_index: None,
            block_param_counts: std::collections::HashMap::new(),
            local_slots: std::collections::HashMap::new(),
            compiled_closure: None,
            desired_argc: 0,
            desired_has_ret: true,
            desired_ret_is_f64: false,
            typed_sig_prepared: false,
            ret_hint_is_b1: false,
            ret_block: None,
            ret_slot: None,
            pending_blocks: 0,
            cur_needs_term: false,
            sealed_blocks: std::collections::HashSet::new(),
        }
    }
    pub fn take_compiled_closure(
        &mut self,
    ) -> Option<
        std::sync::Arc<
            dyn Fn(&[crate::jit::abi::JitValue]) -> crate::jit::abi::JitValue + Send + Sync,
        >,
    > {
        self.compiled_closure.take()
    }
}
