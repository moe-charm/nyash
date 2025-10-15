
use crate::mir::builder::{MirBuilder, MirInstruction, Effect, EffectMask, ValueId};
use super::policy::BirthPolicyBox;

pub struct BirthCallEmitterBox;

impl BirthCallEmitterBox {
    /// Try build auto-birth Call instruction (None if not applicable)
    pub fn try_build(builder: &mut MirBuilder, class: &str, dst: ValueId, args: Vec<ValueId>) -> Option<MirInstruction> {
        if class == "StringBox" { return None; }
        let full_name = crate::mir::resolve::call_name_resolver::CallNameResolverBox::make_birth_name(class, args.len());
        if !BirthPolicyBox::should_auto_emit(builder, &full_name) { return None; }
        // me + args
        let mut call_args: Vec<ValueId> = Vec::with_capacity(1 + args.len());
        call_args.push(dst);
        call_args.extend(args.into_iter());
        let name_val = match crate::mir::builder::name_const::make_name_const_result(builder, &full_name) {
            Ok(v) => v,
            Err(_) => return None,
        };
        Some(MirInstruction::Call {
            dst: None,
            func: name_val,
            callee: Some(crate::mir::definitions::Callee::ModuleFunction(full_name)),
            args: call_args,
            effects: EffectMask::READ.add(Effect::ReadHeap),
        })
    }
}
