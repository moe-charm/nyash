use super::ast::{ProgramV0, StmtV0};
use crate::mir::{
    BasicBlockId, ConstValue, EffectMask, FunctionSignature, MirFunction, MirInstruction, MirModule,
    MirPrinter, MirType, ValueId,
};
use std::collections::HashMap;

// Split out merge/new_block helpers for readability (no behavior change)
mod merge;
use merge::{merge_var_maps, new_block};
// Feature splits (gradual extraction)
pub(super) mod if_else;
pub(super) mod loop_;
pub(super) mod try_catch;
pub(super) mod expr;
pub(super) mod ternary; // placeholder (not wired)
pub(super) mod peek; // placeholder (not wired)
pub(super) mod throw_ctx; // thread-local ctx for Result-mode throw routing

#[derive(Clone, Copy)]
pub(super) struct LoopContext {
    pub(super) cond_bb: BasicBlockId,
    pub(super) exit_bb: BasicBlockId,
}

#[derive(Clone)]
pub(super) struct BridgeEnv {
    pub(super) throw_enabled: bool,
    pub(super) mir_no_phi: bool,
    pub(super) allow_me_dummy: bool,
    pub(super) me_class: String,
    pub(super) try_result_mode: bool,
}

impl BridgeEnv {
    pub(super) fn load() -> Self {
        let trm = crate::config::env::try_result_mode();
        let no_phi = crate::config::env::mir_no_phi();
        if crate::config::env::cli_verbose() {
            eprintln!("[Bridge] load: try_result_mode={} mir_no_phi={}", trm, no_phi);
        }
        Self {
            throw_enabled: std::env::var("NYASH_BRIDGE_THROW_ENABLE").ok().as_deref() == Some("1"),
            mir_no_phi: no_phi,
            allow_me_dummy: std::env::var("NYASH_BRIDGE_ME_DUMMY").ok().as_deref() == Some("1"),
            me_class: std::env::var("NYASH_BRIDGE_ME_CLASS").unwrap_or_else(|_| "Main".to_string()),
            try_result_mode: trm,
        }
    }
}

/// Small helper: set Jump terminator and record predecessor on the target.
fn jump_with_pred(f: &mut MirFunction, cur_bb: BasicBlockId, target: BasicBlockId) {
    if let Some(bb) = f.get_block_mut(cur_bb) {
        bb.set_terminator(MirInstruction::Jump { target });
    }
    if let Some(succ) = f.get_block_mut(target) {
        succ.add_predecessor(cur_bb);
    }
}

/// Strip Phi instructions by inserting edge copies on each predecessor.
/// This normalizes MIR to PHI-off form for downstream harnesses that synthesize PHIs.
fn strip_phi_functions(f: &mut MirFunction) {
    // Collect block ids to avoid borrow issues while mutating
    let block_ids: Vec<BasicBlockId> = f.blocks.keys().copied().collect();
    for bbid in block_ids {
        // Snapshot phi instructions at the head
        let mut phi_entries: Vec<(ValueId, Vec<(BasicBlockId, ValueId)>)> = Vec::new();
        if let Some(bb) = f.blocks.get(&bbid) {
            for inst in &bb.instructions {
                if let MirInstruction::Phi { dst, inputs } = inst {
                    phi_entries.push((*dst, inputs.clone()));
                } else {
                    // PHIs must be at the beginning; once we see non-Phi, stop
                    break;
                }
            }
        }
        if phi_entries.is_empty() {
            continue;
        }
        // Insert copies on predecessors
        for (dst, inputs) in &phi_entries {
            for (pred, val) in inputs {
                if let Some(pbb) = f.blocks.get_mut(pred) {
                    pbb.add_instruction(MirInstruction::Copy { dst: *dst, src: *val });
                }
            }
        }
        // Remove Phi instructions from the merge block
        if let Some(bb) = f.blocks.get_mut(&bbid) {
            let non_phi: Vec<MirInstruction> = bb
                .instructions
                .iter()
                .cloned()
                .skip_while(|inst| matches!(inst, MirInstruction::Phi { .. }))
                .collect();
            bb.instructions = non_phi;
        }
    }
}

fn lower_break_stmt(f: &mut MirFunction, cur_bb: BasicBlockId, exit_bb: BasicBlockId) {
    jump_with_pred(f, cur_bb, exit_bb);
    crate::jit::events::emit_lower(
        serde_json::json!({ "id": "loop_break","exit_bb": exit_bb.0,"decision": "lower" }),
        "loop",
        "<json_v0>",
    );
}

fn lower_continue_stmt(f: &mut MirFunction, cur_bb: BasicBlockId, cond_bb: BasicBlockId) {
    jump_with_pred(f, cur_bb, cond_bb);
    crate::jit::events::emit_lower(
        serde_json::json!({ "id": "loop_continue","cond_bb": cond_bb.0,"decision": "lower" }),
        "loop",
        "<json_v0>",
    );
}


