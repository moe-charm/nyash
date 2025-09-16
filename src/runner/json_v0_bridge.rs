use crate::mir::{
    BasicBlockId, BinaryOp, ConstValue, EffectMask, FunctionSignature, MirFunction, MirInstruction,
    MirModule, MirPrinter, MirType, ValueId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct ProgramV0 {
    version: i32,
    kind: String,
    body: Vec<StmtV0>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
enum StmtV0 {
    Return {
        expr: ExprV0,
    },
    Extern {
        iface: String,
        method: String,
        args: Vec<ExprV0>,
    },
    // Optional: expression statement (side effects only)
    Expr {
        expr: ExprV0,
    },
    // Optional: local binding (Stage-2)
    Local {
        name: String,
        expr: ExprV0,
    },
    // Optional: if/else (Stage-2)
    If {
        cond: ExprV0,
        then: Vec<StmtV0>,
        #[serde(rename = "else", default)]
        r#else: Option<Vec<StmtV0>>,
    },
    // Optional: loop (Stage-2)
    Loop {
        cond: ExprV0,
        body: Vec<StmtV0>,
    },
    Break,
    Continue,
    Try {
        #[serde(rename = "try")]
        try_body: Vec<StmtV0>,
        #[serde(default)]
        catches: Vec<CatchV0>,
        #[serde(default)]
        finally: Vec<StmtV0>,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
struct CatchV0 {
    #[serde(rename = "param", default)]
    param: Option<String>,
    #[serde(rename = "typeHint", default)]
    type_hint: Option<String>,
    #[serde(default)]
    body: Vec<StmtV0>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
enum ExprV0 {
    Int {
        value: serde_json::Value,
    },
    Str {
        value: String,
    },
    Bool {
        value: bool,
    },
    Binary {
        op: String,
        lhs: Box<ExprV0>,
        rhs: Box<ExprV0>,
    },
    Extern {
        iface: String,
        method: String,
        args: Vec<ExprV0>,
    },
    Compare {
        op: String,
        lhs: Box<ExprV0>,
        rhs: Box<ExprV0>,
    },
    Logical {
        op: String,
        lhs: Box<ExprV0>,
        rhs: Box<ExprV0>,
    }, // short-circuit: &&, || (or: "and"/"or")
    // Stage-2 additions (optional):
    Call {
        name: String,
        args: Vec<ExprV0>,
    },
    Method {
        recv: Box<ExprV0>,
        method: String,
        args: Vec<ExprV0>,
    },
    New {
        class: String,
        args: Vec<ExprV0>,
    },
    Var {
        name: String,
    },
    Throw {
        expr: Box<ExprV0>,
    },
}

#[derive(Clone, Copy)]
struct LoopContext {
    cond_bb: BasicBlockId,
    exit_bb: BasicBlockId,
}

fn lower_throw(
    f: &mut MirFunction,
    cur_bb: BasicBlockId,
    exception_value: ValueId,
) -> (ValueId, BasicBlockId) {
    if std::env::var("NYASH_BRIDGE_THROW_ENABLE").ok().as_deref() == Some("1") {
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

pub fn parse_json_v0_to_module(json: &str) -> Result<MirModule, String> {
    let prog: ProgramV0 =
        serde_json::from_str(json).map_err(|e| format!("invalid JSON v0: {}", e))?;
    if prog.version != 0 || prog.kind != "Program" {
        return Err("unsupported IR: expected {version:0, kind:\"Program\"}".into());
    }
    // Create module and main function
    let mut module = MirModule::new("ny_json_v0".into());
    let sig = FunctionSignature {
        name: "main".into(),
        params: vec![],
        return_type: MirType::Integer,
        effects: EffectMask::PURE,
    };
    let entry = BasicBlockId::new(0);
    let mut f = MirFunction::new(sig, entry);

    if prog.body.is_empty() {
        return Err("empty body".into());
    }

    // Variable map for simple locals (Stage-2; currently minimal)
    let mut var_map: std::collections::HashMap<String, crate::mir::ValueId> =
        std::collections::HashMap::new();
    let mut loop_stack: Vec<LoopContext> = Vec::new();
    let start_bb = f.entry_block;
    let end_bb =
        lower_stmt_list_with_vars(&mut f, start_bb, &prog.body, &mut var_map, &mut loop_stack)?;
    // Ensure function terminates: add `ret 0` to last un-terminated block (prefer end_bb else entry)
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
    // Keep return type unknown to allow dynamic display (VM/Interpreter)
    f.signature.return_type = MirType::Unknown;
    module.add_function(f);
    Ok(module)
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

fn lower_expr(
    f: &mut MirFunction,
    cur_bb: BasicBlockId,
    e: &ExprV0,
) -> Result<(crate::mir::ValueId, BasicBlockId), String> {
    match e {
        ExprV0::Int { value } => {
            // Accept number or stringified digits
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
            let (l, cur_after_l) = lower_expr(f, cur_bb, lhs)?;
            let (r, cur_after_r) = lower_expr(f, cur_after_l, rhs)?;
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
            let (arg_ids, cur2) = lower_args(f, cur_bb, args)?;
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
            let (l, cur_after_l) = lower_expr(f, cur_bb, lhs)?;
            let (r, cur_after_r) = lower_expr(f, cur_after_l, rhs)?;
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
            // Short-circuit boolean logic with branches (+phi or edge-copy)
            let (l, cur_after_l) = lower_expr(f, cur_bb, lhs)?;
            let rhs_bb = next_block_id(f);
            let fall_bb = BasicBlockId::new(rhs_bb.0 + 1);
            let merge_bb = BasicBlockId::new(rhs_bb.0 + 2);
            f.add_block(crate::mir::BasicBlock::new(rhs_bb));
            f.add_block(crate::mir::BasicBlock::new(fall_bb));
            f.add_block(crate::mir::BasicBlock::new(merge_bb));
            // Branch depending on op
            let is_and = matches!(op.as_str(), "&&" | "and");
            if let Some(bb) = f.get_block_mut(cur_after_l) {
                if is_and {
                    bb.set_terminator(MirInstruction::Branch {
                        condition: l,
                        then_bb: rhs_bb,
                        else_bb: fall_bb,
                    });
                } else {
                    // OR: if lhs true, go to fall_bb (true path), else evaluate rhs
                    bb.set_terminator(MirInstruction::Branch {
                        condition: l,
                        then_bb: fall_bb,
                        else_bb: rhs_bb,
                    });
                }
            }
            // Telemetry: note short-circuit lowering
            crate::jit::events::emit_lower(
                serde_json::json!({
                    "id": "shortcircuit",
                    "op": if is_and { "and" } else { "or" },
                    "rhs_bb": rhs_bb.0,
                    "fall_bb": fall_bb.0,
                    "merge_bb": merge_bb.0
                }),
                "shortcircuit",
                "<json_v0>",
            );
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!(
                    "[bridge/logical] op={} rhs_bb={} fall_bb={} merge_bb={}",
                    if is_and { "and" } else { "or" },
                    rhs_bb.0,
                    fall_bb.0,
                    merge_bb.0
                );
            }
            // false/true constant in fall_bb depending on op
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
            // evaluate rhs starting at rhs_bb and ensure the terminal block jumps to merge
            let (rval, rhs_end) = lower_expr(f, rhs_bb, rhs)?;
            if let Some(bb) = f.get_block_mut(rhs_end) {
                if !bb.is_terminated() {
                    bb.set_terminator(MirInstruction::Jump { target: merge_bb });
                }
            }
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!(
                    "[bridge/logical] rhs_end={} jump->merge_bb={}",
                    rhs_end.0, merge_bb.0
                );
            }
            // Merge: PHI または edge-copy で合流値を定義
            let no_phi = crate::config::env::mir_no_phi();
            let out = f.next_value_id();
            if no_phi {
                // Edge copies in predecessors
                if let Some(bb) = f.get_block_mut(fall_bb) {
                    bb.add_instruction(MirInstruction::Copy { dst: out, src: cdst });
                }
                if let Some(bb) = f.get_block_mut(rhs_end) {
                    bb.add_instruction(MirInstruction::Copy { dst: out, src: rval });
                }
            } else if let Some(bb) = f.get_block_mut(merge_bb) {
                let mut inputs: Vec<(BasicBlockId, ValueId)> = vec![(fall_bb, cdst)];
                if rhs_end != fall_bb {
                    inputs.push((rhs_end, rval));
                } else {
                    // Degenerate case: RHS ended in fall_bb (e.g., constant expression).
                    // Reuse the constant to keep PHI well-formed.
                    inputs.push((fall_bb, rval));
                }
                inputs.sort_by_key(|(bbid, _)| bbid.0);
                bb.insert_instruction_after_phis(MirInstruction::Phi { dst: out, inputs });
            }
            Ok((out, merge_bb))
        }
        ExprV0::Call { name, args } => {
            // Special: array literal lowering — Call{name:"array.of", args:[...]} → new ArrayBox(); push(...); result=array
            if name == "array.of" {
                // Create array first
                let arr = f.next_value_id();
                if let Some(bb) = f.get_block_mut(cur_bb) {
                    bb.add_instruction(MirInstruction::NewBox {
                        dst: arr,
                        box_type: "ArrayBox".into(),
                        args: vec![],
                    });
                }
                // For each element: eval then push
                let mut cur = cur_bb;
                for e in args {
                    let (v, c) = lower_expr(f, cur, e)?;
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
            // Special: map literal lowering — Call{name:"map.of", args:[k1, v1, k2, v2, ...]} → new MapBox(); set(k,v)...; result=map
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
                        let (kv, cur2) = lower_expr(f, cur, k)?;
                        cur = cur2;
                        let (vv, cur3) = lower_expr(f, cur, v)?;
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
            // Fallback: treat as normal dynamic call
            let (arg_ids, cur) = lower_args(f, cur_bb, args)?;
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
            // Heuristic: new ConsoleBox().println(x) → externcall env.console.log(x)
            let recv_is_console_new =
                matches!(&**recv, ExprV0::New { class, .. } if class == "ConsoleBox");
            if recv_is_console_new && (method == "println" || method == "print" || method == "log")
            {
                let (arg_ids, cur2) = lower_args(f, cur_bb, args)?;
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
            let (recv_v, cur) = lower_expr(f, cur_bb, recv)?;
            let (arg_ids, cur2) = lower_args(f, cur, args)?;
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
            let (arg_ids, cur) = lower_args(f, cur_bb, args)?;
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
        ExprV0::Var { name } => Err(format!("undefined variable in this context: {}", name)),
        ExprV0::Throw { expr } => {
            let (exc, cur) = lower_expr(f, cur_bb, expr)?;
            let (dst, cur) = lower_throw(f, cur, exc);
            Ok((dst, cur))
        }
    }
}

fn lower_expr_with_vars(
    f: &mut MirFunction,
    cur_bb: BasicBlockId,
    e: &ExprV0,
    vars: &mut std::collections::HashMap<String, crate::mir::ValueId>,
) -> Result<(crate::mir::ValueId, BasicBlockId), String> {
    match e {
        ExprV0::Var { name } => {
            if let Some(&vid) = vars.get(name) {
                return Ok((vid, cur_bb));
            }
            if name == "me" {
                // Optional gate: allow a dummy 'me' instance for Stage-2 JSON smoke
                if std::env::var("NYASH_BRIDGE_ME_DUMMY").ok().as_deref() == Some("1") {
                    let class = std::env::var("NYASH_BRIDGE_ME_CLASS")
                        .unwrap_or_else(|_| "Main".to_string());
                    let dst = f.next_value_id();
                    if let Some(bb) = f.get_block_mut(cur_bb) {
                        bb.add_instruction(MirInstruction::NewBox {
                            dst,
                            box_type: class,
                            args: vec![],
                        });
                    }
                    vars.insert("me".to_string(), dst);
                    return Ok((dst, cur_bb));
                } else {
                    return Err("undefined 'me' outside box context (set NYASH_BRIDGE_ME_DUMMY=1 to inject placeholder)".into());
                }
            }
            Err(format!("undefined variable: {}", name))
        }
        ExprV0::Throw { expr } => {
            let (exc, cur) = lower_expr_with_vars(f, cur_bb, expr, vars)?;
            let (dst, cur) = lower_throw(f, cur, exc);
            Ok((dst, cur))
        }
        ExprV0::Call { name, args } => {
            // Special: array literal lowering in vars context
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
                    let (v, c) = lower_expr_with_vars(f, cur, e, vars)?;
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
            // Special: map literal lowering in vars context
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
                        let (kv, cur2) = lower_expr_with_vars(f, cur, k, vars)?;
                        cur = cur2;
                        let (vv, cur3) = lower_expr_with_vars(f, cur, v, vars)?;
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
            // Lower args
            let (arg_ids, cur) = lower_args_with_vars(f, cur_bb, args, vars)?;
            // Encode as: const fun_name; call
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
                let (arg_ids, cur2) = lower_args_with_vars(f, cur_bb, args, vars)?;
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
            let (recv_v, cur) = lower_expr_with_vars(f, cur_bb, recv, vars)?;
            let (arg_ids, cur2) = lower_args_with_vars(f, cur, args, vars)?;
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
            let (arg_ids, cur) = lower_args_with_vars(f, cur_bb, args, vars)?;
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
        ExprV0::Binary { op, lhs, rhs } => {
            let (l, cur_after_l) = lower_expr_with_vars(f, cur_bb, lhs, vars)?;
            let (r, cur_after_r) = lower_expr_with_vars(f, cur_after_l, rhs, vars)?;
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
        ExprV0::Compare { op, lhs, rhs } => {
            let (l, cur_after_l) = lower_expr_with_vars(f, cur_bb, lhs, vars)?;
            let (r, cur_after_r) = lower_expr_with_vars(f, cur_after_l, rhs, vars)?;
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
            let (l, cur_after_l) = lower_expr_with_vars(f, cur_bb, lhs, vars)?;
            let rhs_bb = next_block_id(f);
            let fall_bb = BasicBlockId::new(rhs_bb.0 + 1);
            let merge_bb = BasicBlockId::new(rhs_bb.0 + 2);
            f.add_block(crate::mir::BasicBlock::new(rhs_bb));
            f.add_block(crate::mir::BasicBlock::new(fall_bb));
            f.add_block(crate::mir::BasicBlock::new(merge_bb));
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
            let (rval, rhs_end) = lower_expr_with_vars(f, rhs_bb, rhs, vars)?;
            if let Some(bb) = f.get_block_mut(rhs_end) {
                if !bb.is_terminated() {
                    bb.set_terminator(MirInstruction::Jump { target: merge_bb });
                }
            }
            let out = f.next_value_id();
            if let Some(bb) = f.get_block_mut(merge_bb) {
                bb.insert_instruction_after_phis(MirInstruction::Phi {
                    dst: out,
                    inputs: vec![(rhs_end, rval), (fall_bb, cdst)],
                });
            }
            Ok((out, merge_bb))
        }
        _ => lower_expr(f, cur_bb, e),
    }
}

fn lower_stmt_with_vars(
    f: &mut MirFunction,
    cur_bb: BasicBlockId,
    s: &StmtV0,
    vars: &mut std::collections::HashMap<String, crate::mir::ValueId>,
    loop_stack: &mut Vec<LoopContext>,
) -> Result<BasicBlockId, String> {
    match s {
        StmtV0::Return { expr } => {
            let (v, cur) = lower_expr_with_vars(f, cur_bb, expr, vars)?;
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
            let (arg_ids, cur) = lower_args_with_vars(f, cur_bb, args, vars)?;
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
            let (_v, cur) = lower_expr_with_vars(f, cur_bb, expr, vars)?;
            Ok(cur)
        }
        StmtV0::Local { name, expr } => {
            let (v, cur) = lower_expr_with_vars(f, cur_bb, expr, vars)?;
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
                    serde_json::json!({
                        "id": "loop_break",
                        "exit_bb": ctx.exit_bb.0,
                        "decision": "lower",
                    }),
                    "loop",
                    "<json_v0>",
                );
            } else if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!("[bridge/break] ignoring break outside loop context");
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
                    serde_json::json!({
                        "id": "loop_continue",
                        "cond_bb": ctx.cond_bb.0,
                        "decision": "lower",
                    }),
                    "loop",
                    "<json_v0>",
                );
            } else if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!("[bridge/continue] ignoring continue outside loop context");
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
                    lower_stmt_list_with_vars(f, cur_bb, try_body, &mut tmp_vars, loop_stack)?;
                if !finally.is_empty() {
                    next_bb =
                        lower_stmt_list_with_vars(f, next_bb, finally, &mut tmp_vars, loop_stack)?;
                }
                *vars = tmp_vars;
                return Ok(next_bb);
            }

            let base_vars = vars.clone();
            let try_bb = next_block_id(f);
            f.add_block(crate::mir::BasicBlock::new(try_bb));

            let catch_clause = &catches[0];
            let catch_bb = next_block_id(f);
            f.add_block(crate::mir::BasicBlock::new(catch_bb));

            let finally_bb = if !finally.is_empty() {
                let id = next_block_id(f);
                f.add_block(crate::mir::BasicBlock::new(id));
                Some(id)
            } else {
                None
            };

            let exit_bb = next_block_id(f);
            f.add_block(crate::mir::BasicBlock::new(exit_bb));

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
                lower_stmt_list_with_vars(f, try_bb, try_body, &mut try_vars, loop_stack)?;
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
            let merge_target = handler_target;

            if let Some(finally_block) = finally_bb {
                // ensure finally block exists before inserting phi
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
            // Lower condition first
            let (cval, cur) = lower_expr_with_vars(f, cur_bb, cond, vars)?;
            // Create then/else/merge blocks
            let then_bb = next_block_id(f);
            let else_bb = BasicBlockId::new(then_bb.0 + 1);
            let merge_bb = BasicBlockId::new(then_bb.0 + 2);
            f.add_block(crate::mir::BasicBlock::new(then_bb));
            f.add_block(crate::mir::BasicBlock::new(else_bb));
            f.add_block(crate::mir::BasicBlock::new(merge_bb));
            // Branch to then/else
            if let Some(bb) = f.get_block_mut(cur) {
                bb.set_terminator(MirInstruction::Branch {
                    condition: cval,
                    then_bb,
                    else_bb,
                });
            }

            // Clone current vars as branch-local maps
            let base_vars = vars.clone();
            let mut then_vars = base_vars.clone();
            let tend = lower_stmt_list_with_vars(f, then_bb, then, &mut then_vars, loop_stack)?;
            if let Some(bb) = f.get_block_mut(tend) {
                if !bb.is_terminated() {
                    bb.set_terminator(MirInstruction::Jump { target: merge_bb });
                }
            }

            let (else_end_pred, else_vars) = if let Some(elses) = r#else {
                let mut ev = base_vars.clone();
                let eend = lower_stmt_list_with_vars(f, else_bb, elses, &mut ev, loop_stack)?;
                if let Some(bb) = f.get_block_mut(eend) {
                    if !bb.is_terminated() {
                        bb.set_terminator(MirInstruction::Jump { target: merge_bb });
                    }
                }
                (eend, ev)
            } else {
                // No else: empty path falls through with base vars
                if let Some(bb) = f.get_block_mut(else_bb) {
                    bb.set_terminator(MirInstruction::Jump { target: merge_bb });
                }
                (else_bb, base_vars.clone())
            };

            // Merge at then/else predecessors（PHI or edge-copy）
            use std::collections::HashSet;
            let no_phi = crate::config::env::mir_no_phi();
            let mut names: HashSet<String> = base_vars.keys().cloned().collect();
            for k in then_vars.keys() { names.insert(k.clone()); }
            for k in else_vars.keys() { names.insert(k.clone()); }

            for name in names {
                let tv = then_vars.get(&name).copied();
                let ev = else_vars.get(&name).copied();
                let exists_base = base_vars.contains_key(&name);
                match (tv, ev, exists_base) {
                    (Some(tval), Some(eval), _) => {
                        let merged = if tval == eval { tval } else {
                            let dst = f.next_value_id();
                            if no_phi {
                                if let Some(bb) = f.get_block_mut(tend) { bb.add_instruction(MirInstruction::Copy { dst, src: tval }); }
                                if let Some(bb) = f.get_block_mut(else_end_pred) { bb.add_instruction(MirInstruction::Copy { dst, src: eval }); }
                            } else if let Some(bb) = f.get_block_mut(merge_bb) {
                                bb.insert_instruction_after_phis(MirInstruction::Phi { dst, inputs: vec![(tend, tval), (else_end_pred, eval)] });
                            }
                            dst
                        };
                        vars.insert(name, merged);
                    }
                    (Some(tval), None, true) => {
                        if let Some(&bval) = base_vars.get(&name) {
                            let merged = if tval == bval { tval } else {
                                let dst = f.next_value_id();
                                if no_phi {
                                    if let Some(bb) = f.get_block_mut(tend) { bb.add_instruction(MirInstruction::Copy { dst, src: tval }); }
                                    if let Some(bb) = f.get_block_mut(else_end_pred) { bb.add_instruction(MirInstruction::Copy { dst, src: bval }); }
                                } else if let Some(bb) = f.get_block_mut(merge_bb) {
                                    bb.insert_instruction_after_phis(MirInstruction::Phi { dst, inputs: vec![(tend, tval), (else_end_pred, bval)] });
                                }
                                dst
                            };
                            vars.insert(name, merged);
                        }
                    }
                    (None, Some(eval), true) => {
                        if let Some(&bval) = base_vars.get(&name) {
                            let merged = if eval == bval { eval } else {
                                let dst = f.next_value_id();
                                if no_phi {
                                    if let Some(bb) = f.get_block_mut(tend) { bb.add_instruction(MirInstruction::Copy { dst, src: bval }); }
                                    if let Some(bb) = f.get_block_mut(else_end_pred) { bb.add_instruction(MirInstruction::Copy { dst, src: eval }); }
                                } else if let Some(bb) = f.get_block_mut(merge_bb) {
                                    bb.insert_instruction_after_phis(MirInstruction::Phi { dst, inputs: vec![(tend, bval), (else_end_pred, eval)] });
                                }
                                dst
                            };
                            vars.insert(name, merged);
                        }
                    }
                    _ => {}
                }
            }

            Ok(merge_bb)
        }
        StmtV0::Loop { cond, body } => {
            // Create loop blocks
            let cond_bb = next_block_id(f);
            let body_bb = BasicBlockId::new(cond_bb.0 + 1);
            let exit_bb = BasicBlockId::new(cond_bb.0 + 2);
            f.add_block(crate::mir::BasicBlock::new(cond_bb));
            f.add_block(crate::mir::BasicBlock::new(body_bb));
            f.add_block(crate::mir::BasicBlock::new(exit_bb));

            // Preheader jump into cond
            if let Some(bb) = f.get_block_mut(cur_bb) {
                if !bb.is_terminated() {
                    bb.add_instruction(MirInstruction::Jump { target: cond_bb });
                }
            }

            // Snapshot base vars and set up merged ids for loop-carried vars
            let no_phi = crate::config::env::mir_no_phi();
            let base_vars = vars.clone();
            let orig_names: Vec<String> = base_vars.keys().cloned().collect();
            let mut phi_map: std::collections::HashMap<String, crate::mir::ValueId> =
                std::collections::HashMap::new();
            for name in &orig_names {
                if let Some(&bval) = base_vars.get(name) {
                    let dst = f.next_value_id();
                    if no_phi {
                        // Preheader edge-copy（cur_bb -> cond）
                        if let Some(bb) = f.get_block_mut(cur_bb) {
                            bb.add_instruction(MirInstruction::Copy { dst, src: bval });
                        }
                    } else if let Some(bb) = f.get_block_mut(cond_bb) {
                        // Initial incoming from preheader via PHI
                        bb.insert_instruction_after_phis(MirInstruction::Phi { dst, inputs: vec![(cur_bb, bval)] });
                    }
                    phi_map.insert(name.clone(), dst);
                }
            }
            // Redirect current vars to PHIs for use in cond/body
            for (name, &phi) in &phi_map {
                vars.insert(name.clone(), phi);
            }

            // Lower condition using phi-backed vars
            let (cval, _cend) = lower_expr_with_vars(f, cond_bb, cond, vars)?;
            if let Some(bb) = f.get_block_mut(cond_bb) {
                bb.set_terminator(MirInstruction::Branch {
                    condition: cval,
                    then_bb: body_bb,
                    else_bb: exit_bb,
                });
            }

            // Lower body; record end block and body-out vars
            let mut body_vars = vars.clone();
            loop_stack.push(LoopContext { cond_bb, exit_bb });
            let bend_res = lower_stmt_list_with_vars(f, body_bb, body, &mut body_vars, loop_stack);
            loop_stack.pop();
            let bend = bend_res?;
            if let Some(bb) = f.get_block_mut(bend) {
                if !bb.is_terminated() {
                    bb.set_terminator(MirInstruction::Jump { target: cond_bb });
                }
            }

            // Wire second incoming from latch (body end)
            let backedge_to_cond = matches!(
                f.blocks.get(&bend).and_then(|bb| bb.terminator.as_ref()),
                Some(MirInstruction::Jump { target, .. }) if *target == cond_bb
            );
            if backedge_to_cond {
                if no_phi {
                    // Latch edge-copy（bend -> cond）
                    for (name, &phi_dst) in &phi_map {
                        if let Some(&latch_val) = body_vars.get(name) {
                            if let Some(bb) = f.get_block_mut(bend) {
                                bb.add_instruction(MirInstruction::Copy { dst: phi_dst, src: latch_val });
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
            } else if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!(
                    "[bridge/loop] skipped latch bb{} -> cond bb{}",
                    bend.0, cond_bb.0
                );
            }

            // After the loop, keep vars mapped to the PHI values (current loop state)
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
    vars: &mut std::collections::HashMap<String, crate::mir::ValueId>,
    loop_stack: &mut Vec<LoopContext>,
) -> Result<BasicBlockId, String> {
    let mut cur = start_bb;
    for s in stmts {
        cur = lower_stmt_with_vars(f, cur, s, vars, loop_stack)?;
        if let Some(bb) = f.blocks.get(&cur) {
            if bb.is_terminated() {
                break;
            }
        }
    }
    Ok(cur)
}
fn lower_args_with_vars(
    f: &mut MirFunction,
    cur_bb: BasicBlockId,
    args: &[ExprV0],
    vars: &mut std::collections::HashMap<String, crate::mir::ValueId>,
) -> Result<(Vec<crate::mir::ValueId>, BasicBlockId), String> {
    let mut out = Vec::with_capacity(args.len());
    let mut cur = cur_bb;
    for a in args {
        let (v, c) = lower_expr_with_vars(f, cur, a, vars)?;
        out.push(v);
        cur = c;
    }
    Ok((out, cur))
}

fn lower_args(
    f: &mut MirFunction,
    cur_bb: BasicBlockId,
    args: &[ExprV0],
) -> Result<(Vec<crate::mir::ValueId>, BasicBlockId), String> {
    let mut out = Vec::with_capacity(args.len());
    let mut cur = cur_bb;
    for a in args {
        let (v, c) = lower_expr(f, cur, a)?;
        out.push(v);
        cur = c;
    }
    Ok((out, cur))
}

pub fn maybe_dump_mir(module: &MirModule) {
    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
        let p = MirPrinter::new();
        println!("{}", p.print_module(module));
    }
}

#[cfg(test)]
mod tests {
    use super::parse_json_v0_to_module;
    use crate::mir::{BasicBlockId, MirInstruction, MirModule};

    fn block_terminator(module: &MirModule, block: BasicBlockId) -> MirInstruction {
        module
            .get_function("main")
            .unwrap()
            .get_block(block)
            .and_then(|bb| bb.terminator.clone())
            .expect("terminator")
    }

    #[test]
    fn stage3_break_jumps_to_exit() {
        let json = "{\"version\":0,\"kind\":\"Program\",\"body\":[{\"type\":\"Loop\",\"cond\":{\"type\":\"Bool\",\"value\":true},\"body\":[{\"type\":\"Break\"}]},{\"type\":\"Return\",\"expr\":{\"type\":\"Int\",\"value\":0}}]}";
        let module = parse_json_v0_to_module(json).unwrap();
        match block_terminator(&module, BasicBlockId::new(2)) {
            MirInstruction::Jump { target, .. } => assert_eq!(target, BasicBlockId::new(3)),
            other => panic!("expected jump, got {:?}", other),
        }
        module.verify().unwrap();
    }

    #[test]
    fn stage3_continue_jumps_to_head() {
        let json = "{\"version\":0,\"kind\":\"Program\",\"body\":[{\"type\":\"Loop\",\"cond\":{\"type\":\"Bool\",\"value\":true},\"body\":[{\"type\":\"Continue\"}]},{\"type\":\"Return\",\"expr\":{\"type\":\"Int\",\"value\":0}}]}";
        let module = parse_json_v0_to_module(json).unwrap();
        match block_terminator(&module, BasicBlockId::new(2)) {
            MirInstruction::Jump { target, .. } => assert_eq!(target, BasicBlockId::new(1)),
            other => panic!("expected jump, got {:?}", other),
        }
        module.verify().unwrap();
    }
}

// ========== Direct bridge (source → JSON v0 → MIR) ==========

#[derive(Clone, Debug)]
enum Tok {
    Return,
    Int(i64),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Eof,
}

fn lex(input: &str) -> Result<Vec<Tok>, String> {
    let bytes = input.as_bytes();
    let mut i = 0usize;
    let n = bytes.len();
    let mut toks = Vec::new();
    while i < n {
        let c = bytes[i] as char;
        // Treat semicolon as whitespace (Stage-1 minimal ASI: optional ';')
        if c.is_whitespace() || c == ';' {
            i += 1;
            continue;
        }
        match c {
            '+' => {
                toks.push(Tok::Plus);
                i += 1;
            }
            '-' => {
                toks.push(Tok::Minus);
                i += 1;
            }
            '*' => {
                toks.push(Tok::Star);
                i += 1;
            }
            '/' => {
                toks.push(Tok::Slash);
                i += 1;
            }
            '(' => {
                toks.push(Tok::LParen);
                i += 1;
            }
            ')' => {
                toks.push(Tok::RParen);
                i += 1;
            }
            '0'..='9' => {
                let start = i;
                while i < n {
                    let cc = bytes[i] as char;
                    if cc.is_ascii_digit() {
                        i += 1;
                    } else {
                        break;
                    }
                }
                let s = std::str::from_utf8(&bytes[start..i]).unwrap();
                let v: i64 = s.parse().map_err(|_| "invalid int")?;
                toks.push(Tok::Int(v));
            }
            'r' => {
                // return
                if i + 6 <= n && &input[i..i + 6] == "return" {
                    toks.push(Tok::Return);
                    i += 6;
                } else {
                    return Err("unexpected 'r'".into());
                }
            }
            _ => return Err(format!("unexpected char '{}'", c)),
        }
    }
    toks.push(Tok::Eof);
    Ok(toks)
}

struct P {
    toks: Vec<Tok>,
    pos: usize,
}
impl P {
    fn new(toks: Vec<Tok>) -> Self {
        Self { toks, pos: 0 }
    }
    fn peek(&self) -> &Tok {
        self.toks.get(self.pos).unwrap()
    }
    fn next(&mut self) -> Tok {
        let t = self.toks.get(self.pos).unwrap().clone();
        self.pos += 1;
        t
    }
    fn expect_return(&mut self) -> Result<(), String> {
        match self.next() {
            Tok::Return => Ok(()),
            _ => Err("expected 'return'".into()),
        }
    }
    fn parse_program(&mut self) -> Result<ExprV0, String> {
        self.expect_return()?;
        self.parse_expr()
    }
    fn parse_expr(&mut self) -> Result<ExprV0, String> {
        let mut left = self.parse_term()?;
        loop {
            match self.peek() {
                Tok::Plus => {
                    self.next();
                    let r = self.parse_term()?;
                    left = ExprV0::Binary {
                        op: "+".into(),
                        lhs: Box::new(left),
                        rhs: Box::new(r),
                    };
                }
                Tok::Minus => {
                    self.next();
                    let r = self.parse_term()?;
                    left = ExprV0::Binary {
                        op: "-".into(),
                        lhs: Box::new(left),
                        rhs: Box::new(r),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }
    fn parse_term(&mut self) -> Result<ExprV0, String> {
        let mut left = self.parse_factor()?;
        loop {
            match self.peek() {
                Tok::Star => {
                    self.next();
                    let r = self.parse_factor()?;
                    left = ExprV0::Binary {
                        op: "*".into(),
                        lhs: Box::new(left),
                        rhs: Box::new(r),
                    };
                }
                Tok::Slash => {
                    self.next();
                    let r = self.parse_factor()?;
                    left = ExprV0::Binary {
                        op: "/".into(),
                        lhs: Box::new(left),
                        rhs: Box::new(r),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }
    fn parse_factor(&mut self) -> Result<ExprV0, String> {
        match self.next() {
            Tok::Int(v) => Ok(ExprV0::Int {
                value: serde_json::Value::from(v),
            }),
            Tok::LParen => {
                let e = self.parse_expr()?;
                match self.next() {
                    Tok::RParen => Ok(e),
                    _ => Err(") expected".into()),
                }
            }
            _ => Err("factor expected".into()),
        }
    }
}

pub fn parse_source_v0_to_json(input: &str) -> Result<String, String> {
    let toks = lex(input)?;
    let mut p = P::new(toks);
    let expr = p.parse_program()?;
    let prog = ProgramV0 {
        version: 0,
        kind: "Program".into(),
        body: vec![StmtV0::Return { expr }],
    };
    serde_json::to_string(&prog).map_err(|e| e.to_string())
}

pub fn parse_source_v0_to_module(input: &str) -> Result<MirModule, String> {
    let json = parse_source_v0_to_json(input)?;
    if std::env::var("NYASH_DUMP_JSON_IR").ok().as_deref() == Some("1") {
        println!("{}", json);
    }
    parse_json_v0_to_module(&json)
}
