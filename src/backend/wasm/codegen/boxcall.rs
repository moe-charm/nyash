/*!
 * BoxCall Method Handler Generation
 */

use crate::backend::wasm::WasmError;
use crate::mir::ValueId;

impl super::WasmCodegen {
    /// Generate BoxCall method invocation
    /// Implements critical Box methods: toString, print, equals, clone
    pub(super) fn generate_box_call(
        &mut self,
        dst: Option<ValueId>,
        box_val: ValueId,
        method: &str,
        args: &[ValueId],
    ) -> Result<Vec<String>, WasmError> {
        match method {
            "toString" => self.generate_to_string_call(dst, box_val),
            "print" => self.generate_print_call(dst, box_val),
            "equals" => self.generate_equals_call(dst, box_val, args),
            "clone" => self.generate_clone_call(dst, box_val),
            "log" => self.generate_log_call(dst, box_val, args),
            _ => Err(WasmError::UnsupportedInstruction(format!(
                "Unsupported BoxCall method: {}",
                method
            ))),
        }
    }

    /// Generate toString() method call - Box → String conversion
    fn generate_to_string_call(
        &mut self,
        dst: Option<ValueId>,
        box_val: ValueId,
    ) -> Result<Vec<String>, WasmError> {
        let Some(dst) = dst else {
            return Err(WasmError::CodegenError(
                "toString() requires destination".to_string(),
            ));
        };

        Ok(vec![
            format!(
                ";; toString() implementation for ValueId({})",
                box_val.as_u32()
            ),
            format!("local.get ${}", self.get_local_index(box_val)?),
            "call $box_to_string".to_string(),
            format!("local.set ${}", self.get_local_index(dst)?),
        ])
    }

    /// Generate print() method call - Basic output
    fn generate_print_call(
        &mut self,
        dst: Option<ValueId>,
        box_val: ValueId,
    ) -> Result<Vec<String>, WasmError> {
        let mut instructions = vec![
            format!(
                ";; print() implementation for ValueId({})",
                box_val.as_u32()
            ),
            format!("local.get ${}", self.get_local_index(box_val)?),
            "call $box_print".to_string(),
        ];

        // Store void result if destination is provided
        if let Some(dst) = dst {
            instructions.extend(vec![
                "i32.const 0".to_string(), // Void result
                format!("local.set ${}", self.get_local_index(dst)?),
            ]);
        }

        Ok(instructions)
    }

    /// Generate equals() method call - Box comparison
    fn generate_equals_call(
        &mut self,
        dst: Option<ValueId>,
        box_val: ValueId,
        args: &[ValueId],
    ) -> Result<Vec<String>, WasmError> {
        let Some(dst) = dst else {
            return Err(WasmError::CodegenError(
                "equals() requires destination".to_string(),
            ));
        };

        if args.len() != 1 {
            return Err(WasmError::CodegenError(format!(
                "equals() expects 1 argument, got {}",
                args.len()
            )));
        }

        Ok(vec![
            format!(
                ";; equals() implementation for ValueId({}) == ValueId({})",
                box_val.as_u32(),
                args[0].as_u32()
            ),
            format!("local.get ${}", self.get_local_index(box_val)?),
            format!("local.get ${}", self.get_local_index(args[0])?),
            "call $box_equals".to_string(),
            format!("local.set ${}", self.get_local_index(dst)?),
        ])
    }

    /// Generate clone() method call - Box duplication
    fn generate_clone_call(
        &mut self,
        dst: Option<ValueId>,
        box_val: ValueId,
    ) -> Result<Vec<String>, WasmError> {
        let Some(dst) = dst else {
            return Err(WasmError::CodegenError(
                "clone() requires destination".to_string(),
            ));
        };

        Ok(vec![
            format!(
                ";; clone() implementation for ValueId({})",
                box_val.as_u32()
            ),
            format!("local.get ${}", self.get_local_index(box_val)?),
            "call $box_clone".to_string(),
            format!("local.set ${}", self.get_local_index(dst)?),
        ])
    }

    /// Generate log() method call - Console logging (ConsoleBox.log)
    fn generate_log_call(
        &mut self,
        dst: Option<ValueId>,
        box_val: ValueId,
        args: &[ValueId],
    ) -> Result<Vec<String>, WasmError> {
        let mut instructions = vec![format!(
            ";; log() implementation for ValueId({})",
            box_val.as_u32()
        )];

        // Load box_val (ConsoleBox instance)
        instructions.push(format!("local.get ${}", self.get_local_index(box_val)?));

        // Load all arguments
        for arg in args {
            instructions.push(format!("local.get ${}", self.get_local_index(*arg)?));
        }

        // Call console log function
        instructions.push("call $console_log".to_string());

        // Store void result if destination is provided
        if let Some(dst) = dst {
            instructions.extend(vec![
                "i32.const 0".to_string(), // Void result
                format!("local.set ${}", self.get_local_index(dst)?),
            ]);
        }

        Ok(instructions)
    }
}
