//! IR builder abstraction for JIT lowering
//!
//! This trait lets LowerCore target an abstract IR so we can plug Cranelift later
//! behind a feature flag. For now, we provide a NoopBuilder that counts calls.

#[derive(Debug, Clone, Copy)]
pub enum BinOpKind { Add, Sub, Mul, Div, Mod }

#[derive(Debug, Clone, Copy)]
pub enum CmpKind { Eq, Ne, Lt, Le, Gt, Ge }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind { I64, F64, B1 }

pub trait IRBuilder {
    fn begin_function(&mut self, name: &str);
    fn end_function(&mut self);
    /// Optional: prepare a simple `i64` ABI signature with `argc` params
    fn prepare_signature_i64(&mut self, _argc: usize, _has_ret: bool) { }
    /// Optional: prepare typed ABI signature for params and f64 return flag
    fn prepare_signature_typed(&mut self, _params: &[ParamKind], _ret_is_f64: bool) { }
    /// Load i64 parameter at index and push to value stack (Core-1 path)
    fn emit_param_i64(&mut self, _index: usize) { }
    fn emit_const_i64(&mut self, _val: i64);
    fn emit_const_f64(&mut self, _val: f64);
    fn emit_binop(&mut self, _op: BinOpKind);
    fn emit_compare(&mut self, _op: CmpKind);
    fn emit_jump(&mut self);
    fn emit_branch(&mut self);
    fn emit_return(&mut self);
    /// Select between two i64 values using top-of-stack condition (b1 or i64 0/1).
    /// Stack order: ... cond then_val else_val -> result
    fn emit_select_i64(&mut self) { }
    /// Phase 10_d scaffolding: host-call emission (symbolic)
    fn emit_host_call(&mut self, _symbol: &str, _argc: usize, _has_ret: bool) { }
    /// Typed host-call emission: params kinds and return type hint (f64 when true)
    fn emit_host_call_typed(&mut self, _symbol: &str, _params: &[ParamKind], _has_ret: bool, _ret_is_f64: bool) { }
    /// Phase 10.2: plugin invoke emission (symbolic; type_id/method_id based)
    fn emit_plugin_invoke(&mut self, _type_id: u32, _method_id: u32, _argc: usize, _has_ret: bool) { }
    /// Phase 10.5c: plugin invoke by method-name (box_type unknown at compile-time)
    fn emit_plugin_invoke_by_name(&mut self, _method: &str, _argc: usize, _has_ret: bool) { }
    // ==== Phase 10.7 (control-flow wiring, default no-op) ====
    /// Optional: prepare N basic blocks and return their handles (0..N-1)
    fn prepare_blocks(&mut self, _count: usize) { }
    /// Optional: switch current insertion point to a given block index
    fn switch_to_block(&mut self, _index: usize) { }
    /// Optional: seal a block after all predecessors are known
    fn seal_block(&mut self, _index: usize) { }
    /// Optional: conditional branch, treating the top-of-stack as condition (i64!=0 or b1)
    fn br_if_top_is_true(&mut self, _then_index: usize, _else_index: usize) { }
    /// Optional: unconditional jump to target block index
    fn jump_to(&mut self, _target_index: usize) { }
    /// Optional: ensure target block has N i64 block params (for PHI)
    fn ensure_block_params_i64(&mut self, _index: usize, _count: usize) { }
    /// Optional: ensure target block has N b1 block params (for PHI of bool)
    fn ensure_block_params_b1(&mut self, index: usize, count: usize) { self.ensure_block_params_i64(index, count); }
    /// Optional: ensure target block has one i64 block param (backward compat)
    fn ensure_block_param_i64(&mut self, index: usize) { self.ensure_block_params_i64(index, 1); }
    /// Optional: push current block's param at position onto the value stack (default=0)
    fn push_block_param_i64_at(&mut self, _pos: usize) { }
    /// Optional: push current block's boolean param (b1) at position; default converts i64 0/1 → b1
    fn push_block_param_b1_at(&mut self, _pos: usize) { self.push_block_param_i64_at(_pos); }
    /// Optional: push current block's first param (i64) onto the value stack (backward compat)
    fn push_block_param_i64(&mut self) { self.push_block_param_i64_at(0); }
    /// Optional: conditional branch with explicit arg counts for then/else; pops args from stack
    fn br_if_with_args(&mut self, _then_index: usize, _else_index: usize, _then_n: usize, _else_n: usize) {
        // fallback to no-arg br_if
        self.br_if_top_is_true(_then_index, _else_index);
    }
    /// Optional: jump with explicit arg count; pops args from stack
    fn jump_with_args(&mut self, _target_index: usize, _n: usize) { self.jump_to(_target_index); }
    /// Optional: hint that function returns a boolean (b1) value (footing only)
    fn hint_ret_bool(&mut self, _is_b1: bool) { }

    // ==== Minimal local slots for Load/Store (i64 only) ====
    /// Ensure an i64 local slot exists for the given index
    fn ensure_local_i64(&mut self, _index: usize) { }
    /// Store top-of-stack (normalized to i64) into local slot
    fn store_local_i64(&mut self, _index: usize) { }
    /// Load i64 from local slot and push to stack
    fn load_local_i64(&mut self, _index: usize) { }
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
    fn emit_param_i64(&mut self, _index: usize) { self.consts += 1; }
    fn emit_const_i64(&mut self, _val: i64) { self.consts += 1; }
    fn emit_const_f64(&mut self, _val: f64) { self.consts += 1; }
    fn emit_binop(&mut self, _op: BinOpKind) { self.binops += 1; }
    fn emit_compare(&mut self, _op: CmpKind) { self.cmps += 1; }
    fn emit_jump(&mut self) { self.branches += 1; }
    fn emit_branch(&mut self) { self.branches += 1; }
    fn emit_return(&mut self) { self.rets += 1; }
    fn emit_select_i64(&mut self) { self.binops += 1; }
    fn emit_host_call_typed(&mut self, _symbol: &str, _params: &[ParamKind], has_ret: bool, _ret_is_f64: bool) { if has_ret { self.consts += 1; } }
    fn emit_plugin_invoke(&mut self, _type_id: u32, _method_id: u32, _argc: usize, has_ret: bool) { if has_ret { self.consts += 1; } }
    fn emit_plugin_invoke_by_name(&mut self, _method: &str, _argc: usize, has_ret: bool) { if has_ret { self.consts += 1; } }
    fn ensure_local_i64(&mut self, _index: usize) { /* no-op */ }
    fn store_local_i64(&mut self, _index: usize) { self.consts += 1; }
    fn load_local_i64(&mut self, _index: usize) { self.consts += 1; }
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
    // Phase 10.7: basic block wiring state
    blocks: Vec<cranelift_codegen::ir::Block>,
    current_block_index: Option<usize>,
    block_param_counts: std::collections::HashMap<usize, usize>,
    // Local stack slots for minimal Load/Store lowering (i64 only)
    local_slots: std::collections::HashMap<usize, cranelift_codegen::ir::StackSlot>,
    // Finalized function pointer (if any)
    compiled_closure: Option<std::sync::Arc<dyn Fn(&[crate::jit::abi::JitValue]) -> crate::jit::abi::JitValue + Send + Sync>>, 
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
}

#[cfg(feature = "cranelift-jit")]
use cranelift_module::Module;
#[cfg(feature = "cranelift-jit")]
use cranelift_codegen::ir::InstBuilder;

// TLS: 単一関数あたり1つの FunctionBuilder を保持（jit-direct 専用）
#[cfg(feature = "cranelift-jit")]
mod clif_tls {
    use super::*;
    thread_local! {
        pub static FB: std::cell::RefCell<Option<TlsCtx>> = std::cell::RefCell::new(None);
    }
    pub struct TlsCtx {
        pub ctx: Box<cranelift_codegen::Context>,
        pub fbc: Box<cranelift_frontend::FunctionBuilderContext>,
        fb: *mut cranelift_frontend::FunctionBuilder<'static>,
    }
    impl TlsCtx {
        pub fn new() -> Self {
            Self { ctx: Box::new(cranelift_codegen::Context::new()), fbc: Box::new(cranelift_frontend::FunctionBuilderContext::new()), fb: core::ptr::null_mut() }
        }
        pub unsafe fn create(&mut self) {
            let func_ptr: *mut cranelift_codegen::ir::Function = &mut self.ctx.func;
            let fbc_ptr: *mut cranelift_frontend::FunctionBuilderContext = &mut *self.fbc;
            let fb = Box::new(cranelift_frontend::FunctionBuilder::new(&mut *func_ptr, &mut *fbc_ptr));
            self.fb = Box::into_raw(fb);
        }
        pub fn with<R>(&mut self, f: impl FnOnce(&mut cranelift_frontend::FunctionBuilder<'static>) -> R) -> R {
            unsafe { f(&mut *self.fb) }
        }
        pub unsafe fn finalize_drop(&mut self) {
            if !self.fb.is_null() {
                let fb = Box::from_raw(self.fb);
                fb.finalize();
                self.fb = core::ptr::null_mut();
            }
        }
    }
}

// Small TLS helpers to call imported functions via the single FunctionBuilder
#[cfg(feature = "cranelift-jit")]
fn tls_call_import_ret(
    module: &mut cranelift_jit::JITModule,
    func_id: cranelift_module::FuncId,
    args: &[cranelift_codegen::ir::Value],
    has_ret: bool,
) -> Option<cranelift_codegen::ir::Value> {
    clif_tls::FB.with(|cell| {
        let mut opt = cell.borrow_mut();
        let tls = opt.as_mut().expect("FunctionBuilder TLS not initialized");
        tls.with(|fb| {
            let fref = module.declare_func_in_func(func_id, fb.func);
            let call_inst = fb.ins().call(fref, args);
            if has_ret { fb.inst_results(call_inst).get(0).copied() } else { None }
        })
    })
}

#[cfg(feature = "cranelift-jit")]
fn tls_call_import_with_iconsts(
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
            for &c in iconsts { all_args.push(fb.ins().iconst(types::I64, c)); }
            all_args.extend_from_slice(tail_args);
            let fref = module.declare_func_in_func(func_id, fb.func);
            let call_inst = fb.ins().call(fref, &all_args);
            if has_ret { fb.inst_results(call_inst).get(0).copied() } else { None }
        })
    })
}

