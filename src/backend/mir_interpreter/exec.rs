use super::*;
use crate::mir::basic_block::BasicBlock;
use std::mem;

impl MirInterpreter {
    fn trace_enabled() -> bool {
        let cfg = super::VmConfig::global();
        cfg.general_trace || cfg.trace_exec
    }

    pub(super) fn exec_function_inner(
        &mut self,
        func: &MirFunction,
        arg_vals: Option<&[VMValue]>,
    ) -> Result<VMValue, VMError> {
        self.exec_function_inner_with_mode(func, arg_vals, super::CallMode::Normal)
    }

    pub(super) fn exec_function_inner_with_mode(
        &mut self,
        func: &MirFunction,
        arg_vals: Option<&[VMValue]>,
        mode: super::CallMode,
    ) -> Result<VMValue, VMError> {
        if Self::trace_enabled() { eprintln!("[DEBUG-EXEC-ENTRY] About to call function: {}", func.signature.name); }
        // Centralized OperatorBox guard: intercept before any frame setup (skip in NoOperatorGuard mode)
        if mode == super::CallMode::Normal {
            if let Some(r) = self.operator_guard_intercept_entry(func.signature.name.as_str(), arg_vals) {
                if Self::trace_enabled() { eprintln!("[DEBUG-EXEC-ENTRY] Early return from operator_guard"); }
                return r;
            }
        }

        // Phase 1: delegate cross-class reroute / narrow fallbacks to method_router (skip in NoOperatorGuard mode)
        if mode == super::CallMode::Normal {
            if let Some(r) = super::method_router::pre_exec_reroute(self, func, arg_vals) {
                if Self::trace_enabled() { eprintln!("[DEBUG-EXEC-ENTRY] Early return from pre_exec_reroute"); }
                return r;
            }
        }
        let saved_regs = mem::take(&mut self.regs);
        let saved_fn = self.cur_fn.clone();
        self.cur_fn = Some(func.signature.name.clone());
        if Self::trace_enabled() { eprintln!("[DEBUG-EXEC] Entering function: {:?}, prev_fn: {:?}", self.cur_fn, saved_fn); }

        // Enter a new scope for this function frame (lifetime management)
        self.scope.push_scope();

        if let Some(args) = arg_vals {
            for (i, pid) in func.params.iter().enumerate() {
                let v = args.get(i).cloned().unwrap_or(VMValue::Void);
                self.regs.insert(*pid, v.clone());
            }
            if super::VmConfig::global().param_trace {
                let mut pairs = Vec::new();
                for (i, pid) in func.params.iter().enumerate() {
                    let vi = self.regs.get(pid).cloned().unwrap_or(VMValue::Void);
                    pairs.append(&mut vec![format!("p{}=%{:?}:{:?}", i, pid, vi)]);
                }
                eprintln!("[vm-params] fn={} {}", func.signature.name, pairs.join(" "));
            }
        }

        let mut cur = func.entry_block;
        let mut last_pred: Option<BasicBlockId> = None;

        loop {
            let block = func
                .blocks
                .get(&cur)
                .ok_or_else(|| VMError::InvalidBasicBlock(format!("bb {:?} not found", cur)))?;

            // B案: Basic Block 実行回数の上限（オプトイン）。
            if let Some(limit) = self.max_block_exec {
                let c = self.block_exec_count.entry(block.id).or_insert(0);
                *c = c.saturating_add(1);
                if *c > limit {
                    return Err(VMError::InvalidInstruction(format!(
                        "VM block execution limit exceeded for bb={:?} (count={} > limit={}) in fn={}",
                        block.id,
                        *c,
                        limit,
                        self.cur_fn.as_deref().unwrap_or("")
                    )));
                }
            }

            if Self::trace_enabled() {
                eprintln!(
                    "[vm-trace] enter bb={:?} pred={:?} fn={}",
                    cur,
                    last_pred,
                    self.cur_fn.as_deref().unwrap_or("")
                );
            }

            self.apply_phi_nodes(block, last_pred)?;
            if let Err(e) = self.execute_block_instructions(block) {
                if Self::trace_enabled() {
                    eprintln!(
                        "[vm-trace] error in bb={:?}: {:?}\n  last_inst={:?}",
                        cur, e, self.last_inst
                    );
                }
                return Err(e);
            }

            match self.handle_terminator(block)? {
                BlockOutcome::Return(result) => {
                    // Leave function scope before restoring caller state.
                    // Scope pop will call fini() only for boxes that do not escape
                    // (i.e., Arc strong_count == 1). Returned BoxRef remains alive.
                    self.scope.pop_scope();
                    self.cur_fn = saved_fn;
                    self.regs = saved_regs;
                    return Ok(result);
                }
                BlockOutcome::Next {
                    target,
                    predecessor,
                } => {
                    last_pred = Some(predecessor);
                    cur = target;
                }
            }
        }
    }

