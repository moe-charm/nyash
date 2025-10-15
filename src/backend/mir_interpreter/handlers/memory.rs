use super::*;

impl MirInterpreter {
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