#[cfg(feature = "cranelift-jit")]
extern "C" fn nyash_host_stub0() -> i64 { 0 }
#[cfg(feature = "cranelift-jit")]
extern "C" fn nyash_jit_dbg_i64(tag: i64, val: i64) -> i64 {
    eprintln!("[JIT-DBG] tag={} val={}", tag, val);
    val
}
#[cfg(feature = "cranelift-jit")]
extern "C" fn nyash_jit_block_enter(idx: i64) {
    eprintln!("[JIT-BLOCK] enter={}", idx);
}
#[cfg(feature = "cranelift-jit")]
extern "C" fn nyash_plugin_invoke3_i64(type_id: i64, method_id: i64, argc: i64, a0: i64, a1: i64, a2: i64) -> i64 {
    use crate::runtime::plugin_loader_v2::PluginBoxV2;
    let trace = crate::jit::observe::trace_enabled();
    // Emit early shim-enter event for observability regardless of path taken
    crate::jit::events::emit_runtime(
        serde_json::json!({
            "id": "shim.enter.i64", "type_id": type_id, "method_id": method_id, "argc": argc
        }),
        "shim", "<jit>"
    );
    // Resolve receiver instance from legacy VM args (param index)
    let mut instance_id: u32 = 0;
    let mut invoke: Option<unsafe extern "C" fn(u32,u32,u32,*const u8,usize,*mut u8,*mut usize)->i32> = None;
    // Try handle registry first: a0 may be a handle (preferred)
    if a0 > 0 {
        if let Some(obj) = crate::jit::rt::handles::get(a0 as u64) {
            if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                instance_id = p.instance_id();
                invoke = Some(p.inner.invoke_fn);
            } else {
                // Builtin/native object fallback for common methods
                if method_id as u32 == 1 {
                    // length
                    if let Some(arr) = obj.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                        if let Some(ib) = arr.length().as_any().downcast_ref::<crate::box_trait::IntegerBox>() { return ib.value; }
                    }
                    if let Some(sb) = obj.as_any().downcast_ref::<crate::box_trait::StringBox>() {
                        return sb.value.len() as i64;
                    }
                }
            }
        }
    }
    // Also capture a direct pointer to native objects via legacy VM args index (compat)
    let mut native_array_len: Option<i64> = None;
    if a0 >= 0 && std::env::var("NYASH_JIT_ARGS_HANDLE_ONLY").ok().as_deref() != Some("1") {
        crate::jit::rt::with_legacy_vm_args(|args| {
            let idx = a0 as usize;
            if let Some(crate::backend::vm::VMValue::BoxRef(b)) = args.get(idx) {
                if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                    instance_id = p.instance_id();
                    invoke = Some(p.inner.invoke_fn);
                } else if let Some(arr) = b.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                    // Fallback length for ArrayBox when not plugin-backed
                    if method_id as u32 == 1 { // length
                        if let Some(ib) = arr.length().as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                            native_array_len = Some(ib.value);
                        }
                    }
                }
            }
        });
    }
    if invoke.is_none() {
        if let Some(v) = native_array_len {
            if trace { eprintln!("[JIT-SHIM i64] native_fallback return {}", v); }
            crate::jit::events::emit_runtime(
                serde_json::json!({
                    "id": "shim.native.i64", "type_id": type_id, "method_id": method_id, "argc": argc, "ret": v
                }),
                "shim", "<jit>"
            );
            return v;
        }
    }
    // If not resolved, scan all VM args for a matching PluginBoxV2 by type_id
    if invoke.is_none() {
        crate::jit::rt::with_legacy_vm_args(|args| {
            for v in args.iter() {
                if let crate::backend::vm::VMValue::BoxRef(b) = v {
                    if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                        // type_id compatibility is best-effort; fall back to first PluginBoxV2
                        if p.inner.type_id == (type_id as u32) || invoke.is_none() {
                            instance_id = p.instance_id();
                            invoke = Some(p.inner.invoke_fn);
                            if p.inner.type_id == (type_id as u32) { break; }
                        }
                    }
                }
            }
        });
    }
    if invoke.is_none() { return 0; }
    // Build TLV args from a1/a2 if present
    let mut buf = crate::runtime::plugin_ffi_common::encode_tlv_header((argc.saturating_sub(1).max(0) as u16));
    let mut add_i64 = |v: i64| { crate::runtime::plugin_ffi_common::encode::i64(&mut buf, v); };
    if argc >= 2 { add_i64(a1); }
    if argc >= 3 { add_i64(a2); }
    // Prepare output buffer with canaries for overrun detection
    let mut out = vec![0xCDu8; 4096 + 32];
    let canary_val = 0xABu8;
    let canary_len = 16usize;
    for i in 0..canary_len { out[i] = canary_val; }
    for i in 0..canary_len { out[4096 + canary_len + i] = canary_val; }
    let mut out_len: usize = 4096;
    let out_ptr = unsafe { out.as_mut_ptr().add(canary_len) };
    if trace { eprintln!("[JIT-SHIM i64] invoke type={} method={} argc={} inst_id={} a1={} a2={} buf_len={}", type_id, method_id, argc, instance_id, a1, a2, buf.len()); }
    crate::jit::observe::runtime_plugin_shim_i64(type_id, method_id, argc, instance_id);
    crate::jit::observe::trace_push(format!("i64.start type={} method={} argc={} inst={} a1={} a2={}", type_id, method_id, argc, instance_id, a1, a2));
    let rc = unsafe { invoke.unwrap()(type_id as u32, method_id as u32, instance_id, buf.as_ptr(), buf.len(), out_ptr, &mut out_len) };
    // Canary check
    let pre_ok = out[..canary_len].iter().all(|&b| b==canary_val);
    let post_ok = out[canary_len + out_len .. canary_len + out_len + canary_len].iter().all(|&b| b==canary_val);
    if trace { eprintln!("[JIT-SHIM i64] rc={} out_len={} canary_pre={} canary_post={}", rc, out_len, pre_ok, post_ok); }
    crate::jit::observe::trace_push(format!("i64.end rc={} out_len={} pre_ok={} post_ok={}", rc, out_len, pre_ok, post_ok));
    if rc != 0 { return 0; }
    let out_slice = unsafe { std::slice::from_raw_parts(out_ptr, out_len) };
        if let Some((tag, sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(out_slice) {
            if trace { eprintln!("[JIT-SHIM i64] TLV tag={} sz={}", tag, sz); }
            crate::jit::observe::trace_push(format!("i64.tlv tag={} sz={}", tag, sz));
            match tag {
                2 => { // I32
                    if let Some(v) = crate::runtime::plugin_ffi_common::decode::i32(payload) { return v as i64; }
                }
                3 => { // I64
                    if let Some(v) = crate::runtime::plugin_ffi_common::decode::i32(payload) { return v as i64; }
                    if payload.len() == 8 { let mut b=[0u8;8]; b.copy_from_slice(payload); return i64::from_le_bytes(b); }
                }
                8 => { // Handle(tag=8)
                    if sz == 8 {
                        let mut t=[0u8;4]; t.copy_from_slice(&payload[0..4]);
                        let mut i=[0u8;4]; i.copy_from_slice(&payload[4..8]);
                        let r_type = u32::from_le_bytes(t); let r_inst = u32::from_le_bytes(i);
                        let box_type_name = crate::runtime::plugin_loader_unified::get_global_plugin_host()
                            .read().ok()
                            .and_then(|h| h.config_ref().map(|cfg| cfg.box_types.clone()))
                            .and_then(|m| m.into_iter().find(|(_k,v)| *v == r_type).map(|(k,_v)| k))
                            .unwrap_or_else(|| "PluginBox".to_string());
                        let pb = crate::runtime::plugin_loader_v2::make_plugin_box_v2(box_type_name, r_type, r_inst, invoke.unwrap());
                        let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> = std::sync::Arc::new(pb);
                        let h = crate::jit::rt::handles::to_handle(arc);
                        return h as i64;
                    }
                }
                1 => { // Bool
                    return if crate::runtime::plugin_ffi_common::decode::bool(payload).unwrap_or(false) { 1 } else { 0 };
                }
                5 => { // F64 → optional conversion to i64 when enabled
                    if std::env::var("NYASH_JIT_NATIVE_F64").ok().as_deref() == Some("1") {
                    if sz == 8 {
                        let mut b=[0u8;8]; b.copy_from_slice(payload);
                        let f = f64::from_le_bytes(b);
                        return f as i64;
                    }
                }
            }
            _ => {}
        }
    }
    0
}

// F64-typed shim: decodes TLV first entry and returns f64 when possible
extern "C" fn nyash_plugin_invoke3_f64(type_id: i64, method_id: i64, argc: i64, a0: i64, a1: i64, a2: i64) -> f64 {
    use crate::runtime::plugin_loader_v2::PluginBoxV2;
    let trace = crate::jit::observe::trace_enabled();
    crate::jit::events::emit_runtime(
        serde_json::json!({
            "id": "shim.enter.f64", "type_id": type_id, "method_id": method_id, "argc": argc
        }),
        "shim", "<jit>"
    );
    // Resolve receiver + invoke_fn from legacy VM args
    let mut instance_id: u32 = 0;
    let mut invoke: Option<unsafe extern "C" fn(u32,u32,u32,*const u8,usize,*mut u8,*mut usize)->i32> = None;
    // Try handle registry first
    let mut native_array_len: Option<f64> = None;
    if a0 > 0 {
        if let Some(obj) = crate::jit::rt::handles::get(a0 as u64) {
            if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                instance_id = p.instance_id();
                invoke = Some(p.inner.invoke_fn);
            } else if method_id as u32 == 1 {
                if let Some(arr) = obj.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                    if let Some(ib) = arr.length().as_any().downcast_ref::<crate::box_trait::IntegerBox>() { native_array_len = Some(ib.value as f64); }
                }
                if let Some(sb) = obj.as_any().downcast_ref::<crate::box_trait::StringBox>() { native_array_len = Some(sb.value.len() as f64); }
            }
        }
    }
    if a0 >= 0 && std::env::var("NYASH_JIT_ARGS_HANDLE_ONLY").ok().as_deref() != Some("1") {
        crate::jit::rt::with_legacy_vm_args(|args| {
            let idx = a0 as usize;
            if let Some(crate::backend::vm::VMValue::BoxRef(b)) = args.get(idx) {
                if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                    instance_id = p.instance_id();
                    invoke = Some(p.inner.invoke_fn);
                } else if let Some(arr) = b.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                    if method_id as u32 == 1 { // length
                        if let Some(ib) = arr.length().as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                            native_array_len = Some(ib.value as f64);
                        }
                    }
                }
            }
        });
    }
    if invoke.is_none() {
        if let Some(v) = native_array_len {
            if trace { eprintln!("[JIT-SHIM f64] native_fallback return {}", v); }
            crate::jit::events::emit_runtime(
                serde_json::json!({
                    "id": "shim.native.f64", "type_id": type_id, "method_id": method_id, "argc": argc, "ret": v
                }),
                "shim", "<jit>"
            );
            return v;
        }
    }
    if invoke.is_none() {
        crate::jit::rt::with_legacy_vm_args(|args| {
            for v in args.iter() {
                if let crate::backend::vm::VMValue::BoxRef(b) = v {
                    if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                        if p.inner.type_id == (type_id as u32) || invoke.is_none() {
                            instance_id = p.instance_id();
                            invoke = Some(p.inner.invoke_fn);
                            if p.inner.type_id == (type_id as u32) { break; }
                        }
                    }
                }
            }
        });
    }
    if invoke.is_none() { return 0.0; }
    // Build TLV args from a1/a2 if present (i64 only for now)
    let mut buf = crate::runtime::plugin_ffi_common::encode_tlv_header((argc.saturating_sub(1).max(0) as u16));
    let mut add_i64 = |v: i64| { crate::runtime::plugin_ffi_common::encode::i64(&mut buf, v); };
    if argc >= 2 { add_i64(a1); }
    if argc >= 3 { add_i64(a2); }
    // Prepare output buffer with canaries
    let mut out = vec![0xCDu8; 4096 + 32];
    let canary_val = 0xABu8;
    let canary_len = 16usize;
    for i in 0..canary_len { out[i] = canary_val; }
    for i in 0..canary_len { out[4096 + canary_len + i] = canary_val; }
    let mut out_len: usize = 4096;
    let out_ptr = unsafe { out.as_mut_ptr().add(canary_len) };
    if trace { eprintln!("[JIT-SHIM f64] invoke type={} method={} argc={} inst_id={} a1={} a2={} buf_len={}", type_id, method_id, argc, instance_id, a1, a2, buf.len()); }
    crate::jit::events::emit_runtime(
        serde_json::json!({
            "id": "plugin_invoke.f64",
            "type_id": type_id,
            "method_id": method_id,
            "argc": argc,
            "inst": instance_id
        }),
        "plugin", "<jit>"
    );
    crate::jit::observe::runtime_plugin_shim_i64(type_id, method_id, argc, instance_id);
    crate::jit::observe::trace_push(format!("f64.start type={} method={} argc={} inst={} a1={} a2={}", type_id, method_id, argc, instance_id, a1, a2));
    let rc = unsafe { invoke.unwrap()(type_id as u32, method_id as u32, instance_id, buf.as_ptr(), buf.len(), out_ptr, &mut out_len) };
    let pre_ok = out[..canary_len].iter().all(|&b| b==canary_val);
    let post_ok = out[canary_len + out_len .. canary_len + out_len + canary_len].iter().all(|&b| b==canary_val);
    if trace { eprintln!("[JIT-SHIM f64] rc={} out_len={} canary_pre={} canary_post={}", rc, out_len, pre_ok, post_ok); }
    crate::jit::observe::trace_push(format!("f64.end rc={} out_len={} pre_ok={} post_ok={}", rc, out_len, pre_ok, post_ok));
    if rc != 0 { return 0.0; }
    let out_slice = unsafe { std::slice::from_raw_parts(out_ptr, out_len) };
    if let Some((tag, sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(out_slice) {
        if trace { eprintln!("[JIT-SHIM f64] TLV tag={} sz={}", tag, sz); }
        crate::jit::observe::trace_push(format!("f64.tlv tag={} sz={}", tag, sz));
        match tag {
            5 => { // F64
                if sz == 8 { let mut b=[0u8;8]; b.copy_from_slice(payload); return f64::from_le_bytes(b); }
            }
            3 => { // I64 → f64
                if payload.len() == 8 { let mut b=[0u8;8]; b.copy_from_slice(payload); return (i64::from_le_bytes(b)) as f64; }
                if let Some(v) = crate::runtime::plugin_ffi_common::decode::i32(payload) { return (v as i64) as f64; }
            }
            2 => { // I32 → f64
                if let Some(v) = crate::runtime::plugin_ffi_common::decode::i32(payload) { return (v as i64) as f64; }
            }
            1 => { // Bool → 0.0/1.0
                return if crate::runtime::plugin_ffi_common::decode::bool(payload).unwrap_or(false) { 1.0 } else { 0.0 };
            }
            _ => {}
        }
    }
    0.0
}

// === By-name plugin shims (JIT) ===
#[cfg(feature = "cranelift-jit")]
extern "C" fn nyash_plugin_invoke_name_getattr_i64(argc: i64, a0: i64, a1: i64, a2: i64) -> i64 {
    nyash_plugin_invoke_name_common_i64("getattr", argc, a0, a1, a2)
}
#[cfg(feature = "cranelift-jit")]
extern "C" fn nyash_plugin_invoke_name_call_i64(argc: i64, a0: i64, a1: i64, a2: i64) -> i64 {
    nyash_plugin_invoke_name_common_i64("call", argc, a0, a1, a2)
}
#[cfg(feature = "cranelift-jit")]
fn nyash_plugin_invoke_name_common_i64(method: &str, argc: i64, a0: i64, a1: i64, a2: i64) -> i64 {
    use crate::runtime::plugin_loader_v2::PluginBoxV2;
    // Resolve receiver
    let mut instance_id: u32 = 0;
    let mut type_id: u32 = 0;
    let mut box_type: Option<String> = None;
    let mut invoke: Option<unsafe extern "C" fn(u32,u32,u32,*const u8,usize,*mut u8,*mut usize)->i32> = None;
    if a0 > 0 {
        if let Some(obj) = crate::jit::rt::handles::get(a0 as u64) {
            if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                instance_id = p.instance_id(); type_id = p.inner.type_id; box_type = Some(p.box_type.clone());
                invoke = Some(p.inner.invoke_fn);
            }
        }
    }
    if invoke.is_none() && std::env::var("NYASH_JIT_ARGS_HANDLE_ONLY").ok().as_deref() != Some("1") {
        crate::jit::rt::with_legacy_vm_args(|args| {
            let idx = a0.max(0) as usize;
            if let Some(crate::backend::vm::VMValue::BoxRef(b)) = args.get(idx) {
                if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                    instance_id = p.instance_id(); type_id = p.inner.type_id; box_type = Some(p.box_type.clone());
                    invoke = Some(p.inner.invoke_fn);
                }
            }
        });
    }
    if invoke.is_none() {
        crate::jit::rt::with_legacy_vm_args(|args| {
            for v in args.iter() {
                if let crate::backend::vm::VMValue::BoxRef(b) = v {
                    if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                        instance_id = p.instance_id(); type_id = p.inner.type_id; box_type = Some(p.box_type.clone());
                        invoke = Some(p.inner.invoke_fn); break;
                    }
                }
            }
        });
    }
    if invoke.is_none() { return 0; }
    let box_type = box_type.unwrap_or_default();
    // Resolve method_id via host
    let mh = if let Ok(host) = crate::runtime::plugin_loader_unified::get_global_plugin_host().read() { host.resolve_method(&box_type, method) } else { return 0 };
    let method_id = match mh { Ok(h) => h.method_id, Err(_) => return 0 } as u32;
    // TLV args from legacy (skip receiver)
    let mut buf = crate::runtime::plugin_ffi_common::encode_tlv_header((argc.saturating_sub(1).max(0) as u16));
    let mut add_from_legacy = |pos: usize| {
        crate::jit::rt::with_legacy_vm_args(|args| {
            if let Some(v) = args.get(pos) {
                use crate::backend::vm::VMValue as V;
                match v {
                    V::String(s) => crate::runtime::plugin_ffi_common::encode::string(&mut buf, s),
                    V::Integer(i) => crate::runtime::plugin_ffi_common::encode::i64(&mut buf, *i),
                    V::Float(f) => crate::runtime::plugin_ffi_common::encode::f64(&mut buf, *f),
                    V::Bool(b) => crate::runtime::plugin_ffi_common::encode::bool(&mut buf, *b),
                    V::BoxRef(b) => {
                        if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                            let host = crate::runtime::get_global_plugin_host();
                            if let Ok(hg) = host.read() {
                            if p.box_type == "StringBox" {
                                    if let Ok(Some(sb)) = hg.invoke_instance_method("StringBox", "toUtf8", p.instance_id(), &[]) {
                                        if let Some(s) = sb.as_any().downcast_ref::<crate::box_trait::StringBox>() { crate::runtime::plugin_ffi_common::encode::string(&mut buf, &s.value); return; }
                                    }
                                } else if p.box_type == "IntegerBox" {
                                    if let Ok(Some(ibx)) = hg.invoke_instance_method("IntegerBox", "get", p.instance_id(), &[]) {
                                        if let Some(i) = ibx.as_any().downcast_ref::<crate::box_trait::IntegerBox>() { crate::runtime::plugin_ffi_common::encode::i64(&mut buf, i.value); return; }
                                    }
                                }
                            }
                            crate::runtime::plugin_ffi_common::encode::plugin_handle(&mut buf, p.inner.type_id, p.instance_id());
                        } else {
                            let s = b.to_string_box().value; crate::runtime::plugin_ffi_common::encode::string(&mut buf, &s)
                        }
                    }
                    _ => {}
                }
            }
        });
    };
    if argc >= 2 { add_from_legacy(1); }
    if argc >= 3 { add_from_legacy(2); }
    let mut out = vec![0u8; 4096]; let mut out_len: usize = out.len();
    let rc = unsafe { invoke.unwrap()(type_id as u32, method_id, instance_id, buf.as_ptr(), buf.len(), out.as_mut_ptr(), &mut out_len) };
    if rc != 0 { return 0; }
    let out_slice = &out[..out_len];
    if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(out_slice) {
        match tag {
            3 => { if payload.len()==8 { let mut b=[0u8;8]; b.copy_from_slice(payload); return i64::from_le_bytes(b); } }
            1 => { return if crate::runtime::plugin_ffi_common::decode::bool(payload).unwrap_or(false) { 1 } else { 0 }; }
            5 => { if std::env::var("NYASH_JIT_NATIVE_F64").ok().as_deref()==Some("1") { if payload.len()==8 { let mut b=[0u8;8]; b.copy_from_slice(payload); let f=f64::from_le_bytes(b); return f as i64; } } }
            _ => {}
        }
    }
    0
}
#[cfg(feature = "cranelift-jit")]
use super::extern_thunks::{
    nyash_math_sin_f64, nyash_math_cos_f64, nyash_math_abs_f64, nyash_math_min_f64, nyash_math_max_f64,
    nyash_array_len_h, nyash_array_get_h, nyash_array_set_h, nyash_array_push_h, 
    nyash_array_last_h, nyash_map_size_h, nyash_map_get_h, nyash_map_get_hh,
    nyash_map_set_h, nyash_map_has_h,
    nyash_string_charcode_at_h, nyash_string_birth_h, nyash_integer_birth_h,
    nyash_string_concat_hh, nyash_string_eq_hh, nyash_string_lt_hh,
    nyash_any_length_h, nyash_any_is_empty_h, nyash_console_birth_h,
};
#[cfg(feature = "cranelift-jit")]
use crate::jit::r#extern::r#async::nyash_future_await_h;
#[cfg(feature = "cranelift-jit")]
use crate::jit::r#extern::result::{nyash_result_ok_h, nyash_result_err_h};