    fn apply_phi_nodes(
        &mut self,
        block: &BasicBlock,
        last_pred: Option<BasicBlockId>,
    ) -> Result<(), VMError> {
        for inst in block.phi_instructions() {
            if let MirInstruction::Phi { dst, inputs } = inst {
                let dst_id = *dst;
                if let Some(pred) = last_pred {
                    if let Some((_, val)) = inputs.iter().find(|(bb, _)| *bb == pred) {
                        let v = match self.reg_load(*val) {
                            Ok(v) => v,
                            Err(e) => {
                                // Dev safety valve: tolerate undefined phi inputs by substituting Void
                                if super::VmConfig::global().phi_tolerate_undefined {
                                    if Self::trace_enabled() {
                                        eprintln!("[vm-trace] phi tolerate undefined input {:?} -> Void (err={:?})", val, e);
                                    }
                                    VMValue::Void
                                } else {
                                    return Err(e);
                                }
                            }
                        };
                        self.regs.insert(dst_id, v);
                        if super::VmConfig::global().phi_trace {
                            eprintln!(
                                "[vm-phi] fn={} bb={} dst=v%{} pred={} val=v%{}",
                                self.cur_fn.as_deref().unwrap_or("") ,
                                block.id.as_u32(),
                                dst_id.as_u32(),
                                pred.as_u32(),
                                val.as_u32()
                            );
                        }
                        if Self::trace_enabled() {
                            eprintln!(
                                "[vm-trace] phi dst={:?} take pred={:?} val={:?}",
                                dst_id, pred, val
                            );
                        }
                    }
                } else if let Some((_, val)) = inputs.first() {
                    let v = match self.reg_load(*val) {
                        Ok(v) => v,
                        Err(e) => {
                            if super::VmConfig::global().phi_tolerate_undefined {
                                if Self::trace_enabled() {
                                    eprintln!("[vm-trace] phi tolerate undefined default input {:?} -> Void (err={:?})", val, e);
                                }
                                VMValue::Void
                            } else {
                                return Err(e);
                            }
                        }
                    };
                    self.regs.insert(dst_id, v);
                    if super::VmConfig::global().phi_trace {
                        eprintln!(
                            "[vm-phi] fn={} bb={} dst=v%{} pred=<default> val=v%{}",
                            self.cur_fn.as_deref().unwrap_or("") ,
                            block.id.as_u32(),
                            dst_id.as_u32(),
                            val.as_u32()
                        );
                    }
                    if Self::trace_enabled() {
                        eprintln!(
                            "[vm-trace] phi dst={:?} take default val={:?}",
                            dst_id, val
                        );
                    }
                }
            }
        }
        Ok(())
    }

    fn execute_block_instructions(&mut self, block: &BasicBlock) -> Result<(), VMError> {
        // Dev trace: rich per‑instruction tracer (opt‑in)
        let trace_cfg = TraceCfg::from_env();
        let mut inst_idx: usize = 0;
        for inst in block.non_phi_instructions() {
            // Fuel: increment and enforce optional max_inst limit
            if let Some(limit) = self.max_inst {
                self.inst_count = self.inst_count.saturating_add(1);
                if self.inst_count > limit {
                    return Err(VMError::InvalidInstruction(format!(
                        "VM instruction limit exceeded (count={} > limit={}) at fn={} bb={:?} inst_idx={}",
                        self.inst_count,
                        limit,
                        self.cur_fn.as_deref().unwrap_or(""),
                        block.id,
                        inst_idx
                    )));
                }
            }
            self.last_block = Some(block.id);
            self.last_inst = Some(inst.clone());
            // Legacy one‑liner (kept for compatibility)
            if Self::trace_enabled() { eprintln!("[vm-trace] inst bb={:?} {:?}", block.id, inst); }
            // New rich trace (env: HAKO_VM_TRACE="op=...,externcall;block=0,3;regs=1")
            if let Some(cfg) = &trace_cfg {
                if cfg.should_trace(block.id, op_name(inst)) {
                    eprintln!("{}", cfg.render_before(self, block.id, inst_idx, inst));
                }
            }
            // Optional stepper before execute
            if StepperCfg::is_enabled() { maybe_stepper_prompt(block.id, inst_idx, inst, &trace_cfg, self); }
            self.execute_instruction(inst)?;
            if crate::runtime::env_gate_box::bool_any(&["HAKO_CALLABLE_ASYNC","NYASH_CALLABLE_ASYNC","HAKO_VM_POLL_SCHED","NYASH_VM_POLL_SCHED"]) {
                crate::runtime::global_hooks::safepoint_and_poll();
            }
            if let Some(cfg) = &trace_cfg { if cfg.should_trace(block.id, op_name(inst)) {
                if let Some(line) = cfg.render_after(self, block.id, inst_idx, inst) { eprintln!("{}", line); }
            }}
            inst_idx += 1;
        }
        Ok(())
    }

