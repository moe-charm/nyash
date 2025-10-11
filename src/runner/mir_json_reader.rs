use serde_json::Value as J;
use crate::mir::{self, basic_block::BasicBlock, basic_block::BasicBlockId, function::{FunctionSignature, MirFunction, MirModule}, instruction::MirInstruction as I, types::{ConstValue as C, BinaryOp as B, CompareOp as Cmp, MirType}, ValueId};

fn as_u32(v: &J) -> Result<u32, String> { v.as_u64().map(|x| x as u32).ok_or_else(|| "expected u32".into()) }
fn as_i64(v: &J) -> Result<i64, String> { v.as_i64().ok_or_else(|| "expected i64".into()) }
fn as_str<'a>(v:&'a J)->Result<&'a str,String>{v.as_str().ok_or_else(||"expected string".into())}

fn parse_binop(s: &str) -> Result<B, String> {
    Ok(match s {
        "Add"=>B::Add, "Sub"=>B::Sub, "Mul"=>B::Mul, "Div"=>B::Div, "Mod"=>B::Mod,
        "BitAnd"=>B::BitAnd, "BitOr"=>B::BitOr, "BitXor"=>B::BitXor, "Shl"=>B::Shl, "Shr"=>B::Shr,
        "And"=>B::And, "Or"=>B::Or,
        other => return Err(format!("unsupported binop: {}", other))
    })
}
fn parse_cmp(s:&str)->Result<Cmp,String>{Ok(match s{ "Eq"=>Cmp::Eq,"Ne"=>Cmp::Ne,"Lt"=>Cmp::Lt,"Le"=>Cmp::Le,"Gt"=>Cmp::Gt,"Ge"=>Cmp::Ge, o=>return Err(format!("unsupported cmp: {}",o))})}

fn parse_const(val:&J)->Result<C,String>{
    let t = as_str(&val["type"])?.to_lowercase();
    match t.as_str(){
        "i64"|"int"|"integer"=> Ok(C::Integer(as_i64(&val["value"]) ?)),
        "bool"=> Ok(C::Bool(val["value"].as_bool().unwrap_or(false))),
        "string"=> Ok(C::String(as_str(&val["value"])?.to_string())),
        "void"=> Ok(C::Void),
        "null"=> Ok(C::Null),
        other=>Err(format!("unsupported const type: {}",other)),
    }
}

fn vid(x:u32)->ValueId{ ValueId::new(x) }
fn bbid(x:u32)->BasicBlockId{ BasicBlockId::new(x) }

fn parse_block(func:&mut MirFunction, f: &J) -> Result<(), String> {
    let id = as_u32(&f["id"]).unwrap_or(0);
    if id != func.entry_block.to_u32(){ func.add_block(BasicBlock::new(bbid(id))); }
    let blk = func.get_block_mut(bbid(id)).ok_or("missing block")?;
    if let Some(insts)=f.get("instructions").and_then(|x|x.as_array()){
        for inst in insts{
            let op = as_str(&inst["op"]).unwrap_or("");
            match op{
                "const"=>{
                    let dst = as_u32(&inst["dst"]) ?;
                    let cv = parse_const(&inst["value"]) ?;
                    blk.add_instruction(I::Const{dst:vid(dst), value:cv});
                }
                "binop"=>{
                    let dst=as_u32(&inst["dst"]) ?; let lhs=as_u32(&inst["lhs"]) ?; let rhs=as_u32(&inst["rhs"]) ?;
                    let kind=parse_binop(as_str(&inst["op_kind"])?)?;
                    blk.add_instruction(I::BinOp{dst:vid(dst), op:kind, lhs:vid(lhs), rhs:vid(rhs)});
                }
                "compare"=>{
                    let dst=as_u32(&inst["dst"]) ?; let lhs=as_u32(&inst["lhs"]) ?; let rhs=as_u32(&inst["rhs"]) ?;
                    let cmp=parse_cmp(as_str(&inst["cmp"])?)?;
                    blk.add_instruction(I::Compare{dst:vid(dst), op:cmp, lhs:vid(lhs), rhs:vid(rhs)});
                }
                "branch"=>{
                    let c=as_u32(&inst["cond"]) ?; let tb=as_u32(&inst["then"]) ?; let eb=as_u32(&inst["else"]) ?;
                    blk.add_instruction(I::Branch{condition:vid(c), then_bb:bbid(tb), else_bb:bbid(eb)});
                }
                "jump"=>{
                    let t=as_u32(&inst["target"]).unwrap_or(0);
                    blk.add_instruction(I::Jump{target:bbid(t)});
                }
                "ret"|"return"=>{
                    let v = inst.get("value").and_then(|x|x.as_u64()).map(|n| vid(n as u32));
                    blk.add_instruction(I::Return{ value: v });
                }
                other=>{
                    return Err(format!("unsupported op in reader: {}", other));
                }
            }
        }
    }
    Ok(())
}

/// Minimal MIR(JSON v0) reader (dev/optional): supports const/binop/compare/branch/jump/ret
pub fn parse_mir_json_v0_to_module(json: &str) -> Result<MirModule, String> {
    let v: J = serde_json::from_str(json).map_err(|e| format!("invalid MIR JSON: {}", e))?;
    if let Some(k)=v.get("kind").and_then(|x|x.as_str()){ if k != "MIR" { return Err("unsupported kind (expected MIR)".into()); } }
    let funs = v.get("functions").and_then(|x|x.as_array()).ok_or("missing functions")?;
    let mut module = MirModule::new("json_v0".into());
    for f in funs{
        let name = f.get("name").and_then(|x|x.as_str()).unwrap_or("main").to_string();
        let sig = FunctionSignature{ name: name.clone(), params: vec![], return_type: MirType::Integer, effects: mir::EffectMask::PURE };
        let mut func = MirFunction::new(sig, bbid(0));
        if let Some(blocks)=f.get("blocks").and_then(|x|x.as_array()){
            for b in blocks { parse_block(&mut func, b)?; }
        } else if let Some(insts)=f.get("instructions").and_then(|x|x.as_array()){
            parse_block(&mut func, &serde_json::json!({"id":0,"instructions":insts}))?;
        }
        module.add_function(func);
    }
    Ok(module)
}
