#![cfg(feature = "cranelift-jit")]
use super::{IRBuilder, ParamKind};
use cranelift_codegen::ir::InstBuilder;
use cranelift_module::Module;

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
    pub stats: (u64, u64, u64, u64, u64),
    pub object_bytes: Option<Vec<u8>>,
    // Track rough kinds of values on the stack for bridging (e.g., plugin tagged invoke)
    value_tags: Vec<ValueTag>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ValueTag {
    I64,
    F64,
    Handle,
    Unknown,
}

impl ObjectBuilder {
    pub fn new() -> Self {
        use cranelift_codegen::settings;
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
            stats: (0, 0, 0, 0, 0),
            object_bytes: None,
            value_tags: Vec::new(),
        }
    }

    fn fresh_module() -> cranelift_object::ObjectModule {
        use cranelift_codegen::settings;
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
        cranelift_object::ObjectModule::new(obj_builder)
    }

    pub fn take_object_bytes(&mut self) -> Option<Vec<u8>> {
        self.object_bytes.take()
    }

    fn entry_param(&mut self, index: usize) -> Option<cranelift_codegen::ir::Value> {
        use cranelift_frontend::FunctionBuilder;
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(b) = self.entry_block {
            fb.switch_to_block(b);
            let params = fb.func.dfg.block_params(b).to_vec();
            if let Some(v) = params.get(index).copied() {
                return Some(v);
            }
        }
        None
    }
}