#[cfg(feature = "cranelift-jit")]
use crate::{
    mir::{MirType, Effect as OpEffect, MirFunction},
    jit::events,
};

#[cfg(feature = "cranelift-jit")]
extern "C" fn nyash_array_len(arr_param_index: i64) -> i64 {
    // Interpret first arg as function param index and fetch from thread-local args
    if arr_param_index < 0 { return 0; }
    crate::jit::rt::with_legacy_vm_args(|args| {
        let idx = arr_param_index as usize;
        if let Some(crate::backend::vm::VMValue::BoxRef(b)) = args.get(idx) {
            if let Some(ab) = b.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                if let Some(ib) = ab.length().as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                    return ib.value;
                }
            }
        }
        0
    })
}
#[cfg(feature = "cranelift-jit")]
extern "C" fn nyash_array_push(arr_param_index: i64, val: i64) -> i64 {
    if arr_param_index < 0 { return 0; }
    crate::jit::rt::with_legacy_vm_args(|args| {
        let idx = arr_param_index as usize;
        if let Some(crate::backend::vm::VMValue::BoxRef(b)) = args.get(idx) {
            if let Some(ab) = b.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                // Push integer value only (PoC)
                let ib = crate::box_trait::IntegerBox::new(val);
                let _ = ab.push(Box::new(ib));
                return 0;
            }
        }
        0
    })
}
#[cfg(feature = "cranelift-jit")]
extern "C" fn nyash_array_get(arr_param_index: i64, idx: i64) -> i64 {
    if arr_param_index < 0 { return 0; }
    crate::jit::rt::with_legacy_vm_args(|args| {
        let pidx = arr_param_index as usize;
        if let Some(crate::backend::vm::VMValue::BoxRef(b)) = args.get(pidx) {
            if let Some(ab) = b.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                let val = ab.get(Box::new(crate::box_trait::IntegerBox::new(idx)));
                if let Some(ib) = val.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                    return ib.value;
                }
            }
        }
        0
    })
}
#[cfg(feature = "cranelift-jit")]
extern "C" fn nyash_array_set(arr_param_index: i64, idx: i64, val: i64) -> i64 {
    if arr_param_index < 0 { return 0; }
    crate::jit::rt::with_legacy_vm_args(|args| {
        let pidx = arr_param_index as usize;
        if let Some(crate::backend::vm::VMValue::BoxRef(b)) = args.get(pidx) {
            if let Some(ab) = b.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                let _ = ab.set(
                    Box::new(crate::box_trait::IntegerBox::new(idx)),
                    Box::new(crate::box_trait::IntegerBox::new(val)),
                );
                return 0;
            }
        }
        0
    })
}
#[cfg(feature = "cranelift-jit")]
extern "C" fn nyash_map_get(_map: u64, _key: i64) -> i64 { 0 }
#[cfg(feature = "cranelift-jit")]
extern "C" fn nyash_map_set(_map: u64, _key: i64, _val: i64) -> i64 { 0 }
#[cfg(feature = "cranelift-jit")]
extern "C" fn nyash_map_size(map_param_index: i64) -> i64 {
    if map_param_index < 0 { return 0; }
    crate::jit::rt::with_legacy_vm_args(|args| {
        let idx = map_param_index as usize;
        if let Some(crate::backend::vm::VMValue::BoxRef(b)) = args.get(idx) {
            if let Some(mb) = b.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                if let Some(ib) = mb.size().as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                    return ib.value;
                }
            }
        }
        0
    })
}

// === Handle-based externs (10.7c) ===
#[cfg(feature = "cranelift-jit")]
#[cfg(feature = "cranelift-jit")]
#[cfg(feature = "cranelift-jit")]
#[cfg(feature = "cranelift-jit")]
#[cfg(feature = "cranelift-jit")]
#[cfg(feature = "cranelift-jit")]
#[cfg(feature = "cranelift-jit")]
#[cfg(feature = "cranelift-jit")]
#[cfg(feature = "cranelift-jit")]
#[cfg(feature = "cranelift-jit")]
#[cfg(feature = "cranelift-jit")]
#[cfg(feature = "cranelift-jit")]
#[cfg(feature = "cranelift-jit")]

