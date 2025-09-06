//! AOT-Plan v1 → MIR13 importer (Phase 15.1)
//! Feature-gated behind `aot-plan-import`.

use crate::mir::{MirModule, MirFunction, FunctionSignature, BasicBlockId, MirInstruction, EffectMask, MirType, ConstValue};

#[derive(Debug, serde::Deserialize)]
struct PlanV1 {
    version: String, // "1"
    name: Option<String>,
    functions: Vec<PlanFunction>,
}

#[derive(Debug, serde::Deserialize)]
struct PlanFunction {
    name: String,
    #[serde(default)]
    params: Vec<PlanParam>,
    return_type: Option<String>, // "integer" | "string" | ...
    #[serde(default)]
    body: Option<PlanBody>,
}

#[derive(Debug, serde::Deserialize)]
struct PlanParam { name: String, r#type: Option<String> }

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "kind")]
enum PlanBody {
    #[serde(rename = "const_return")]
    ConstReturn { value: serde_json::Value },
    #[serde(rename = "empty")]
    Empty,
}

fn map_type(s: Option<&str>) -> MirType {
    match s.unwrap_or("") {
        "integer" => MirType::Integer,
        "float" => MirType::Float,
        "bool" => MirType::Bool,
        "string" => MirType::String,
        "void" => MirType::Void,
        _ => MirType::Unknown,
    }
}

fn const_from_json(v: &serde_json::Value) -> Option<ConstValue> {
    if let Some(i) = v.as_i64() { return Some(ConstValue::Integer(i)); }
    if let Some(b) = v.as_bool() { return Some(ConstValue::Bool(b)); }
    if let Some(f) = v.as_f64() { return Some(ConstValue::Float(f)); }
    if let Some(s) = v.as_str() { return Some(ConstValue::String(s.to_string())); }
    None
}

/// Import a v1 plan JSON string into a MIR13 module with skeleton bodies.
pub fn import_from_str(plan_json: &str) -> Result<MirModule, String> {
    let plan: PlanV1 = serde_json::from_str(plan_json).map_err(|e| format!("invalid plan json: {}", e))?;
    if plan.version != "1" { return Err("unsupported plan version".into()); }
    let mut module = MirModule::new(plan.name.unwrap_or_else(|| "aot_plan".into()));

    for f in plan.functions.iter() {
        // Signatures: keep types minimal; params exist but VM uses stackless calling for main
        let ret_ty = map_type(f.return_type.as_deref());
        let sig = FunctionSignature { name: f.name.clone(), params: vec![], return_type: ret_ty.clone(), effects: EffectMask::PURE };
        let mut mf = MirFunction::new(sig, BasicBlockId::new(0));
        let bb = mf.entry_block;
        // Body lowering (skeleton)
        match &f.body {
            Some(PlanBody::ConstReturn { value }) => {
                let dst = mf.next_value_id();
                let cst = const_from_json(value).ok_or_else(|| format!("unsupported const value in {}", f.name))?;
                if let Some(b) = mf.get_block_mut(bb) { b.add_instruction(MirInstruction::Const { dst, value: cst }); b.set_terminator(MirInstruction::Return { value: Some(dst) }); }
                // If return_type is unspecified, set Unknown to allow VM dynamic display
                // Otherwise retain declared type
                if matches!(ret_ty, MirType::Unknown) { /* keep Unknown */ }
            }
            Some(PlanBody::Empty) | None => {
                // Return void or default 0 for integer; choose Unknown for display stability
                let dst = mf.next_value_id();
                if let Some(b) = mf.get_block_mut(bb) { b.add_instruction(MirInstruction::Const { dst, value: ConstValue::Integer(0) }); b.set_terminator(MirInstruction::Return { value: Some(dst) }); }
                mf.signature.return_type = MirType::Unknown;
            }
        }
        module.add_function(mf);
    }
    Ok(module)
}

