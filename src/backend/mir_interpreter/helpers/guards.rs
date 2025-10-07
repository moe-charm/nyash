use super::super::*;

impl MirInterpreter {
    /// Unified unborn guard: forbid operations on unborn instances.
    /// Allows if `born` is true or currently `in_birth`.
    pub(in crate::backend::mir_interpreter) fn check_unborn_guard(&self, recv_id: ValueId) -> Result<(), VMError> {
        if !crate::config::env::check_contracts() { return Ok(()); }
        let key = self.object_key_for(recv_id);
        let seen_new = self.contracts_new.contains(&key);
        let seen_birth = self.contracts_born.contains(&key) || self.contracts_in_birth.contains(&key);
        if crate::common::lifecycle_contracts::is_unborn_violation(seen_new, seen_birth) {
            return Err(VMError::InvalidInstruction(
                crate::common::lifecycle_contracts::unborn_error_message().to_string(),
            ));
        }
        Ok(())
    }
}