#[cfg(feature = "cranelift-jit")]
impl IRBuilder for CraneliftBuilder {
    fn prepare_signature_typed(&mut self, params: &[ParamKind], ret_is_f64: bool) {
        use cranelift_codegen::ir::{AbiParam, Signature, types};
        fn abi_param_for_kind(k: ParamKind, cfg: &crate::jit::config::JitConfig) -> cranelift_codegen::ir::AbiParam {
            use cranelift_codegen::ir::types;
            match k {
                ParamKind::I64 => cranelift_codegen::ir::AbiParam::new(types::I64),
                ParamKind::F64 => cranelift_codegen::ir::AbiParam::new(types::F64),
                ParamKind::B1 => {
                    let _ = cfg.native_bool_abi;
                    #[cfg(feature = "jit-b1-abi")]
                    {
                        if crate::jit::config::probe_capabilities().supports_b1_sig && cfg.native_bool_abi { return cranelift_codegen::ir::AbiParam::new(types::B1); }
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
        for &k in params { sig.params.push(abi_param_for_kind(k, &cfg_now)); }
        if self.desired_has_ret {
            // Decide return ABI: prefer F64 if requested; otherwise Bool may use B1 when supported
            if self.desired_ret_is_f64 { sig.returns.push(AbiParam::new(types::F64)); }
            else {
                let mut used_b1 = false;
                #[cfg(feature = "jit-b1-abi")]
                {
                    let cfg_now = crate::jit::config::current();
                    if crate::jit::config::probe_capabilities().supports_b1_sig && cfg_now.native_bool_abi && self.ret_hint_is_b1 {
                        sig.returns.push(AbiParam::new(types::B1));
                        used_b1 = true;
                    }
                }
                if !used_b1 { sig.returns.push(AbiParam::new(types::I64)); }
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
    fn prepare_signature_i64(&mut self, argc: usize, has_ret: bool) {
        self.desired_argc = argc;
        self.desired_has_ret = has_ret;
        self.desired_ret_is_f64 = crate::jit::config::current().native_f64;
    }
    fn begin_function(&mut self, name: &str) {
        use cranelift_codegen::ir::{AbiParam, Signature, types};
        self.current_name = Some(name.to_string());
        self.value_stack.clear();
        // Create TLS context + single FB
        clif_tls::FB.with(|cell| {
            let mut tls = clif_tls::TlsCtx::new();
            let call_conv = self.module.isa().default_call_conv();
            let mut sig = Signature::new(call_conv);
            if !self.typed_sig_prepared {
                for _ in 0..self.desired_argc { sig.params.push(AbiParam::new(types::I64)); }
                if self.desired_has_ret {
                    if self.desired_ret_is_f64 { sig.returns.push(AbiParam::new(types::F64)); }
                    else { sig.returns.push(AbiParam::new(types::I64)); }
                }
            } else {
                for _ in 0..self.desired_argc { sig.params.push(AbiParam::new(types::I64)); }
                if self.desired_has_ret {
                    if self.desired_ret_is_f64 { sig.returns.push(AbiParam::new(types::F64)); }
                    else {
                        let mut used_b1 = false;
                        #[cfg(feature = "jit-b1-abi")]
                        {
                            let cfg_now = crate::jit::config::current();
                            if crate::jit::config::probe_capabilities().supports_b1_sig && cfg_now.native_bool_abi && self.ret_hint_is_b1 {
                                sig.returns.push(AbiParam::new(types::B1));
                                used_b1 = true;
                            }
                        }
                        if !used_b1 { sig.returns.push(AbiParam::new(types::I64)); }
                    }
                }
            }
            tls.ctx.func.signature = sig;
            tls.ctx.func.name = cranelift_codegen::ir::UserFuncName::user(0, 0);
            unsafe { tls.create(); }
            tls.with(|fb| {
                if self.blocks.is_empty() {
                    let block = fb.create_block();
                    self.blocks.push(block);
                }
                if self.pending_blocks > self.blocks.len() {
                    let to_create = self.pending_blocks - self.blocks.len();
                    for _ in 0..to_create { self.blocks.push(fb.create_block()); }
                }
                let entry = self.blocks[0];
                fb.append_block_params_for_function_params(entry);
                fb.switch_to_block(entry);
                fb.seal_block(entry);
                self.entry_block = Some(entry);
                self.current_block_index = Some(0);
                self.cur_needs_term = true;
                let rb = fb.create_block();
                self.ret_block = Some(rb);
                fb.append_block_param(rb, types::I64);
                self.ret_slot = None;
            });
            cell.replace(Some(tls));
        });
    }

    fn end_function(&mut self) {
        use cranelift_module::{Linkage, Module};
        if self.entry_block.is_none() { return; }
        // Epilogue + seal + finalize builder in TLS, then take Context out
        let mut ctx_opt: Option<cranelift_codegen::Context> = None;
        clif_tls::FB.with(|cell| {
            if let Some(mut tls) = cell.take() {
                tls.with(|fb| {
                    use cranelift_codegen::ir::types;
                    if let Some(rb) = self.ret_block {
                        fb.switch_to_block(rb);
                        if fb.func.signature.returns.is_empty() {
                            fb.ins().return_(&[]);
                        } else {
                            let params = fb.func.dfg.block_params(rb).to_vec();
                            let mut v = params.get(0).copied().unwrap_or_else(|| fb.ins().iconst(types::I64, 0));
                            if std::env::var("NYASH_JIT_TRACE_RET").ok().as_deref() == Some("1") {
                                use cranelift_codegen::ir::{AbiParam, Signature};
                                let mut sig = Signature::new(self.module.isa().default_call_conv());
                                sig.params.push(AbiParam::new(types::I64));
                                sig.params.push(AbiParam::new(types::I64));
                                sig.returns.push(AbiParam::new(types::I64));
                                let fid = self.module.declare_function("nyash.jit.dbg_i64", Linkage::Import, &sig).expect("declare dbg_i64");
                                let fref = self.module.declare_func_in_func(fid, fb.func);
                                let tag = fb.ins().iconst(types::I64, 200);
                                let _ = fb.ins().call(fref, &[tag, v]);
                            }
                            let ret_ty = fb.func.signature.returns.get(0).map(|p| p.value_type).unwrap_or(types::I64);
                            if ret_ty == types::F64 { v = fb.ins().fcvt_from_sint(types::F64, v); }
                            fb.ins().return_(&[v]);
                        }
                    }
                    if let Some(en) = self.entry_block { fb.seal_block(en); }
                    for b in &self.blocks { fb.seal_block(*b); }
                    if let Some(rb) = self.ret_block { fb.seal_block(rb); }
                });
                unsafe { tls.finalize_drop(); }
                ctx_opt = Some(*tls.ctx);
            }
        });
        let mut ctx = ctx_opt.expect("missing TLS context");
        let sym_name = self.current_name.clone().unwrap_or_else(|| "jit_fn".to_string());
        let func_id = self.module.declare_function(&sym_name, Linkage::Local, &ctx.func.signature).expect("declare_function failed");
        self.module.define_function(func_id, &mut ctx).expect("define_function failed");
        self.module.clear_context(&mut ctx);
        let _ = self.module.finalize_definitions();
        let code = self.module.get_finalized_function(func_id);

        // SAFETY: We compiled a function with simple (i64 x N) -> i64/f64 というABIだよ。
        // ランタイムでは JitValue から i64 へ正規化して、引数個数に応じた関数型にtransmuteして呼び出すにゃ。
        let argc = self.desired_argc;
        let has_ret = self.desired_has_ret;
        let ret_is_f64 = self.desired_has_ret && self.desired_ret_is_f64;
        // capture code as usize to avoid raw pointer Send/Sync issues in closure
        let code_usize = code as usize;
        unsafe {
            let closure = std::sync::Arc::new(move |args: &[crate::jit::abi::JitValue]| -> crate::jit::abi::JitValue {
                // 正規化: 足りなければ0で埋め、余分は切り捨て
                let mut a: [i64; 6] = [0; 6];
                let take = core::cmp::min(core::cmp::min(argc, 6), args.len());
                for i in 0..take {
                    a[i] = match args[i] { crate::jit::abi::JitValue::I64(v) => v, crate::jit::abi::JitValue::Bool(b) => if b {1} else {0}, crate::jit::abi::JitValue::F64(f) => f as i64, crate::jit::abi::JitValue::Handle(h) => h as i64 };
                }
                // Call according to return type expectation
                let ret_i64 = if has_ret {
                    match argc {
                        0 => { let f: extern "C" fn() -> i64 = std::mem::transmute(code_usize); f() }
                        1 => { let f: extern "C" fn(i64) -> i64 = std::mem::transmute(code_usize); f(a[0]) }
                        2 => { let f: extern "C" fn(i64,i64) -> i64 = std::mem::transmute(code_usize); f(a[0],a[1]) }
                        3 => { let f: extern "C" fn(i64,i64,i64) -> i64 = std::mem::transmute(code_usize); f(a[0],a[1],a[2]) }
                        4 => { let f: extern "C" fn(i64,i64,i64,i64) -> i64 = std::mem::transmute(code_usize); f(a[0],a[1],a[2],a[3]) }
                        5 => { let f: extern "C" fn(i64,i64,i64,i64,i64) -> i64 = std::mem::transmute(code_usize); f(a[0],a[1],a[2],a[3],a[4]) }
                        _ => { let f: extern "C" fn(i64,i64,i64,i64,i64,i64) -> i64 = std::mem::transmute(code_usize); f(a[0],a[1],a[2],a[3],a[4],a[5]) }
                    }
                } else {
                    match argc {
                        0 => { let f: extern "C" fn() = std::mem::transmute(code_usize); f(); 0 }
                        1 => { let f: extern "C" fn(i64) = std::mem::transmute(code_usize); f(a[0]); 0 }
                        2 => { let f: extern "C" fn(i64,i64) = std::mem::transmute(code_usize); f(a[0],a[1]); 0 }
                        3 => { let f: extern "C" fn(i64,i64,i64) = std::mem::transmute(code_usize); f(a[0],a[1],a[2]); 0 }
                        4 => { let f: extern "C" fn(i64,i64,i64,i64) = std::mem::transmute(code_usize); f(a[0],a[1],a[2],a[3]); 0 }
                        5 => { let f: extern "C" fn(i64,i64,i64,i64,i64) = std::mem::transmute(code_usize); f(a[0],a[1],a[2],a[3],a[4]); 0 }
                        _ => { let f: extern "C" fn(i64,i64,i64,i64,i64,i64) = std::mem::transmute(code_usize); f(a[0],a[1],a[2],a[3],a[4],a[5]); 0 }
                    }
                };
                if ret_is_f64 {
                    let ret_f64 = match argc {
                        0 => { let f: extern "C" fn() -> f64 = std::mem::transmute(code_usize); f() }
                        1 => { let f: extern "C" fn(i64) -> f64 = std::mem::transmute(code_usize); f(a[0]) }
                        2 => { let f: extern "C" fn(i64,i64) -> f64 = std::mem::transmute(code_usize); f(a[0],a[1]) }
                        3 => { let f: extern "C" fn(i64,i64,i64) -> f64 = std::mem::transmute(code_usize); f(a[0],a[1],a[2]) }
                        4 => { let f: extern "C" fn(i64,i64,i64,i64) -> f64 = std::mem::transmute(code_usize); f(a[0],a[1],a[2],a[3]) }
                        5 => { let f: extern "C" fn(i64,i64,i64,i64,i64) -> f64 = std::mem::transmute(code_usize); f(a[0],a[1],a[2],a[3],a[4]) }
                        _ => { let f: extern "C" fn(i64,i64,i64,i64,i64,i64) -> f64 = std::mem::transmute(code_usize); f(a[0],a[1],a[2],a[3],a[4],a[5]) }
                    };
                    return crate::jit::abi::JitValue::F64(ret_f64);
                }
                crate::jit::abi::JitValue::I64(ret_i64)
            });
            self.compiled_closure = Some(closure);
        }
        // Important: keep finalized code alive by preserving the JITModule.
        // Swap current module with a fresh one and leak the old module to avoid freeing code memory.
        {
            // Build a fresh JITModule with the same symbol registrations for the next compilation
            let mut jb = cranelift_jit::JITBuilder::new(cranelift_module::default_libcall_names())
                .expect("failed to create JITBuilder");
            // Register host-call symbols (keep in sync with new())
            jb.symbol("nyash.host.stub0", nyash_host_stub0 as *const u8);
            {
                use crate::jit::r#extern::collections as c;
                use crate::jit::r#extern::{handles as h, birth as b, runtime as r};
                use super::extern_thunks::{
                    nyash_plugin_invoke_name_getattr_i64, nyash_plugin_invoke_name_call_i64,
                    nyash_handle_of, nyash_box_birth_h, nyash_box_birth_i64,
                    nyash_rt_checkpoint, nyash_gc_barrier_write,
                };
                jb.symbol(c::SYM_ARRAY_LEN, nyash_array_len as *const u8);
                jb.symbol(c::SYM_ARRAY_GET, nyash_array_get as *const u8);
                jb.symbol(c::SYM_ARRAY_SET, nyash_array_set as *const u8);
                jb.symbol(c::SYM_ARRAY_PUSH, nyash_array_push as *const u8);
                jb.symbol(c::SYM_MAP_GET, nyash_map_get as *const u8);
                jb.symbol(c::SYM_MAP_SET, nyash_map_set as *const u8);
                jb.symbol(c::SYM_MAP_SIZE, nyash_map_size as *const u8);
                jb.symbol("nyash.math.sin_f64", nyash_math_sin_f64 as *const u8);
                jb.symbol("nyash.math.cos_f64", nyash_math_cos_f64 as *const u8);
                jb.symbol("nyash.math.abs_f64", nyash_math_abs_f64 as *const u8);
                jb.symbol("nyash.math.min_f64", nyash_math_min_f64 as *const u8);
                jb.symbol("nyash.math.max_f64", nyash_math_max_f64 as *const u8);
                // Handle-based symbols
                jb.symbol(c::SYM_ARRAY_LEN_H, nyash_array_len_h as *const u8);
                jb.symbol(c::SYM_ARRAY_GET_H, nyash_array_get_h as *const u8);
                jb.symbol(c::SYM_ARRAY_SET_H, nyash_array_set_h as *const u8);
                jb.symbol(c::SYM_ARRAY_PUSH_H, nyash_array_push_h as *const u8);
                jb.symbol(c::SYM_ARRAY_LAST_H, nyash_array_last_h as *const u8);
                jb.symbol(c::SYM_MAP_SIZE_H, nyash_map_size_h as *const u8);
                jb.symbol(c::SYM_MAP_GET_H, nyash_map_get_h as *const u8);
                jb.symbol(c::SYM_MAP_GET_HH, nyash_map_get_hh as *const u8);
                jb.symbol(c::SYM_MAP_SET_H, nyash_map_set_h as *const u8);
                jb.symbol(c::SYM_MAP_HAS_H, nyash_map_has_h as *const u8);
                jb.symbol(c::SYM_ANY_LEN_H, nyash_any_length_h as *const u8);
                jb.symbol(c::SYM_ANY_IS_EMPTY_H, nyash_any_is_empty_h as *const u8);
                jb.symbol(c::SYM_STRING_CHARCODE_AT_H, nyash_string_charcode_at_h as *const u8);
                jb.symbol(c::SYM_STRING_BIRTH_H, nyash_string_birth_h as *const u8);
                jb.symbol(c::SYM_INTEGER_BIRTH_H, nyash_integer_birth_h as *const u8);
                // String-like binary ops (handle, handle)
                jb.symbol(c::SYM_STRING_CONCAT_HH, nyash_string_concat_hh as *const u8);
                jb.symbol(c::SYM_STRING_EQ_HH, nyash_string_eq_hh as *const u8);
                jb.symbol(c::SYM_STRING_LT_HH, nyash_string_lt_hh as *const u8);
                use crate::jit::lower::extern_thunks::nyash_semantics_add_hh;
                // Unified semantics ops
                jb.symbol(crate::jit::r#extern::collections::SYM_SEMANTICS_ADD_HH, nyash_semantics_add_hh as *const u8);
                jb.symbol(b::SYM_BOX_BIRTH_H, nyash_box_birth_h as *const u8);
                jb.symbol("nyash.box.birth_i64", nyash_box_birth_i64 as *const u8);
                // Handle helpers
                jb.symbol(h::SYM_HANDLE_OF, nyash_handle_of as *const u8);
                // Plugin invoke shims (i64/f64)
                jb.symbol("nyash_plugin_invoke3_i64", nyash_plugin_invoke3_i64 as *const u8);
                jb.symbol("nyash_plugin_invoke3_f64", nyash_plugin_invoke3_f64 as *const u8);
                // By-name plugin invoke shims (method-name specific)
                jb.symbol("nyash_plugin_invoke_name_getattr_i64", nyash_plugin_invoke_name_getattr_i64 as *const u8);
                jb.symbol("nyash_plugin_invoke_name_call_i64", nyash_plugin_invoke_name_call_i64 as *const u8);
                // Reserved runtime/GC symbols
                jb.symbol(r::SYM_RT_CHECKPOINT, nyash_rt_checkpoint as *const u8);
                jb.symbol(r::SYM_GC_BARRIER_WRITE, nyash_gc_barrier_write as *const u8);
            }
            let new_module = cranelift_jit::JITModule::new(jb);
            // Leak the old module so finalized code stays valid
            let old = std::mem::replace(&mut self.module, new_module);
            let _leaked: &'static mut cranelift_jit::JITModule = Box::leak(Box::new(old));
        }
        // Reset typed signature flag for next function
        self.typed_sig_prepared = false;
    }

    fn emit_const_i64(&mut self, val: i64) {
        use cranelift_codegen::ir::types;
        let v = CraneliftBuilder::with_fb(|fb| fb.ins().iconst(types::I64, val));
        self.value_stack.push(v);
        self.stats.0 += 1;
    }

    fn emit_const_f64(&mut self, val: f64) {
        self.stats.0 += 1;
        if !crate::jit::config::current().native_f64 { return; }
        use cranelift_codegen::ir::types;
        let v = CraneliftBuilder::with_fb(|fb| fb.ins().f64const(val));
        self.value_stack.push(v);
    }

    fn emit_binop(&mut self, op: BinOpKind) {
        use cranelift_codegen::ir::types;
        if self.value_stack.len() < 2 { return; }
        let mut rhs = self.value_stack.pop().unwrap();
        let mut lhs = self.value_stack.pop().unwrap();
        let res = CraneliftBuilder::with_fb(|fb| {
            let lty = fb.func.dfg.value_type(lhs);
            let rty = fb.func.dfg.value_type(rhs);
            let native_f64 = crate::jit::config::current().native_f64;
            let mut use_f64 = native_f64 && (lty == types::F64 || rty == types::F64);
            if use_f64 {
                if lty != types::F64 { lhs = fb.ins().fcvt_from_sint(types::F64, lhs); }
                if rty != types::F64 { rhs = fb.ins().fcvt_from_sint(types::F64, rhs); }
            }
            if use_f64 {
                match op { BinOpKind::Add => fb.ins().fadd(lhs, rhs), BinOpKind::Sub => fb.ins().fsub(lhs, rhs), BinOpKind::Mul => fb.ins().fmul(lhs, rhs), BinOpKind::Div => fb.ins().fdiv(lhs, rhs), BinOpKind::Mod => fb.ins().f64const(0.0) }
            } else {
                match op { BinOpKind::Add => fb.ins().iadd(lhs, rhs), BinOpKind::Sub => fb.ins().isub(lhs, rhs), BinOpKind::Mul => fb.ins().imul(lhs, rhs), BinOpKind::Div => fb.ins().sdiv(lhs, rhs), BinOpKind::Mod => fb.ins().srem(lhs, rhs) }
            }
        });
        self.value_stack.push(res);
        self.stats.1 += 1;
    }

    fn emit_compare(&mut self, op: CmpKind) {
        use cranelift_codegen::ir::{condcodes::{IntCC, FloatCC}, types};
        if self.value_stack.len() < 2 { return; }
        let mut rhs = self.value_stack.pop().unwrap();
        let mut lhs = self.value_stack.pop().unwrap();
        CraneliftBuilder::with_fb(|fb| {
            if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
            else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
            let lty = fb.func.dfg.value_type(lhs);
            let rty = fb.func.dfg.value_type(rhs);
            let native_f64 = crate::jit::config::current().native_f64;
            let use_f64 = native_f64 && (lty == types::F64 || rty == types::F64);
            let b1 = if use_f64 {
                if lty != types::F64 { lhs = fb.ins().fcvt_from_sint(types::F64, lhs); }
                if rty != types::F64 { rhs = fb.ins().fcvt_from_sint(types::F64, rhs); }
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
        use cranelift_codegen::ir::{types, condcodes::IntCC};
        // Expect stack: ... cond then else
        if self.value_stack.len() < 3 { return; }
        let mut else_v = self.value_stack.pop().unwrap();
        let mut then_v = self.value_stack.pop().unwrap();
        let mut cond_v = self.value_stack.pop().unwrap();
        let sel = CraneliftBuilder::with_fb(|fb| {
            // Normalize types
            let cty = fb.func.dfg.value_type(cond_v);
            if cty == types::I64 {
                cond_v = fb.ins().icmp_imm(IntCC::NotEqual, cond_v, 0);
                crate::jit::rt::b1_norm_inc(1);
            }
            let tty = fb.func.dfg.value_type(then_v);
            if tty != types::I64 { then_v = fb.ins().fcvt_to_sint(types::I64, then_v); }
            let ety = fb.func.dfg.value_type(else_v);
            if ety != types::I64 { else_v = fb.ins().fcvt_to_sint(types::I64, else_v); }
            // Optional runtime debug
        if std::env::var("NYASH_JIT_TRACE_SEL").ok().as_deref() == Some("1") {
            use cranelift_codegen::ir::{AbiParam, Signature};
            use cranelift_module::Linkage;
            let mut sig = Signature::new(self.module.isa().default_call_conv());
            sig.params.push(AbiParam::new(types::I64));
            sig.params.push(AbiParam::new(types::I64));
            sig.returns.push(AbiParam::new(types::I64));
            let fid = self.module
                .declare_function("nyash.jit.dbg_i64", Linkage::Import, &sig)
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
    fn emit_jump(&mut self) { self.stats.3 += 1; }
    fn emit_branch(&mut self) { self.stats.3 += 1; }

    fn emit_return(&mut self) {
        use cranelift_codegen::ir::{types, condcodes::IntCC};
        self.stats.4 += 1;
        CraneliftBuilder::with_fb(|fb| {
            if fb.func.signature.returns.is_empty() { fb.ins().return_(&[]); return; }
            let mut v = if let Some(x) = self.value_stack.pop() { x } else { fb.ins().iconst(types::I64, 0) };
            let v_ty = fb.func.dfg.value_type(v);
            if v_ty != types::I64 {
                if v_ty == types::F64 { v = fb.ins().fcvt_to_sint(types::I64, v); }
                else { let one = fb.ins().iconst(types::I64, 1); let zero = fb.ins().iconst(types::I64, 0); v = fb.ins().select(v, one, zero); }
            }
            if std::env::var("NYASH_JIT_TRACE_RET").ok().as_deref() == Some("1") {
                use cranelift_codegen::ir::{AbiParam, Signature}; use cranelift_module::Linkage;
                let mut sig = Signature::new(self.module.isa().default_call_conv());
                sig.params.push(AbiParam::new(types::I64));
                sig.params.push(AbiParam::new(types::I64));
                sig.returns.push(AbiParam::new(types::I64));
                let fid = self.module.declare_function("nyash.jit.dbg_i64", Linkage::Import, &sig).expect("declare dbg_i64");
                let fref = self.module.declare_func_in_func(fid, fb.func);
                let tag = fb.ins().iconst(types::I64, 201);
                let _ = fb.ins().call(fref, &[tag, v]);
            }
            if let Some(rb) = self.ret_block { fb.ins().jump(rb, &[v]); }
        });
        self.cur_needs_term = false;
    }

    fn emit_host_call(&mut self, symbol: &str, _argc: usize, has_ret: bool) {
        use cranelift_codegen::ir::{AbiParam, Signature, types};
        use cranelift_frontend::FunctionBuilder;
        use cranelift_module::{Linkage, Module};

        // Minimal import+call to a registered stub symbol; ignore args for now
        let call_conv = self.module.isa().default_call_conv();
        let mut sig = Signature::new(call_conv);
        // Collect up to _argc i64 values from stack as arguments (right-to-left)
        let mut args: Vec<cranelift_codegen::ir::Value> = Vec::new();
        let take_n = _argc.min(self.value_stack.len());
        for _ in 0..take_n { if let Some(v) = self.value_stack.pop() { args.push(v); } }
        args.reverse();
        // Build params for each collected arg
        for _ in 0..args.len() { sig.params.push(AbiParam::new(types::I64)); }
        if has_ret { sig.returns.push(AbiParam::new(types::I64)); }

        let func_id = self.module
            .declare_function(symbol, Linkage::Import, &sig)
            .expect("declare import failed");
        let ret = tls_call_import_ret(&mut self.module, func_id, &args, has_ret);
        if let Some(v) = ret { self.value_stack.push(v); }
    }

    fn emit_host_call_typed(&mut self, symbol: &str, params: &[ParamKind], has_ret: bool, ret_is_f64: bool) {
        use cranelift_codegen::ir::{AbiParam, Signature, types};
        use cranelift_frontend::FunctionBuilder;
        use cranelift_module::{Linkage, Module};

        // Pop values according to params length (right-to-left), then reverse
        let mut args: Vec<cranelift_codegen::ir::Value> = Vec::new();
        let take_n = params.len().min(self.value_stack.len());
        for _ in 0..take_n { if let Some(v) = self.value_stack.pop() { args.push(v); } }
        args.reverse();

        // Build typed signature
        let call_conv = self.module.isa().default_call_conv();
        let mut sig = Signature::new(call_conv);
        let abi_param_for_kind = |k: &ParamKind| {
            match k {
                ParamKind::I64 => AbiParam::new(types::I64),
                ParamKind::F64 => AbiParam::new(types::F64),
                ParamKind::B1 => {
                    // Map b1 to I64 unless native-b1 ABI is enabled; keep simple here
                    AbiParam::new(types::I64)
                }
            }
        };
        for k in params { sig.params.push(abi_param_for_kind(k)); }
        if has_ret {
            if ret_is_f64 { sig.returns.push(AbiParam::new(types::F64)); }
            else { sig.returns.push(AbiParam::new(types::I64)); }
        }

        let func_id = self.module
            .declare_function(symbol, Linkage::Import, &sig)
            .expect("declare typed import failed");

        let ret = tls_call_import_ret(&mut self.module, func_id, &args, has_ret);
        if let Some(v) = ret { self.value_stack.push(v); }
    }

    fn emit_plugin_invoke(&mut self, type_id: u32, method_id: u32, argc: usize, has_ret: bool) {
        use cranelift_codegen::ir::{AbiParam, Signature, types};
        use cranelift_frontend::FunctionBuilder;
        use cranelift_module::{Linkage, Module};

        // Pop argc values (right-to-left): receiver + up to 2 args
        let mut arg_vals: Vec<cranelift_codegen::ir::Value> = {
            let take_n = argc.min(self.value_stack.len());
            let mut tmp = Vec::new();
            for _ in 0..take_n { if let Some(v) = self.value_stack.pop() { tmp.push(v); } }
            tmp.reverse();
            tmp
        };

        // Ensure receiver (a0) is a runtime handle via nyash.handle.of (Handle-First)
        let a0_handle = {
            use cranelift_module::Linkage;
            use crate::jit::r#extern::handles as h;
            let call_conv_h = self.module.isa().default_call_conv();
            let mut sig_h = Signature::new(call_conv_h);
            sig_h.params.push(AbiParam::new(types::I64));
            sig_h.returns.push(AbiParam::new(types::I64));
            let func_id_h = self.module
                .declare_function(h::SYM_HANDLE_OF, Linkage::Import, &sig_h)
                .expect("declare handle.of failed");
            // call handle.of(a0)
            tls_call_import_ret(&mut self.module, func_id_h, &arg_vals[0..1], true).expect("handle.of ret")
        };
        arg_vals[0] = a0_handle;

        // Choose f64 shim if allowlisted
        let use_f64 = if has_ret {
            if let Ok(list) = std::env::var("NYASH_JIT_PLUGIN_F64") {
                list.split(',').any(|e| {
                    let mut it = e.split(':');
                    match (it.next(), it.next()) { (Some(t), Some(m)) => t.parse::<u32>().ok()==Some(type_id) && m.parse::<u32>().ok()==Some(method_id), _ => false }
                })
            } else { false }
        } else { false };
        // Build signature: (i64 type_id, i64 method_id, i64 argc, i64 a0, i64 a1, i64 a2) -> i64/f64
        let call_conv = self.module.isa().default_call_conv();
        let mut sig = Signature::new(call_conv);
        for _ in 0..6 { sig.params.push(AbiParam::new(types::I64)); }
        if has_ret { sig.returns.push(AbiParam::new(if use_f64 { types::F64 } else { types::I64 })); }

        let symbol = if use_f64 { "nyash_plugin_invoke3_f64" } else { "nyash_plugin_invoke3_i64" };
        let func_id = self.module
            .declare_function(symbol, Linkage::Import, &sig)
            .expect("declare plugin shim failed");
        // Now emit calls via the single FB
        let ret_val = CraneliftBuilder::with_fb(|fb| {
            if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
            else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
            while arg_vals.len() < 3 { let z = fb.ins().iconst(types::I64, 0); arg_vals.push(z); }
            // Handle-of transform for receiver
            {
            use cranelift_module::Linkage;
            use crate::jit::r#extern::handles as h;
            let call_conv_h = self.module.isa().default_call_conv();
            let mut sig_h = Signature::new(call_conv_h);
            sig_h.params.push(AbiParam::new(types::I64));
            sig_h.returns.push(AbiParam::new(types::I64));
            let func_id_h = self.module
                .declare_function(h::SYM_HANDLE_OF, Linkage::Import, &sig_h)
                .expect("declare handle.of failed");
            let fref_h = self.module.declare_func_in_func(func_id_h, fb.func);
            let call_h = fb.ins().call(fref_h, &[arg_vals[0]]);
            if let Some(rv) = fb.inst_results(call_h).get(0).copied() { arg_vals[0] = rv; }
            }
            let fref = self.module.declare_func_in_func(func_id, fb.func);
            let c_type = fb.ins().iconst(types::I64, type_id as i64);
            let c_meth = fb.ins().iconst(types::I64, method_id as i64);
            let c_argc = fb.ins().iconst(types::I64, argc as i64);
            let call_inst = fb.ins().call(fref, &[c_type, c_meth, c_argc, arg_vals[0], arg_vals[1], arg_vals[2]]);
            if has_ret { fb.inst_results(call_inst).get(0).copied() } else { None }
        });
        if let Some(v) = ret_val { self.value_stack.push(v); }
    }

    fn emit_plugin_invoke_by_name(&mut self, method: &str, argc: usize, has_ret: bool) {
        use cranelift_codegen::ir::{AbiParam, Signature, types};
        use cranelift_frontend::FunctionBuilder;
        use cranelift_module::{Linkage, Module};

        CraneliftBuilder::with_fb(|fb| {
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
        else if let Some(b) = self.entry_block { fb.switch_to_block(b); }

        // Pop argc-1 values (a0 receiver included in argc? Here argc = 1 + args.len())
        let mut arg_vals: Vec<cranelift_codegen::ir::Value> = Vec::new();
        let take_n = argc.min(self.value_stack.len());
        for _ in 0..take_n { if let Some(v) = self.value_stack.pop() { arg_vals.push(v); } }
        arg_vals.reverse();
        while arg_vals.len() < 3 {
            let z = fb.ins().iconst(types::I64, 0);
            arg_vals.push(z);
        }

        // Ensure receiver (a0) is a runtime handle via nyash.handle.of
        {
            use cranelift_module::Linkage;
            use crate::jit::r#extern::handles as h;
            let call_conv_h = self.module.isa().default_call_conv();
            let mut sig_h = Signature::new(call_conv_h);
            sig_h.params.push(AbiParam::new(types::I64));
            sig_h.returns.push(AbiParam::new(types::I64));
            let func_id_h = self.module
                .declare_function(h::SYM_HANDLE_OF, Linkage::Import, &sig_h)
                .expect("declare handle.of failed");
            let fref_h = self.module.declare_func_in_func(func_id_h, fb.func);
            let call_h = fb.ins().call(fref_h, &[arg_vals[0]]);
            // Replace a0 with handle result
            if let Some(rv) = fb.inst_results(call_h).get(0).copied() { arg_vals[0] = rv; }
        }

        // Signature: (i64 argc, i64 a0, i64 a1, i64 a2) -> i64
        let call_conv = self.module.isa().default_call_conv();
        let mut sig = Signature::new(call_conv);
        for _ in 0..4 { sig.params.push(AbiParam::new(types::I64)); }
        if has_ret { sig.returns.push(AbiParam::new(types::I64)); }

        let sym = format!("nyash_plugin_invoke_name_{}_i64", method);
        let func_id = self.module
            .declare_function(&sym, Linkage::Import, &sig)
            .expect("declare by-name plugin shim failed");
        let fref = self.module.declare_func_in_func(func_id, fb.func);
        let c_argc = fb.ins().iconst(types::I64, argc as i64);
        let call_inst = fb.ins().call(fref, &[c_argc, arg_vals[0], arg_vals[1], arg_vals[2]]);
        if has_ret { let results = fb.inst_results(call_inst).to_vec(); if let Some(v) = results.get(0).copied() { self.value_stack.push(v); } }
        });
    }

    // ==== Phase 10.7 block APIs ====
    fn prepare_blocks(&mut self, count: usize) {
        // Do not touch TLS before begin_function; just remember the desired count.
        if count > self.pending_blocks { self.pending_blocks = count; }
    }
    fn switch_to_block(&mut self, index: usize) {
        if index >= self.blocks.len() { return; }
        CraneliftBuilder::with_fb(|fb| { fb.switch_to_block(self.blocks[index]); });
        self.current_block_index = Some(index);
    }
    fn seal_block(&mut self, index: usize) {
        if index >= self.blocks.len() { return; }
        CraneliftBuilder::with_fb(|fb| { fb.seal_block(self.blocks[index]); });
    }
    fn br_if_top_is_true(&mut self, then_index: usize, else_index: usize) {
        use cranelift_codegen::ir::{types, condcodes::IntCC};
        if then_index >= self.blocks.len() || else_index >= self.blocks.len() { return; }
        let had_cond = !self.value_stack.is_empty();
        let popped = self.value_stack.pop();
        CraneliftBuilder::with_fb(|fb| {
            if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
            else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
            let cond_b1 = if let Some(v) = popped {
                let ty = fb.func.dfg.value_type(v);
                if ty == types::I64 {
                    let out = fb.ins().icmp_imm(IntCC::NotEqual, v, 0);
                    crate::jit::rt::b1_norm_inc(1);
                    out
                } else { v }
            } else {
                let zero = fb.ins().iconst(types::I64, 0);
                let out = fb.ins().icmp_imm(IntCC::NotEqual, zero, 0);
                crate::jit::rt::b1_norm_inc(1);
                out
            };
            if std::env::var("NYASH_JIT_TRACE_BR").ok().as_deref() == Some("1") {
                eprintln!("[JIT-CLIF] br_if cond_present={} then={} else={}", had_cond, then_index, else_index);
            }
            let swap = std::env::var("NYASH_JIT_SWAP_THEN_ELSE").ok().as_deref() == Some("1");
            if swap {
                fb.ins().brif(cond_b1, self.blocks[else_index], &[], self.blocks[then_index], &[]);
            } else {
                fb.ins().brif(cond_b1, self.blocks[then_index], &[], self.blocks[else_index], &[]);
            }
        });
        self.stats.3 += 1;
    }
    fn jump_to(&mut self, target_index: usize) {
        if target_index >= self.blocks.len() { return; }
        CraneliftBuilder::with_fb(|fb| {
            if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
            else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
            fb.ins().jump(self.blocks[target_index], &[]);
        });
        self.stats.3 += 1;
    }
    fn ensure_block_param_i64(&mut self, index: usize) {
        self.ensure_block_params_i64(index, 1);
    }
    fn ensure_block_params_i64(&mut self, index: usize, needed: usize) {
        use cranelift_codegen::ir::types;
        if index >= self.blocks.len() { return; }
        let have = self.block_param_counts.get(&index).copied().unwrap_or(0);
        if needed > have {
            CraneliftBuilder::with_fb(|fb| {
                let b = self.blocks[index];
                for _ in have..needed {
                    let _v = fb.append_block_param(b, types::I64);
                }
            });
            self.block_param_counts.insert(index, needed);
            if std::env::var("NYASH_JIT_DUMP").ok().as_deref() == Some("1") {
                eprintln!("[JIT-CLIF] ensure_block_params bb={} now={}", index, needed);
            }
        }
    }
    fn ensure_block_params_b1(&mut self, index: usize, needed: usize) {
        // Store as i64 block params for ABI stability; consumers can convert to b1
        self.ensure_block_params_i64(index, needed);
    }
    fn push_block_param_i64_at(&mut self, pos: usize) {
        use cranelift_codegen::ir::types;
        let v = CraneliftBuilder::with_fb(|fb| {
            let b_opt = if let Some(idx) = self.current_block_index { Some(self.blocks[idx]) } else { self.entry_block };
            let params = if let Some(b) = b_opt { fb.switch_to_block(b); fb.func.dfg.block_params(b).to_vec() } else { Vec::new() };
            if std::env::var("NYASH_JIT_DUMP").ok().as_deref() == Some("1") {
                eprintln!("[JIT-CLIF] push_block_param_i64_at pos={} available={}", pos, params.len());
            }
            if let Some(v) = params.get(pos).copied() { v } else { fb.ins().iconst(types::I64, 0) }
        });
        self.value_stack.push(v);
    }
    fn push_block_param_b1_at(&mut self, pos: usize) {
        use cranelift_codegen::ir::{types, condcodes::IntCC};
        let b1 = CraneliftBuilder::with_fb(|fb| {
            let b_opt = if let Some(idx) = self.current_block_index { Some(self.blocks[idx]) } else { self.entry_block };
            let params = if let Some(b) = b_opt { fb.func.dfg.block_params(b).to_vec() } else { Vec::new() };
            if std::env::var("NYASH_JIT_DUMP").ok().as_deref() == Some("1") {
                eprintln!("[JIT-CLIF] push_block_param_b1_at pos={} available={}", pos, params.len());
            }
            if let Some(v) = params.get(pos).copied() {
                let ty = fb.func.dfg.value_type(v);
                if ty == types::I64 { fb.ins().icmp_imm(IntCC::NotEqual, v, 0) } else { v }
            } else {
                let zero = fb.ins().iconst(types::I64, 0);
                fb.ins().icmp_imm(IntCC::NotEqual, zero, 0)
            }
        });
        self.value_stack.push(b1);
    }
    fn br_if_with_args(&mut self, then_index: usize, else_index: usize, then_n: usize, else_n: usize) {
        use cranelift_codegen::ir::{types, condcodes::IntCC};
        use cranelift_frontend::FunctionBuilder;
        if then_index >= self.blocks.len() || else_index >= self.blocks.len() { return; }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
        else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        // Pop else args then then args (so stack order can be value-friendly)
        let mut else_args: Vec<cranelift_codegen::ir::Value> = Vec::new();
        for _ in 0..else_n { if let Some(v) = self.value_stack.pop() { else_args.push(v); } }
        else_args.reverse();
        let mut then_args: Vec<cranelift_codegen::ir::Value> = Vec::new();
        for _ in 0..then_n { if let Some(v) = self.value_stack.pop() { then_args.push(v); } }
        then_args.reverse();
        // Now pop condition last (it was pushed first by LowerCore)
        let cond_b1 = if let Some(v) = self.value_stack.pop() {
            let ty = fb.func.dfg.value_type(v);
            if ty == types::I64 { let out = fb.ins().icmp_imm(IntCC::NotEqual, v, 0); crate::jit::rt::b1_norm_inc(1); out } else { v }
        } else {
            let zero = fb.ins().iconst(types::I64, 0);
            let out = fb.ins().icmp_imm(IntCC::NotEqual, zero, 0);
            crate::jit::rt::b1_norm_inc(1);
            out
        };
        if std::env::var("NYASH_JIT_DUMP").ok().as_deref() == Some("1") {
            eprintln!("[JIT-CLIF] br_if_with_args then_n={} else_n={}", then_n, else_n);
        }
        fb.ins().brif(cond_b1, self.blocks[then_index], &then_args, self.blocks[else_index], &else_args);
        self.stats.3 += 1;
        fb.finalize();
    }
    fn jump_with_args(&mut self, target_index: usize, n: usize) {
        use cranelift_frontend::FunctionBuilder;
        if target_index >= self.blocks.len() { return; }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
        else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        let mut args: Vec<cranelift_codegen::ir::Value> = Vec::new();
        for _ in 0..n { if let Some(v) = self.value_stack.pop() { args.push(v); } }
        args.reverse();
        if std::env::var("NYASH_JIT_DUMP").ok().as_deref() == Some("1") {
            eprintln!("[JIT-CLIF] jump_with_args target={} n={}", target_index, n);
        }
        fb.ins().jump(self.blocks[target_index], &args);
        self.stats.3 += 1;
        fb.finalize();
    }

    fn hint_ret_bool(&mut self, is_b1: bool) { self.ret_hint_is_b1 = is_b1; }

    fn ensure_local_i64(&mut self, index: usize) {
        use cranelift_codegen::ir::{StackSlotData, StackSlotKind};
        use cranelift_frontend::FunctionBuilder;
        if self.local_slots.contains_key(&index) { return; }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        let slot = fb.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8));
        self.local_slots.insert(index, slot);
        fb.finalize();
    }
    fn store_local_i64(&mut self, index: usize) {
        use cranelift_codegen::ir::{types, condcodes::IntCC};
        use cranelift_frontend::FunctionBuilder;
        if let Some(mut v) = self.value_stack.pop() {
            // Ensure slot without overlapping FunctionBuilder borrows
            if !self.local_slots.contains_key(&index) { self.ensure_local_i64(index); }
            let slot = self.local_slots.get(&index).copied();
            let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
            if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
            else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
            let ty = fb.func.dfg.value_type(v);
            if ty != types::I64 {
                if ty == types::F64 {
                    v = fb.ins().fcvt_to_sint(types::I64, v);
                } else {
                    // Convert unknown ints/bools to i64 via (v!=0)?1:0
                    let one = fb.ins().iconst(types::I64, 1);
                    let zero = fb.ins().iconst(types::I64, 0);
                    let b1 = fb.ins().icmp_imm(IntCC::NotEqual, v, 0);
                    v = fb.ins().select(b1, one, zero);
                }
            }
            if let Some(slot) = slot { fb.ins().stack_store(v, slot, 0); }
            fb.finalize();
        }
    }
    fn load_local_i64(&mut self, index: usize) {
        use cranelift_codegen::ir::types;
        use cranelift_frontend::FunctionBuilder;
        if !self.local_slots.contains_key(&index) { self.ensure_local_i64(index); }
        if let Some(&slot) = self.local_slots.get(&index) {
            let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
            if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
            else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
            let v = fb.ins().stack_load(types::I64, slot, 0);
            self.value_stack.push(v);
            self.stats.0 += 1;
            fb.finalize();
        }
    }
}

#[cfg(feature = "cranelift-jit")]
impl CraneliftBuilder {
    #[cfg(feature = "cranelift-jit")]
    fn mk_fb(&mut self) -> cranelift_frontend::FunctionBuilder {
        // Reset FunctionBuilderContext to satisfy Cranelift's requirement when instantiating a new builder.
        self.fbc = cranelift_frontend::FunctionBuilderContext::new();
        cranelift_frontend::FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc)
    }
    fn entry_param(&mut self, index: usize) -> Option<cranelift_codegen::ir::Value> {
        if let Some(b) = self.entry_block {
            return CraneliftBuilder::with_fb(|fb| {
                fb.switch_to_block(b);
                fb.func.dfg.block_params(b).get(index).copied()
            });
        }
        None
    }
}

// ==== Minimal ObjectModule-based builder for AOT .o emission (Phase 10.2) ====
#[cfg(feature = "cranelift-jit")]
pub struct ObjectBuilder {
    module: cranelift_object::ObjectModule,
    ctx: cranelift_codegen::Context,
    fbc: cranelift_frontend::FunctionBuilderContext,
    current_name: Option<String>,
    entry_block: Option<cranelift_codegen::ir::Block>,
    blocks: Vec<cranelift_codegen::ir::Block>,
    current_block_index: Option<usize>,
    value_stack: Vec<cranelift_codegen::ir::Value>,
    typed_sig_prepared: bool,
    desired_argc: usize,
    desired_has_ret: bool,
    desired_ret_is_f64: bool,
    ret_hint_is_b1: bool,
    local_slots: std::collections::HashMap<usize, cranelift_codegen::ir::StackSlot>,
    block_param_counts: std::collections::HashMap<usize, usize>,
    pub stats: (u64,u64,u64,u64,u64),
    pub object_bytes: Option<Vec<u8>>,
}

#[cfg(feature = "cranelift-jit")]
impl ObjectBuilder {
    pub fn new() -> Self {
        use cranelift_codegen::settings;
        // Host ISA
        let isa = cranelift_native::builder()
            .expect("host ISA")
            .finish(settings::Flags::new(settings::builder()))
            .expect("finish ISA");
        let obj_builder = cranelift_object::ObjectBuilder::new(
            isa,
            "nyash_aot".to_string(),
            cranelift_module::default_libcall_names(),
        )
        .expect("ObjectBuilder");
        let module = cranelift_object::ObjectModule::new(obj_builder);
        Self {
            module,
            ctx: cranelift_codegen::Context::new(),
            fbc: cranelift_frontend::FunctionBuilderContext::new(),
            current_name: None,
            entry_block: None,
            blocks: Vec::new(),
            current_block_index: None,
            value_stack: Vec::new(),
            typed_sig_prepared: false,
            desired_argc: 0,
            desired_has_ret: true,
            desired_ret_is_f64: false,
            ret_hint_is_b1: false,
            local_slots: std::collections::HashMap::new(),
            block_param_counts: std::collections::HashMap::new(),
            stats: (0,0,0,0,0),
            object_bytes: None,
        }
    }

    pub fn take_object_bytes(&mut self) -> Option<Vec<u8>> { self.object_bytes.take() }

    fn entry_param(&mut self, index: usize) -> Option<cranelift_codegen::ir::Value> {
        use cranelift_frontend::FunctionBuilder;
        let mut fb = cranelift_frontend::FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(b) = self.entry_block {
            fb.switch_to_block(b);
            let params = fb.func.dfg.block_params(b).to_vec();
            if let Some(v) = params.get(index).copied() { return Some(v); }
        }
        None
    }
}

#[cfg(feature = "cranelift-jit")]
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
            if self.desired_has_ret {
                if self.desired_ret_is_f64 { sig.returns.push(AbiParam::new(types::F64)); }
                else { sig.returns.push(AbiParam::new(types::I64)); }
            }
            self.ctx.func.signature = sig;
        }
        self.ctx.func.name = cranelift_codegen::ir::UserFuncName::user(0, 0);
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if self.blocks.is_empty() { self.blocks.push(fb.create_block()); }
        let entry = self.blocks[0];
        fb.append_block_params_for_function_params(entry);
        fb.switch_to_block(entry);
        // Defer sealing to allow entry PHI params when needed
        self.entry_block = Some(entry);
        self.current_block_index = Some(0);
        fb.finalize();
    }
    fn end_function(&mut self) {
        use cranelift_module::{Linkage, Module};
        if self.entry_block.is_none() { return; }
        let orig = self.current_name.clone().unwrap_or_else(|| "nyash_fn".to_string());
        let export = if orig == "main" { "ny_main".to_string() } else { orig };
        let func_id = self.module.declare_function(&export, Linkage::Export, &self.ctx.func.signature).expect("declare object function");
        self.module.define_function(func_id, &mut self.ctx).expect("define object function");
        self.module.clear_context(&mut self.ctx);
        // Finish current module and immediately replace with a fresh one for next function
        let finished_module = {
            // swap out with a fresh empty module
            use cranelift_codegen::settings;
            let isa = cranelift_native::builder().expect("host ISA").finish(settings::Flags::new(settings::builder())).expect("finish ISA");
            let fresh = cranelift_object::ObjectModule::new(
                cranelift_object::ObjectBuilder::new(isa, "nyash_aot".to_string(), cranelift_module::default_libcall_names()).expect("ObjectBuilder")
            );
            std::mem::replace(&mut self.module, fresh)
        };
        let obj = finished_module.finish();
        let bytes = obj.emit().expect("emit object");
        self.object_bytes = Some(bytes);
        // reset for next
        self.blocks.clear();
        self.entry_block = None;
        self.current_block_index = None;
        self.typed_sig_prepared = false;
    }
    fn prepare_signature_i64(&mut self, argc: usize, has_ret: bool) { self.desired_argc = argc; self.desired_has_ret = has_ret; self.desired_ret_is_f64 = false; }
    fn prepare_signature_typed(&mut self, params: &[ParamKind], ret_is_f64: bool) {
        use cranelift_codegen::ir::{AbiParam, Signature, types};
        let call_conv = self.module.isa().default_call_conv();
        let mut sig = Signature::new(call_conv);
        for p in params {
            match p { ParamKind::I64 => sig.params.push(AbiParam::new(types::I64)), ParamKind::F64 => sig.params.push(AbiParam::new(types::F64)), ParamKind::B1 => sig.params.push(AbiParam::new(types::I64)) }
        }
        if ret_is_f64 { sig.returns.push(AbiParam::new(types::F64)); } else { sig.returns.push(AbiParam::new(types::I64)); }
        self.ctx.func.signature = sig; self.typed_sig_prepared = true; self.desired_argc = params.len(); self.desired_has_ret = true; self.desired_ret_is_f64 = ret_is_f64;
    }
    fn emit_param_i64(&mut self, index: usize) { if let Some(v) = self.entry_param(index) { self.value_stack.push(v); } }
    fn emit_const_i64(&mut self, val: i64) {
        use cranelift_codegen::ir::types; use cranelift_frontend::FunctionBuilder;
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); } else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        let v = fb.ins().iconst(types::I64, val); self.value_stack.push(v); self.stats.0 += 1; fb.finalize();
    }
    fn emit_const_f64(&mut self, val: f64) {
        use cranelift_codegen::ir::types; use cranelift_frontend::FunctionBuilder;
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); } else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        let v = fb.ins().f64const(val); self.value_stack.push(v); fb.finalize();
    }
    fn emit_binop(&mut self, op: BinOpKind) {
        use cranelift_frontend::FunctionBuilder; use cranelift_codegen::ir::types;
        if self.value_stack.len() < 2 { return; }
        let mut rhs = self.value_stack.pop().unwrap(); let mut lhs = self.value_stack.pop().unwrap();
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); } else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        let lty = fb.func.dfg.value_type(lhs); let rty = fb.func.dfg.value_type(rhs);
        let use_f64 = lty == types::F64 || rty == types::F64;
        if use_f64 { if lty != types::F64 { lhs = fb.ins().fcvt_from_sint(types::F64, lhs); } if rty != types::F64 { rhs = fb.ins().fcvt_from_sint(types::F64, rhs); } }
        let res = if use_f64 { match op { BinOpKind::Add => fb.ins().fadd(lhs, rhs), BinOpKind::Sub => fb.ins().fsub(lhs, rhs), BinOpKind::Mul => fb.ins().fmul(lhs, rhs), BinOpKind::Div => fb.ins().fdiv(lhs, rhs), BinOpKind::Mod => fb.ins().f64const(0.0) } } else { match op { BinOpKind::Add => fb.ins().iadd(lhs, rhs), BinOpKind::Sub => fb.ins().isub(lhs, rhs), BinOpKind::Mul => fb.ins().imul(lhs, rhs), BinOpKind::Div => fb.ins().sdiv(lhs, rhs), BinOpKind::Mod => fb.ins().srem(lhs, rhs) } };
        self.value_stack.push(res); self.stats.1 += 1; fb.finalize();
    }
    fn emit_compare(&mut self, op: CmpKind) {
        use cranelift_frontend::FunctionBuilder; use cranelift_codegen::ir::{types, condcodes::{IntCC, FloatCC}};
        if self.value_stack.len() < 2 { return; }
        let mut rhs = self.value_stack.pop().unwrap(); let mut lhs = self.value_stack.pop().unwrap();
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); } else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        let lty = fb.func.dfg.value_type(lhs); let rty = fb.func.dfg.value_type(rhs);
        let use_f64 = lty == types::F64 || rty == types::F64;
        let b1 = if use_f64 {
            if lty != types::F64 { lhs = fb.ins().fcvt_from_sint(types::F64, lhs); }
            if rty != types::F64 { rhs = fb.ins().fcvt_from_sint(types::F64, rhs); }
            let cc = match op { CmpKind::Eq => FloatCC::Equal, CmpKind::Ne => FloatCC::NotEqual, CmpKind::Lt => FloatCC::LessThan, CmpKind::Le => FloatCC::LessThanOrEqual, CmpKind::Gt => FloatCC::GreaterThan, CmpKind::Ge => FloatCC::GreaterThanOrEqual };
            fb.ins().fcmp(cc, lhs, rhs)
        } else {
            let cc = match op { CmpKind::Eq => IntCC::Equal, CmpKind::Ne => IntCC::NotEqual, CmpKind::Lt => IntCC::SignedLessThan, CmpKind::Le => IntCC::SignedLessThanOrEqual, CmpKind::Gt => IntCC::SignedGreaterThan, CmpKind::Ge => IntCC::SignedGreaterThanOrEqual };
            fb.ins().icmp(cc, lhs, rhs)
        };
        self.value_stack.push(b1); self.stats.2 += 1; fb.finalize();
    }
    fn emit_jump(&mut self) { self.stats.3 += 1; }
    fn emit_branch(&mut self) { self.stats.3 += 1; }
    fn emit_return(&mut self) {
        // ObjectBuilder: return directly (no dedicated ret_block)
        use cranelift_frontend::FunctionBuilder; use cranelift_codegen::ir::{types, condcodes::IntCC};
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); } else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        if fb.func.signature.returns.is_empty() {
            fb.ins().return_(&[]);
            fb.finalize();
            return;
        }
        let mut v = if let Some(x) = self.value_stack.pop() { x } else { fb.ins().iconst(types::I64, 0) };
        let v_ty = fb.func.dfg.value_type(v);
        if v_ty != types::I64 {
            if v_ty == types::F64 { v = fb.ins().fcvt_to_sint(types::I64, v); }
            else { let one = fb.ins().iconst(types::I64, 1); let zero = fb.ins().iconst(types::I64, 0); v = fb.ins().select(v, one, zero); }
        }
        fb.ins().return_(&[v]);
        fb.finalize();
    }
    fn emit_host_call(&mut self, symbol: &str, argc: usize, has_ret: bool) {
        use cranelift_codegen::ir::{AbiParam, Signature, types}; use cranelift_frontend::FunctionBuilder; use cranelift_module::{Linkage, Module};
        let call_conv = self.module.isa().default_call_conv(); let mut sig = Signature::new(call_conv);
        let mut args: Vec<cranelift_codegen::ir::Value> = Vec::new(); let take_n = argc.min(self.value_stack.len()); for _ in 0..take_n { if let Some(v) = self.value_stack.pop() { args.push(v); } } args.reverse(); for _ in 0..args.len() { sig.params.push(AbiParam::new(types::I64)); } if has_ret { sig.returns.push(AbiParam::new(types::I64)); }
        let func_id = self.module.declare_function(symbol, Linkage::Import, &sig).expect("declare import");
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc); if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); } else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        let fref = self.module.declare_func_in_func(func_id, fb.func); let call_inst = fb.ins().call(fref, &args);
        if has_ret { let results = fb.inst_results(call_inst).to_vec(); if let Some(v) = results.get(0).copied() { self.value_stack.push(v); } }
        fb.finalize();
    }
    fn emit_host_call_typed(&mut self, symbol: &str, params: &[ParamKind], has_ret: bool, ret_is_f64: bool) {
        use cranelift_codegen::ir::{AbiParam, Signature, types}; use cranelift_frontend::FunctionBuilder; use cranelift_module::{Linkage, Module};
        let mut args: Vec<cranelift_codegen::ir::Value> = Vec::new(); let take_n = params.len().min(self.value_stack.len()); for _ in 0..take_n { if let Some(v) = self.value_stack.pop() { args.push(v); } } args.reverse();
        let call_conv = self.module.isa().default_call_conv(); let mut sig = Signature::new(call_conv);
        for k in params { match k { ParamKind::I64 => sig.params.push(AbiParam::new(types::I64)), ParamKind::F64 => sig.params.push(AbiParam::new(types::F64)), ParamKind::B1 => sig.params.push(AbiParam::new(types::I64)) } }
        if has_ret { if ret_is_f64 { sig.returns.push(AbiParam::new(types::F64)); } else { sig.returns.push(AbiParam::new(types::I64)); } }
        let func_id = self.module.declare_function(symbol, Linkage::Import, &sig).expect("declare typed import");
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc); if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); } else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        let fref = self.module.declare_func_in_func(func_id, fb.func); let call_inst = fb.ins().call(fref, &args);
        if has_ret { let results = fb.inst_results(call_inst).to_vec(); if let Some(v) = results.get(0).copied() { self.value_stack.push(v); } }
        fb.finalize();
    }
    fn emit_plugin_invoke(&mut self, type_id: u32, method_id: u32, argc: usize, has_ret: bool) {
        use cranelift_codegen::ir::{AbiParam, Signature, types}; use cranelift_frontend::FunctionBuilder; use cranelift_module::{Linkage, Module};
        let mut arg_vals: Vec<cranelift_codegen::ir::Value> = Vec::new(); let take_n = argc.min(self.value_stack.len()); for _ in 0..take_n { if let Some(v) = self.value_stack.pop() { arg_vals.push(v); } } arg_vals.reverse();
        while arg_vals.len() < 3 { let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc); if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); } else if let Some(b) = self.entry_block { fb.switch_to_block(b); } let z = fb.ins().iconst(types::I64, 0); fb.finalize(); arg_vals.push(z); }
        // Choose f64 or i64 shim based on env allowlist: NYASH_JIT_PLUGIN_F64="type:method,type:method"
        let use_f64 = if has_ret {
            if let Ok(list) = std::env::var("NYASH_JIT_PLUGIN_F64") {
                list.split(',').any(|e| {
                    let mut it = e.split(':');
                    match (it.next(), it.next()) { (Some(t), Some(m)) => t.parse::<u32>().ok()==Some(type_id) && m.parse::<u32>().ok()==Some(method_id), _ => false }
                })
            } else { false }
        } else { false };
        let call_conv = self.module.isa().default_call_conv(); let mut sig = Signature::new(call_conv); for _ in 0..6 { sig.params.push(AbiParam::new(types::I64)); } if has_ret { if use_f64 { sig.returns.push(AbiParam::new(types::F64)); } else { sig.returns.push(AbiParam::new(types::I64)); } }
        let symbol = if use_f64 { "nyash_plugin_invoke3_f64" } else { "nyash_plugin_invoke3_i64" };
        let func_id = self.module.declare_function(symbol, Linkage::Import, &sig).expect("declare plugin shim");
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc); if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); } else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        let fref = self.module.declare_func_in_func(func_id, fb.func);
        let c_type = fb.ins().iconst(types::I64, type_id as i64); let c_meth = fb.ins().iconst(types::I64, method_id as i64); let c_argc = fb.ins().iconst(types::I64, argc as i64);
        let call_inst = fb.ins().call(fref, &[c_type, c_meth, c_argc, arg_vals[0], arg_vals[1], arg_vals[2]]);
        if has_ret { let results = fb.inst_results(call_inst).to_vec(); if let Some(v) = results.get(0).copied() { self.value_stack.push(v); } }
        fb.finalize();
    }
    fn prepare_blocks(&mut self, count: usize) { use cranelift_frontend::FunctionBuilder; if count == 0 { return; } let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc); if self.blocks.len() < count { for _ in 0..(count - self.blocks.len()) { self.blocks.push(fb.create_block()); } } fb.finalize(); }
    fn switch_to_block(&mut self, index: usize) { use cranelift_frontend::FunctionBuilder; use cranelift_codegen::ir::{types, AbiParam, Signature}; use cranelift_module::Linkage; if index >= self.blocks.len() { return; } let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc); fb.switch_to_block(self.blocks[index]); if std::env::var("NYASH_JIT_TRACE_BLOCKS").ok().as_deref() == Some("1") { let mut sig = Signature::new(self.module.isa().default_call_conv()); sig.params.push(AbiParam::new(types::I64)); let fid = self.module.declare_function("nyash.jit.block_enter", Linkage::Import, &sig).expect("declare block_enter"); let fref = self.module.declare_func_in_func(fid, fb.func); let bi = fb.ins().iconst(types::I64, index as i64); let _ = fb.ins().call(fref, &[bi]); } self.current_block_index = Some(index); fb.finalize(); }
    fn seal_block(&mut self, index: usize) { use cranelift_frontend::FunctionBuilder; if index >= self.blocks.len() { return; } let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc); fb.seal_block(self.blocks[index]); fb.finalize(); }
    fn br_if_top_is_true(&mut self, then_index: usize, else_index: usize) {
        use cranelift_codegen::ir::{types, condcodes::IntCC};
        use cranelift_frontend::FunctionBuilder;
        if then_index >= self.blocks.len() || else_index >= self.blocks.len() { return; }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
        else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        let cond_b1 = if let Some(v) = self.value_stack.pop() {
            let ty = fb.func.dfg.value_type(v);
            if ty == types::I64 { fb.ins().icmp_imm(IntCC::NotEqual, v, 0) } else { v }
        } else {
            let z = fb.ins().iconst(types::I64, 0);
            fb.ins().icmp_imm(IntCC::NotEqual, z, 0)
        };
        fb.ins().brif(cond_b1, self.blocks[then_index], &[], self.blocks[else_index], &[]);
        self.stats.3 += 1;
        fb.finalize();
    }
    fn ensure_block_params_i64(&mut self, index: usize, count: usize) {
        use cranelift_frontend::FunctionBuilder;
        if index >= self.blocks.len() { return; }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        let have = self.block_param_counts.get(&index).copied().unwrap_or(0);
        if count > have {
            let b = self.blocks[index];
            for _ in have..count { let _ = fb.append_block_param(b, cranelift_codegen::ir::types::I64); }
            self.block_param_counts.insert(index, count);
            if std::env::var("NYASH_JIT_DUMP").ok().as_deref() == Some("1") {
                eprintln!("[JIT-CLIF] ensure_block_params bb={} now={}", index, count);
            }
        }
        fb.finalize();
    }
    fn push_block_param_i64_at(&mut self, pos: usize) {
        use cranelift_frontend::FunctionBuilder;
        use cranelift_codegen::ir::types;
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        let b = if let Some(idx) = self.current_block_index { self.blocks[idx] } else if let Some(b) = self.entry_block { b } else { fb.create_block() };
        fb.switch_to_block(b);
        let params = fb.func.dfg.block_params(b).to_vec();
        if std::env::var("NYASH_JIT_DUMP").ok().as_deref() == Some("1") {
            eprintln!("[JIT-CLIF] push_block_param_i64_at pos={} available={}", pos, params.len());
        }
        if let Some(v) = params.get(pos).copied() { self.value_stack.push(v); }
        else { let z = fb.ins().iconst(types::I64, 0); self.value_stack.push(z); }
    }
    fn jump_to(&mut self, target_index: usize) {
        use cranelift_frontend::FunctionBuilder;
        if target_index >= self.blocks.len() { return; }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
        else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        fb.ins().jump(self.blocks[target_index], &[]);
        self.stats.3 += 1;
    }
    fn hint_ret_bool(&mut self, is_b1: bool) { self.ret_hint_is_b1 = is_b1; }
    fn ensure_local_i64(&mut self, index: usize) { use cranelift_codegen::ir::{StackSlotData, StackSlotKind}; use cranelift_frontend::FunctionBuilder; if self.local_slots.contains_key(&index) { return; } let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc); let slot = fb.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8)); self.local_slots.insert(index, slot); }
    fn store_local_i64(&mut self, index: usize) { use cranelift_codegen::ir::{types, condcodes::IntCC}; use cranelift_frontend::FunctionBuilder; if let Some(mut v) = self.value_stack.pop() { if !self.local_slots.contains_key(&index) { self.ensure_local_i64(index); } let slot = self.local_slots.get(&index).copied(); let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc); if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); } else if let Some(b) = self.entry_block { fb.switch_to_block(b); } let ty = fb.func.dfg.value_type(v); if ty != types::I64 { if ty == types::F64 { v = fb.ins().fcvt_to_sint(types::I64, v); } else { let one = fb.ins().iconst(types::I64, 1); let zero = fb.ins().iconst(types::I64, 0); let b1 = fb.ins().icmp_imm(IntCC::NotEqual, v, 0); v = fb.ins().select(b1, one, zero); } } if let Some(slot) = slot { fb.ins().stack_store(v, slot, 0); } } }
    fn load_local_i64(&mut self, index: usize) { use cranelift_codegen::ir::types; use cranelift_frontend::FunctionBuilder; if !self.local_slots.contains_key(&index) { self.ensure_local_i64(index); } if let Some(&slot) = self.local_slots.get(&index) { let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc); if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); } else if let Some(b) = self.entry_block { fb.switch_to_block(b); } let v = fb.ins().stack_load(types::I64, slot, 0); self.value_stack.push(v); self.stats.0 += 1; } }
}

