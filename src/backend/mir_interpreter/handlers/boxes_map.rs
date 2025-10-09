use super::*;
use crate::box_trait::NyashBox;
use std::sync::OnceLock;

#[inline]
fn map_handlers_disable() -> bool {
    matches!(std::env::var("NYASH_VM_DISABLE_MAP_HANDLERS").ok().as_deref(), Some("1"|"true"|"on"))
}

#[inline]
fn map_handlers_deprecate() -> bool {
    matches!(std::env::var("NYASH_VM_MAP_HANDLERS_DEPRECATE").ok().as_deref(), Some("1"|"true"|"on"))
}

fn warn_once() {
    static WARNED: OnceLock<bool> = OnceLock::new();
    if *WARNED.get_or_init(|| false) == false {
        eprintln!("[deprecate] VM map handlers are deprecated and will be removed (prefer plugin/User route)");
        WARNED.set(true).ok();
    }
}

pub(super) fn try_handle_map_box(
    this: &mut MirInterpreter,
    dst: Option<ValueId>,
    box_val: ValueId,
    method: &str,
    args: &[ValueId],
) -> Result<bool, VMError> {
        let recv = this.reg_load(box_val)?;
        let recv_box_any: Box<dyn NyashBox> = match recv.clone() {
            VMValue::BoxRef(b) => b.share_box(),
            other => other.to_nyash_box(),
        };
        if let Some(mb) = recv_box_any
            .as_any()
            .downcast_ref::<crate::boxes::map_box::MapBox>()
        {
            if map_handlers_disable() { return Ok(false); }
            if map_handlers_deprecate() { warn_once(); }
            match method {
                "birth" => {
                    // No-op constructor init for MapBox
                    if let Some(d) = dst { this.regs.insert(d, VMValue::Void); }
                    return Ok(true);
                }
                "set" => {
                    if args.len() != 2 {
                        if crate::config::env::check_contracts() {
                            eprintln!(r#"{{"kind":"contracts_arity","box":"MapBox","method":"set","expected":2,"got":{}}}"#, args.len());
                        }
                        return Err(VMError::InvalidInstruction("MapBox.set expects 2 args".into()));
                    }
                    let k = this.reg_load(args[0])?.to_nyash_box();
                    if crate::config::env::check_contracts() && k.as_any().downcast_ref::<crate::box_trait::StringBox>().is_none() {
                        eprintln!(r#"{{"kind":"contracts_type","box":"MapBox","method":"set","expected":"String","actual":"{}"}}"#, k.type_name());
                    }
                    let v = this.reg_load(args[1])?.to_nyash_box();
                    let ret = mb.set(k, v);
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "get" => {
                    if args.len() != 1 {
                        if crate::config::env::check_contracts() {
                            eprintln!(r#"{{"kind":"contracts_arity","box":"MapBox","method":"get","expected":1,"got":{}}}"#, args.len());
                        }
                        return Err(VMError::InvalidInstruction("MapBox.get expects 1 arg".into()));
                    }
                    let k = this.reg_load(args[0])?.to_nyash_box();
                    if crate::config::env::check_contracts() && k.as_any().downcast_ref::<crate::box_trait::StringBox>().is_none() {
                        eprintln!(r#"{{"kind":"contracts_type","box":"MapBox","method":"get","expected":"String","actual":"{}"}}"#, k.type_name());
                    }
                    let ret = mb.get(k);
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "has" => {
                    if args.len() != 1 {
                        if crate::config::env::check_contracts() {
                            eprintln!(r#"{{"kind":"contracts_arity","box":"MapBox","method":"has","expected":1,"got":{}}}"#, args.len());
                        }
                        return Err(VMError::InvalidInstruction("MapBox.has expects 1 arg".into()));
                    }
                    let k = this.reg_load(args[0])?.to_nyash_box();
                    if crate::config::env::check_contracts() && k.as_any().downcast_ref::<crate::box_trait::StringBox>().is_none() {
                        eprintln!(r#"{{"kind":"contracts_type","box":"MapBox","method":"has","expected":"String","actual":"{}"}}"#, k.type_name());
                    }
                    let ret = mb.has(k);
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "delete" => {
                    if args.len() != 1 {
                        if crate::config::env::check_contracts() {
                            eprintln!(r#"{{"kind":"contracts_arity","box":"MapBox","method":"delete","expected":1,"got":{}}}"#, args.len());
                        }
                        return Err(VMError::InvalidInstruction("MapBox.delete expects 1 arg".into()));
                    }
                    let k = this.reg_load(args[0])?.to_nyash_box();
                    if crate::config::env::check_contracts() && k.as_any().downcast_ref::<crate::box_trait::StringBox>().is_none() {
                        eprintln!(r#"{{"kind":"contracts_type","box":"MapBox","method":"delete","expected":"String","actual":"{}"}}"#, k.type_name());
                    }
                    let ret = mb.delete(k);
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "size" => {
                    let ret = mb.size();
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "keys" => {
                    let ret = mb.keys();
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "values" => {
                    let ret = mb.values();
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "isEmpty" => {
                    // Compute via size()==0 to avoid exposing internals
                    let n = mb.size();
                    let is_empty = match n.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                        Some(i) => i.value == 0,
                        None => false,
                    };
                    if let Some(d) = dst { this.regs.insert(d, VMValue::Bool(is_empty)); }
                    return Ok(true);
                }
                "toJSON" => {
                    if !args.is_empty() {
                        if crate::config::env::check_contracts() {
                            eprintln!(r#"{{"kind":"contracts_arity","box":"MapBox","method":"toJSON","expected":0,"got":{}}}"#, args.len());
                        }
                        return Err(VMError::InvalidInstruction("MapBox.toJSON expects 0 args".into()));
                    }
                    let ret = mb.toJSON();
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                _ => {}
            }
        }
        Ok(false)
}
