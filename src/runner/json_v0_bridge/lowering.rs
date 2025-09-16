use super::ast::{CatchV0, ExprV0, ProgramV0, StmtV0};
use crate::mir::{
    BasicBlock, BasicBlockId, BinaryOp, ConstValue, EffectMask, FunctionSignature, MirFunction,
    MirInstruction, MirModule, MirPrinter, MirType, ValueId,
};
use std::collections::HashMap;
use std::collections::HashSet;

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

fn next_block_id(f: &MirFunction) -> BasicBlockId {
    let mut mx = 0u32;
    for k in f.blocks.keys() {
        if k.0 >= mx {
            mx = k.0 + 1;
        }
    }
    BasicBlockId::new(mx)
}

/// Create a fresh basic block and insert it into the function.
fn new_block(f: &mut MirFunction) -> BasicBlockId {
    let id = next_block_id(f);
    f.add_block(BasicBlock::new(id));
    id
}

/// Merge two incoming values either by inserting Copy on predecessor edges
/// (no_phi mode) or by adding a Phi at the merge block head.
fn merge_values(
    f: &mut MirFunction,
    no_phi: bool,
    merge_bb: BasicBlockId,
    pred_a: BasicBlockId,
    val_a: ValueId,
    pred_b: BasicBlockId,
    val_b: ValueId,
) -> ValueId {
    if val_a == val_b {
        return val_a;
    }
    let dst = f.next_value_id();
    if no_phi {
        if let Some(bb) = f.get_block_mut(pred_a) {
            bb.add_instruction(MirInstruction::Copy { dst, src: val_a });
        }
        if let Some(bb) = f.get_block_mut(pred_b) {
            bb.add_instruction(MirInstruction::Copy { dst, src: val_b });
        }
    } else if let Some(bb) = f.get_block_mut(merge_bb) {
        bb.insert_instruction_after_phis(MirInstruction::Phi { dst, inputs: vec![(pred_a, val_a), (pred_b, val_b)] });
    }
    dst
}

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

