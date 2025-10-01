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
                    match host.invoke_instance_method(
                        &p.box_type,
                        method,
                        p.inner.instance_id,
                        &argv,
                    ) {
                        Ok(Some(ret)) => Ok(VMValue::from_nyash_box(ret)),
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

