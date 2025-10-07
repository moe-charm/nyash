/*!
 * Utility Functions for WASM Codegen
 */

use crate::backend::wasm::WasmError;
use crate::mir::ValueId;

impl super::WasmCodegen {
    /// Get WASM local variable index for ValueId
    pub(super) fn get_local_index(&self, value_id: ValueId) -> Result<u32, WasmError> {
        self.current_locals.get(&value_id).copied().ok_or_else(|| {
            WasmError::CodegenError(format!(
                "Local variable not found for ValueId: {:?}",
                value_id
            ))
        })
    }

    /// Generate print instruction (calls env.print import)
    pub(super) fn generate_print(&self, value: ValueId) -> Result<Vec<String>, WasmError> {
        Ok(vec![
            format!("local.get ${}", self.get_local_index(value)?),
            "call $print".to_string(),
        ])
    }
}