// removed duplicate impl IRBuilder for CraneliftBuilder (emit_param_i64 moved into main impl)

#[cfg(feature = "cranelift-jit")]
impl CraneliftBuilder {
    #[cfg(feature = "cranelift-jit")]
    fn with_fb<R>(f: impl FnOnce(&mut cranelift_frontend::FunctionBuilder<'static>) -> R) -> R {
        clif_tls::FB.with(|cell| {
            let mut opt = cell.borrow_mut();
            let tls = opt.as_mut().expect("FunctionBuilder TLS not initialized");
            tls.with(f)
        })
    }
    pub fn new() -> Self {
        // Initialize a minimal JITModule to validate linking; not used yet
        let mut builder = cranelift_jit::JITBuilder::new(cranelift_module::default_libcall_names())
            .expect("failed to create JITBuilder");
            // Register host-call symbols (PoC: map to simple C-ABI stubs)
            builder.symbol("nyash.host.stub0", nyash_host_stub0 as *const u8);
            builder.symbol("nyash.jit.dbg_i64", nyash_jit_dbg_i64 as *const u8);
            // Async/Future
            builder.symbol(crate::jit::r#extern::r#async::SYM_FUTURE_AWAIT_H, nyash_future_await_h as *const u8);
            builder.symbol(crate::jit::r#extern::result::SYM_RESULT_OK_H, nyash_result_ok_h as *const u8);
            builder.symbol(crate::jit::r#extern::result::SYM_RESULT_ERR_H, nyash_result_err_h as *const u8);
            builder.symbol("nyash.jit.block_enter", nyash_jit_block_enter as *const u8);
            // Async/Future
            builder.symbol(crate::jit::r#extern::r#async::SYM_FUTURE_AWAIT_H, nyash_future_await_h as *const u8);
            builder.symbol(crate::jit::r#extern::result::SYM_RESULT_OK_H, nyash_result_ok_h as *const u8);
            builder.symbol(crate::jit::r#extern::result::SYM_RESULT_ERR_H, nyash_result_err_h as *const u8);
            builder.symbol("nyash.jit.dbg_i64", nyash_jit_dbg_i64 as *const u8);
        {
            use crate::jit::r#extern::collections as c;
            use crate::jit::r#extern::{handles as h, birth as b, runtime as r};
            use super::extern_thunks::{
                nyash_plugin_invoke_name_getattr_i64, nyash_plugin_invoke_name_call_i64,
                nyash_handle_of, nyash_box_birth_h, nyash_box_birth_i64,
                nyash_rt_checkpoint, nyash_gc_barrier_write,
            };
            builder.symbol(c::SYM_ARRAY_LEN, nyash_array_len as *const u8);
            builder.symbol(c::SYM_ARRAY_GET, nyash_array_get as *const u8);
            builder.symbol(c::SYM_ARRAY_SET, nyash_array_set as *const u8);
            builder.symbol(c::SYM_ARRAY_PUSH, nyash_array_push as *const u8);
            builder.symbol(c::SYM_MAP_GET, nyash_map_get as *const u8);
            builder.symbol(c::SYM_MAP_SET, nyash_map_set as *const u8);
            builder.symbol(c::SYM_MAP_SIZE, nyash_map_size as *const u8);
            // Math f64 externs
            builder.symbol("nyash.math.sin_f64", nyash_math_sin_f64 as *const u8);
            builder.symbol("nyash.math.cos_f64", nyash_math_cos_f64 as *const u8);
            builder.symbol("nyash.math.abs_f64", nyash_math_abs_f64 as *const u8);
            builder.symbol("nyash.math.min_f64", nyash_math_min_f64 as *const u8);
            builder.symbol("nyash.math.max_f64", nyash_math_max_f64 as *const u8);
            // Handle-based symbols
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
            builder.symbol(c::SYM_ANY_IS_EMPTY_H, nyash_any_is_empty_h as *const u8);
            builder.symbol(c::SYM_STRING_CHARCODE_AT_H, nyash_string_charcode_at_h as *const u8);
            builder.symbol(c::SYM_STRING_BIRTH_H, nyash_string_birth_h as *const u8);
            builder.symbol(c::SYM_INTEGER_BIRTH_H, nyash_integer_birth_h as *const u8);
            builder.symbol("nyash.console.birth_h", nyash_console_birth_h as *const u8);
            // String-like binary ops (handle, handle)
            builder.symbol(c::SYM_STRING_CONCAT_HH, nyash_string_concat_hh as *const u8);
            builder.symbol(c::SYM_STRING_EQ_HH, nyash_string_eq_hh as *const u8);
            builder.symbol(c::SYM_STRING_LT_HH, nyash_string_lt_hh as *const u8);
            builder.symbol(b::SYM_BOX_BIRTH_H, nyash_box_birth_h as *const u8);
            builder.symbol("nyash.box.birth_i64", nyash_box_birth_i64 as *const u8);
            // Handle helpers
            builder.symbol(h::SYM_HANDLE_OF, nyash_handle_of as *const u8);
            // Plugin invoke shims (i64/f64)
            builder.symbol("nyash_plugin_invoke3_i64", nyash_plugin_invoke3_i64 as *const u8);
            builder.symbol("nyash_plugin_invoke3_f64", nyash_plugin_invoke3_f64 as *const u8);
            // JIT debug helpers (conditionally called when env toggles are set)
            builder.symbol("nyash.jit.dbg_i64", nyash_jit_dbg_i64 as *const u8);
            builder.symbol("nyash.jit.block_enter", nyash_jit_block_enter as *const u8);
            // By-name plugin invoke shims (method-name specific)
            builder.symbol("nyash_plugin_invoke_name_getattr_i64", nyash_plugin_invoke_name_getattr_i64 as *const u8);
            builder.symbol("nyash_plugin_invoke_name_call_i64", nyash_plugin_invoke_name_call_i64 as *const u8);
            // Reserved runtime/GC symbols
            builder.symbol(r::SYM_RT_CHECKPOINT, nyash_rt_checkpoint as *const u8);
            builder.symbol(r::SYM_GC_BARRIER_WRITE, nyash_gc_barrier_write as *const u8);
        }
        let module = cranelift_jit::JITModule::new(builder);
        let ctx = cranelift_codegen::Context::new();
        let fbc = cranelift_frontend::FunctionBuilderContext::new();
        CraneliftBuilder { 
            module, ctx, fbc, 
            stats: (0,0,0,0,0),
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
        }
    }

    /// Take ownership of compiled closure if available
    pub fn take_compiled_closure(&mut self) -> Option<std::sync::Arc<dyn Fn(&[crate::jit::abi::JitValue]) -> crate::jit::abi::JitValue + Send + Sync>> {
        self.compiled_closure.take()
    }
}
