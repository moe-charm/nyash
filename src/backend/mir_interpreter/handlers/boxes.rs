use super::*;
use crate::box_trait::NyashBox;

impl MirInterpreter {
    pub(super) fn handle_new_box(
        &mut self,
        dst: ValueId,
        box_type: &str,
        args: &[ValueId],
    ) -> Result<(), VMError> {
        // Provider Lock guard (受け口・既定は挙動不変)
        if let Err(e) = crate::runtime::provider_lock::guard_before_new_box(box_type) {
            return Err(VMError::InvalidInstruction(e));
        }
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
        // Graceful void guard for common short-circuit patterns in user code
        // e.g., `A or not last.is_eof()` should not crash when last is absent.
        match self.reg_load(box_val)? {
            VMValue::Void => {
                match method {
                    "is_eof" => { if let Some(d) = dst { self.regs.insert(d, VMValue::Bool(false)); } return Ok(()); }
                    "length" => { if let Some(d) = dst { self.regs.insert(d, VMValue::Integer(0)); } return Ok(()); }
                    "substring" => { if let Some(d) = dst { self.regs.insert(d, VMValue::String(String::new())); } return Ok(()); }
                    "push" => { if let Some(d) = dst { self.regs.insert(d, VMValue::Void); } return Ok(()); }
                    "get_position" => { if let Some(d) = dst { self.regs.insert(d, VMValue::Integer(0)); } return Ok(()); }
                    "get_line" => { if let Some(d) = dst { self.regs.insert(d, VMValue::Integer(1)); } return Ok(()); }
                    "get_column" => { if let Some(d) = dst { self.regs.insert(d, VMValue::Integer(1)); } return Ok(()); }
                    _ => {}
                }
            }
            VMValue::BoxRef(ref b) => {
                if b.as_any().downcast_ref::<crate::box_trait::VoidBox>().is_some() {
                    match method {
                        "is_eof" => { if let Some(d) = dst { self.regs.insert(d, VMValue::Bool(false)); } return Ok(()); }
                        "length" => { if let Some(d) = dst { self.regs.insert(d, VMValue::Integer(0)); } return Ok(()); }
                        "substring" => { if let Some(d) = dst { self.regs.insert(d, VMValue::String(String::new())); } return Ok(()); }
                        "push" => { if let Some(d) = dst { self.regs.insert(d, VMValue::Void); } return Ok(()); }
                        "get_position" => { if let Some(d) = dst { self.regs.insert(d, VMValue::Integer(0)); } return Ok(()); }
                        "get_line" => { if let Some(d) = dst { self.regs.insert(d, VMValue::Integer(1)); } return Ok(()); }
                        "get_column" => { if let Some(d) = dst { self.regs.insert(d, VMValue::Integer(1)); } return Ok(()); }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        if self.try_handle_object_fields(dst, box_val, method, args)? {
            return Ok(());
        }
        if self.try_handle_instance_box(dst, box_val, method, args)? {
            return Ok(());
        }
        if self.try_handle_string_box(dst, box_val, method, args)? {
            return Ok(());
        }
        if self.try_handle_array_box(dst, box_val, method, args)? {
            return Ok(());
        }
        if self.try_handle_map_box(dst, box_val, method, args)? {
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

    fn try_handle_map_box(
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
        if let Some(mb) = recv_box_any
            .as_any()
            .downcast_ref::<crate::boxes::map_box::MapBox>()
        {
            match method {
                "birth" => {
                    // No-op constructor init for MapBox
                    if let Some(d) = dst { self.regs.insert(d, VMValue::Void); }
                    return Ok(true);
                }
                "set" => {
                    if args.len() != 2 { return Err(VMError::InvalidInstruction("MapBox.set expects 2 args".into())); }
                    let k = self.reg_load(args[0])?.to_nyash_box();
                    let v = self.reg_load(args[1])?.to_nyash_box();
                    let ret = mb.set(k, v);
                    if let Some(d) = dst { self.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "get" => {
                    if args.len() != 1 { return Err(VMError::InvalidInstruction("MapBox.get expects 1 arg".into())); }
                    let k = self.reg_load(args[0])?.to_nyash_box();
                    let ret = mb.get(k);
                    if let Some(d) = dst { self.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "has" => {
                    if args.len() != 1 { return Err(VMError::InvalidInstruction("MapBox.has expects 1 arg".into())); }
                    let k = self.reg_load(args[0])?.to_nyash_box();
                    let ret = mb.has(k);
                    if let Some(d) = dst { self.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "delete" => {
                    if args.len() != 1 { return Err(VMError::InvalidInstruction("MapBox.delete expects 1 arg".into())); }
                    let k = self.reg_load(args[0])?.to_nyash_box();
                    let ret = mb.delete(k);
                    if let Some(d) = dst { self.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "size" => {
                    let ret = mb.size();
                    if let Some(d) = dst { self.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "keys" => {
                    let ret = mb.keys();
                    if let Some(d) = dst { self.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "values" => {
                    let ret = mb.values();
                    if let Some(d) = dst { self.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                _ => {}
            }
        }
        Ok(false)
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
                "substring" => {
                    if args.len() != 2 {
                        return Err(VMError::InvalidInstruction(
                            "substring expects 2 args (start, end)".into(),
                        ));
                    }
                    let s_idx = self.reg_load(args[0])?.as_integer().unwrap_or(0);
                    let e_idx = self.reg_load(args[1])?.as_integer().unwrap_or(0);
                    let len = sb.value.chars().count() as i64;
                    let start = s_idx.max(0).min(len) as usize;
                    let end = e_idx.max(start as i64).min(len) as usize;
                    let chars: Vec<char> = sb.value.chars().collect();
                    let sub: String = chars[start..end].iter().collect();
                    if let Some(d) = dst {
                        self.regs.insert(
                            d,
                            VMValue::from_nyash_box(Box::new(crate::box_trait::StringBox::new(
                                sub,
                            ))),
                        );
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

    fn try_handle_instance_box(
        &mut self,
        dst: Option<ValueId>,
        box_val: ValueId,
        method: &str,
        args: &[ValueId],
    ) -> Result<bool, VMError> {
        let recv_vm = self.reg_load(box_val)?;
        let recv_box_any: Box<dyn NyashBox> = match recv_vm.clone() {
            VMValue::BoxRef(b) => b.share_box(),
            other => other.to_nyash_box(),
        };
        if let Some(inst) = recv_box_any.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
            // Resolve lowered method function: "Class.method/arity"
            let fname = format!("{}.{}{}", inst.class_name, method, format!("/{}", args.len()));
            if let Some(func) = self.functions.get(&fname).cloned() {
                // Build argv: me + args
                let mut argv: Vec<VMValue> = Vec::with_capacity(1 + args.len());
                argv.push(recv_vm.clone());
                for a in args {
                    argv.push(self.reg_load(*a)?);
                }
                let ret = self.exec_function_inner(&func, Some(&argv))?;
                if let Some(d) = dst {
                    self.regs.insert(d, ret);
                }
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn try_handle_array_box(
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
        if let Some(ab) = recv_box_any
            .as_any()
            .downcast_ref::<crate::boxes::array::ArrayBox>()
        {
            match method {
                "birth" => {
                    // No-op constructor init
                    if let Some(d) = dst { self.regs.insert(d, VMValue::Void); }
                    return Ok(true);
                }
                "push" => {
                    if args.len() != 1 { return Err(VMError::InvalidInstruction("push expects 1 arg".into())); }
                    let val = self.reg_load(args[0])?.to_nyash_box();
                    let _ = ab.push(val);
                    if let Some(d) = dst { self.regs.insert(d, VMValue::Void); }
                    return Ok(true);
                }
                "len" | "length" | "size" => {
                    let ret = ab.length();
                    if let Some(d) = dst { self.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "get" => {
                    if args.len() != 1 { return Err(VMError::InvalidInstruction("get expects 1 arg".into())); }
                    let idx = self.reg_load(args[0])?.to_nyash_box();
                    let ret = ab.get(idx);
                    if let Some(d) = dst { self.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "set" => {
                    if args.len() != 2 { return Err(VMError::InvalidInstruction("set expects 2 args".into())); }
                    let idx = self.reg_load(args[0])?.to_nyash_box();
                    let val = self.reg_load(args[1])?.to_nyash_box();
                    let _ = ab.set(idx, val);
                    if let Some(d) = dst { self.regs.insert(d, VMValue::Void); }
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
