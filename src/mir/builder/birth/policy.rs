
use crate::mir::builder::MirBuilder;

pub struct BirthPolicyBox;

impl BirthPolicyBox {
    /// Decide if auto-birth should be emitted based on module function table.
    pub fn should_auto_emit(builder: &MirBuilder, full_name: &str) -> bool {
        if let Some(module) = builder.current_module.as_ref() {
            module.functions.contains_key(full_name)
        } else {
            false
        }
    }
}
