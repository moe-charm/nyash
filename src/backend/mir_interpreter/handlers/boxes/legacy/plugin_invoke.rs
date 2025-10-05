//! Plugin box method invocation handler
//!
//! Handles PluginInvoke instructions for plugin-backed boxes.
//! Provides auto-birth handling and fallback to builtin handlers.

use super::super::super::*;
use crate::box_trait::NyashBox;
use crate::backend::mir_interpreter::handlers::{
    boxes_array, boxes_string, boxes_map,
};

impl MirInterpreter {
    pub(crate) fn handle_plugin_invoke(
        &mut self,
        dst: Option<ValueId>,
        box_val: ValueId,
        method: &str,
        args: &[ValueId],
    ) -> Result<(), VMError> {
        // Unborn guard for plugin instance methods (except birth)
        if method != "birth" {
            if crate::config::env::check_contracts() {
                let key = self.object_key_for(box_val);
                let seen_new = self.contracts_new.contains(&key);
                let seen_birth =
                    self.contracts_born.contains(&key) || self.contracts_in_birth.contains(&key);
                if seen_new && !seen_birth {
                    return Err(VMError::InvalidInstruction(
                        "operation on unborn plugin instance (call birth() first)".to_string(),
                    ));
                }
            }
        }

        // Dev-only call trace for PluginInvoke (parity aid)
        let cls = match self.reg_load(box_val).unwrap_or(VMValue::Void) {
            VMValue::BoxRef(b) => b.type_name().to_string(),
            _ => "<unknown>".to_string(),
        };
        let label = format!("PluginInvoke:{}.{}", cls, method);
        self.emit_call_trace_label(&label, args.len(), None);
        let recv = self.reg_load(box_val)?;
        let recv_box: Box<dyn NyashBox> = match recv.clone() {
            VMValue::BoxRef(b) => b.share_box(),
            other => other.to_nyash_box(),
        };

        if let Some(p) = recv_box
            .as_any()
            .downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>()
        {
            let host = crate::runtime::plugin_loader_unified::get_global_plugin_host();
            let host = host.read().unwrap();
            if method == "birth" { let k = self.object_key_for(box_val); if self.contracts_born.contains(&k) { if let Some(d)=dst { self.regs.insert(d, VMValue::Void);} return Ok(()); } if !self.contracts_in_birth.insert(k) { return Err(VMError::InvalidInstruction("reentrant birth()".into())); } }
            // Auto-birth no-op: if plugin does not provide birth, treat as success(void)
            if method == "birth" {
                let need_noop = match host.resolve_method(&p.box_type, method) {
                    Ok(_) => false,
                    Err(_) => true,
                };
                if need_noop {
                    if std::env::var("NYASH_WARN_PLUGIN_NO_BIRTH").ok().as_deref() != Some("0")
                        && !crate::config::env::cli_quiet()
                    {
                        eprintln!(
                            "[plugin-loader] info: {} has no birth(); treating as no-op",
                            p.box_type
                        );
                    }
                    // Mark birth observed to satisfy unborn→alive transition
                    self.lifecycle_contracts_birth(box_val, args.len());
                    if let Some(d) = dst { self.regs.insert(d, VMValue::Void); }
                    return Ok(());
                }
            }
            let mut argv: Vec<Box<dyn NyashBox>> = Vec::with_capacity(args.len());
            for a in args {
                argv.push(self.reg_load(*a)?.to_nyash_box());
            }
            let __birth = method=="birth"; let __key = self.object_key_for(box_val);
            match host.invoke_instance_method(&p.box_type, method, p.inner.instance_id, &argv) {
                Ok(Some(ret)) => { if __birth { self.contracts_in_birth.remove(&__key); self.lifecycle_contracts_birth(box_val, args.len()); }
                    if let Some(d) = dst {
                        self.regs.insert(d, VMValue::from_nyash_box(ret));
                    }
                }
                Ok(None) => { if __birth { self.contracts_in_birth.remove(&__key); self.lifecycle_contracts_birth(box_val, args.len()); }
                    if let Some(d) = dst {
                        self.regs.insert(d, VMValue::Void);
                    }
                }
                Err(e) => { if __birth { self.contracts_in_birth.remove(&__key); }
                    return Err(VMError::InvalidInstruction(format!(
                        "PluginInvoke {}.{} failed: {:?}",
                        p.box_type, method, e
                    )))
                }
            }
            Ok(())
        } else if method == "toString" {
            if let Some(d) = dst {
                self.regs
                    .insert(d, VMValue::String(recv_box.to_string_box().value));
            }
            Ok(())
        } else {
            if crate::config::env::check_contracts() {
                eprintln!(
                    r#"{{"kind":"contracts_warn","what":"plugin_invoke_non_plugin","method":"{}"}}"#,
                    method
                );
            }
            // Fallback: if receiver is a builtin core box (Array/Map/String),
            // route PluginInvoke to the same minimal handlers we use for BoxCall.
            // This keeps behavior stable in dev when optimizer forces PluginInvoke
            // but NewBox still yielded a builtin instance.
            if boxes_array::try_handle_array_box(self, dst, box_val, method, args)? {
                return Ok(());
            }
            if boxes_string::try_handle_string_box(self, dst, box_val, method, args)? {
                return Ok(());
            }
            if boxes_map::try_handle_map_box(self, dst, box_val, method, args)? {
                return Ok(());
            }
            Err(VMError::InvalidInstruction(format!(
                "PluginInvoke unsupported on {} for method {}",
                recv_box.type_name(),
                method
            )))
        }
    }
}
