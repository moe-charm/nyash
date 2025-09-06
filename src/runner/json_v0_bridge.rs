use serde::{Deserialize, Serialize};
use crate::mir::{
    MirModule, MirFunction, FunctionSignature, BasicBlockId, MirInstruction,
    ConstValue, BinaryOp, MirType, EffectMask, MirPrinter,
};

#[derive(Debug, Deserialize, Serialize)]
struct ProgramV0 {
    version: i32,
    kind: String,
    body: Vec<StmtV0>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
enum StmtV0 {
    Return { expr: ExprV0 },
    Extern { iface: String, method: String, args: Vec<ExprV0> },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
enum ExprV0 {
    Int { value: serde_json::Value },
    Str { value: String },
    Bool { value: bool },
    Binary { op: String, lhs: Box<ExprV0>, rhs: Box<ExprV0> },
    Extern { iface: String, method: String, args: Vec<ExprV0> },
    Compare { op: String, lhs: Box<ExprV0>, rhs: Box<ExprV0> },
    Logical { op: String, lhs: Box<ExprV0>, rhs: Box<ExprV0> }, // short-circuit: &&, || (or: "and"/"or")
}

pub fn parse_json_v0_to_module(json: &str) -> Result<MirModule, String> {
    let prog: ProgramV0 = serde_json::from_str(json).map_err(|e| format!("invalid JSON v0: {}", e))?;
    if prog.version != 0 || prog.kind != "Program" {
        return Err("unsupported IR: expected {version:0, kind:\"Program\"}".into());
    }
    // Create module and main function
    let mut module = MirModule::new("ny_json_v0".into());
    let sig = FunctionSignature { name: "main".into(), params: vec![], return_type: MirType::Integer, effects: EffectMask::PURE };
    let entry = BasicBlockId::new(0);
    let mut f = MirFunction::new(sig, entry);
    
    if prog.body.is_empty() { return Err("empty body".into()); }

    // Lower all statements; capture last expression for return when the last is Return
    let mut last_ret: Option<(crate::mir::ValueId, BasicBlockId)> = None;
    for (i, stmt) in prog.body.iter().enumerate() {
        match stmt {
            StmtV0::Extern { iface, method, args } => {
                // void extern call
                let entry_bb = f.entry_block;
                let (arg_ids, _cur) = lower_args(&mut f, entry_bb, args)?;
                if let Some(bb) = f.get_block_mut(entry) {
                    bb.add_instruction(MirInstruction::ExternCall { dst: None, iface_name: iface.clone(), method_name: method.clone(), args: arg_ids, effects: EffectMask::IO });
                }
                if i == prog.body.len()-1 { last_ret = None; }
            }
            StmtV0::Return { expr } => {
                let entry_bb = f.entry_block;
                last_ret = Some(lower_expr(&mut f, entry_bb, expr)?);
            }
        }
    }
    // Return last value (or 0)
    if let Some((rv, cur)) = last_ret {
        if let Some(bb) = f.get_block_mut(cur) {
            bb.set_terminator(MirInstruction::Return { value: Some(rv) });
        } else {
            return Err("invalid block when setting return".into());
        }
    } else {
        let dst_id = f.next_value_id();
        if let Some(bb) = f.get_block_mut(entry) {
            bb.add_instruction(MirInstruction::Const { dst: dst_id, value: ConstValue::Integer(0) });
            bb.set_terminator(MirInstruction::Return { value: Some(dst_id) });
        }
    }
    // Keep return type unknown to allow dynamic display (VM/Interpreter)
    f.signature.return_type = MirType::Unknown;
    module.add_function(f);
    Ok(module)
}

fn next_block_id(f: &MirFunction) -> BasicBlockId {
    let mut mx = 0u32;
    for k in f.blocks.keys() { if k.0 >= mx { mx = k.0 + 1; } }
    BasicBlockId::new(mx)
}

fn lower_expr(f: &mut MirFunction, cur_bb: BasicBlockId, e: &ExprV0) -> Result<(crate::mir::ValueId, BasicBlockId), String> {
    match e {
        ExprV0::Int { value } => {
            // Accept number or stringified digits
            let ival: i64 = if let Some(n) = value.as_i64() {
                n
            } else if let Some(s) = value.as_str() { s.parse().map_err(|_| "invalid int literal")? } else {
                return Err("invalid int literal".into());
            };
            let dst = f.next_value_id();
            if let Some(bb) = f.get_block_mut(cur_bb) {
                bb.add_instruction(MirInstruction::Const { dst, value: ConstValue::Integer(ival) });
            }
            Ok((dst, cur_bb))
        }
        ExprV0::Str { value } => {
            let dst = f.next_value_id();
            if let Some(bb) = f.get_block_mut(cur_bb) {
                bb.add_instruction(MirInstruction::Const { dst, value: ConstValue::String(value.clone()) });
            }
            Ok((dst, cur_bb))
        }
        ExprV0::Bool { value } => {
            let dst = f.next_value_id();
            if let Some(bb) = f.get_block_mut(cur_bb) {
                bb.add_instruction(MirInstruction::Const { dst, value: ConstValue::Bool(*value) });
            }
            Ok((dst, cur_bb))
        }
        ExprV0::Binary { op, lhs, rhs } => {
            let (l, cur_after_l) = lower_expr(f, cur_bb, lhs)?;
            let (r, cur_after_r) = lower_expr(f, cur_after_l, rhs)?;
            let bop = match op.as_str() { "+" => BinaryOp::Add, "-" => BinaryOp::Sub, "*" => BinaryOp::Mul, "/" => BinaryOp::Div, _ => return Err("unsupported op".into()) };
            let dst = f.next_value_id();
            if let Some(bb) = f.get_block_mut(cur_after_r) {
                bb.add_instruction(MirInstruction::BinOp { dst, op: bop, lhs: l, rhs: r });
            }
            Ok((dst, cur_after_r))
        }
        ExprV0::Extern { iface, method, args } => {
            let (arg_ids, cur2) = lower_args(f, cur_bb, args)?;
            let dst = f.next_value_id();
            if let Some(bb) = f.get_block_mut(cur2) {
                bb.add_instruction(MirInstruction::ExternCall { dst: Some(dst), iface_name: iface.clone(), method_name: method.clone(), args: arg_ids, effects: EffectMask::IO });
            }
            Ok((dst, cur2))
        }
        ExprV0::Compare { op, lhs, rhs } => {
            let (l, cur_after_l) = lower_expr(f, cur_bb, lhs)?;
            let (r, cur_after_r) = lower_expr(f, cur_after_l, rhs)?;
            let cop = match op.as_str() {
                "==" => crate::mir::CompareOp::Eq,
                "!=" => crate::mir::CompareOp::Ne,
                "<"  => crate::mir::CompareOp::Lt,
                "<=" => crate::mir::CompareOp::Le,
                ">"  => crate::mir::CompareOp::Gt,
                ">=" => crate::mir::CompareOp::Ge,
                _ => return Err("unsupported compare op".into()),
            };
            let dst = f.next_value_id();
            if let Some(bb) = f.get_block_mut(cur_after_r) {
                bb.add_instruction(MirInstruction::Compare { dst, op: cop, lhs: l, rhs: r });
            }
            Ok((dst, cur_after_r))
        }
        ExprV0::Logical { op, lhs, rhs } => {
            // Short-circuit boolean logic with branches + phi
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
                    bb.set_terminator(MirInstruction::Branch { condition: l, then_bb: rhs_bb, else_bb: fall_bb });
                } else {
                    // OR: if lhs true, go to fall_bb (true path), else evaluate rhs
                    bb.set_terminator(MirInstruction::Branch { condition: l, then_bb: fall_bb, else_bb: rhs_bb });
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
                "<json_v0>"
            );
            // false/true constant in fall_bb depending on op
            let cdst = f.next_value_id();
            if let Some(bb) = f.get_block_mut(fall_bb) {
                let cval = if is_and { ConstValue::Bool(false) } else { ConstValue::Bool(true) };
                bb.add_instruction(MirInstruction::Const { dst: cdst, value: cval });
                bb.set_terminator(MirInstruction::Jump { target: merge_bb });
            }
            // evaluate rhs in rhs_bb
            let (rval, _rhs_end) = lower_expr(f, rhs_bb, rhs)?;
            if let Some(bb) = f.get_block_mut(rhs_bb) {
                if !bb.is_terminated() { bb.set_terminator(MirInstruction::Jump { target: merge_bb }); }
            }
            // merge with phi
            let out = f.next_value_id();
            if let Some(bb) = f.get_block_mut(merge_bb) {
                bb.insert_instruction_after_phis(MirInstruction::Phi { dst: out, inputs: vec![(rhs_bb, rval), (fall_bb, cdst)] });
            }
            Ok((out, merge_bb))
        }
    }
}

fn lower_args(f: &mut MirFunction, cur_bb: BasicBlockId, args: &[ExprV0]) -> Result<(Vec<crate::mir::ValueId>, BasicBlockId), String> {
    let mut out = Vec::with_capacity(args.len());
    let mut cur = cur_bb;
    for a in args {
        let (v, c) = lower_expr(f, cur, a)?; out.push(v); cur = c;
    }
    Ok((out, cur))
}

pub fn maybe_dump_mir(module: &MirModule) {
    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
        let mut p = MirPrinter::new();
        println!("{}", p.print_module(module));
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
        if c.is_whitespace() { i += 1; continue; }
        match c {
            '+' => { toks.push(Tok::Plus); i+=1; }
            '-' => { toks.push(Tok::Minus); i+=1; }
            '*' => { toks.push(Tok::Star); i+=1; }
            '/' => { toks.push(Tok::Slash); i+=1; }
            '(' => { toks.push(Tok::LParen); i+=1; }
            ')' => { toks.push(Tok::RParen); i+=1; }
            '0'..='9' => {
                let start = i; while i<n { let cc = bytes[i] as char; if cc.is_ascii_digit() { i+=1; } else { break; } }
                let s = std::str::from_utf8(&bytes[start..i]).unwrap();
                let v: i64 = s.parse().map_err(|_| "invalid int")?;
                toks.push(Tok::Int(v));
            }
            'r' => {
                // return
                if i+6<=n && &input[i..i+6]=="return" { toks.push(Tok::Return); i+=6; } else { return Err("unexpected 'r'".into()); }
            }
            _ => return Err(format!("unexpected char '{}'", c)),
        }
    }
    toks.push(Tok::Eof);
    Ok(toks)
}

