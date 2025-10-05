//! method.rs — Method dispatch
//!
//! Behavior-preserving extraction of execute_method_call from legacy.

use super::super::*;

impl MirInterpreter {
    pub(crate) fn execute_method_call(
        &mut self,
        receiver: &VMValue,
        method: &str,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
        // Built-in arity validation (Fail-Fast)
        {
            let arity = args.len();
            let mut type_name: Option<&str> = None;
            match receiver {
                VMValue::String(_) => { type_name = Some("StringBox"); }
                VMValue::BoxRef(bx) => {
                    let tn = bx.type_name();
                    if matches!(tn, "ArrayBox" | "MapBox" | "StringBox") { type_name = Some(tn); }
                }
                _ => {}
            }
            if let Some(tn) = type_name {
                if method != "birth" {
                    if crate::runtime::type_registry::resolve_typebox_by_name(tn).is_some() {
                        if crate::runtime::type_registry::resolve_slot_by_name(tn, method, arity).is_none() {
                            if let Some(known) = crate::runtime::type_registry::known_arities_for(tn, method) {
                                if !known.is_empty() {
                                    return Err(VMError::InvalidInstruction(format!(
                                        "No matching method: {}.{}({} args). Available arities: {:?}", tn, method, arity, known
                                    )));
                                }
                            }
                        }
                    }
                }
            }
        }

        match receiver {
            VMValue::String(s) => match method {
                "length" => Ok(VMValue::Integer(s.len() as i64)),
                "concat" => {
                    if let Some(arg_id) = args.get(0) {
                        let arg_val = self.reg_load(*arg_id)?;
                        let new_str = format!("{}{}", s, arg_val.to_string());
                        Ok(VMValue::String(new_str))
                    } else {
                        Err(VMError::InvalidInstruction(
                            "concat requires 1 argument".into(),
                        ))
                    }
                }
                "indexOf" => {
                    if args.len() != 1 {
                        return Err(VMError::InvalidInstruction(format!("No matching method: StringBox.indexOf({} args). Available arities: [1]", args.len())));
                    }
                    if let Some(arg_id) = args.get(0) {
                        let needle = self.reg_load(*arg_id)?.to_string();
                        let idx = s.find(&needle).map(|i| i as i64).unwrap_or(-1);
                        Ok(VMValue::Integer(idx))
                    } else {
                        Err(VMError::InvalidInstruction(
                            "indexOf requires 1 argument".into(),
                        ))
                    }
                }
                "substring" => {
                    let start = if let Some(a0) = args.get(0) {
                        self.reg_load(*a0)?.as_integer().unwrap_or(0)
                    } else {
                        0
                    };
                    let end = if let Some(a1) = args.get(1) {
                        self.reg_load(*a1)?.as_integer().unwrap_or(s.len() as i64)
                    } else {
                        s.len() as i64
                    };
                    let len = s.len() as i64;
                    let i0 = start.max(0).min(len) as usize;
                    let i1 = end.max(0).min(len) as usize;
                    if i0 > i1 {
                        return Ok(VMValue::String(String::new()));
                    }
                    // Note: operating on bytes; Nyash strings are UTF‑8, but tests are ASCII only here
                    let bytes = s.as_bytes();
                    let sub =
                        String::from_utf8(bytes[i0..i1].to_vec()).unwrap_or_default();
                    Ok(VMValue::String(sub))
                }
                _ => Err(VMError::InvalidInstruction(format!(
                    "Unknown String method: {}",
                    method
                ))),
            },
            VMValue::BoxRef(box_ref) => {
                if let Some(p) = box_ref
                    .as_any()
                    .downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>()
                {
                    let host =
                        crate::runtime::plugin_loader_unified::get_global_plugin_host();
                    let host = host.read().unwrap();
                    let mut argv: Vec<Box<dyn crate::box_trait::NyashBox>> =
                        Vec::with_capacity(args.len());
                    for a in args {
                        argv.push(self.reg_load(*a)?.to_nyash_box());
                    }
                    let out = host.invoke_instance_method(&p.box_type, method, p.inner.instance_id, &argv);
                    match out {
                        Ok(Some(ret)) => { let v = VMValue::from_nyash_box(ret); self.maybe_register_scope_value(&v); Ok(v) },
                        Ok(None) => Ok(VMValue::Void),
                        Err(e) => Err(VMError::InvalidInstruction(format!(
                            "Plugin method {}.{} failed: {:?}",
                            p.box_type, method, e
                        ))),
                    }
                } else {
                    Err(VMError::InvalidInstruction(format!(
                        "Method {} not supported on BoxRef({})",
                        method,
                        box_ref.type_name()
                    )))
                }
            }
            _ => Err(VMError::InvalidInstruction(format!(
                "Method {} not supported on {:?}",
                method, receiver
            ))),
        }
    }
}
