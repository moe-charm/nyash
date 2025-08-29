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
    /// Phase 10_d scaffolding: host-call emission (symbolic)
    fn emit_host_call(&mut self, _symbol: &str, _argc: usize, _has_ret: bool) { }
    /// Typed host-call emission: params kinds and return type hint (f64 when true)
    fn emit_host_call_typed(&mut self, _symbol: &str, _params: &[ParamKind], _has_ret: bool, _ret_is_f64: bool) { }
    /// Phase 10.2: plugin invoke emission (symbolic; type_id/method_id based)
    fn emit_plugin_invoke(&mut self, _type_id: u32, _method_id: u32, _argc: usize, _has_ret: bool) { }
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
    fn emit_host_call_typed(&mut self, _symbol: &str, _params: &[ParamKind], has_ret: bool, _ret_is_f64: bool) { if has_ret { self.consts += 1; } }
    fn emit_plugin_invoke(&mut self, _type_id: u32, _method_id: u32, _argc: usize, has_ret: bool) { if has_ret { self.consts += 1; } }
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
}

#[cfg(feature = "cranelift-jit")]
use cranelift_module::Module;
#[cfg(feature = "cranelift-jit")]
use cranelift_codegen::ir::InstBuilder;

#[cfg(feature = "cranelift-jit")]
extern "C" fn nyash_host_stub0() -> i64 { 0 }
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
#[cfg(feature = "cranelift-jit")]
use super::extern_thunks::{
    nyash_math_sin_f64, nyash_math_cos_f64, nyash_math_abs_f64, nyash_math_min_f64, nyash_math_max_f64,
    nyash_array_len_h, nyash_array_get_h, nyash_array_set_h, nyash_array_push_h, 
    nyash_array_last_h, nyash_map_size_h, nyash_map_get_h, nyash_map_get_hh,
    nyash_map_set_h, nyash_map_has_h,
    nyash_string_charcode_at_h, nyash_string_birth_h, nyash_integer_birth_h,
    nyash_any_length_h, nyash_any_is_empty_h,
};

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
        use cranelift_frontend::FunctionBuilder;

        self.current_name = Some(name.to_string());
        self.value_stack.clear();
        // Keep any pre-created blocks (from prepare_blocks or typed signature)

        // Build default signature only if a typed one wasn't prepared
        if !self.typed_sig_prepared {
            // Minimal signature: (i64 x argc) -> i64?  (Core-1 integer path)
            let call_conv = self.module.isa().default_call_conv();
            let mut sig = Signature::new(call_conv);
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
            self.ctx.func.signature = sig;
        }
        self.ctx.func.name = cranelift_codegen::ir::UserFuncName::user(0, 0);

        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        // Prepare entry block: use pre-created block[0] if present, otherwise create
        if self.blocks.is_empty() {
            let block = fb.create_block();
            self.blocks.push(block);
        }
        let entry = self.blocks[0];
        fb.append_block_params_for_function_params(entry);
        fb.switch_to_block(entry);
        // Entry block can be sealed immediately
        fb.seal_block(entry);
        self.entry_block = Some(entry);
        self.current_block_index = Some(0);
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
        let _ = self.module.finalize_definitions();

        // Get finalized code pointer and wrap into a safe closure
        let code = self.module.get_finalized_function(func_id);

        // SAFETY: We compiled a function with simple (i64 x N) -> i64/f64 というABIだよ。
        // ランタイムでは JitValue から i64 へ正規化して、引数個数に応じた関数型にtransmuteして呼び出すにゃ。
        let argc = self.desired_argc;
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
                let ret_i64 = match argc {
                    0 => {
                        let f: extern "C" fn() -> i64 = std::mem::transmute(code_usize);
                        f()
                    }
                    1 => {
                        let f: extern "C" fn(i64) -> i64 = std::mem::transmute(code_usize);
                        f(a[0])
                    }
                    2 => {
                        let f: extern "C" fn(i64, i64) -> i64 = std::mem::transmute(code_usize);
                        f(a[0], a[1])
                    }
                    3 => {
                        let f: extern "C" fn(i64, i64, i64) -> i64 = std::mem::transmute(code_usize);
                        f(a[0], a[1], a[2])
                    }
                    4 => {
                        let f: extern "C" fn(i64, i64, i64, i64) -> i64 = std::mem::transmute(code_usize);
                        f(a[0], a[1], a[2], a[3])
                    }
                    5 => {
                        let f: extern "C" fn(i64, i64, i64, i64, i64) -> i64 = std::mem::transmute(code_usize);
                        f(a[0], a[1], a[2], a[3], a[4])
                    }
                    _ => {
                        // 上限6（十分なPoC）
                        let f: extern "C" fn(i64, i64, i64, i64, i64, i64) -> i64 = std::mem::transmute(code_usize);
                        f(a[0], a[1], a[2], a[3], a[4], a[5])
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
        // Reset typed signature flag for next function
        self.typed_sig_prepared = false;
    }

    fn emit_const_i64(&mut self, val: i64) {
        use cranelift_codegen::ir::types;
        use cranelift_frontend::FunctionBuilder;
        // Recreate FunctionBuilder each emit (lightweight wrapper around ctx+fbc)
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
        else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        let v = fb.ins().iconst(types::I64, val);
        self.value_stack.push(v);
        self.stats.0 += 1;
        fb.finalize();
    }

    fn emit_const_f64(&mut self, val: f64) {
        self.stats.0 += 1;
        if !crate::jit::config::current().native_f64 { return; }
        use cranelift_codegen::ir::types;
        use cranelift_frontend::FunctionBuilder;
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
        else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        let v = fb.ins().f64const(val);
        self.value_stack.push(v);
        fb.finalize();
    }

    fn emit_binop(&mut self, op: BinOpKind) {
        use cranelift_frontend::FunctionBuilder;
        use cranelift_codegen::ir::types;
        if self.value_stack.len() < 2 { return; }
        let mut rhs = self.value_stack.pop().unwrap();
        let mut lhs = self.value_stack.pop().unwrap();
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
        else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        // Choose op by operand type (I64 vs F64). If mixed and native_f64, promote to F64.
        let lty = fb.func.dfg.value_type(lhs);
        let rty = fb.func.dfg.value_type(rhs);
        let native_f64 = crate::jit::config::current().native_f64;
        let mut use_f64 = native_f64 && (lty == types::F64 || rty == types::F64);
        if use_f64 {
            if lty != types::F64 { lhs = fb.ins().fcvt_from_sint(types::F64, lhs); }
            if rty != types::F64 { rhs = fb.ins().fcvt_from_sint(types::F64, rhs); }
        }
        let res = if use_f64 {
            match op {
                BinOpKind::Add => fb.ins().fadd(lhs, rhs),
                BinOpKind::Sub => fb.ins().fsub(lhs, rhs),
                BinOpKind::Mul => fb.ins().fmul(lhs, rhs),
                BinOpKind::Div => fb.ins().fdiv(lhs, rhs),
                BinOpKind::Mod => {
                    // Minimal path: produce 0.0 (fmod未実装)。将来はホストコール/Libcallに切替
                    fb.ins().f64const(0.0)
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
        };
        self.value_stack.push(res);
        self.stats.1 += 1;
        fb.finalize();
    }

    fn emit_compare(&mut self, op: CmpKind) {
        use cranelift_codegen::ir::{condcodes::{IntCC, FloatCC}, types};
        use cranelift_frontend::FunctionBuilder;
        if self.value_stack.len() < 2 { return; }
        let mut rhs = self.value_stack.pop().unwrap();
        let mut lhs = self.value_stack.pop().unwrap();
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
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
        // Keep b1 on the stack; users (branch) can consume directly
        self.value_stack.push(b1);
        self.stats.2 += 1;
        fb.finalize();
    }
    fn emit_jump(&mut self) { self.stats.3 += 1; }
    fn emit_branch(&mut self) { self.stats.3 += 1; }

    fn emit_return(&mut self) {
        use cranelift_frontend::FunctionBuilder;
        self.stats.4 += 1;
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
        else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        if let Some(mut v) = self.value_stack.pop() {
            // Normalize return type if needed
            let ret_ty = fb.func.signature.returns.get(0).map(|p| p.value_type).unwrap_or(cranelift_codegen::ir::types::I64);
            let v_ty = fb.func.dfg.value_type(v);
            if v_ty != ret_ty {
                use cranelift_codegen::ir::types;
                if ret_ty == types::F64 && v_ty == types::I64 {
                    v = fb.ins().fcvt_from_sint(types::F64, v);
                } else if ret_ty == types::I64 && v_ty == types::F64 {
                    v = fb.ins().fcvt_to_sint(types::I64, v);
                } else if ret_ty == types::I64 {
                    // If returning i64 but we currently have a boolean, normalize via select(b1,1,0)
                    use cranelift_codegen::ir::types;
                    let one = fb.ins().iconst(types::I64, 1);
                    let zero = fb.ins().iconst(types::I64, 0);
                    v = fb.ins().select(v, one, zero);
                }
                #[cfg(feature = "jit-b1-abi")]
                {
                    use cranelift_codegen::ir::types;
                    if ret_ty == types::B1 && v_ty == types::I64 {
                        use cranelift_codegen::ir::condcodes::IntCC;
                        v = fb.ins().icmp_imm(IntCC::NotEqual, v, 0);
                    }
                }
            }
            fb.ins().return_(&[v]);
        } else {
            // Return 0 if empty stack (defensive)
            use cranelift_codegen::ir::types;
            let ret_ty = fb.func.signature.returns.get(0).map(|p| p.value_type).unwrap_or(types::I64);
            if ret_ty == types::F64 {
                let z = fb.ins().f64const(0.0);
                fb.ins().return_(&[z]);
            } else {
                let zero = fb.ins().iconst(types::I64, 0);
                fb.ins().return_(&[zero]);
            }
        }
        fb.finalize();
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

        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
        else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        let fref = self.module.declare_func_in_func(func_id, fb.func);
        let call_inst = fb.ins().call(fref, &args);
        if has_ret {
            let results = fb.inst_results(call_inst).to_vec();
            if let Some(v) = results.get(0).copied() {
                self.value_stack.push(v);
            }
        }
        fb.finalize();
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

        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
        else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        let fref = self.module.declare_func_in_func(func_id, fb.func);
        let call_inst = fb.ins().call(fref, &args);
        if has_ret {
            let results = fb.inst_results(call_inst).to_vec();
            if let Some(v) = results.get(0).copied() { self.value_stack.push(v); }
        }
        fb.finalize();
    }

    fn emit_plugin_invoke(&mut self, type_id: u32, method_id: u32, argc: usize, has_ret: bool) {
        use cranelift_codegen::ir::{AbiParam, Signature, types};
        use cranelift_frontend::FunctionBuilder;
        use cranelift_module::{Linkage, Module};

        // Use a single FunctionBuilder to construct all IR in this method to avoid dominance issues
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
        else if let Some(b) = self.entry_block { fb.switch_to_block(b); }

        // Pop argc values (right-to-left): receiver + up to 2 args
        let mut arg_vals: Vec<cranelift_codegen::ir::Value> = Vec::new();
        let take_n = argc.min(self.value_stack.len());
        for _ in 0..take_n { if let Some(v) = self.value_stack.pop() { arg_vals.push(v); } }
        arg_vals.reverse();
        // Pad to 3 values (receiver + a1 + a2) using the same builder
        while arg_vals.len() < 3 {
            let z = fb.ins().iconst(types::I64, 0);
            arg_vals.push(z);
        }

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
        let fref = self.module.declare_func_in_func(func_id, fb.func);

        let c_type = fb.ins().iconst(types::I64, type_id as i64);
        let c_meth = fb.ins().iconst(types::I64, method_id as i64);
        let c_argc = fb.ins().iconst(types::I64, argc as i64);
        // Pass receiver param index (a0) when known; shim will fallback-scan if invalid (<0)
        let call_inst = fb.ins().call(fref, &[c_type, c_meth, c_argc, arg_vals[0], arg_vals[1], arg_vals[2]]);
        if has_ret {
            let results = fb.inst_results(call_inst).to_vec();
            if let Some(v) = results.get(0).copied() { self.value_stack.push(v); }
        }
        fb.finalize();
    }

    // ==== Phase 10.7 block APIs ====
    fn prepare_blocks(&mut self, count: usize) {
        use cranelift_frontend::FunctionBuilder;
        if count == 0 { return; }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        // Only create if not already created
        if self.blocks.len() < count {
            let to_create = count - self.blocks.len();
            for _ in 0..to_create { self.blocks.push(fb.create_block()); }
        }
        fb.finalize();
    }
    fn switch_to_block(&mut self, index: usize) {
        use cranelift_frontend::FunctionBuilder;
        if index >= self.blocks.len() { return; }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        fb.switch_to_block(self.blocks[index]);
        self.current_block_index = Some(index);
        fb.finalize();
    }
    fn seal_block(&mut self, index: usize) {
        use cranelift_frontend::FunctionBuilder;
        if index >= self.blocks.len() { return; }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        fb.seal_block(self.blocks[index]);
        fb.finalize();
    }
    fn br_if_top_is_true(&mut self, then_index: usize, else_index: usize) {
        use cranelift_codegen::ir::{types, condcodes::IntCC};
        use cranelift_frontend::FunctionBuilder;
        if then_index >= self.blocks.len() || else_index >= self.blocks.len() { return; }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        // Ensure we are in a block
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
        else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        // Take top-of-stack as cond; if it's i64, normalize to b1 via icmp_imm != 0
        let cond_b1 = if let Some(v) = self.value_stack.pop() {
            let ty = fb.func.dfg.value_type(v);
            if ty == types::I64 {
                let out = fb.ins().icmp_imm(IntCC::NotEqual, v, 0);
                crate::jit::rt::b1_norm_inc(1);
                out
            } else {
                // assume already b1
                v
            }
        } else {
            let zero = fb.ins().iconst(types::I64, 0);
            let out = fb.ins().icmp_imm(IntCC::NotEqual, zero, 0);
            crate::jit::rt::b1_norm_inc(1);
            out
        };
        fb.ins().brif(cond_b1, self.blocks[then_index], &[], self.blocks[else_index], &[]);
        self.stats.3 += 1;
        fb.finalize();
    }
    fn jump_to(&mut self, target_index: usize) {
        use cranelift_frontend::FunctionBuilder;
        if target_index >= self.blocks.len() { return; }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
        else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        fb.ins().jump(self.blocks[target_index], &[]);
        self.stats.3 += 1;
        fb.finalize();
    }
    fn ensure_block_param_i64(&mut self, index: usize) {
        self.ensure_block_params_i64(index, 1);
    }
    fn ensure_block_params_i64(&mut self, index: usize, needed: usize) {
        use cranelift_codegen::ir::types;
        use cranelift_frontend::FunctionBuilder;
        if index >= self.blocks.len() { return; }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        let have = self.block_param_counts.get(&index).copied().unwrap_or(0);
        if needed > have {
            let b = self.blocks[index];
            for _ in have..needed {
                let _v = fb.append_block_param(b, types::I64);
            }
            self.block_param_counts.insert(index, needed);
        }
        fb.finalize();
    }
    fn ensure_block_params_b1(&mut self, index: usize, needed: usize) {
        // Store as i64 block params for ABI stability; consumers can convert to b1
        self.ensure_block_params_i64(index, needed);
    }
    fn push_block_param_i64_at(&mut self, pos: usize) {
        use cranelift_frontend::FunctionBuilder;
        use cranelift_codegen::ir::types;
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        let b = if let Some(idx) = self.current_block_index { self.blocks[idx] } else if let Some(b) = self.entry_block { b } else { fb.create_block() };
        // Ensure we have an active insertion point before emitting any instructions
        fb.switch_to_block(b);
        let params = fb.func.dfg.block_params(b).to_vec();
        if let Some(v) = params.get(pos).copied() { self.value_stack.push(v); }
        else {
            // defensive fallback
            let zero = fb.ins().iconst(types::I64, 0);
            self.value_stack.push(zero);
        }
        fb.finalize();
    }
    fn push_block_param_b1_at(&mut self, pos: usize) {
        use cranelift_frontend::FunctionBuilder;
        use cranelift_codegen::ir::{types, condcodes::IntCC};
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        let b = if let Some(idx) = self.current_block_index { self.blocks[idx] } else if let Some(b) = self.entry_block { b } else { fb.create_block() };
        let params = fb.func.dfg.block_params(b).to_vec();
        if let Some(v) = params.get(pos).copied() {
            let ty = fb.func.dfg.value_type(v);
            let b1 = if ty == types::I64 { fb.ins().icmp_imm(IntCC::NotEqual, v, 0) } else { v };
            self.value_stack.push(b1);
        } else {
            let zero = fb.ins().iconst(types::I64, 0);
            let b1 = fb.ins().icmp_imm(IntCC::NotEqual, zero, 0);
            self.value_stack.push(b1);
        }
        fb.finalize();
    }
    fn br_if_with_args(&mut self, then_index: usize, else_index: usize, then_n: usize, else_n: usize) {
        use cranelift_codegen::ir::{types, condcodes::IntCC};
        use cranelift_frontend::FunctionBuilder;
        if then_index >= self.blocks.len() || else_index >= self.blocks.len() { return; }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
        else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        // Condition
        let cond_b1 = if let Some(v) = self.value_stack.pop() {
            let ty = fb.func.dfg.value_type(v);
            if ty == types::I64 { let out = fb.ins().icmp_imm(IntCC::NotEqual, v, 0); crate::jit::rt::b1_norm_inc(1); out } else { v }
        } else {
            let zero = fb.ins().iconst(types::I64, 0);
            let out = fb.ins().icmp_imm(IntCC::NotEqual, zero, 0);
            crate::jit::rt::b1_norm_inc(1);
            out
        };
        // Pop else args then then args (so stack order can be value-friendly)
        let mut else_args: Vec<cranelift_codegen::ir::Value> = Vec::new();
        for _ in 0..else_n { if let Some(v) = self.value_stack.pop() { else_args.push(v); } }
        else_args.reverse();
        let mut then_args: Vec<cranelift_codegen::ir::Value> = Vec::new();
        for _ in 0..then_n { if let Some(v) = self.value_stack.pop() { then_args.push(v); } }
        then_args.reverse();
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
        fb.seal_block(entry);
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
        use cranelift_frontend::FunctionBuilder; use cranelift_codegen::ir::types;
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); } else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        if let Some(mut v) = self.value_stack.pop() {
            let ret_ty = fb.func.signature.returns.get(0).map(|p| p.value_type).unwrap_or(types::I64);
            let v_ty = fb.func.dfg.value_type(v);
            if v_ty != ret_ty {
                if ret_ty == types::F64 && v_ty == types::I64 { v = fb.ins().fcvt_from_sint(types::F64, v); }
                else if ret_ty == types::I64 && v_ty == types::F64 { v = fb.ins().fcvt_to_sint(types::I64, v); }
                else if ret_ty == types::I64 { let one = fb.ins().iconst(types::I64, 1); let zero = fb.ins().iconst(types::I64, 0); use cranelift_codegen::ir::condcodes::IntCC; let b1 = fb.ins().icmp_imm(IntCC::NotEqual, v, 0); v = fb.ins().select(b1, one, zero); }
            }
            fb.ins().return_(&[v]);
        } else {
            let z = fb.ins().iconst(types::I64, 0); fb.ins().return_(&[z]);
        }
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
    fn switch_to_block(&mut self, index: usize) { use cranelift_frontend::FunctionBuilder; if index >= self.blocks.len() { return; } let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc); fb.switch_to_block(self.blocks[index]); self.current_block_index = Some(index); fb.finalize(); }
    fn seal_block(&mut self, index: usize) { use cranelift_frontend::FunctionBuilder; if index >= self.blocks.len() { return; } let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc); fb.seal_block(self.blocks[index]); fb.finalize(); }
    fn br_if_top_is_true(&mut self, _: usize, _: usize) { /* control-flow omitted for AOT PoC */ }
    fn ensure_block_params_i64(&mut self, _index: usize, _count: usize) { /* PHI omitted for AOT PoC */ }
    fn push_block_param_i64_at(&mut self, _pos: usize) { /* omitted */ }
    fn hint_ret_bool(&mut self, is_b1: bool) { self.ret_hint_is_b1 = is_b1; }
    fn ensure_local_i64(&mut self, index: usize) { use cranelift_codegen::ir::{StackSlotData, StackSlotKind}; use cranelift_frontend::FunctionBuilder; if self.local_slots.contains_key(&index) { return; } let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc); let slot = fb.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8)); self.local_slots.insert(index, slot); fb.finalize(); }
    fn store_local_i64(&mut self, index: usize) { use cranelift_codegen::ir::{types, condcodes::IntCC}; use cranelift_frontend::FunctionBuilder; if let Some(mut v) = self.value_stack.pop() { if !self.local_slots.contains_key(&index) { self.ensure_local_i64(index); } let slot = self.local_slots.get(&index).copied(); let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc); if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); } else if let Some(b) = self.entry_block { fb.switch_to_block(b); } let ty = fb.func.dfg.value_type(v); if ty != types::I64 { if ty == types::F64 { v = fb.ins().fcvt_to_sint(types::I64, v); } else { let one = fb.ins().iconst(types::I64, 1); let zero = fb.ins().iconst(types::I64, 0); let b1 = fb.ins().icmp_imm(IntCC::NotEqual, v, 0); v = fb.ins().select(b1, one, zero); } } if let Some(slot) = slot { fb.ins().stack_store(v, slot, 0); } fb.finalize(); } }
    fn load_local_i64(&mut self, index: usize) { use cranelift_codegen::ir::types; use cranelift_frontend::FunctionBuilder; if !self.local_slots.contains_key(&index) { self.ensure_local_i64(index); } if let Some(&slot) = self.local_slots.get(&index) { let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc); if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); } else if let Some(b) = self.entry_block { fb.switch_to_block(b); } let v = fb.ins().stack_load(types::I64, slot, 0); self.value_stack.push(v); self.stats.0 += 1; fb.finalize(); } }
}

// removed duplicate impl IRBuilder for CraneliftBuilder (emit_param_i64 moved into main impl)

#[cfg(feature = "cranelift-jit")]
impl CraneliftBuilder {
    pub fn new() -> Self {
        // Initialize a minimal JITModule to validate linking; not used yet
        let mut builder = cranelift_jit::JITBuilder::new(cranelift_module::default_libcall_names())
            .expect("failed to create JITBuilder");
        // Register host-call symbols (PoC: map to simple C-ABI stubs)
        builder.symbol("nyash.host.stub0", nyash_host_stub0 as *const u8);
        {
            use crate::jit::r#extern::collections as c;
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
            // Plugin invoke shims (i64/f64)
            builder.symbol("nyash_plugin_invoke3_i64", nyash_plugin_invoke3_i64 as *const u8);
            builder.symbol("nyash_plugin_invoke3_f64", nyash_plugin_invoke3_f64 as *const u8);
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
        }
    }

    /// Take ownership of compiled closure if available
    pub fn take_compiled_closure(&mut self) -> Option<std::sync::Arc<dyn Fn(&[crate::jit::abi::JitValue]) -> crate::jit::abi::JitValue + Send + Sync>> {
        self.compiled_closure.take()
    }
}
