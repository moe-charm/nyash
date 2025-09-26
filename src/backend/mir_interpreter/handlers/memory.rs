use super::*;

impl MirInterpreter {
    pub(super) fn handle_ref_set(
        &mut self,
        reference: ValueId,
        field: &str,
        value: ValueId,
    ) -> Result<(), VMError> {
        let v = self.reg_load(value)?;
        let key = self.object_key_for(reference);
        self.obj_fields
            .entry(key)
            .or_default()
            .insert(field.into(), v);
        Ok(())
    }

    pub(super) fn handle_ref_get(
        &mut self,
        dst: ValueId,
        reference: ValueId,
        field: &str,
    ) -> Result<(), VMError> {
        let key = self.object_key_for(reference);
        let v = self
            .obj_fields
            .get(&key)
            .and_then(|m| m.get(field))
            .cloned()
            .unwrap_or(VMValue::Void);
        self.regs.insert(dst, v);
        Ok(())
    }

    pub(super) fn handle_load(&mut self, dst: ValueId, ptr: ValueId) -> Result<(), VMError> {
        let v = self.mem.get(&ptr).cloned().unwrap_or(VMValue::Void);
        self.regs.insert(dst, v);
        Ok(())
    }

    pub(super) fn handle_store(&mut self, ptr: ValueId, value: ValueId) -> Result<(), VMError> {
        let v = self.reg_load(value)?;
        self.mem.insert(ptr, v);
        Ok(())
    }
}
