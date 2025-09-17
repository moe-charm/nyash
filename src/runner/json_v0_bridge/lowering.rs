use super::ast::{CatchV0, ExprV0, ProgramV0, StmtV0};
use crate::mir::{
    BasicBlock, BasicBlockId, BinaryOp, ConstValue, EffectMask, FunctionSignature, MirFunction,
    MirInstruction, MirModule, MirPrinter, MirType, ValueId,
};
use std::collections::HashMap;
use std::collections::HashSet;

// Split out merge/new_block helpers for readability (no behavior change)
mod merge;
use merge::{merge_var_maps, merge_values, new_block};
// Feature splits (gradual extraction)
pub(super) mod if_else;
pub(super) mod loop_;
pub(super) mod try_catch;

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

trait VarScope {
    fn resolve(
        &mut self,
        env: &BridgeEnv,
        f: &mut MirFunction,
        cur_bb: BasicBlockId,
        name: &str,
    ) -> Result<Option<ValueId>, String>;
}

struct NoVars;
impl VarScope for NoVars {
    fn resolve(
        &mut self,
        _env: &BridgeEnv,
        _f: &mut MirFunction,
        _cur_bb: BasicBlockId,
        name: &str,
    ) -> Result<Option<ValueId>, String> {
        Err(format!("undefined variable in this context: {}", name))
    }
}

struct MapVars<'a> {
    vars: &'a mut HashMap<String, ValueId>,
}
impl<'a> MapVars<'a> {
    fn new(vars: &'a mut HashMap<String, ValueId>) -> Self {
        Self { vars }
    }
}
impl<'a> VarScope for MapVars<'a> {
    fn resolve(
        &mut self,
        env: &BridgeEnv,
        f: &mut MirFunction,
        cur_bb: BasicBlockId,
        name: &str,
    ) -> Result<Option<ValueId>, String> {
        if let Some(&vid) = self.vars.get(name) {
            return Ok(Some(vid));
        }
        if name == "me" {
            if env.allow_me_dummy {
                let dst = f.next_value_id();
                if let Some(bb) = f.get_block_mut(cur_bb) {
                    bb.add_instruction(MirInstruction::NewBox {
                        dst,
                        box_type: env.me_class.clone(),
                        args: vec![],
                    });
                }
                self.vars.insert(name.to_string(), dst);
                Ok(Some(dst))
            } else {
                Err("undefined 'me' outside box context (set NYASH_BRIDGE_ME_DUMMY=1 to inject placeholder)".into())
            }
        } else {
            Ok(None)
        }
    }
}

// moved helpers are imported above

fn lower_throw(
    env: &BridgeEnv,
    f: &mut MirFunction,
    cur_bb: BasicBlockId,
    exception_value: ValueId,
) -> (ValueId, BasicBlockId) {
    if env.throw_enabled {
        if let Some(bb) = f.get_block_mut(cur_bb) {
            bb.set_terminator(MirInstruction::Throw {
                exception: exception_value,
                effects: EffectMask::PANIC,
            });
        }
        (exception_value, cur_bb)
    } else {
        let dst = f.next_value_id();
        if let Some(bb) = f.get_block_mut(cur_bb) {
            bb.add_instruction(MirInstruction::Const {
                dst,
                value: ConstValue::Integer(0),
            });
        }
        (dst, cur_bb)
    }
}

