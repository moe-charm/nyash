use crate::mir::builder::MirBuilder;
use crate::mir::definitions::call_unified::Callee;
use crate::mir::ValueId;

/// Normalize Array length methods into a single extern form.
///
/// Rewrites:
/// - Method(receiver, method in {size,len,length}, 0 args)
/// - ModuleFunction("ArrayBox.{size|len|length}/1")
/// into: Extern("nyrt.array.size"), args=[recv_local]
///
/// Returns true if the callee/args were rewritten.
pub fn normalize_array_length_call(
    builder: &mut MirBuilder,
    callee: &mut Callee,
    args: &mut Vec<ValueId>,
) -> bool {
    super::normalize_length_call(builder, callee, args, "ArrayBox", "nyrt.array.size")
}
