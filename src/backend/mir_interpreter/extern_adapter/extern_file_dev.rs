// extern_file_dev.rs — File I/O externs (dev convenience)
use std::collections::HashMap;

use crate::backend::vm_types::{VMError, VMValue};

pub fn register(map: &mut HashMap<(String, String), super::HandlerFn>) {
    // nyrt.file.read(path: String) -> String
    map.insert(("nyrt.file".into(), "read".into()), |args: &[VMValue]| {
        if args.is_empty() {
            return Err(VMError::InvalidInstruction(
                "nyrt.file.read requires path argument".into(),
            ));
        }
        let path = match &args[0] {
            VMValue::String(s) => s.clone(),
            v => v.to_string(),
        };

        match std::fs::read_to_string(&path) {
            Ok(content) => Ok(VMValue::String(content)),
            Err(e) => Err(VMError::IoError(format!(
                "Failed to read file '{}': {}",
                path, e
            ))),
        }
    });

    // nyrt.file.write(path: String, content: String) -> Void
    map.insert(("nyrt.file".into(), "write".into()), |args: &[VMValue]| {
        if args.len() < 2 {
            return Err(VMError::InvalidInstruction(
                "nyrt.file.write requires path and content arguments".into(),
            ));
        }
        let path = match &args[0] {
            VMValue::String(s) => s.clone(),
            v => v.to_string(),
        };
        let content = match &args[1] {
            VMValue::String(s) => s.clone(),
            v => v.to_string(),
        };

        match std::fs::write(&path, content) {
            Ok(_) => Ok(VMValue::Void),
            Err(e) => Err(VMError::IoError(format!(
                "Failed to write file '{}': {}",
                path, e
            ))),
        }
    });
}

