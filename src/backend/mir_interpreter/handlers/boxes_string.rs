use super::*;
use crate::box_trait::NyashBox;

pub(super) fn try_handle_string_box(
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
        if let Some(sb) = recv_box_any
            .as_any()
            .downcast_ref::<crate::box_trait::StringBox>()
        {
            match method {
                "length" => {
                    let ret = sb.length();
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "size" => {
                    let ret = sb.length();
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "isEmpty" => {
                    let is_empty = sb.value.is_empty();
                    if let Some(d) = dst { this.regs.insert(d, VMValue::Bool(is_empty)); }
                    return Ok(true);
                }
                "upper" | "to_upper" => {
                    if !args.is_empty() {
                        if crate::config::env::check_contracts() {
                            eprintln!(r#"{{"kind":"contracts_arity","box":"StringBox","method":"{}","expected":0,"got":{}}}"#, method, args.len());
                        }
                        return Err(VMError::InvalidInstruction(format!("{} expects 0 args", method)));
                    }
                    let upper = sb.value.to_uppercase();
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(Box::new(crate::box_trait::StringBox::new(upper)))); }
                    return Ok(true);
                }
                "lower" | "to_lower" => {
                    if !args.is_empty() {
                        if crate::config::env::check_contracts() {
                            eprintln!(r#"{{"kind":"contracts_arity","box":"StringBox","method":"{}","expected":0,"got":{}}}"#, method, args.len());
                        }
                        return Err(VMError::InvalidInstruction(format!("{} expects 0 args", method)));
                    }
                    let lower = sb.value.to_lowercase();
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(Box::new(crate::box_trait::StringBox::new(lower)))); }
                    return Ok(true);
                }
                "indexOf" => {
                    // Enforce arity=1 (Fail‑Fast). Previously a dev-only 2-arg form existed but was inconsistent.
                    if args.len() != 1 {
                        if crate::config::env::check_contracts() {
                            eprintln!(r#"{{"kind":"contracts_arity","box":"StringBox","method":"indexOf","expected":1,"got":{}}}"#, args.len());
                        }
                        return Err(VMError::InvalidInstruction("indexOf expects 1 arg".into()));
                    }
                    let needle = this.reg_load(args[0])?.to_string();
                    let idx = sb.value.find(&needle).map(|i| i as i64).unwrap_or(-1);
                    if let Some(d) = dst { this.regs.insert(d, VMValue::Integer(idx)); }
                    return Ok(true);
                }
                "lastIndexOf" => {
                    // lastIndexOf(substr) -> last index or -1 (byte index basis)
                    if args.is_empty() {
                        if crate::config::env::check_contracts() {
                            eprintln!(r#"{{"kind":"contracts_arity_min","box":"StringBox","method":"lastIndexOf","min_expected":1,"got":0}}"#);
                        }
                        return Err(VMError::InvalidInstruction("lastIndexOf expects at least 1 arg".into()));
                    }
                    let needle = this.reg_load(args[0])?.to_string();
                    let idx = sb.value.rfind(&needle).map(|i| i as i64).unwrap_or(-1);
                    if let Some(d) = dst { this.regs.insert(d, VMValue::Integer(idx)); }
                    return Ok(true);
                }
                "stringify" => {
                    // JSON-style stringify for strings: quote and escape common characters
                    let mut quoted = String::with_capacity(sb.value.len() + 2);
                    quoted.push('"');
                    for ch in sb.value.chars() {
                        match ch {
                            '"' => quoted.push_str("\\\""),
                            '\\' => quoted.push_str("\\\\"),
                            '\n' => quoted.push_str("\\n"),
                            '\r' => quoted.push_str("\\r"),
                            '\t' => quoted.push_str("\\t"),
                            c if c.is_control() => quoted.push(' '),
                            c => quoted.push(c),
                        }
                    }
                    quoted.push('"');
                    if let Some(d) = dst {
                        this.regs.insert(d, VMValue::from_nyash_box(Box::new(crate::box_trait::StringBox::new(quoted))));
                    }
                    return Ok(true);
                }
                "substring" => {
                    if args.len() != 2 {
                        if crate::config::env::check_contracts() {
                            eprintln!(r#"{{"kind":"contracts_arity","box":"StringBox","method":"substring","expected":2,"got":{}}}"#, args.len());
                        }
                        return Err(VMError::InvalidInstruction(
                            "substring expects 2 args (start, end)".into(),
                        ));
                    }
                    let s_idx = this.reg_load(args[0])?.as_integer().unwrap_or(0);
                    let e_idx = this.reg_load(args[1])?.as_integer().unwrap_or(0);
                    let len = sb.value.chars().count() as i64;
                    let start = s_idx.max(0).min(len) as usize;
                    let end = e_idx.max(start as i64).min(len) as usize;
                    let chars: Vec<char> = sb.value.chars().collect();
                    let sub: String = chars[start..end].iter().collect();
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(Box::new(crate::box_trait::StringBox::new(sub)))) ; }
                    return Ok(true);
                }
                "charAt" => {
                    if args.len() != 1 {
                        if crate::config::env::check_contracts() {
                            eprintln!(r#"{{"kind":"contracts_arity","box":"StringBox","method":"charAt","expected":1,"got":{}}}"#, args.len());
                        }
                        return Err(VMError::InvalidInstruction("charAt expects 1 arg".into()));
                    }
                    let idx = this.reg_load(args[0])?.as_integer().unwrap_or(-1);
                    let ch = if idx >= 0 { sb.value.chars().nth(idx as usize) } else { None };
                    let out = match ch {
                        Some(c) => Box::new(crate::box_trait::StringBox::new(c.to_string())),
                        None => Box::new(crate::box_trait::StringBox::new("")),
                    };
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(out)); }
                    return Ok(true);
                }
                "concat" => {
                    if args.len() != 1 {
                        if crate::config::env::check_contracts() {
                            eprintln!(r#"{{"kind":"contracts_arity","box":"StringBox","method":"concat","expected":1,"got":{}}}"#, args.len());
                        }
                        return Err(VMError::InvalidInstruction("concat expects 1 arg".into()));
                    }
                    let rhs = this.reg_load(args[0])?;
                    let new_s = format!("{}{}", sb.value, rhs.to_string());
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(Box::new(crate::box_trait::StringBox::new(new_s)))) ; }
                    return Ok(true);
                }
                "is_digit_char" => {
                    // Accept either 0-arg (use first char of receiver) or 1-arg (string/char to test)
                    let ch_opt = if args.is_empty() {
                        sb.value.chars().next()
                    } else if args.len() == 1 {
                        let s = this.reg_load(args[0])?.to_string();
                        s.chars().next()
                    } else {
                        if crate::config::env::check_contracts() {
                            eprintln!(r#"{{"kind":"contracts_arity_range","box":"StringBox","method":"is_digit_char","min":0,"max":1,"got":{}}}"#, args.len());
                        }
                        return Err(VMError::InvalidInstruction("is_digit_char expects 0 or 1 arg".into()));
                    };
                    let is_digit = ch_opt.map(|c| c.is_ascii_digit()).unwrap_or(false);
                    if let Some(d) = dst { this.regs.insert(d, VMValue::Bool(is_digit)); }
                    return Ok(true);
                }
                "is_hex_digit_char" => {
                    let ch_opt = if args.is_empty() {
                        sb.value.chars().next()
                    } else if args.len() == 1 {
                        let s = this.reg_load(args[0])?.to_string();
                        s.chars().next()
                    } else {
                        if crate::config::env::check_contracts() {
                            eprintln!(r#"{{"kind":"contracts_arity_range","box":"StringBox","method":"is_hex_digit_char","min":0,"max":1,"got":{}}}"#, args.len());
                        }
                        return Err(VMError::InvalidInstruction("is_hex_digit_char expects 0 or 1 arg".into()));
                    };
                    let is_hex = ch_opt.map(|c| c.is_ascii_hexdigit()).unwrap_or(false);
                    if let Some(d) = dst { this.regs.insert(d, VMValue::Bool(is_hex)); }
                    return Ok(true);
                }
                _ => {}
            }
        }
        Ok(false)
}