fn lower_expr_with_vars(
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

fn lower_args_with_vars(
    env: &BridgeEnv,
    f: &mut MirFunction,
    cur_bb: BasicBlockId,
    args: &[ExprV0],
    vars: &mut HashMap<String, ValueId>,
) -> Result<(Vec<ValueId>, BasicBlockId), String> {
    let mut scope = MapVars::new(vars);
    lower_args_with_scope(env, f, cur_bb, args, &mut scope)
}

fn lower_stmt_with_vars(
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
            let try_enabled = std::env::var("NYASH_BRIDGE_TRY_ENABLE").ok().as_deref() == Some("1");
            if !try_enabled || catches.is_empty() || catches.len() > 1 {
                let mut tmp_vars = vars.clone();
                let mut next_bb =
                    lower_stmt_list_with_vars(f, cur_bb, try_body, &mut tmp_vars, loop_stack, env)?;
                if !finally.is_empty() {
                    next_bb = lower_stmt_list_with_vars(
                        f,
                        next_bb,
                        finally,
                        &mut tmp_vars,
                        loop_stack,
                        env,
                    )?;
                }
                *vars = tmp_vars;
                return Ok(next_bb);
            }

            let base_vars = vars.clone();
            let try_bb = new_block(f);
            let catch_clause = &catches[0];
            let catch_bb = new_block(f);
            let finally_bb = if !finally.is_empty() {
                let id = new_block(f);
                Some(id)
            } else {
                None
            };
            let exit_bb = new_block(f);
            let handler_target = finally_bb.unwrap_or(exit_bb);
            let exception_value = f.next_value_id();
            if let Some(bb) = f.get_block_mut(cur_bb) {
                bb.add_instruction(MirInstruction::Catch {
                    exception_type: catch_clause.type_hint.clone(),
                    exception_value,
                    handler_bb: catch_bb,
                });
                bb.set_terminator(MirInstruction::Jump { target: try_bb });
            }
            let mut try_vars = vars.clone();
            let try_end =
                lower_stmt_list_with_vars(f, try_bb, try_body, &mut try_vars, loop_stack, env)?;
            if let Some(bb) = f.get_block_mut(try_end) {
                if !bb.is_terminated() {
                    bb.set_terminator(MirInstruction::Jump {
                        target: handler_target,
                    });
                }
            }
            let try_branch_vars = try_vars.clone();

            let mut catch_vars = base_vars.clone();
            if let Some(param) = &catch_clause.param {
                catch_vars.insert(param.clone(), exception_value);
            }
            let catch_end = lower_stmt_list_with_vars(
                f,
                catch_bb,
                &catch_clause.body,
                &mut catch_vars,
                loop_stack,
                env,
            )?;
            if let Some(param) = &catch_clause.param {
                catch_vars.remove(param);
            }
            if let Some(bb) = f.get_block_mut(catch_end) {
                if !bb.is_terminated() {
                    bb.set_terminator(MirInstruction::Jump {
                        target: handler_target,
                    });
                }
            }
            let catch_branch_vars = catch_vars.clone();

            use std::collections::HashSet;
            let mut branch_vars = vec![(try_end, try_branch_vars), (catch_end, catch_branch_vars)];
            if let Some(finally_block) = finally_bb {
                let names: HashSet<String> = {
                    let mut set: HashSet<String> = base_vars.keys().cloned().collect();
                    for (_, map) in &branch_vars {
                        set.extend(map.keys().cloned());
                    }
                    set
                };
                let mut merged_vars = base_vars.clone();
                let mut phi_entries: Vec<(ValueId, Vec<(BasicBlockId, ValueId)>)> = Vec::new();
                for name in names {
                    let mut inputs: Vec<(BasicBlockId, ValueId)> = Vec::new();
                    for (bbid, map) in &branch_vars {
                        if let Some(&val) = map.get(&name) {
                            inputs.push((*bbid, val));
                        }
                    }
                    if inputs.is_empty() {
                        if let Some(&base_val) = base_vars.get(&name) {
                            merged_vars.insert(name.clone(), base_val);
                        }
                        continue;
                    }
                    let unique: HashSet<ValueId> = inputs.iter().map(|(_, v)| *v).collect();
                    if unique.len() == 1 {
                        merged_vars.insert(name.clone(), inputs[0].1);
                        continue;
                    }
                    let dst = f.next_value_id();
                    inputs.sort_by_key(|(bbid, _)| bbid.0);
                    phi_entries.push((dst, inputs));
                    merged_vars.insert(name.clone(), dst);
                }
                if let Some(bb) = f.get_block_mut(finally_block) {
                    for (dst, inputs) in phi_entries {
                        bb.insert_instruction_after_phis(MirInstruction::Phi { dst, inputs });
                    }
                }
                let mut finally_vars = merged_vars.clone();
                let final_end = lower_stmt_list_with_vars(
                    f,
                    finally_block,
                    finally,
                    &mut finally_vars,
                    loop_stack,
                    env,
                )?;
                if let Some(bb) = f.get_block_mut(final_end) {
                    if !bb.is_terminated() {
                        bb.set_terminator(MirInstruction::Jump { target: exit_bb });
                    }
                }
                *vars = finally_vars;
                Ok(exit_bb)
            } else {
                let names: HashSet<String> = {
                    let mut set: HashSet<String> = base_vars.keys().cloned().collect();
                    for (_, map) in &branch_vars {
                        set.extend(map.keys().cloned());
                    }
                    set
                };
                let mut merged_vars = base_vars.clone();
                let mut phi_entries: Vec<(ValueId, Vec<(BasicBlockId, ValueId)>)> = Vec::new();
                for name in names {
                    let mut inputs: Vec<(BasicBlockId, ValueId)> = Vec::new();
                    for (bbid, map) in &branch_vars {
                        if let Some(&val) = map.get(&name) {
                            inputs.push((*bbid, val));
                        }
                    }
                    if inputs.is_empty() {
                        if let Some(&base_val) = base_vars.get(&name) {
                            merged_vars.insert(name.clone(), base_val);
                        }
                        continue;
                    }
                    let unique: HashSet<ValueId> = inputs.iter().map(|(_, v)| *v).collect();
                    if unique.len() == 1 {
                        merged_vars.insert(name.clone(), inputs[0].1);
                        continue;
                    }
                    let dst = f.next_value_id();
                    inputs.sort_by_key(|(bbid, _)| bbid.0);
                    phi_entries.push((dst, inputs));
                    merged_vars.insert(name.clone(), dst);
                }
                if let Some(bb) = f.get_block_mut(exit_bb) {
                    for (dst, inputs) in phi_entries {
                        bb.insert_instruction_after_phis(MirInstruction::Phi { dst, inputs });
                    }
                }
                *vars = merged_vars;
                Ok(exit_bb)
            }
        }
        StmtV0::If { cond, then, r#else } => {
            let (cval, cur) = lower_expr_with_vars(env, f, cur_bb, cond, vars)?;
            let then_bb = next_block_id(f);
            let else_bb = BasicBlockId::new(then_bb.0 + 1);
            let merge_bb = BasicBlockId::new(then_bb.0 + 2);
            f.add_block(BasicBlock::new(then_bb));
            f.add_block(BasicBlock::new(else_bb));
            f.add_block(BasicBlock::new(merge_bb));
            if let Some(bb) = f.get_block_mut(cur) {
                bb.set_terminator(MirInstruction::Branch {
                    condition: cval,
                    then_bb,
                    else_bb,
                });
            }
            let base_vars = vars.clone();
            let mut then_vars = base_vars.clone();
            let tend =
                lower_stmt_list_with_vars(f, then_bb, then, &mut then_vars, loop_stack, env)?;
            if let Some(bb) = f.get_block_mut(tend) {
                if !bb.is_terminated() {
                    bb.set_terminator(MirInstruction::Jump { target: merge_bb });
                }
            }
            let (else_end_pred, else_vars) = if let Some(elses) = r#else {
                let mut ev = base_vars.clone();
                let eend = lower_stmt_list_with_vars(f, else_bb, elses, &mut ev, loop_stack, env)?;
                if let Some(bb) = f.get_block_mut(eend) {
                    if !bb.is_terminated() {
                        bb.set_terminator(MirInstruction::Jump { target: merge_bb });
                    }
                }
                (eend, ev)
            } else {
                if let Some(bb) = f.get_block_mut(else_bb) {
                    bb.set_terminator(MirInstruction::Jump { target: merge_bb });
                }
                (else_bb, base_vars.clone())
            };
            let no_phi = env.mir_no_phi;
            let mut names: HashSet<String> = base_vars.keys().cloned().collect();
            for k in then_vars.keys() {
                names.insert(k.clone());
            }
            for k in else_vars.keys() {
                names.insert(k.clone());
            }
            for name in names {
                let tv = then_vars.get(&name).copied();
                let ev = else_vars.get(&name).copied();
                let exists_base = base_vars.contains_key(&name);
                match (tv, ev, exists_base) {
                    (Some(tval), Some(eval), _) => {
                        let merged = merge_values(f, no_phi, merge_bb, tend, tval, else_end_pred, eval);
                        vars.insert(name, merged);
                    }
                    (Some(tval), None, true) => {
                        if let Some(&bval) = base_vars.get(&name) {
                            let merged = merge_values(f, no_phi, merge_bb, tend, tval, else_end_pred, bval);
                            vars.insert(name, merged);
                        }
                    }
                    (None, Some(eval), true) => {
                        if let Some(&bval) = base_vars.get(&name) {
                            let merged = merge_values(f, no_phi, merge_bb, tend, bval, else_end_pred, eval);
                            vars.insert(name, merged);
                        }
                    }
                    _ => {}
                }
            }
            Ok(merge_bb)
        }
        StmtV0::Loop { cond, body } => {
            let cond_bb = new_block(f);
            let body_bb = new_block(f);
            let exit_bb = new_block(f);
            if let Some(bb) = f.get_block_mut(cur_bb) {
                if !bb.is_terminated() {
                    bb.add_instruction(MirInstruction::Jump { target: cond_bb });
                }
            }
            let no_phi = env.mir_no_phi;
            let base_vars = vars.clone();
            let orig_names: Vec<String> = base_vars.keys().cloned().collect();
            let mut phi_map: HashMap<String, ValueId> = HashMap::new();
            for name in &orig_names {
                if let Some(&bval) = base_vars.get(name) {
                    let dst = f.next_value_id();
                    if no_phi {
                        if let Some(bb) = f.get_block_mut(cur_bb) {
                            bb.add_instruction(MirInstruction::Copy { dst, src: bval });
                        }
                    } else if let Some(bb) = f.get_block_mut(cond_bb) {
                        bb.insert_instruction_after_phis(MirInstruction::Phi {
                            dst,
                            inputs: vec![(cur_bb, bval)],
                        });
                    }
                    phi_map.insert(name.clone(), dst);
                }
            }
            for (name, &phi) in &phi_map {
                vars.insert(name.clone(), phi);
            }
            let (cval, _cend) = lower_expr_with_vars(env, f, cond_bb, cond, vars)?;
            if let Some(bb) = f.get_block_mut(cond_bb) {
                bb.set_terminator(MirInstruction::Branch {
                    condition: cval,
                    then_bb: body_bb,
                    else_bb: exit_bb,
                });
            }
            let mut body_vars = vars.clone();
            loop_stack.push(LoopContext { cond_bb, exit_bb });
            let bend_res =
                lower_stmt_list_with_vars(f, body_bb, body, &mut body_vars, loop_stack, env);
            loop_stack.pop();
            let bend = bend_res?;
            if let Some(bb) = f.get_block_mut(bend) {
                if !bb.is_terminated() {
                    bb.set_terminator(MirInstruction::Jump { target: cond_bb });
                }
            }
            let backedge_to_cond = matches!(f.blocks.get(&bend).and_then(|bb| bb.terminator.as_ref()), Some(MirInstruction::Jump { target, .. }) if *target == cond_bb);
            if backedge_to_cond {
                if no_phi {
                    for (name, &phi_dst) in &phi_map {
                        if let Some(&latch_val) = body_vars.get(name) {
                            if let Some(bb) = f.get_block_mut(bend) {
                                bb.add_instruction(MirInstruction::Copy {
                                    dst: phi_dst,
                                    src: latch_val,
                                });
                            }
                        }
                    }
                } else if let Some(bb) = f.get_block_mut(cond_bb) {
                    for (name, &phi_dst) in &phi_map {
                        if let Some(&latch_val) = body_vars.get(name) {
                            for inst in &mut bb.instructions {
                                if let MirInstruction::Phi { dst, inputs } = inst {
                                    if *dst == phi_dst {
                                        inputs.push((bend, latch_val));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            for (name, &phi) in &phi_map {
                vars.insert(name.clone(), phi);
            }
            Ok(exit_bb)
        }
    }
}

fn lower_stmt_list_with_vars(
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