fn lower_expr_with_scope<S: VarScope>(
    env: &BridgeEnv,
    f: &mut MirFunction,
    cur_bb: BasicBlockId,
    e: &ExprV0,
    vars: &mut S,
) -> Result<(ValueId, BasicBlockId), String> {
    match e {
        ExprV0::Int { value } => {
            let ival: i64 = if let Some(n) = value.as_i64() {
                n
            } else if let Some(s) = value.as_str() {
                s.parse().map_err(|_| "invalid int literal")?
            } else {
                return Err("invalid int literal".into());
            };
            let dst = f.next_value_id();
            if let Some(bb) = f.get_block_mut(cur_bb) {
                bb.add_instruction(MirInstruction::Const {
                    dst,
                    value: ConstValue::Integer(ival),
                });
            }
            Ok((dst, cur_bb))
        }
        ExprV0::Str { value } => {
            let dst = f.next_value_id();
            if let Some(bb) = f.get_block_mut(cur_bb) {
                bb.add_instruction(MirInstruction::Const {
                    dst,
                    value: ConstValue::String(value.clone()),
                });
            }
            Ok((dst, cur_bb))
        }
        ExprV0::Bool { value } => {
            let dst = f.next_value_id();
            if let Some(bb) = f.get_block_mut(cur_bb) {
                bb.add_instruction(MirInstruction::Const {
                    dst,
                    value: ConstValue::Bool(*value),
                });
            }
            Ok((dst, cur_bb))
        }
        ExprV0::Binary { op, lhs, rhs } => {
            let (l, cur_after_l) = lower_expr_with_scope(env, f, cur_bb, lhs, vars)?;
            let (r, cur_after_r) = lower_expr_with_scope(env, f, cur_after_l, rhs, vars)?;
            let bop = match op.as_str() {
                "+" => BinaryOp::Add,
                "-" => BinaryOp::Sub,
                "*" => BinaryOp::Mul,
                "/" => BinaryOp::Div,
                _ => return Err("unsupported op".into()),
            };
            let dst = f.next_value_id();
            if let Some(bb) = f.get_block_mut(cur_after_r) {
                bb.add_instruction(MirInstruction::BinOp {
                    dst,
                    op: bop,
                    lhs: l,
                    rhs: r,
                });
            }
            Ok((dst, cur_after_r))
        }
        ExprV0::Extern {
            iface,
            method,
            args,
        } => {
            let (arg_ids, cur2) = lower_args_with_scope(env, f, cur_bb, args, vars)?;
            let dst = f.next_value_id();
            if let Some(bb) = f.get_block_mut(cur2) {
                bb.add_instruction(MirInstruction::ExternCall {
                    dst: Some(dst),
                    iface_name: iface.clone(),
                    method_name: method.clone(),
                    args: arg_ids,
                    effects: EffectMask::IO,
                });
            }
            Ok((dst, cur2))
        }
        ExprV0::Compare { op, lhs, rhs } => {
            let (l, cur_after_l) = lower_expr_with_scope(env, f, cur_bb, lhs, vars)?;
            let (r, cur_after_r) = lower_expr_with_scope(env, f, cur_after_l, rhs, vars)?;
            let cop = match op.as_str() {
                "==" => crate::mir::CompareOp::Eq,
                "!=" => crate::mir::CompareOp::Ne,
                "<" => crate::mir::CompareOp::Lt,
                "<=" => crate::mir::CompareOp::Le,
                ">" => crate::mir::CompareOp::Gt,
                ">=" => crate::mir::CompareOp::Ge,
                _ => return Err("unsupported compare op".into()),
            };
            let dst = f.next_value_id();
            if let Some(bb) = f.get_block_mut(cur_after_r) {
                bb.add_instruction(MirInstruction::Compare {
                    dst,
                    op: cop,
                    lhs: l,
                    rhs: r,
                });
            }
            Ok((dst, cur_after_r))
        }
        ExprV0::Logical { op, lhs, rhs } => {
            let (l, cur_after_l) = lower_expr_with_scope(env, f, cur_bb, lhs, vars)?;
            let rhs_bb = new_block(f);
            let fall_bb = new_block(f);
            let merge_bb = new_block(f);
            let is_and = matches!(op.as_str(), "&&" | "and");
            if let Some(bb) = f.get_block_mut(cur_after_l) {
                if is_and {
                    bb.set_terminator(MirInstruction::Branch {
                        condition: l,
                        then_bb: rhs_bb,
                        else_bb: fall_bb,
                    });
                } else {
                    bb.set_terminator(MirInstruction::Branch {
                        condition: l,
                        then_bb: fall_bb,
                        else_bb: rhs_bb,
                    });
                }
            }
            crate::jit::events::emit_lower(
                serde_json::json!({ "id":"shortcircuit","op": if is_and {"and"} else {"or"},"rhs_bb":rhs_bb.0,"fall_bb":fall_bb.0,"merge_bb":merge_bb.0 }),
                "shortcircuit",
                "<json_v0>",
            );
            let cdst = f.next_value_id();
            if let Some(bb) = f.get_block_mut(fall_bb) {
                let cval = if is_and {
                    ConstValue::Bool(false)
                } else {
                    ConstValue::Bool(true)
                };
                bb.add_instruction(MirInstruction::Const {
                    dst: cdst,
                    value: cval,
                });
                bb.set_terminator(MirInstruction::Jump { target: merge_bb });
            }
            let (rval, rhs_end) = lower_expr_with_scope(env, f, rhs_bb, rhs, vars)?;
            if let Some(bb) = f.get_block_mut(rhs_end) {
                if !bb.is_terminated() {
                    bb.set_terminator(MirInstruction::Jump { target: merge_bb });
                }
            }
            let out = f.next_value_id();
            if env.mir_no_phi {
                if let Some(bb) = f.get_block_mut(fall_bb) {
                    bb.add_instruction(MirInstruction::Copy {
                        dst: out,
                        src: cdst,
                    });
                }
                if let Some(bb) = f.get_block_mut(rhs_end) {
                    bb.add_instruction(MirInstruction::Copy {
                        dst: out,
                        src: rval,
                    });
                }
            } else if let Some(bb) = f.get_block_mut(merge_bb) {
                let mut inputs: Vec<(BasicBlockId, ValueId)> = vec![(fall_bb, cdst)];
                if rhs_end != fall_bb {
                    inputs.push((rhs_end, rval));
                } else {
                    inputs.push((fall_bb, rval));
                }
                inputs.sort_by_key(|(bbid, _)| bbid.0);
                bb.insert_instruction_after_phis(MirInstruction::Phi { dst: out, inputs });
            }
            Ok((out, merge_bb))
        }
        ExprV0::Call { name, args } => {
            if name == "array.of" {
                let arr = f.next_value_id();
                if let Some(bb) = f.get_block_mut(cur_bb) {
                    bb.add_instruction(MirInstruction::NewBox {
                        dst: arr,
                        box_type: "ArrayBox".into(),
                        args: vec![],
                    });
                }
                let mut cur = cur_bb;
                for e in args {
                    let (v, c) = lower_expr_with_scope(env, f, cur, e, vars)?;
                    cur = c;
                    let tmp = f.next_value_id();
                    if let Some(bb) = f.get_block_mut(cur) {
                        bb.add_instruction(MirInstruction::BoxCall {
                            dst: Some(tmp),
                            box_val: arr,
                            method: "push".into(),
                            method_id: None,
                            args: vec![v],
                            effects: EffectMask::READ,
                        });
                    }
                }
                return Ok((arr, cur));
            }
            if name == "map.of" {
                let mapv = f.next_value_id();
                if let Some(bb) = f.get_block_mut(cur_bb) {
                    bb.add_instruction(MirInstruction::NewBox {
                        dst: mapv,
                        box_type: "MapBox".into(),
                        args: vec![],
                    });
                }
                let mut cur = cur_bb;
                let mut it = args.iter();
                while let Some(k) = it.next() {
                    if let Some(v) = it.next() {
                        let (kv, cur2) = lower_expr_with_scope(env, f, cur, k, vars)?;
                        cur = cur2;
                        let (vv, cur3) = lower_expr_with_scope(env, f, cur, v, vars)?;
                        cur = cur3;
                        let tmp = f.next_value_id();
                        if let Some(bb) = f.get_block_mut(cur) {
                            bb.add_instruction(MirInstruction::BoxCall {
                                dst: Some(tmp),
                                box_val: mapv,
                                method: "set".into(),
                                method_id: None,
                                args: vec![kv, vv],
                                effects: EffectMask::READ,
                            });
                        }
                    } else {
                        break;
                    }
                }
                return Ok((mapv, cur));
            }
            let (arg_ids, cur) = lower_args_with_scope(env, f, cur_bb, args, vars)?;
            let fun_val = f.next_value_id();
            if let Some(bb) = f.get_block_mut(cur) {
                bb.add_instruction(MirInstruction::Const {
                    dst: fun_val,
                    value: ConstValue::String(name.clone()),
                });
            }
            let dst = f.next_value_id();
            if let Some(bb) = f.get_block_mut(cur) {
                bb.add_instruction(MirInstruction::Call {
                    dst: Some(dst),
                    func: fun_val,
                    args: arg_ids,
                    effects: EffectMask::READ,
                });
            }
            Ok((dst, cur))
        }
        ExprV0::Method { recv, method, args } => {
            let recv_is_console_new =
                matches!(&**recv, ExprV0::New { class, .. } if class == "ConsoleBox");
            if recv_is_console_new && (method == "println" || method == "print" || method == "log")
            {
                let (arg_ids, cur2) = lower_args_with_scope(env, f, cur_bb, args, vars)?;
                let dst = f.next_value_id();
                if let Some(bb) = f.get_block_mut(cur2) {
                    bb.add_instruction(MirInstruction::ExternCall {
                        dst: Some(dst),
                        iface_name: "env.console".into(),
                        method_name: "log".into(),
                        args: arg_ids,
                        effects: EffectMask::READ,
                    });
                }
                return Ok((dst, cur2));
            }
            let (recv_v, cur) = lower_expr_with_scope(env, f, cur_bb, recv, vars)?;
            let (arg_ids, cur2) = lower_args_with_scope(env, f, cur, args, vars)?;
            let dst = f.next_value_id();
            if let Some(bb) = f.get_block_mut(cur2) {
                bb.add_instruction(MirInstruction::BoxCall {
                    dst: Some(dst),
                    box_val: recv_v,
                    method: method.clone(),
                    method_id: None,
                    args: arg_ids,
                    effects: EffectMask::READ,
                });
            }
            Ok((dst, cur2))
        }
        ExprV0::New { class, args } => {
            let (arg_ids, cur) = lower_args_with_scope(env, f, cur_bb, args, vars)?;
            let dst = f.next_value_id();
            if let Some(bb) = f.get_block_mut(cur) {
                bb.add_instruction(MirInstruction::NewBox {
                    dst,
                    box_type: class.clone(),
                    args: arg_ids,
                });
            }
            Ok((dst, cur))
        }
        ExprV0::Var { name } => match vars.resolve(env, f, cur_bb, name)? {
            Some(v) => Ok((v, cur_bb)),
            None => Err(format!("undefined variable: {}", name)),
        },
        ExprV0::Throw { expr } => {
            let (exc, cur) = lower_expr_with_scope(env, f, cur_bb, expr, vars)?;
            Ok(lower_throw(env, f, cur, exc))
        }
    }
}

