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
}

impl BridgeEnv {
    pub(super) fn load() -> Self {
        Self {
            throw_enabled: std::env::var("NYASH_BRIDGE_THROW_ENABLE").ok().as_deref() == Some("1"),
            mir_no_phi: crate::config::env::mir_no_phi(),
            allow_me_dummy: std::env::var("NYASH_BRIDGE_ME_DUMMY").ok().as_deref() == Some("1"),
            me_class: std::env::var("NYASH_BRIDGE_ME_CLASS").unwrap_or_else(|_| "Main".to_string()),
        }
    }
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
                if let Some(bb) = f.get_block_mut(cur_bb) {
                    bb.set_terminator(MirInstruction::Jump {
                        target: ctx.exit_bb,
                    });
                }
                crate::jit::events::emit_lower(
                    serde_json::json!({ "id": "loop_break","exit_bb": ctx.exit_bb.0,"decision": "lower" }),
                    "loop",
                    "<json_v0>",
                );
            }
            Ok(cur_bb)
        }
        StmtV0::Continue => {
            if let Some(ctx) = loop_stack.last().copied() {
                if let Some(bb) = f.get_block_mut(cur_bb) {
                    bb.set_terminator(MirInstruction::Jump {
                        target: ctx.cond_bb,
                    });
                }
                crate::jit::events::emit_lower(
                    serde_json::json!({ "id": "loop_continue","cond_bb": ctx.cond_bb.0,"decision": "lower" }),
                    "loop",
                    "<json_v0>",
                );
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
    module.add_function(f);
    Ok(module)
}

pub(super) fn maybe_dump_mir(module: &MirModule) {
    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
        let p = MirPrinter::new();
        println!("{}", p.print_module(module));
    }
}
