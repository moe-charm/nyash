/*!
 * String Literal Management for WASM Codegen
 */

use crate::backend::wasm::WasmError;
use crate::mir::ValueId;
use std::collections::HashMap;

impl super::WasmCodegen {
    /// Register a string literal and return its data offset
    pub(super) fn register_string_literal(&mut self, string: &str) -> u32 {
        if let Some(&offset) = self.string_literals.get(string) {
            return offset;
        }

        let offset = self.next_data_offset;
        let string_bytes = string.as_bytes();
        self.string_literals.insert(string.to_string(), offset);
        self.next_data_offset += string_bytes.len() as u32;

        offset
    }

    /// Generate data segments for all registered string literals
    pub(super) fn generate_data_segments(&self) -> Vec<String> {
        let mut segments = Vec::new();

        for (string, &offset) in &self.string_literals {
            let string_bytes = string.as_bytes();

            // Convert to hex-escaped string for WAT
            let byte_string = string_bytes
                .iter()
                .map(|b| format!("\\{:02x}", b))
                .collect::<String>();

            let data_segment = format!("(data (i32.const {}) \"{}\")", offset, byte_string);

            segments.push(data_segment);
        }

        segments
    }

    /// Generate StringBox allocation for a string constant
    pub(super) fn generate_string_box_const(
        &self,
        dst: ValueId,
        data_offset: u32,
        string_len: u32,
    ) -> Result<Vec<String>, WasmError> {
        // Allocate a StringBox using the StringBox allocator
        // StringBox layout: [type_id:0x1001][ref_count:1][field_count:2][data_ptr:offset][length:len]
        Ok(vec![
            // Call StringBox allocator function
            "call $alloc_stringbox".to_string(),
            // Store the result (StringBox pointer) in local variable
            format!("local.set ${}", self.get_local_index(dst)?),
            // Initialize StringBox fields
            // Get StringBox pointer back
            format!("local.get ${}", self.get_local_index(dst)?),
            // Set data_ptr field (offset 12 from StringBox pointer)
            "i32.const 12".to_string(),
            "i32.add".to_string(),
            format!("i32.const {}", data_offset),
            "i32.store".to_string(),
            // Get StringBox pointer again
            format!("local.get ${}", self.get_local_index(dst)?),
            // Set length field (offset 16 from StringBox pointer)
            "i32.const 16".to_string(),
            "i32.add".to_string(),
            format!("i32.const {}", string_len),
            "i32.store".to_string(),
        ])
    }
}
