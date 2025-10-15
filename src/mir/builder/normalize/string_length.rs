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
    builder: &mut MirBuilder,
    callee: &mut Callee,
    args: &mut Vec<ValueId>,
) -> bool {
    super::normalize_length_call(builder, callee, args, "StringBox", "nyrt.string.length")
}

