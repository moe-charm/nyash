use crate::mir::builder::MirBuilder;
use crate::mir::definitions::call_unified::Callee;
use crate::mir::ValueId;

/// Normalize String length methods into a single extern form.
///
/// Rewrites:
/// - Method(receiver, method in {size,len,length}, 0 args)
/// - ModuleFunction("StringBox.{size|len|length}/1")
/// into: Extern("nyrt.string.length"), args=[recv_local]
///
/// Returns true if the callee/args were rewritten.
pub fn normalize_string_length_call(
    _builder: &mut MirBuilder,
    callee: &mut Callee,
    args: &mut Vec<ValueId>,
) -> bool {
    // Already normalized
    if matches!(callee, Callee::Extern(name) if name == "nyrt.string.length") {
        return false;
    }

    // Method form with 0 args
    if let Callee::Method { method, receiver: Some(r), .. } = callee.clone() {
        if (method.as_str() == "size" || method.as_str() == "len" || method.as_str() == "length") && args.is_empty() {
            // receiver is already materialized by finalize_call_operands
            let recv_local = r;
            *callee = Callee::Extern("nyrt.string.length".to_string());
            args.clear();
            args.push(recv_local);
            return true;
        }
    }

    // ModuleFunction form with 1 arg (the receiver)
    if let Callee::ModuleFunction(name) = callee.clone() {
        if (name.starts_with("StringBox.size/") || name.starts_with("StringBox.len/") || name.starts_with("StringBox.length/"))
            && args.len() == 1
        {
            // receiver is already materialized by finalize_call_operands
            let recv_local = args[0];
            *callee = Callee::Extern("nyrt.string.length".to_string());
            args.clear();
            args.push(recv_local);
            return true;
        }
    }

    false
}

