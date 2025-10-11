use super::*;

impl MirInterpreter {
    pub(super) fn handle_debug(&mut self, message: &str, value: ValueId) -> Result<(), VMError> {
        let v = self.reg_load(value)?;
        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
            eprintln!("[mir-debug] {} => {:?}", message, v);
        }
        Ok(())
    }

    pub(super) fn handle_print(&mut self, value: ValueId) -> Result<(), VMError> {
        let v = self.reg_load(value)?;
        crate::runtime::console_adapter::print_value(&v);
        Ok(())
    }
}
