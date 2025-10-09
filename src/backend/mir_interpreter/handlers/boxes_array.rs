use super::*;
use crate::box_trait::NyashBox;

pub(super) fn try_handle_array_box(
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
        if let Some(ab) = recv_box_any
            .as_any()
            .downcast_ref::<crate::boxes::array::ArrayBox>()
        {
            match method {
                "birth" => {
                    // No-op constructor init
                    if let Some(d) = dst { this.regs.insert(d, VMValue::Void); }
                    return Ok(true);
                }
                "push" => {
                    if args.len() != 1 {
                        if crate::config::env::check_contracts() {
                            eprintln!(r#"{{"kind":"contracts_arity","box":"ArrayBox","method":"push","expected":1,"got":{}}}"#, args.len());
                        }
                        return Err(VMError::InvalidInstruction("push expects 1 arg".into()));
                    }
                    let val = this.reg_load(args[0])?.to_nyash_box();
                    let _ = ab.push(val);
                    if let Some(d) = dst { this.regs.insert(d, VMValue::Void); }
                    return Ok(true);
                }
                "len" | "length" | "size" => {
                    let ret = ab.length();
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "isEmpty" => {
                    let is_empty = ab.len() == 0;
                    if let Some(d) = dst { this.regs.insert(d, VMValue::Bool(is_empty)); }
                    return Ok(true);
                }
                "get" => {
                    if args.len() != 1 {
                        if crate::config::env::check_contracts() {
                            eprintln!(r#"{{"kind":"contracts_arity","box":"ArrayBox","method":"get","expected":1,"got":{}}}"#, args.len());
                        }
                        return Err(VMError::InvalidInstruction("get expects 1 arg".into()));
                    }
                    let idx = this.reg_load(args[0])?.to_nyash_box();
                    if crate::config::env::check_contracts() {
                        if let Some(ib) = idx.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                            let len = ab.len();
                            let neg = if ib.value < 0 { 1 } else { 0 };
                            let oob = if ib.value < 0 || (ib.value as usize) >= len { 1 } else { 0 };
                            eprintln!(r#"{{"kind":"contracts_index","box":"ArrayBox","method":"get","neg":{},"oob":{},"idx":{},"len":{}}}"#, neg, oob, ib.value, len);
                        } else {
                            eprintln!(r#"{{"kind":"contracts_type","box":"ArrayBox","method":"get","expected":"Integer","actual":"{}"}}"#, idx.type_name());
                        }
                    }
                    let ret = ab.get(idx);
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "set" => {
                    if args.len() != 2 {
                        if crate::config::env::check_contracts() {
                            eprintln!(r#"{{"kind":"contracts_arity","box":"ArrayBox","method":"set","expected":2,"got":{}}}"#, args.len());
                        }
                        return Err(VMError::InvalidInstruction("set expects 2 args".into()));
                    }
                    let idx = this.reg_load(args[0])?.to_nyash_box();
                    if crate::config::env::check_contracts() {
                        if let Some(ib) = idx.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                            let len = ab.len();
                            let neg = if ib.value < 0 { 1 } else { 0 };
                            // set は len と等しい場合は append 許可
                            let oob = if ib.value < 0 || (ib.value as usize) > len { 1 } else { 0 };
                            eprintln!(r#"{{"kind":"contracts_index","box":"ArrayBox","method":"set","neg":{},"oob":{},"idx":{},"len":{}}}"#, neg, oob, ib.value, len);
                        } else {
                            eprintln!(r#"{{"kind":"contracts_type","box":"ArrayBox","method":"set","expected":"Integer","actual":"{}"}}"#, idx.type_name());
                        }
                    }
                    let val = this.reg_load(args[1])?.to_nyash_box();
                    let _ = ab.set(idx, val);
                    if let Some(d) = dst { this.regs.insert(d, VMValue::Void); }
                    return Ok(true);
                }
                "toJSON" => {
                    if !args.is_empty() {
                        if crate::config::env::check_contracts() {
                            eprintln!(r#"{{"kind":"contracts_arity","box":"ArrayBox","method":"toJSON","expected":0,"got":{}}}"#, args.len());
                        }
                        return Err(VMError::InvalidInstruction("ArrayBox.toJSON expects 0 args".into()));
                    }
                    let ret = ab.toJSON();
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                _ => {}
            }
        }
        Ok(false)
}
