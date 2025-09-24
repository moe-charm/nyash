use super::*;
use crate::box_trait::NyashBox;

impl MirInterpreter {
    pub(super) fn handle_new_box(
        &mut self,
        dst: ValueId,
        box_type: &str,
        args: &[ValueId],
    ) -> Result<(), VMError> {
        let mut converted: Vec<Box<dyn NyashBox>> = Vec::with_capacity(args.len());
        for vid in args {
            converted.push(self.reg_load(*vid)?.to_nyash_box());
        }
        let reg = crate::runtime::unified_registry::get_global_unified_registry();
        let created = reg
            .lock()
            .unwrap()
            .create_box(box_type, &converted)
            .map_err(|e| {
                VMError::InvalidInstruction(format!("NewBox {} failed: {}", box_type, e))
            })?;
        self.regs.insert(dst, VMValue::from_nyash_box(created));
        Ok(())
    }

    pub(super) fn handle_plugin_invoke(
        &mut self,
        dst: Option<ValueId>,
        box_val: ValueId,
        method: &str,
        args: &[ValueId],
    ) -> Result<(), VMError> {
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
            let mut argv: Vec<Box<dyn NyashBox>> = Vec::with_capacity(args.len());
            for a in args {
                argv.push(self.reg_load(*a)?.to_nyash_box());
            }
            match host.invoke_instance_method(&p.box_type, method, p.inner.instance_id, &argv) {
                Ok(Some(ret)) => {
                    if let Some(d) = dst {
                        self.regs.insert(d, VMValue::from_nyash_box(ret));
                    }
                }
                Ok(None) => {
                    if let Some(d) = dst {
                        self.regs.insert(d, VMValue::Void);
                    }
                }
                Err(e) => {
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
            Err(VMError::InvalidInstruction(format!(
                "PluginInvoke unsupported on {} for method {}",
                recv_box.type_name(),
                method
            )))
        }
    }

    pub(super) fn handle_box_call(
        &mut self,
        dst: Option<ValueId>,
        box_val: ValueId,
        method: &str,
        args: &[ValueId],
    ) -> Result<(), VMError> {
        if self.try_handle_object_fields(dst, box_val, method, args)? {
            return Ok(());
        }
        if self.try_handle_string_box(dst, box_val, method, args)? {
            return Ok(());
        }
        self.invoke_plugin_box(dst, box_val, method, args)
    }

    fn try_handle_object_fields(
        &mut self,
        dst: Option<ValueId>,
        box_val: ValueId,
        method: &str,
        args: &[ValueId],
    ) -> Result<bool, VMError> {
        match method {
            "getField" => {
                if args.len() != 1 {
                    return Err(VMError::InvalidInstruction("getField expects 1 arg".into()));
                }
                let fname = match self.reg_load(args[0])? {
                    VMValue::String(s) => s,
                    v => v.to_string(),
                };
                let v = self
                    .obj_fields
                    .get(&box_val)
                    .and_then(|m| m.get(&fname))
                    .cloned()
                    .unwrap_or(VMValue::Void);
                if let Some(d) = dst {
                    self.regs.insert(d, v);
                }
                Ok(true)
            }
            "setField" => {
                if args.len() != 2 {
                    return Err(VMError::InvalidInstruction(
                        "setField expects 2 args".into(),
                    ));
                }
                let fname = match self.reg_load(args[0])? {
                    VMValue::String(s) => s,
                    v => v.to_string(),
                };
                let valv = self.reg_load(args[1])?;
                self.obj_fields
                    .entry(box_val)
                    .or_default()
                    .insert(fname, valv);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn try_handle_string_box(
        &mut self,
        dst: Option<ValueId>,
        box_val: ValueId,
        method: &str,
        args: &[ValueId],
    ) -> Result<bool, VMError> {
        let recv = self.reg_load(box_val)?;
        let recv_box_any: Box<dyn NyashBox> = match recv.clone() {
            VMValue::BoxRef(b) => b.share_box(),
            other => other.to_nyash_box(),
        };
        if let Some(sb) = recv_box_any
            .as_any()
            .downcast_ref::<crate::box_trait::StringBox>()
        {
            match method {
                "length" => {
                    let ret = sb.length();
                    if let Some(d) = dst {
                        self.regs.insert(d, VMValue::from_nyash_box(ret));
                    }
                    return Ok(true);
                }
                "concat" => {
                    if args.len() != 1 {
                        return Err(VMError::InvalidInstruction("concat expects 1 arg".into()));
                    }
                    let rhs = self.reg_load(args[0])?;
                    let new_s = format!("{}{}", sb.value, rhs.to_string());
                    if let Some(d) = dst {
                        self.regs.insert(
                            d,
                            VMValue::from_nyash_box(Box::new(crate::box_trait::StringBox::new(
                                new_s,
                            ))),
                        );
                    }
                    return Ok(true);
                }
                _ => {}
            }
        }
        Ok(false)
    }

    fn invoke_plugin_box(
        &mut self,
        dst: Option<ValueId>,
        box_val: ValueId,
        method: &str,
        args: &[ValueId],
    ) -> Result<(), VMError> {
        let recv = self.reg_load(box_val)?;
        let recv_box: Box<dyn NyashBox> = match recv.clone() {
            VMValue::BoxRef(b) => b.share_box(),
            other => other.to_nyash_box(),
        };
        if let Some(p) = recv_box
            .as_any()
            .downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>()
        {
            if p.box_type == "ConsoleBox" && method == "readLine" {
                use std::io::{self, Read};
                let mut s = String::new();
                let mut stdin = io::stdin();
                let mut buf = [0u8; 1];
                while let Ok(n) = stdin.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    let ch = buf[0] as char;
                    if ch == '\n' {
                        break;
                    }
                    s.push(ch);
                    if s.len() > 1_000_000 {
                        break;
                    }
                }
                if let Some(d) = dst {
                    self.regs.insert(d, VMValue::String(s));
                }
                return Ok(());
            }
            let host = crate::runtime::plugin_loader_unified::get_global_plugin_host();
            let host = host.read().unwrap();
            let mut argv: Vec<Box<dyn NyashBox>> = Vec::with_capacity(args.len());
            for a in args {
                argv.push(self.reg_load(*a)?.to_nyash_box());
            }
            match host.invoke_instance_method(&p.box_type, method, p.inner.instance_id, &argv) {
                Ok(Some(ret)) => {
                    if let Some(d) = dst {
                        self.regs.insert(d, VMValue::from_nyash_box(ret));
                    }
                    Ok(())
                }
                Ok(None) => {
                    if let Some(d) = dst {
                        self.regs.insert(d, VMValue::Void);
                    }
                    Ok(())
                }
                Err(e) => Err(VMError::InvalidInstruction(format!(
                    "BoxCall {}.{} failed: {:?}",
                    p.box_type, method, e
                ))),
            }
        } else {
            Err(VMError::InvalidInstruction(format!(
                "BoxCall unsupported on {}.{}",
                recv_box.type_name(),
                method
            )))
        }
    }
}