    fn handle_terminator(&mut self, block: &BasicBlock) -> Result<BlockOutcome, VMError> {
        match &block.terminator {
            Some(MirInstruction::Return { value }) => {
                let result = if let Some(v) = value {
                    self.reg_load(*v)?
                } else {
                    VMValue::Void
                };
                if super::VmConfig::global().ret_trace {
                    let kind = crate::backend::abi_util::tag_of_vm(&result);
                    eprintln!(
                        "[vm-ret] fn={} kind={} bb={}",
                        self.cur_fn.as_deref().unwrap_or("") ,
                        kind,
                        block.id.as_u32()
                    );
                }
                Ok(BlockOutcome::Return(result))
            }
            Some(MirInstruction::Jump { target }) => Ok(BlockOutcome::Next {
                target: *target,
                predecessor: block.id,
            }),
            Some(MirInstruction::Branch {
                condition,
                then_bb,
                else_bb,
            }) => {
                let cond = self.reg_load(*condition)?;
                if super::VmConfig::global().branch_trace {
                    // Lightweight condition trace: tag and class name for BoxRef
                    let tag = crate::backend::abi_util::tag_of_vm(&cond);
                    let class = match &cond {
                        VMValue::BoxRef(bx) => bx.type_name(),
                        _ => "-",
                    };
                    eprintln!(
                        "[vm-branch] fn={} bb={:?} cond=%{:?} tag={} class={}",
                        self.cur_fn.as_deref().unwrap_or(""),
                        block.id,
                        condition,
                        tag,
                        class
                    );
                }
                let branch = to_bool_vm(&cond).map_err(VMError::TypeError)?;
                let target = if branch { *then_bb } else { *else_bb };
                Ok(BlockOutcome::Next {
                    target,
                    predecessor: block.id,
                })
            }
            None => Err(VMError::InvalidBasicBlock(format!(
                "unterminated block {:?}",
                block.id
            ))),
            Some(MirInstruction::Throw { .. }) => {
                // Minimal support: treat throw as immediate function return (void)
                // This allows non-taken throw arms to be lowered without crashing the VM.
                Ok(BlockOutcome::Return(VMValue::Void))
            }
            Some(other) => Err(VMError::InvalidInstruction(format!(
                "invalid terminator in MIR interp: {:?}",
                other
            ))),
        }
    }
}

enum BlockOutcome {
    Return(VMValue),
    Next {
        target: BasicBlockId,
        predecessor: BasicBlockId,
    },
}

// ----- Dev Tracer (opt‑in) -----
use std::collections::HashSet;

struct TraceCfg {
    ops: Option<HashSet<String>>,   // e.g., {"boxcall","externcall"}
    blocks: Option<HashSet<u32>>,   // e.g., {0,3}; None => all; '*' => all
    regs: bool,
}