fn lower_args_with_scope<S: VarScope>(
    env: &BridgeEnv,
    f: &mut MirFunction,
    cur_bb: BasicBlockId,
    args: &[ExprV0],
    scope: &mut S,
) -> Result<(Vec<ValueId>, BasicBlockId), String> {
    let mut out = Vec::with_capacity(args.len());
    let mut cur = cur_bb;
    for a in args {
        let (v, c) = lower_expr_with_scope(env, f, cur, a, scope)?;
        out.push(v);
        cur = c;
    }
    Ok((out, cur))
}

#[allow(dead_code)]
fn lower_expr(
    env: &BridgeEnv,
    f: &mut MirFunction,
    cur_bb: BasicBlockId,
    e: &ExprV0,
) -> Result<(ValueId, BasicBlockId), String> {
    let mut scope = NoVars;
    lower_expr_with_scope(env, f, cur_bb, e, &mut scope)
}

pub(super) fn lower_expr_with_vars(
    env: &BridgeEnv,
    f: &mut MirFunction,
    cur_bb: BasicBlockId,
    e: &ExprV0,
    vars: &mut HashMap<String, ValueId>,
) -> Result<(ValueId, BasicBlockId), String> {
    let mut scope = MapVars::new(vars);
    lower_expr_with_scope(env, f, cur_bb, e, &mut scope)
}

