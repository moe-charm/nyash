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
        // Align with calls.rs behavior: Void/BoxRef(VoidBox) prints as null; raw String/StringBox unquoted
        match &v {
            VMValue::Void => { println!("null"); return Ok(()); }
            VMValue::BoxRef(bx) => {
                if bx.as_any().downcast_ref::<crate::box_trait::VoidBox>().is_some() {
                    println!("null"); return Ok(());
                }
                if let Some(sb) = bx.as_any().downcast_ref::<crate::box_trait::StringBox>() {
                    println!("{}", sb.value); return Ok(());
                }
            }
            VMValue::String(s) => { println!("{}", s); return Ok(()); }
            _ => {}
        }
        println!("{}", v.to_string());
        Ok(())
    }
}