impl IRBuilder for ObjectBuilder {
    fn begin_function(&mut self, name: &str) {
        use cranelift_codegen::ir::{types, AbiParam, Signature};
        use cranelift_frontend::FunctionBuilder;
        self.current_name = Some(name.to_string());
        self.value_stack.clear();
        self.value_tags.clear();
        // Reset contexts to satisfy Cranelift requirements when reusing the builder
        self.ctx = cranelift_codegen::Context::new();
        self.fbc = cranelift_frontend::FunctionBuilderContext::new();
        self.blocks.clear();
        self.entry_block = None;
        self.current_block_index = None;
        if !self.typed_sig_prepared {
            let call_conv = self.module.isa().default_call_conv();
            let mut sig = Signature::new(call_conv);
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
            self.ctx.func.signature = sig;
        }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if self.blocks.is_empty() {
            self.blocks.push(fb.create_block());
        }
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
        if let Some(b) = self.entry_block {
            fb.switch_to_block(b);
        }
        fb.finalize();
        // Export as ny_main so that nyrt can locate the entrypoint when linking AOT objects
        let obj_id = self
            .module
            .declare_function(
                "ny_main",
                cranelift_module::Linkage::Export,
                &self.ctx.func.signature,
            )
            .expect("declare func");
        self.module
            .define_function(obj_id, &mut self.ctx)
            .expect("define");
        self.module.clear_context(&mut self.ctx);
        let finished = std::mem::replace(&mut self.module, Self::fresh_module());
        let product = finished.finish();
        self.object_bytes = Some(product.emit().expect("emit object"));
        // Clear per-function state to allow reuse
        self.blocks.clear();
        self.entry_block = None;
        self.current_block_index = None;
    }
    fn prepare_signature_i64(&mut self, argc: usize, has_ret: bool) {
        self.desired_argc = argc;
        self.desired_has_ret = has_ret;
    }
    fn prepare_signature_typed(&mut self, _params: &[ParamKind], _ret_is_f64: bool) {
        self.typed_sig_prepared = true;
    }
    fn emit_param_i64(&mut self, index: usize) {
        if let Some(v) = self.entry_param(index) {
            self.value_stack.push(v);
            self.value_tags.push(ValueTag::Unknown);
        }
    }
    fn emit_const_i64(&mut self, val: i64) {
        use cranelift_codegen::ir::types;
        use cranelift_frontend::FunctionBuilder;
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index {
            fb.switch_to_block(self.blocks[idx]);
        } else if let Some(b) = self.entry_block {
            fb.switch_to_block(b);
        }
        let v = fb.ins().iconst(types::I64, val);
        self.value_stack.push(v);
        self.value_tags.push(ValueTag::I64);
        self.stats.0 += 1;
    }
    fn emit_const_f64(&mut self, val: f64) {
        use cranelift_codegen::ir::types;
        use cranelift_frontend::FunctionBuilder;
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index {
            fb.switch_to_block(self.blocks[idx]);
        } else if let Some(b) = self.entry_block {
            fb.switch_to_block(b);
        }
        let v = fb.ins().f64const(val);
        self.value_stack.push(v);
        self.value_tags.push(ValueTag::F64);
    }
    fn emit_binop(&mut self, op: super::BinOpKind) {
        use cranelift_codegen::ir::types;
        use cranelift_frontend::FunctionBuilder;
        if self.value_stack.len() < 2 {
            return;
        }
        let mut rhs = self.value_stack.pop().unwrap();
        let mut lhs = self.value_stack.pop().unwrap();
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index {
            fb.switch_to_block(self.blocks[idx]);
        } else if let Some(b) = self.entry_block {
            fb.switch_to_block(b);
        }
        // Ensure i64 operands
        if fb.func.dfg.value_type(lhs) != types::I64 {
            lhs = fb.ins().fcvt_to_sint(types::I64, lhs);
        }
        if fb.func.dfg.value_type(rhs) != types::I64 {
            rhs = fb.ins().fcvt_to_sint(types::I64, rhs);
        }
        let res = match op {
            super::BinOpKind::Add => fb.ins().iadd(lhs, rhs),
            super::BinOpKind::Sub => fb.ins().isub(lhs, rhs),
            super::BinOpKind::Mul => fb.ins().imul(lhs, rhs),
            super::BinOpKind::Div => fb.ins().sdiv(lhs, rhs),
            super::BinOpKind::Mod => fb.ins().srem(lhs, rhs),
        };
        self.value_stack.push(res);
        self.value_tags.push(ValueTag::I64);
        self.stats.1 += 1;
    }
    fn emit_compare(&mut self, op: super::CmpKind) {
        use cranelift_codegen::ir::{condcodes::IntCC, types};
        use cranelift_frontend::FunctionBuilder;
        if self.value_stack.len() < 2 {
            return;
        }
        let mut rhs = self.value_stack.pop().unwrap();
        let mut lhs = self.value_stack.pop().unwrap();
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index {
            fb.switch_to_block(self.blocks[idx]);
        } else if let Some(b) = self.entry_block {
            fb.switch_to_block(b);
        }
        // Ensure i64 operands
        if fb.func.dfg.value_type(lhs) != types::I64 {
            lhs = fb.ins().fcvt_to_sint(types::I64, lhs);
        }
        if fb.func.dfg.value_type(rhs) != types::I64 {
            rhs = fb.ins().fcvt_to_sint(types::I64, rhs);
        }
        let cc = match op {
            super::CmpKind::Eq => IntCC::Equal,
            super::CmpKind::Ne => IntCC::NotEqual,
            super::CmpKind::Lt => IntCC::SignedLessThan,
            super::CmpKind::Le => IntCC::SignedLessThanOrEqual,
            super::CmpKind::Gt => IntCC::SignedGreaterThan,
            super::CmpKind::Ge => IntCC::SignedGreaterThanOrEqual,
        };
        let b1 = fb.ins().icmp(cc, lhs, rhs);
        let one = fb.ins().iconst(types::I64, 1);
        let zero = fb.ins().iconst(types::I64, 0);
        let sel = fb.ins().select(b1, one, zero);
        self.value_stack.push(sel);
        self.value_tags.push(ValueTag::I64);
        self.stats.2 += 1;
    }
    fn emit_jump(&mut self) {
        self.stats.3 += 1;
    }
    fn emit_branch(&mut self) {
        self.stats.3 += 1;
    }
    fn emit_return(&mut self) {
        use cranelift_frontend::FunctionBuilder;
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index {
            fb.switch_to_block(self.blocks[idx]);
        } else if let Some(b) = self.entry_block {
            fb.switch_to_block(b);
        }
        if self.desired_has_ret {
            if self.desired_ret_is_f64 {
                use cranelift_codegen::ir::types;
                let v = if let Some(v) = self.value_stack.pop() {
                    v
                } else {
                    fb.ins().f64const(0.0)
                };
                // Coerce i64 to f64 if needed
                let v2 = if fb.func.dfg.value_type(v) != types::F64 {
                    fb.ins().fcvt_from_sint(types::F64, v)
                } else {
                    v
                };
                fb.ins().return_(&[v2]);
            } else {
                use cranelift_codegen::ir::types;
                let v = if let Some(v) = self.value_stack.pop() {
                    v
                } else {
                    fb.ins().iconst(types::I64, 0)
                };
                let v2 = if fb.func.dfg.value_type(v) != types::I64 {
                    fb.ins().fcvt_to_sint(types::I64, v)
                } else {
                    v
                };
                fb.ins().return_(&[v2]);
            }
        } else {
            fb.ins().return_(&[]);
        }
        self.stats.4 += 1;
    }
    fn ensure_local_i64(&mut self, index: usize) {
        use cranelift_codegen::ir::StackSlotData;
        use cranelift_frontend::FunctionBuilder;
        if self.local_slots.contains_key(&index) {
            return;
        }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        let slot = fb.create_sized_stack_slot(StackSlotData::new(
            cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
            8,
        ));
        self.local_slots.insert(index, slot);
    }
    fn store_local_i64(&mut self, index: usize) {
        use cranelift_codegen::ir::{condcodes::IntCC, types};
        use cranelift_frontend::FunctionBuilder;
        if let Some(mut v) = self.value_stack.pop() {
            let _ = self.value_tags.pop();
            if !self.local_slots.contains_key(&index) {
                self.ensure_local_i64(index);
            }
            let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
            if let Some(idx) = self.current_block_index {
                fb.switch_to_block(self.blocks[idx]);
            } else if let Some(b) = self.entry_block {
                fb.switch_to_block(b);
            }
            // Coerce to i64 if needed
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
            if let Some(&slot) = self.local_slots.get(&index) {
                fb.ins().stack_store(v, slot, 0);
            }
        }
    }
    fn load_local_i64(&mut self, index: usize) {
        use cranelift_codegen::ir::types;
        use cranelift_frontend::FunctionBuilder;
        if !self.local_slots.contains_key(&index) {
            self.ensure_local_i64(index);
        }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index {
            fb.switch_to_block(self.blocks[idx]);
        } else if let Some(b) = self.entry_block {
            fb.switch_to_block(b);
        }
        if let Some(&slot) = self.local_slots.get(&index) {
            let v = fb.ins().stack_load(types::I64, slot, 0);
            self.value_stack.push(v);
            self.value_tags.push(ValueTag::Unknown);
        }
    }
    fn prepare_blocks(&mut self, count: usize) {
        use cranelift_frontend::FunctionBuilder;
        if count == 0 {
            return;
        }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if self.blocks.len() < count {
            for _ in 0..(count - self.blocks.len()) {
                self.blocks.push(fb.create_block());
            }
        }
    }
    fn switch_to_block(&mut self, index: usize) {
        use cranelift_frontend::FunctionBuilder;
        if index >= self.blocks.len() {
            return;
        }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        fb.switch_to_block(self.blocks[index]);
        self.current_block_index = Some(index);
    }
    fn ensure_block_params_i64(&mut self, index: usize, count: usize) {
        use cranelift_codegen::ir::types;
        use cranelift_frontend::FunctionBuilder;
        if index >= self.blocks.len() {
            return;
        }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        let b = self.blocks[index];
        let has_inst = fb.func.layout.first_inst(b).is_some();
        if !has_inst {
            let current = fb.func.dfg.block_params(b).len();
            if count > current {
                for _ in current..count {
                    let _ = fb.append_block_param(b, types::I64);
                }
            }
        }
        self.block_param_counts.insert(index, count);
    }
    fn push_block_param_i64_at(&mut self, pos: usize) {
        use cranelift_codegen::ir::types;
        use cranelift_frontend::FunctionBuilder;
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        let b = if let Some(i) = self.current_block_index {
            self.blocks[i]
        } else if let Some(e) = self.entry_block {
            e
        } else {
            return;
        };
        let params = fb.func.dfg.block_params(b).to_vec();
        let v = params
            .get(pos)
            .copied()
            .unwrap_or_else(|| fb.ins().iconst(types::I64, 0));
        self.value_stack.push(v);
    }
    fn br_if_top_is_true(&mut self, then_index: usize, else_index: usize) {
        use cranelift_codegen::ir::{condcodes::IntCC, types};
        use cranelift_frontend::FunctionBuilder;
        if then_index >= self.blocks.len() || else_index >= self.blocks.len() {
            return;
        }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        let cond_val = if let Some(v) = self.value_stack.pop() {
            v
        } else {
            fb.ins().iconst(types::I64, 0)
        };
        if let Some(idx) = self.current_block_index {
            fb.switch_to_block(self.blocks[idx]);
        } else if let Some(b) = self.entry_block {
            fb.switch_to_block(b);
        }
        let b1 = if fb.func.dfg.value_type(cond_val) == types::I64 {
            fb.ins().icmp_imm(IntCC::NotEqual, cond_val, 0)
        } else {
            fb.ins().icmp_imm(IntCC::NotEqual, cond_val, 0)
        };
        fb.ins().brif(
            b1,
            self.blocks[then_index],
            &[],
            self.blocks[else_index],
            &[],
        );
        self.stats.3 += 1;
    }
    fn jump_to(&mut self, target_index: usize) {
        use cranelift_frontend::FunctionBuilder;
        if target_index >= self.blocks.len() {
            return;
        }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index {
            fb.switch_to_block(self.blocks[idx]);
        } else if let Some(b) = self.entry_block {
            fb.switch_to_block(b);
        }
        fb.ins().jump(self.blocks[target_index], &[]);
        self.stats.3 += 1;
    }
    fn emit_select_i64(&mut self) {
        use cranelift_codegen::ir::{condcodes::IntCC, types};
        use cranelift_frontend::FunctionBuilder;
        if self.value_stack.len() < 3 {
            return;
        }
        let mut else_v = self.value_stack.pop().unwrap();
        let mut then_v = self.value_stack.pop().unwrap();
        let cond_v = self.value_stack.pop().unwrap();
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index {
            fb.switch_to_block(self.blocks[idx]);
        } else if let Some(b) = self.entry_block {
            fb.switch_to_block(b);
        }
        let cond_b1 = if fb.func.dfg.value_type(cond_v) == types::I64 {
            fb.ins().icmp_imm(IntCC::NotEqual, cond_v, 0)
        } else {
            fb.ins().icmp_imm(IntCC::NotEqual, cond_v, 0)
        };
        if fb.func.dfg.value_type(then_v) != types::I64 {
            then_v = fb.ins().fcvt_to_sint(types::I64, then_v);
        }
        if fb.func.dfg.value_type(else_v) != types::I64 {
            else_v = fb.ins().fcvt_to_sint(types::I64, else_v);
        }
        let sel = fb.ins().select(cond_b1, then_v, else_v);
        self.value_stack.push(sel);
    }
    fn emit_host_call(&mut self, symbol: &str, argc: usize, has_ret: bool) {
        use cranelift_codegen::ir::{types, AbiParam, Signature};
        use cranelift_frontend::FunctionBuilder;
        // Structured lower event for import call (AOT builder)
        {
            let mut arg_types: Vec<&'static str> = Vec::new();
            for _ in 0..argc {
                arg_types.push("I64");
            }
            crate::jit::events::emit_lower(
                serde_json::json!({
                    "id": symbol,
                    "decision": "allow",
                    "reason": "import_call",
                    "argc": argc,
                    "arg_types": arg_types,
                    "ret": if has_ret { "I64" } else { "Void" }
                }),
                "hostcall",
                "<aot>",
            );
        }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index {
            fb.switch_to_block(self.blocks[idx]);
        } else if let Some(b) = self.entry_block {
            fb.switch_to_block(b);
        }
        let mut sig = Signature::new(self.module.isa().default_call_conv());
        for _ in 0..argc {
            sig.params.push(AbiParam::new(types::I64));
        }
        if has_ret {
            sig.returns.push(AbiParam::new(types::I64));
        }
        let func_id = self
            .module
            .declare_function(symbol, cranelift_module::Linkage::Import, &sig)
            .expect("declare hostcall");
        let mut args: Vec<cranelift_codegen::ir::Value> = Vec::with_capacity(argc);
        for _ in 0..argc {
            if let Some(v) = self.value_stack.pop() {
                args.push(v);
            } else {
                args.push(fb.ins().iconst(types::I64, 0));
            }
        }
        args.reverse();
        // Ensure i64 for all
        for a in args.iter_mut() {
            if fb.func.dfg.value_type(*a) != types::I64 {
                *a = fb.ins().fcvt_to_sint(types::I64, *a);
            }
        }
        let fref = self.module.declare_func_in_func(func_id, fb.func);
        let call_inst = fb.ins().call(fref, &args);
        if has_ret {
            if let Some(v) = fb.inst_results(call_inst).get(0).copied() {
                self.value_stack.push(v);
            }
        }
    }
    fn emit_host_call_typed(
        &mut self,
        symbol: &str,
        params: &[super::ParamKind],
        has_ret: bool,
        ret_is_f64: bool,
    ) {
        use cranelift_codegen::ir::{types, AbiParam, Signature};
        use cranelift_frontend::FunctionBuilder;
        // Structured lower event for typed import call (AOT builder)
        {
            let mut arg_types: Vec<&'static str> = Vec::new();
            for k in params {
                arg_types.push(match k {
                    super::ParamKind::I64 | super::ParamKind::B1 => "I64",
                    super::ParamKind::F64 => "F64",
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
                "<aot>",
            );
        }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index {
            fb.switch_to_block(self.blocks[idx]);
        } else if let Some(b) = self.entry_block {
            fb.switch_to_block(b);
        }
        let mut sig = Signature::new(self.module.isa().default_call_conv());
        for &k in params {
            match k {
                super::ParamKind::I64 => sig.params.push(AbiParam::new(types::I64)),
                super::ParamKind::F64 => sig.params.push(AbiParam::new(types::F64)),
                super::ParamKind::B1 => sig.params.push(AbiParam::new(types::I64)),
            }
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
            .expect("declare hostcall typed");
        // Gather args from stack (reverse)
        let mut args: Vec<cranelift_codegen::ir::Value> = Vec::with_capacity(params.len());
        for &k in params.iter().rev() {
            let mut v = if let Some(v) = self.value_stack.pop() {
                v
            } else {
                match k {
                    super::ParamKind::I64 | super::ParamKind::B1 => fb.ins().iconst(types::I64, 0),
                    super::ParamKind::F64 => fb.ins().f64const(0.0),
                }
            };
            // Coerce
            v = match k {
                super::ParamKind::I64 | super::ParamKind::B1 => {
                    if fb.func.dfg.value_type(v) != types::I64 {
                        fb.ins().fcvt_to_sint(types::I64, v)
                    } else {
                        v
                    }
                }
                super::ParamKind::F64 => {
                    if fb.func.dfg.value_type(v) != types::F64 {
                        fb.ins().fcvt_from_sint(types::F64, v)
                    } else {
                        v
                    }
                }
            };
            args.push(v);
        }
        args.reverse();
        let fref = self.module.declare_func_in_func(func_id, fb.func);
        let call_inst = fb.ins().call(fref, &args);
        if has_ret {
            if let Some(mut v) = fb.inst_results(call_inst).get(0).copied() {
                if ret_is_f64 && fb.func.dfg.value_type(v) != types::F64 {
                    v = fb.ins().fcvt_from_sint(types::F64, v);
                }
                if !ret_is_f64 && fb.func.dfg.value_type(v) != types::I64 {
                    v = fb.ins().fcvt_to_sint(types::I64, v);
                }
                self.value_stack.push(v);
            }
        }
    }
    fn emit_host_call_fixed3(&mut self, symbol: &str, has_ret: bool) {
        self.emit_host_call(symbol, 3, has_ret);
    }
    fn emit_string_handle_from_literal(&mut self, s: &str) {
        use cranelift_codegen::ir::{types, AbiParam, Signature};
        use cranelift_frontend::FunctionBuilder;
        // Pack up to 16 bytes of the literal into two u64 words
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
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index {
            fb.switch_to_block(self.blocks[idx]);
        } else if let Some(b) = self.entry_block {
            fb.switch_to_block(b);
        }
        // Declare import: nyash.string.from_u64x2(lo, hi, len) -> i64
        let mut sig = Signature::new(self.module.isa().default_call_conv());
        sig.params.push(AbiParam::new(types::I64));
        sig.params.push(AbiParam::new(types::I64));
        sig.params.push(AbiParam::new(types::I64));
        sig.returns.push(AbiParam::new(types::I64));
        let func_id = self
            .module
            .declare_function(
                crate::jit::r#extern::collections::SYM_STRING_FROM_U64X2,
                cranelift_module::Linkage::Import,
                &sig,
            )
            .expect("declare string.from_u64x2");
        let lo_v = fb.ins().iconst(types::I64, lo as i64);
        let hi_v = fb.ins().iconst(types::I64, hi as i64);
        let len_v = fb.ins().iconst(types::I64, bytes.len() as i64);
        let fref = self.module.declare_func_in_func(func_id, fb.func);
        let call_inst = fb.ins().call(fref, &[lo_v, hi_v, len_v]);
        if let Some(v) = fb.inst_results(call_inst).get(0).copied() {
            self.value_stack.push(v);
            self.value_tags.push(ValueTag::Handle);
        }
    }
    fn br_if_with_args(
        &mut self,
        then_index: usize,
        else_index: usize,
        then_n: usize,
        else_n: usize,
    ) {
        use cranelift_codegen::ir::{condcodes::IntCC, types};
        use cranelift_frontend::FunctionBuilder;
        if then_index >= self.blocks.len() || else_index >= self.blocks.len() {
            return;
        }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index {
            fb.switch_to_block(self.blocks[idx]);
        } else if let Some(b) = self.entry_block {
            fb.switch_to_block(b);
        }
        // Pop else args, then then args (stack topに近い方から)
        let mut else_args: Vec<cranelift_codegen::ir::Value> = Vec::new();
        for _ in 0..else_n {
            if let Some(v) = self.value_stack.pop() {
                else_args.push(v);
                let _ = self.value_tags.pop();
            } else {
                else_args.push(fb.ins().iconst(types::I64, 0));
            }
        }
        else_args.reverse();
        let mut then_args: Vec<cranelift_codegen::ir::Value> = Vec::new();
        for _ in 0..then_n {
            if let Some(v) = self.value_stack.pop() {
                then_args.push(v);
                let _ = self.value_tags.pop();
            } else {
                then_args.push(fb.ins().iconst(types::I64, 0));
            }
        }
        then_args.reverse();
        // Cond
        let cond_val = if let Some(v) = self.value_stack.pop() {
            v
        } else {
            fb.ins().iconst(types::I64, 0)
        };
        let b1 = if fb.func.dfg.value_type(cond_val) == types::I64 {
            fb.ins().icmp_imm(IntCC::NotEqual, cond_val, 0)
        } else {
            fb.ins().icmp_imm(IntCC::NotEqual, cond_val, 0)
        };
        // Coerce args to i64
        for a in then_args.iter_mut() {
            if fb.func.dfg.value_type(*a) != types::I64 {
                *a = fb.ins().fcvt_to_sint(types::I64, *a);
            }
        }
        for a in else_args.iter_mut() {
            if fb.func.dfg.value_type(*a) != types::I64 {
                *a = fb.ins().fcvt_to_sint(types::I64, *a);
            }
        }
        fb.ins().brif(
            b1,
            self.blocks[then_index],
            &then_args,
            self.blocks[else_index],
            &else_args,
        );
        self.stats.3 += 1;
    }
    fn jump_with_args(&mut self, target_index: usize, n: usize) {
        use cranelift_codegen::ir::types;
        use cranelift_frontend::FunctionBuilder;
        if target_index >= self.blocks.len() {
            return;
        }
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index {
            fb.switch_to_block(self.blocks[idx]);
        } else if let Some(b) = self.entry_block {
            fb.switch_to_block(b);
        }
        let mut args: Vec<cranelift_codegen::ir::Value> = Vec::new();
        for _ in 0..n {
            if let Some(v) = self.value_stack.pop() {
                args.push(v);
                let _ = self.value_tags.pop();
            } else {
                args.push(fb.ins().iconst(types::I64, 0));
            }
        }
        args.reverse();
        for a in args.iter_mut() {
            if fb.func.dfg.value_type(*a) != types::I64 {
                *a = fb.ins().fcvt_to_sint(types::I64, *a);
            }
        }
        fb.ins().jump(self.blocks[target_index], &args);
        self.stats.3 += 1;
    }

    fn emit_plugin_invoke(&mut self, _type_id: u32, _method_id: u32, argc: usize, has_ret: bool) {
        use cranelift_codegen::ir::{types, AbiParam, Signature};
        use cranelift_frontend::FunctionBuilder;
        // We import NyRT tagged invoke entry (by-id). Signature:
        // nyash_plugin_invoke3_tagged_i64(type_id, method_id, argc, a0, a1, tag1, a2, tag2, a3, tag3, a4, tag4) -> i64
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index {
            fb.switch_to_block(self.blocks[idx]);
        } else if let Some(b) = self.entry_block {
            fb.switch_to_block(b);
        }

        // Pop args in reverse: last pushed is top. Collect up to 4 (excluding recv)
        let mut arg_vals: Vec<cranelift_codegen::ir::Value> = Vec::new();
        let mut arg_tags: Vec<ValueTag> = Vec::new();
        for _ in 0..argc.saturating_sub(1) {
            // exclude receiver (first param)
            if let Some(v) = self.value_stack.pop() {
                arg_vals.push(v);
                arg_tags.push(self.value_tags.pop().unwrap_or(ValueTag::Unknown));
            }
        }
        // Receiver
        let recv = if let Some(v) = self.value_stack.pop() {
            let _ = self.value_tags.pop();
            v
        } else {
            fb.ins().iconst(types::I64, 0)
        };
        arg_vals.reverse();
        arg_tags.reverse();
        let mut tag_i64 = |t: ValueTag| -> i64 {
            match t {
                ValueTag::Handle => 8,
                ValueTag::F64 => 5,
                ValueTag::I64 => 3,
                ValueTag::Unknown => 3,
            }
        };

        // Build signature and declare import
        let mut sig = Signature::new(self.module.isa().default_call_conv());
        for _ in 0..12 {
            sig.params.push(AbiParam::new(types::I64));
        }
        sig.returns.push(AbiParam::new(types::I64));
        let func_id = self
            .module
            .declare_function(
                "nyash_plugin_invoke3_tagged_i64",
                cranelift_module::Linkage::Import,
                &sig,
            )
            .expect("declare plugin invoke tagged");
        let fref = self.module.declare_func_in_func(func_id, fb.func);

        // Prepare args array
        let mut args: Vec<cranelift_codegen::ir::Value> = Vec::with_capacity(12);
        let to_i64 = |fb: &mut FunctionBuilder, v: cranelift_codegen::ir::Value| {
            if fb.func.dfg.value_type(v) != types::I64 {
                fb.ins().fcvt_to_sint(types::I64, v)
            } else {
                v
            }
        };

        let t_i64 = |_fb: &mut FunctionBuilder, x: i64| -> cranelift_codegen::ir::Value {
            _fb.ins().iconst(types::I64, x)
        };

        // Pass through type_id/method_id from lowering (method_id must match plugin vtable)
        args.push(t_i64(&mut fb, _type_id as i64)); // type_id (runtime may override with real_type_id)
        args.push(t_i64(&mut fb, _method_id as i64)); // method_id
        args.push(t_i64(&mut fb, argc as i64 - 1)); // argc excluding recv
        args.push(to_i64(&mut fb, recv)); // a0 (recv)

        // a1/tag1, a2/tag2, a3/tag3, a4/tag4
        for i in 0..4 {
            if let Some(v) = arg_vals.get(i).copied() {
                args.push(to_i64(&mut fb, v));
                let tg = tag_i64(*arg_tags.get(i).unwrap_or(&ValueTag::Unknown));
                args.push(t_i64(&mut fb, tg));
            } else {
                args.push(t_i64(&mut fb, 0));
                args.push(t_i64(&mut fb, 3));
            }
        }

        let call_inst = fb.ins().call(fref, &args);
        if has_ret {
            if let Some(v) = fb.inst_results(call_inst).get(0).copied() {
                self.value_stack.push(v);
                self.value_tags.push(ValueTag::I64);
            }
        }
    }

    fn emit_plugin_invoke_by_name(&mut self, _method: &str, argc: usize, has_ret: bool) {
        use cranelift_codegen::ir::{types, AbiParam, Signature};
        use cranelift_frontend::FunctionBuilder;
        // Use nyash.plugin.invoke_by_name_i64(recv_h, method_cstr, argc, a1, a2)
        // Limit: supports up to 2 args beyond receiver.
        let mut fb = FunctionBuilder::new(&mut self.ctx.func, &mut self.fbc);
        if let Some(idx) = self.current_block_index {
            fb.switch_to_block(self.blocks[idx]);
        } else if let Some(b) = self.entry_block {
            fb.switch_to_block(b);
        }

        // Pop args and recv
        let mut arg_vals: Vec<cranelift_codegen::ir::Value> = Vec::new();
        for _ in 0..argc.saturating_sub(1) {
            if let Some(v) = self.value_stack.pop() {
                arg_vals.push(v);
                let _ = self.value_tags.pop();
            }
        }
        let recv = if let Some(v) = self.value_stack.pop() {
            let _ = self.value_tags.pop();
            v
        } else {
            fb.ins().iconst(types::I64, 0)
        };
        arg_vals.reverse();

        let mut sig = Signature::new(self.module.isa().default_call_conv());
        for _ in 0..5 {
            sig.params.push(AbiParam::new(types::I64));
        }
        sig.returns.push(AbiParam::new(types::I64));
        let func_id = self
            .module
            .declare_function(
                "nyash.plugin.invoke_by_name_i64",
                cranelift_module::Linkage::Import,
                &sig,
            )
            .expect("declare plugin invoke by-name");
        let fref = self.module.declare_func_in_func(func_id, fb.func);

        let to_i64 = |fb: &mut FunctionBuilder, v: cranelift_codegen::ir::Value| {
            if fb.func.dfg.value_type(v) != types::I64 {
                fb.ins().fcvt_to_sint(types::I64, v)
            } else {
                v
            }
        };
        let zero = fb.ins().iconst(types::I64, 0);
        let mut args: Vec<cranelift_codegen::ir::Value> = Vec::with_capacity(5);
        args.push(to_i64(&mut fb, recv));
        // method ptr not supported in object builder (no easy CStr symbol payload); pass 0 to let runtime reject if mistakenly used.
        args.push(zero);
        args.push(fb.ins().iconst(types::I64, (argc as i64).saturating_sub(1)));
        args.push(
            arg_vals
                .get(0)
                .copied()
                .map(|v| to_i64(&mut fb, v))
                .unwrap_or(zero),
        );
        args.push(
            arg_vals
                .get(1)
                .copied()
                .map(|v| to_i64(&mut fb, v))
                .unwrap_or(zero),
        );

        let call_inst = fb.ins().call(fref, &args);
        if has_ret {
            if let Some(v) = fb.inst_results(call_inst).get(0).copied() {
                self.value_stack.push(v);
                self.value_tags.push(ValueTag::I64);
            }
        }
    }
}