pub(super) fn lower_stmt_with_vars(
    f: &mut MirFunction,
    cur_bb: BasicBlockId,
    s: &StmtV0,
    vars: &mut HashMap<String, ValueId>,
    loop_stack: &mut Vec<LoopContext>,
    env: &BridgeEnv,
) -> Result<BasicBlockId, String> {
    match s {
        StmtV0::Return { expr } => {
            let (v, cur) = expr::lower_expr_with_vars(env, f, cur_bb, expr, vars)?;
            if let Some(bb) = f.get_block_mut(cur) {
                bb.set_terminator(MirInstruction::Return { value: Some(v) });
            }
            Ok(cur)
        }
        StmtV0::Extern {
            iface,
            method,
            args,
        } => {
            let (arg_ids, cur) = expr::lower_args_with_vars(env, f, cur_bb, args, vars)?;
            if let Some(bb) = f.get_block_mut(cur) {
                bb.add_instruction(MirInstruction::ExternCall {
                    dst: None,
                    iface_name: iface.clone(),
                    method_name: method.clone(),
                    args: arg_ids,
                    effects: EffectMask::IO,
                });
            }
            Ok(cur)
        }
        StmtV0::Expr { expr } => {
            let (_v, cur) = expr::lower_expr_with_vars(env, f, cur_bb, expr, vars)?;
            Ok(cur)
        }
        StmtV0::Local { name, expr } => {
            let (v, cur) = expr::lower_expr_with_vars(env, f, cur_bb, expr, vars)?;
            vars.insert(name.clone(), v);
            Ok(cur)
        }
        StmtV0::Break => {
            if let Some(ctx) = loop_stack.last().copied() {
                lower_break_stmt(f, cur_bb, ctx.exit_bb);
            }
            Ok(cur_bb)
        }
        StmtV0::Continue => {
            if let Some(ctx) = loop_stack.last().copied() {
                lower_continue_stmt(f, cur_bb, ctx.cond_bb);
            }
            Ok(cur_bb)
        }
        StmtV0::Try {
            try_body,
            catches,
            finally,
        } => {
            try_catch::lower_try_stmt(
                f, cur_bb, try_body, catches, finally, vars, loop_stack, env,
            )
        }
        StmtV0::If { cond, then, r#else } => if_else::lower_if_stmt(
            f, cur_bb, cond, then, r#else, vars, loop_stack, env,
        ),
        StmtV0::Loop { cond, body } => loop_::lower_loop_stmt(
            f, cur_bb, cond, body, vars, loop_stack, env,
        ),
    }
}

pub(super) fn lower_stmt_list_with_vars(
    f: &mut MirFunction,
    start_bb: BasicBlockId,
    stmts: &[StmtV0],
    vars: &mut HashMap<String, ValueId>,
    loop_stack: &mut Vec<LoopContext>,
    env: &BridgeEnv,
) -> Result<BasicBlockId, String> {
    let mut cur = start_bb;
    for s in stmts {
        cur = lower_stmt_with_vars(f, cur, s, vars, loop_stack, env)?;
        if let Some(bb) = f.blocks.get(&cur) {
            if bb.is_terminated() {
                break;
            }
        }
    }
    Ok(cur)
}

pub(super) fn lower_program(prog: ProgramV0) -> Result<MirModule, String> {
    if prog.body.is_empty() {
        return Err("empty body".into());
    }
    let env = BridgeEnv::load();
    let mut module = MirModule::new("ny_json_v0".into());
    let sig = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    let entry = BasicBlockId::new(0);
    let mut f = MirFunction::new(sig, entry);
    let mut var_map: HashMap<String, ValueId> = HashMap::new();
    let mut loop_stack: Vec<LoopContext> = Vec::new();
    let start_bb = f.entry_block;
    let end_bb = lower_stmt_list_with_vars(
        &mut f,
        start_bb,
        &prog.body,
        &mut var_map,
        &mut loop_stack,
        &env,
    )?;
    let need_default_ret = f.blocks.iter().any(|(_k, b)| !b.is_terminated());
    if need_default_ret {
        let target_bb = end_bb;
        let dst_id = f.next_value_id();
        if let Some(bb) = f.get_block_mut(target_bb) {
            if !bb.is_terminated() {
                bb.add_instruction(MirInstruction::Const {
                    dst: dst_id,
                    value: ConstValue::Integer(0),
                });
                bb.set_terminator(MirInstruction::Return {
                    value: Some(dst_id),
                });
            }
        }
    }
    f.signature.return_type = MirType::Unknown;
    // PHI-off normalization for Bridge output
    if env.mir_no_phi {
        strip_phi_functions(&mut f);
    }
    module.add_function(f);
    Ok(module)
}

pub(super) fn maybe_dump_mir(module: &MirModule) {
    if crate::config::env::cli_verbose() {
        let p = MirPrinter::new();
        println!("{}", p.print_module(module));
    }
}
