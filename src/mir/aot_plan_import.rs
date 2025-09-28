//! AOT-Plan v1 → MIR13 importer (Phase 15.1)
//! Feature-gated behind `aot-plan-import`.

use crate::mir::{
    BasicBlockId, ConstValue, EffectMask, FunctionSignature, MirFunction, MirModule, MirType,
};
use crate::mir::function_emission as femit;

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
struct PlanParam {
    name: String,
    r#type: Option<String>,
}

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
    if let Some(i) = v.as_i64() {
        return Some(ConstValue::Integer(i));
    }
    if let Some(b) = v.as_bool() {
        return Some(ConstValue::Bool(b));
    }
    if let Some(f) = v.as_f64() {
        return Some(ConstValue::Float(f));
    }
    if let Some(s) = v.as_str() {
        return Some(ConstValue::String(s.to_string()));
    }
    None
}

/// Import a v1 plan JSON string into a MIR13 module with skeleton bodies.
pub fn import_from_str(plan_json: &str) -> Result<MirModule, String> {
    let plan: PlanV1 =
        serde_json::from_str(plan_json).map_err(|e| format!("invalid plan json: {}", e))?;
    if plan.version != "1" {
        return Err("unsupported plan version".into());
    }
    let mut module = MirModule::new(plan.name.unwrap_or_else(|| "aot_plan".into()));

    for f in plan.functions.iter() {
        // Signatures: keep types minimal; params exist but VM uses stackless calling for main
        let ret_ty = map_type(f.return_type.as_deref());
        let sig = FunctionSignature {
            name: f.name.clone(),
            params: vec![],
            return_type: ret_ty.clone(),
            effects: EffectMask::PURE,
        };
        let mut mf = MirFunction::new(sig, BasicBlockId::new(0));
        let bb = mf.entry_block;
        // Body lowering (skeleton)
        match &f.body {
            Some(PlanBody::ConstReturn { value }) => {
                let cst = const_from_json(value)
                    .ok_or_else(|| format!("unsupported const value in {}", f.name))?;
                let dst = match cst {
                    ConstValue::Integer(i) => femit::emit_const_integer(&mut mf, bb, i),
                    ConstValue::Bool(b) => femit::emit_const_bool(&mut mf, bb, b),
                    ConstValue::Float(fl) => {
                        // function_emission currently has no float helper; use manual emit via integer as placeholder is wrong.
                        // Fall back to direct const emission inline here to avoid adding a new helper unnecessarily.
                        let d = mf.next_value_id();
                        if let Some(block) = mf.get_block_mut(bb) {
                            block.add_instruction(crate::mir::MirInstruction::Const { dst: d, value: ConstValue::Float(fl) });
                        }
                        d
                    }
                    ConstValue::String(s) => femit::emit_const_string(&mut mf, bb, s),
                    other => {
                        // Null/Void are not expected in PlanBody::ConstReturn; still handle gracefully.
                        let d = mf.next_value_id();
                        if let Some(block) = mf.get_block_mut(bb) {
                            block.add_instruction(crate::mir::MirInstruction::Const { dst: d, value: other });
                        }
                        d
                    }
                };
                femit::emit_return_value(&mut mf, bb, dst);
                // If return_type is unspecified, keep Unknown to allow VM dynamic display; otherwise retain declared type
                if matches!(ret_ty, MirType::Unknown) { /* keep Unknown */ }
            }
            Some(PlanBody::Empty) | None => {
                // Return default 0 for display stability; mark signature Unknown for runtime display parity
                let dst = femit::emit_const_integer(&mut mf, bb, 0);
                femit::emit_return_value(&mut mf, bb, dst);
                mf.signature.return_type = MirType::Unknown;
            }
        }
        module.add_function(mf);
    }
    Ok(module)
}
