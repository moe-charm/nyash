use super::*;

// Object field get/set bridging for InstanceBox and legacy object field storage.
// Extracted from boxes.rs (behavior unchanged).
pub(super) fn try_handle_object_fields(
    interp: &mut MirInterpreter,
    dst: Option<ValueId>,
    box_val: ValueId,
    method: &str,
    args: &[ValueId],
) -> Result<bool, VMError> {
    fn vm_to_nv(v: &VMValue) -> crate::value::NyashValue {
        use crate::value::NyashValue as NV;
        use super::VMValue as VV;
        match v {
            VV::Integer(i) => NV::Integer(*i),
            VV::Float(f) => NV::Float(*f),
            VV::Bool(b) => NV::Bool(*b),
            VV::String(s) => NV::String(s.clone()),
            VV::Void => NV::Void,
            VV::Future(_) => NV::Void,
            VV::BoxRef(_) => NV::Void,
        }
    }
    fn nv_to_vm(v: &crate::value::NyashValue) -> VMValue {
        use crate::value::NyashValue as NV;
        use super::VMValue as VV;
        match v {
            NV::Integer(i) => VV::Integer(*i),
            NV::Float(f) => VV::Float(*f),
            NV::Bool(b) => VV::Bool(*b),
            NV::String(s) => VV::String(s.clone()),
            NV::Null | NV::Void => VV::Void,
            NV::Array(_) | NV::Map(_) | NV::Box(_) | NV::WeakBox(_) => VV::Void,
        }
    }

    match method {
        "getField" => {
            if args.len() != 1 {
                return Err(VMError::InvalidInstruction("getField expects 1 arg".into()));
            }
            if std::env::var("NYASH_VM_TRACE").ok().as_deref() == Some("1") {
                let rk = match interp.reg_load(box_val) {
                    Ok(VMValue::BoxRef(ref b)) => format!("BoxRef({})", b.type_name()),
                    Ok(VMValue::Integer(_)) => "Integer".to_string(),
                    Ok(VMValue::Float(_)) => "Float".to_string(),
                    Ok(VMValue::Bool(_)) => "Bool".to_string(),
                    Ok(VMValue::String(_)) => "String".to_string(),
                    Ok(VMValue::Void) => "Void".to_string(),
                    Ok(VMValue::Future(_)) => "Future".to_string(),
                    Err(_) => "<err>".to_string(),
                };
                eprintln!("[vm-trace] getField recv_kind={}", rk);
            }
            let fname = match interp.reg_load(args[0])? {
                VMValue::String(s) => s,
                v => v.to_string(),
            };
            // Prefer InstanceBox internal storage
            if let VMValue::BoxRef(bref) = interp.reg_load(box_val)? {
                if let Some(inst) = bref.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                    if std::env::var("NYASH_VM_TRACE").ok().as_deref() == Some("1") {
                        eprintln!("[vm-trace] getField instance class={}", inst.class_name);
                    }
                    // Special-case bridge: JsonParser.length -> tokens.length()
                    if inst.class_name == "JsonParser" && fname == "length" {
                        if let Some(tokens_shared) = inst.get_field("tokens") {
                            let tokens_box: Box<dyn crate::box_trait::NyashBox> = tokens_shared.share_box();
                            if let Some(arr) = tokens_box.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                                let len_box = arr.length();
                                if let Some(d) = dst { interp.regs.insert(d, VMValue::from_nyash_box(len_box)); }
                                return Ok(true);
                            }
                        }
                    }
                    // Prefer Ng fields
                    if let Some(nv) = inst.get_field_ng(&fname) {
                        if std::env::var("NYASH_VM_TRACE").ok().as_deref() == Some("1") && inst.class_name == "JsonToken" {
                            eprintln!("[vm-trace] JsonToken.getField name={} nv={:?}", fname, nv);
                        }
                        match nv {
                            crate::value::NyashValue::Null
                            | crate::value::NyashValue::Array(_)
                            | crate::value::NyashValue::Map(_)
                            | crate::value::NyashValue::Box(_)
                            | crate::value::NyashValue::WeakBox(_) => {
                                // fall through to legacy store
                            }
                            _ => {
                                if let Some(d) = dst { interp.regs.insert(d, nv_to_vm(&nv)); }
                                return Ok(true);
                            }
                        }
                    }
                    // Legacy store for BoxRefs
                    if let Some(bx) = inst.get_field(&fname) {
                        if let Some(d) = dst { interp.regs.insert(d, VMValue::BoxRef(bx.clone())); }
                        return Ok(true);
                    }
                }
            }
            // Legacy per-object store
            let key = interp.object_key_for(box_val);
            if let Some(map) = interp.obj_fields.get(&key) {
                if let Some(v) = map.get(&fname) {
                    if let Some(d) = dst { interp.regs.insert(d, v.clone()); }
                    return Ok(true);
                }
            }
            if let Some(d) = dst { interp.regs.insert(d, VMValue::Void); }
            Ok(true)
        }
        "setField" => {
            if args.len() != 2 {
                return Err(VMError::InvalidInstruction("setField expects 2 args".into()));
            }
            let fname = match interp.reg_load(args[0])? {
                VMValue::String(s) => s,
                v => v.to_string(),
            };
            let valv = interp.reg_load(args[1])?;
            if std::env::var("NYASH_VM_TRACE").ok().as_deref() == Some("1") {
                if let VMValue::BoxRef(bref) = interp.reg_load(box_val)? {
                    if let Some(inst) = bref.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                        if inst.class_name == "JsonToken" {
                            eprintln!("[vm-trace] JsonToken.setField name={} vmval={:?}", fname, valv);
                        }
                    }
                }
            }
            if MirInterpreter::box_trace_enabled() {
                let vkind = match &valv {
                    VMValue::Integer(_) => "Integer",
                    VMValue::Float(_) => "Float",
                    VMValue::Bool(_) => "Bool",
                    VMValue::String(_) => "String",
                    VMValue::BoxRef(b) => b.type_name(),
                    VMValue::Void => "Void",
                    VMValue::Future(_) => "Future",
                };
                let cls = match interp.reg_load(box_val).unwrap_or(VMValue::Void) {
                    VMValue::BoxRef(b) => {
                        if let Some(inst) = b.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                            inst.class_name.clone()
                        } else { b.type_name().to_string() }
                    }
                    _ => "<unknown>".to_string(),
                };
                interp.box_trace_emit_set(&cls, &fname, vkind);
            }
            // Prefer InstanceBox internal storage for primitives
            if let VMValue::BoxRef(bref) = interp.reg_load(box_val)? {
                if let Some(inst) = bref.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                    if matches!(valv, VMValue::Integer(_) | VMValue::Float(_) | VMValue::Bool(_) | VMValue::String(_) | VMValue::Void) {
                        let _ = inst.set_field_ng(fname.clone(), vm_to_nv(&valv));
                        return Ok(true);
                    }
                    if let VMValue::BoxRef(bx) = &valv {
                        if let Some(ib) = bx.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                            let _ = inst.set_field_ng(fname.clone(), crate::value::NyashValue::Integer(ib.value));
                            return Ok(true);
                        }
                        if let Some(fb) = bx.as_any().downcast_ref::<crate::boxes::FloatBox>() {
                            let _ = inst.set_field_ng(fname.clone(), crate::value::NyashValue::Float(fb.value));
                            return Ok(true);
                        }
                        if let Some(bb) = bx.as_any().downcast_ref::<crate::box_trait::BoolBox>() {
                            let _ = inst.set_field_ng(fname.clone(), crate::value::NyashValue::Bool(bb.value));
                            return Ok(true);
                        }
                        if let Some(sb) = bx.as_any().downcast_ref::<crate::box_trait::StringBox>() {
                            let _ = inst.set_field_ng(fname.clone(), crate::value::NyashValue::String(sb.value.clone()));
                            return Ok(true);
                        }
                        // Complex Box values → legacy object_fields to preserve identity
                        let _ = inst.set_field(fname.as_str(), std::sync::Arc::clone(bx));
                        return Ok(true);
                    }
                }
            }
            // Legacy per-object store
            let key = interp.object_key_for(box_val);
            interp.obj_fields.entry(key).or_default().insert(fname, valv);
            Ok(true)
        }
        _ => Ok(false),
    }
}