#[allow(dead_code)]
fn lower_args(
    env: &BridgeEnv,
    f: &mut MirFunction,
    cur_bb: BasicBlockId,
    args: &[ExprV0],
) -> Result<(Vec<ValueId>, BasicBlockId), String> {
    let mut scope = NoVars;
    lower_args_with_scope(env, f, cur_bb, args, &mut scope)
}

pub(super) fn lower_args_with_vars(
    env: &BridgeEnv,
    f: &mut MirFunction,
    cur_bb: BasicBlockId,
    args: &[ExprV0],
    vars: &mut HashMap<String, ValueId>,
) -> Result<(Vec<ValueId>, BasicBlockId), String> {
    let mut scope = MapVars::new(vars);
    lower_args_with_scope(env, f, cur_bb, args, &mut scope)
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
            let (v, cur) = lower_expr_with_vars(env, f, cur_bb, expr, vars)?;
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
            let (arg_ids, cur) = lower_args_with_vars(env, f, cur_bb, args, vars)?;
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
            let (_v, cur) = lower_expr_with_vars(env, f, cur_bb, expr, vars)?;
            Ok(cur)
        }
        StmtV0::Local { name, expr } => {
            let (v, cur) = lower_expr_with_vars(env, f, cur_bb, expr, vars)?;
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