struct P { toks: Vec<Tok>, pos: usize }
impl P {
    fn new(toks: Vec<Tok>) -> Self { Self{ toks, pos:0 } }
    fn peek(&self) -> &Tok { self.toks.get(self.pos).unwrap() }
    fn next(&mut self) -> Tok { let t = self.toks.get(self.pos).unwrap().clone(); self.pos+=1; t }
    fn expect_return(&mut self) -> Result<(), String> { match self.next() { Tok::Return => Ok(()), _ => Err("expected 'return'".into()) } }
    fn parse_program(&mut self) -> Result<ExprV0, String> { self.expect_return()?; self.parse_expr() }
    fn parse_expr(&mut self) -> Result<ExprV0,String> {
        let mut left = self.parse_term()?;
        loop { match self.peek() { Tok::Plus => { self.next(); let r=self.parse_term()?; left = ExprV0::Binary{op:"+".into(), lhs:Box::new(left), rhs:Box::new(r)}; }, Tok::Minus => { self.next(); let r=self.parse_term()?; left = ExprV0::Binary{op:"-".into(), lhs:Box::new(left), rhs:Box::new(r)}; }, _ => break }
        }
        Ok(left)
    }
    fn parse_term(&mut self) -> Result<ExprV0,String> {
        let mut left = self.parse_factor()?;
        loop { match self.peek() { Tok::Star => { self.next(); let r=self.parse_factor()?; left = ExprV0::Binary{op:"*".into(), lhs:Box::new(left), rhs:Box::new(r)}; }, Tok::Slash => { self.next(); let r=self.parse_factor()?; left = ExprV0::Binary{op:"/".into(), lhs:Box::new(left), rhs:Box::new(r)}; }, _ => break }
        }
        Ok(left)
    }
    fn parse_factor(&mut self) -> Result<ExprV0,String> {
        match self.next() {
            Tok::Int(v) => Ok(ExprV0::Int{ value: serde_json::Value::from(v) }),
            Tok::LParen => { let e = self.parse_expr()?; match self.next() { Tok::RParen => Ok(e), _ => Err(") expected".into()) } }
            _ => Err("factor expected".into()),
        }
    }
}

pub fn parse_source_v0_to_json(input: &str) -> Result<String, String> {
    let toks = lex(input)?; let mut p = P::new(toks);
    let expr = p.parse_program()?;
    let prog = ProgramV0 { version:0, kind: "Program".into(), body: vec![StmtV0::Return{ expr }] };
    serde_json::to_string(&prog).map_err(|e| e.to_string())
}

pub fn parse_source_v0_to_module(input: &str) -> Result<MirModule, String> {
    let json = parse_source_v0_to_json(input)?;
    if std::env::var("NYASH_DUMP_JSON_IR").ok().as_deref() == Some("1") { println!("{}", json); }
    parse_json_v0_to_module(&json)
}
