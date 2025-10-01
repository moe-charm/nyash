use super::*;
use crate::box_trait::NyashBox;

// InstanceBox method dispatch lifted from boxes.rs (behavior unchanged).
pub(super) fn try_handle_instance_box(
    interp: &mut MirInterpreter,
    dst: Option<ValueId>,
    box_val: ValueId,
    method: &str,
    args: &[ValueId],
) -> Result<bool, VMError> {
    let recv_vm = interp.reg_load(box_val)?;
    let recv_box_any: Box<dyn NyashBox> = match recv_vm.clone() {
        VMValue::BoxRef(b) => b.share_box(),
        other => other.to_nyash_box(),
    };

    if std::env::var("NYASH_VM_TRACE").ok().as_deref() == Some("1") && method == "toString" {
        eprintln!(
            "[vm-trace] instance-check recv_box_any.type={} args_len={}",
            recv_box_any.type_name(),
            args.len()
        );
    }

    if let Some(inst) = recv_box_any.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
        // Development guard: ensure JsonScanner core fields have sensible defaults
        if inst.class_name == "JsonScanner" {
            if inst.get_field_ng("position").is_none() {
                let _ = inst.set_field_ng("position".to_string(), crate::value::NyashValue::Integer(0));
            }
            if inst.get_field_ng("length").is_none() {
                let _ = inst.set_field_ng("length".to_string(), crate::value::NyashValue::Integer(0));
            }
            if inst.get_field_ng("line").is_none() {
                let _ = inst.set_field_ng("line".to_string(), crate::value::NyashValue::Integer(1));
            }
            if inst.get_field_ng("column").is_none() {
                let _ = inst.set_field_ng("column".to_string(), crate::value::NyashValue::Integer(1));
            }
            if inst.get_field_ng("text").is_none() {
                let _ = inst.set_field_ng("text".to_string(), crate::value::NyashValue::String(String::new()));
            }
        }

        // Build candidate names
        let primary = format!("{}.{}{}", inst.class_name, method, format!("/{}", args.len()));
        let alt = format!("{}Instance.{}{}", inst.class_name, method, format!("/{}", args.len()));
        let static_variant = format!("{}.{}{}", inst.class_name, method, format!("/{}", args.len() + 1));
        let (stringify_base, stringify_inst) = if method == "toString" && args.is_empty() {
            let base = inst.class_name.strip_suffix("Instance").map(|s| s.to_string());
            let base_name = base.unwrap_or_else(|| inst.class_name.clone());
            (
                Some(format!("{}.stringify/0", base_name)),
                Some(format!("{}.stringify/0", inst.class_name)),
            )
        } else {
            (None, None)
        };

        if std::env::var("NYASH_VM_TRACE").ok().as_deref() == Some("1") {
            eprintln!(
                "[vm-trace] instance-dispatch class={} method={} arity={} candidates=[{}, {}, {}]",
                inst.class_name,
                method,
                args.len(),
                primary,
                alt,
                static_variant
            );
        }

        // Prefer stringify candidates for toString()
        let func_opt = if let Some(ref sname) = stringify_inst {
            interp.functions.get(sname).cloned()
        } else {
            None
        }
        .or_else(|| stringify_base.as_ref().and_then(|n| interp.functions.get(n).cloned()))
        .or_else(|| interp.functions.get(&primary).cloned())
        .or_else(|| interp.functions.get(&alt).cloned())
        .or_else(|| interp.functions.get(&static_variant).cloned());

        if let Some(func) = func_opt {
            if std::env::var("NYASH_VM_TRACE").ok().as_deref() == Some("1") {
                eprintln!("[vm-trace] instance-dispatch hit -> {}", func.signature.name);
            }
            let mut argv: Vec<VMValue> = Vec::with_capacity(1 + args.len());
            // birth() dev assert: forbid birth(me==Void)
            if method == "birth" && crate::config::env::using_is_dev() {
                let recv_v = match &recv_vm {
                    VMValue::BoxRef(_) => recv_vm.clone(),
                    _ => VMValue::Void,
                };
                if matches!(recv_v, VMValue::Void) {
                    if crate::config::env::cli_verbose() && !crate::config::env::cli_quiet() {
                        eprintln!("[warn] dev verify: NewBox→birth invariant warnings: me==Void");
                    }
                }
            }
            argv.push(recv_vm);
            for a in args { argv.push(interp.reg_load(*a)?); }
            let ret = interp.exec_function_inner(&func, Some(&argv))?;
            if let Some(d) = dst { interp.regs.insert(d, ret); }
            return Ok(true);
        }
    }

    // If we reach here, no instance or no function matched
    Ok(false)
}