impl TraceCfg {
    fn from_env() -> Option<Self> {
        let raw = std::env::var("HAKO_VM_TRACE").ok()
            .or_else(|| std::env::var("NYASH_VM_TRACE").ok())?;
        let mut ops: Option<HashSet<String>> = None;
        let mut blocks: Option<HashSet<u32>> = None;
        let mut regs = false;
        for part in raw.split(|c| c == ';' || c == ' ') {
            let p = part.trim(); if p.is_empty() { continue; }
            if let Some((k,v)) = p.split_once('=') {
                match k.trim() {
                    "op"|"ops" => {
                        let mut set = HashSet::new();
                        for t in v.split(',') { let s=t.trim(); if !s.is_empty(){ set.insert(s.to_string()); } }
                        ops = Some(set);
                    }
                    "block"|"blocks" => {
                        if v.trim() == "*" { blocks = None; } else {
                            let mut set = HashSet::new();
                            for t in v.split(',') { let s=t.trim(); if s.is_empty(){continue;} if let Ok(n)=s.parse::<u32>(){ set.insert(n);} }
                            blocks = Some(set);
                        }
                    }
                    "regs" => { regs = v.trim()=="1" || v.trim().eq_ignore_ascii_case("true"); }
                    _ => {}
                }
            } else if p == "regs=1" { regs = true; }
        }
        Some(Self{ops,blocks,regs})
    }

    fn should_trace(&self, bb: BasicBlockId, op: &str) -> bool {
        if let Some(bset) = &self.blocks { if !bset.contains(&bb.as_u32()) { return false; } }
        if let Some(oset) = &self.ops { oset.contains(op) } else { true }
    }

    fn render_before(&self, m: &MirInterpreter, bb: BasicBlockId, idx: usize, inst: &crate::mir::MirInstruction) -> String {
        use crate::mir::MirInstruction as I;
        let mut s = String::new();
        let op = op_name(inst);
        s.push_str(&format!("[vm] bb={} inst={} {}", bb.as_u32(), idx, op));
        match inst {
            I::Const { dst, value } => {
                s.push_str(&format!(" dst=v%{} val={}", dst.as_u32(), const_render(value)));
            }
            I::Copy { dst, src } => {
                s.push_str(&format!(" dst=v%{} src=v%{}", dst.as_u32(), src.as_u32()));
                if self.regs { if let Some(v)=m.regs.get(src){ s.push_str(&format!(" srcv={}", val_preview(v))); } }
            }
            I::BinOp { dst, op, lhs, rhs } => {
                let (av, bv) = (m.regs.get(lhs), m.regs.get(rhs));
                s.push_str(&format!(" kind={:?} lhs=v%{}({}) rhs=v%{}({}) dst=v%{}",
                    op, lhs.as_u32(), av.map(val_preview).unwrap_or("?".into()),
                    rhs.as_u32(), bv.map(val_preview).unwrap_or("?".into()), dst.as_u32()));
            }
            I::Compare { dst, op, lhs, rhs } => {
                let (av, bv) = (m.regs.get(lhs), m.regs.get(rhs));
                s.push_str(&format!(" kind={:?} lhs=v%{}({}) rhs=v%{}({}) dst=v%{}",
                    op, lhs.as_u32(), av.map(val_preview).unwrap_or("?".into()),
                    rhs.as_u32(), bv.map(val_preview).unwrap_or("?".into()), dst.as_u32()));
            }
            I::Call { callee, .. } => { if let Some(c)=callee { s.push_str(&format!(" callee={:?}", c)); } }
            I::BoxCall { method, .. } => { s.push_str(&format!(" boxcall method=\"{}\"", method)); }
            // ExternCall retired
            I::Return { value } => { if let Some(v)=value { s.push_str(&format!(" ret=v%{}", v.as_u32())); } }
            I::Branch { condition, then_bb, else_bb } => {
                let cv = m.regs.get(condition).map(val_preview).unwrap_or("?".into());
                s.push_str(&format!(" cond=v%{}({}) then={} else={}", condition.as_u32(), cv, then_bb.as_u32(), else_bb.as_u32()));
            }
            I::Jump { target } => { s.push_str(&format!(" target={}", target.as_u32())); }
            _ => {}
        }
        s
    }

    fn render_after(&self, m: &MirInterpreter, _bb: BasicBlockId, _idx: usize, inst: &crate::mir::MirInstruction) -> Option<String> {
        if !self.regs { return None; }
        if let Some(dst) = crate::mir::instruction_kinds::CallLikeInst::from_mir(inst).and_then(|c| c.dst()) {
            if let Some(v) = m.regs.get(&dst) {
                return Some(format!("[vm] → v%{}({})", dst.as_u32(), val_preview(v)));
            }
        }
        match inst {
            crate::mir::MirInstruction::Const { dst, .. }
            | crate::mir::MirInstruction::Copy { dst, .. }
            | crate::mir::MirInstruction::BinOp { dst, .. }
            | crate::mir::MirInstruction::Compare { dst, .. } => {
                if let Some(v) = m.regs.get(dst) { return Some(format!("[vm] → v%{}({})", dst.as_u32(), val_preview(v))); }
            }
            _ => {}
        }
        None
    }
}

