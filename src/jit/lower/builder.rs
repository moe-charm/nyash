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
    /// Optional: prepare a simple `i64` ABI signature with `argc` params
    fn prepare_signature_i64(&mut self, _argc: usize, _has_ret: bool) { }
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
    /// Optional: ensure target block has one i64 block param (for minimal PHI)
    fn ensure_block_param_i64(&mut self, _index: usize) { }
    /// Optional: push current block's first param (i64) onto the value stack
    fn push_block_param_i64(&mut self) { }
    /// Optional: conditional branch with explicit arg counts for then/else; pops args from stack
    fn br_if_with_args(&mut self, _then_index: usize, _else_index: usize, _then_n: usize, _else_n: usize) {
        // fallback to no-arg br_if
        self.br_if_top_is_true(_then_index, _else_index);
    }
    /// Optional: jump with explicit arg count; pops args from stack
    fn jump_with_args(&mut self, _target_index: usize, _n: usize) { self.jump_to(_target_index); }
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
    // Finalized function pointer (if any)
    compiled_closure: Option<std::sync::Arc<dyn Fn(&[crate::backend::vm::VMValue]) -> crate::backend::vm::VMValue + Send + Sync>>, 
    // Desired simple ABI (Phase 10_c minimal): i64 params count and i64 return
    desired_argc: usize,
    desired_has_ret: bool,
}

#[cfg(feature = "cranelift-jit")]
use cranelift_module::Module;
#[cfg(feature = "cranelift-jit")]
use cranelift_codegen::ir::InstBuilder;

#[cfg(feature = "cranelift-jit")]
extern "C" fn nyash_host_stub0() -> i64 { 0 }
#[cfg(feature = "cranelift-jit")]
extern "C" fn nyash_array_len(arr_param_index: i64) -> i64 {
    // Interpret first arg as function param index and fetch from thread-local args
    if arr_param_index < 0 { return 0; }
    crate::jit::rt::with_args(|args| {
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
    crate::jit::rt::with_args(|args| {
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
    crate::jit::rt::with_args(|args| {
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
    crate::jit::rt::with_args(|args| {
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
    crate::jit::rt::with_args(|args| {
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

#[cfg(feature = "cranelift-jit")]
impl IRBuilder for CraneliftBuilder {
    fn emit_param_i64(&mut self, index: usize) {
        if let Some(v) = self.entry_param(index) {
            self.value_stack.push(v);
        }
    }
    fn prepare_signature_i64(&mut self, argc: usize, has_ret: bool) {
        self.desired_argc = argc;
        self.desired_has_ret = has_ret;
    }
    fn begin_function(&mut self, name: &str) {
        use cranelift_codegen::ir::{AbiParam, Signature, types};
        use cranelift_frontend::FunctionBuilder;

        self.current_name = Some(name.to_string());
        self.value_stack.clear();
        // Keep any pre-created blocks (from prepare_blocks)

        // Minimal signature: (i64 x argc) -> i64?  (Core-1 integer path)
        let call_conv = self.module.isa().default_call_conv();
        let mut sig = Signature::new(call_conv);
        for _ in 0..self.desired_argc { sig.params.push(AbiParam::new(types::I64)); }
        if self.desired_has_ret { sig.returns.push(AbiParam::new(types::I64)); }
        self.ctx.func.signature = sig;
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
        self.module.finalize_definitions();

        // Get finalized code pointer and wrap into a safe closure
        let code = self.module.get_finalized_function(func_id);

        // SAFETY: We compiled a function with simple i64 ABI; we still call without args for now
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
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
        else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
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
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
        else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
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
        use cranelift_codegen::ir::{condcodes::IntCC};
        use cranelift_frontend::FunctionBuilder;
        if self.value_stack.len() < 2 { return; }
        let rhs = self.value_stack.pop().unwrap();
        let lhs = self.value_stack.pop().unwrap();
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index { fb.switch_to_block(self.blocks[idx]); }
        else if let Some(b) = self.entry_block { fb.switch_to_block(b); }
        let cc = match op {
            CmpKind::Eq => IntCC::Equal,
            CmpKind::Ne => IntCC::NotEqual,
            CmpKind::Lt => IntCC::SignedLessThan,
            CmpKind::Le => IntCC::SignedLessThanOrEqual,
            CmpKind::Gt => IntCC::SignedGreaterThan,
            CmpKind::Ge => IntCC::SignedGreaterThanOrEqual,
        };
        let b1 = fb.ins().icmp(cc, lhs, rhs);
        // Keep b1 on the stack; users (branch) can consume directly, arithmetic should not assume compare value as i64
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
                fb.ins().icmp_imm(IntCC::NotEqual, v, 0)
            } else {
                // assume already b1
                v
            }
        } else {
            let zero = fb.ins().iconst(types::I64, 0);
            fb.ins().icmp_imm(IntCC::NotEqual, zero, 0)
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
        use cranelift_codegen::ir::types;
        use cranelift_frontend::FunctionBuilder;
        if index >= self.blocks.len() { return; }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        let count = self.block_param_counts.get(&index).copied().unwrap_or(0);
        if count == 0 {
            let b = self.blocks[index];
            let _v = fb.append_block_param(b, types::I64);
            self.block_param_counts.insert(index, 1);
        }
        fb.finalize();
    }
    fn push_block_param_i64(&mut self) {
        use cranelift_frontend::FunctionBuilder;
        use cranelift_codegen::ir::types;
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        // Determine current block
        let b = if let Some(idx) = self.current_block_index { self.blocks[idx] } else if let Some(b) = self.entry_block { b } else { fb.create_block() };
        // Fetch first param if exists
        let params = fb.func.dfg.block_params(b).to_vec();
        if let Some(v) = params.get(0).copied() { self.value_stack.push(v); }
        else {
            // defensive: push 0
            let zero = fb.ins().iconst(types::I64, 0);
            self.value_stack.push(zero);
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
            if ty == types::I64 { fb.ins().icmp_imm(IntCC::NotEqual, v, 0) } else { v }
        } else {
            let zero = fb.ins().iconst(types::I64, 0);
            fb.ins().icmp_imm(IntCC::NotEqual, zero, 0)
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
            compiled_closure: None,
            desired_argc: 0,
            desired_has_ret: true,
        }
    }

    /// Take ownership of compiled closure if available
    pub fn take_compiled_closure(&mut self) -> Option<std::sync::Arc<dyn Fn(&[crate::backend::vm::VMValue]) -> crate::backend::vm::VMValue + Send + Sync>> {
        self.compiled_closure.take()
    }
}
