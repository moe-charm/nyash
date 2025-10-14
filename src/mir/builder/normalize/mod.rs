pub mod string_length;
pub mod array_length;

use crate::mir::builder::MirBuilder;
use crate::mir::definitions::call_unified::Callee;
use crate::mir::ValueId;

/// Generic normalize function for length methods (size/len/length)
///
/// Rewrites:
/// - Method(receiver, method in {size,len,length}, 0 args)
/// - ModuleFunction("{box_name}.{size|len|length}/1")
/// into: Extern(extern_name), args=[recv_local]
///
/// Returns true if the callee/args were rewritten.
pub fn normalize_length_call(
    _builder: &mut MirBuilder,
    callee: &mut Callee,
    args: &mut Vec<ValueId>,
    box_name: &str,
    extern_name: &str,
) -> bool {
    // Already normalized
    if matches!(callee, Callee::Extern(name) if name == extern_name) {
        return false;
    }

    // Method form with 0 args
    if let Callee::Method { method, receiver: Some(r), .. } = callee.clone() {
        if (method.as_str() == "size" || method.as_str() == "len" || method.as_str() == "length") && args.is_empty() {
            // receiver is already materialized by finalize_call_operands
            let recv_local = r;
            *callee = Callee::Extern(extern_name.to_string());
            args.clear();
            args.push(recv_local);
            return true;
        }
    }

    // ModuleFunction form with 1 arg (the receiver)
    if let Callee::ModuleFunction(name) = callee.clone() {
        let size_prefix = format!("{}.size/", box_name);
        let len_prefix = format!("{}.len/", box_name);
        let length_prefix = format!("{}.length/", box_name);

        if (name.starts_with(&size_prefix) || name.starts_with(&len_prefix) || name.starts_with(&length_prefix))
            && args.len() == 1
        {
            // receiver is already materialized by finalize_call_operands
            let recv_local = args[0];
            *callee = Callee::Extern(extern_name.to_string());
            args.clear();
            args.push(recv_local);
            return true;
        }
    }

    false
}