fn op_name(i: &crate::mir::MirInstruction) -> &'static str {
    use crate::mir::MirInstruction as I;
    match i {
        I::Const { .. } => "const",
        I::Copy { .. } => "copy",
        I::BinOp { .. } => "binop",
        I::Compare { .. } => "compare",
        I::Call { .. } => "call",
        I::BoxCall { .. } => "boxcall",
        
        // ExternCall retired
        I::Return { .. } => "ret",
        I::Branch { .. } => "branch",
        I::Jump { .. } => "jump",
        _ => "op",
    }
}

fn const_render(cv: &crate::mir::ConstValue) -> String {
    use crate::mir::ConstValue as C;
    match cv {
        C::Integer(n) => format!("{}", n),
        C::Float(f) => format!("{:.3}", f),
        C::Bool(b) => format!("{}", b),
        C::String(s) => format!("\"{}\"", s),
        C::Null => "null".into(),
        _ => "?".into(),
    }
}

fn val_preview(v: &super::VMValue) -> String {
    match v {
        super::VMValue::Integer(n) => format!("{}", n),
        super::VMValue::Float(f) => format!("{:.3}", f),
        super::VMValue::Bool(b) => format!("{}", b),
        super::VMValue::String(s) => format!("\"{}\"", s),
        super::VMValue::Void => "void".into(),
        super::VMValue::BoxRef(bx) => format!("{}", bx.type_name()),
        super::VMValue::Future(_) => "<future>".into(),
    }
}

// ----- Dev Stepper (opt‑in, safe by default) -----
use std::sync::atomic::{AtomicBool, Ordering};
static CONTINUE_ALL: AtomicBool = AtomicBool::new(false);

struct StepperCfg;
impl StepperCfg {
    fn is_enabled() -> bool {
        std::env::var("HAKO_VM_STEP").ok().as_deref() == Some("1")
            || std::env::var("NYASH_VM_STEP").ok().as_deref() == Some("1")
    }
    fn allow_block() -> bool {
        std::env::var("HAKO_VM_STEP_ALLOW_BLOCK").ok().as_deref() == Some("1")
            || std::env::var("NYASH_VM_STEP_ALLOW_BLOCK").ok().as_deref() == Some("1")
    }
}

fn maybe_stepper_prompt(bb: BasicBlockId, idx: usize, inst: &crate::mir::MirInstruction, trace_cfg: &Option<TraceCfg>, m: &MirInterpreter) {
    if CONTINUE_ALL.load(Ordering::Relaxed) { return; }
    let line = match trace_cfg {
        Some(cfg) => cfg.render_before(m, bb, idx, inst),
        None => format!("[vm] bb={} inst={} {}", bb.as_u32(), idx, op_name(inst)),
    };
    eprintln!("> {}", line);
    eprint!("[n]ext/[c]ontinue/[r]egisters/[q]uit? ");
    use std::io::Write;
    let mut auto = true;
    if StepperCfg::allow_block() {
        auto = false;
    }
    if auto {
        eprintln!("n");
        return;
    }
    std::io::stderr().flush().ok();
    let mut buf = String::new();
    if std::io::stdin().read_line(&mut buf).is_err() { return; }
    let t = buf.trim();
    match t.chars().next().unwrap_or('n') {
        'n' | 'N' => { /* step */ }
        'c' | 'C' => { CONTINUE_ALL.store(true, Ordering::Relaxed); }
        'r' | 'R' => { dump_registers(m); }
        'q' | 'Q' => { panic!("VM stopped by user"); }
        _ => {}
    }
}

fn dump_registers(m: &MirInterpreter) {
    eprintln!("\nRegisters (up to 32):");
    let mut pairs: Vec<(u32, String)> = m.regs.iter()
        .filter_map(|(k,v)| Some((k.as_u32(), val_preview(v))))
        .collect();
    pairs.sort_by_key(|p| p.0);
    for (i, (id, val)) in pairs.into_iter().take(32).enumerate() {
        eprintln!("  {:2}: v%{} = {}", i, id, val);
    }
}
