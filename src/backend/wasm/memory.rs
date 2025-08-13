/*!
 * WASM Memory Management - Box layout and heap allocation
 * 
 * Phase 8.3 PoC2: Implements bump allocator and Box memory layout
 * Memory Layout: 0x000-0x3FF (reserved), 0x400-0x7FF (stack), 0x800+ (heap)
 */

use super::WasmError;
use std::collections::HashMap;

/// Box memory layout definition
#[derive(Debug, Clone)]
pub struct BoxLayout {
    pub type_id: u32,
    pub size: u32,
    pub field_offsets: HashMap<String, u32>,
}

impl BoxLayout {
    pub fn new(type_name: &str) -> Self {
        // Simple type ID generation (hash of name for now)
        let type_id = type_name.chars().map(|c| c as u32).sum::<u32>() % 65536;
        
        Self {
            type_id,
            size: 8, // Minimum size: type_id + field_count
            field_offsets: HashMap::new(),
        }
    }
    
    pub fn add_field(&mut self, field_name: String) {
        let offset = self.size;
        self.field_offsets.insert(field_name, offset);
        self.size += 4; // Each field is 4 bytes (i32)
    }
    
    pub fn get_field_offset(&self, field_name: &str) -> Option<u32> {
        self.field_offsets.get(field_name).copied()
    }
}

/// WASM memory manager
pub struct MemoryManager {
    /// Known Box layouts
    box_layouts: HashMap<String, BoxLayout>,
    /// Current heap pointer (starts at 0x800)
    heap_start: u32,
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {
            box_layouts: HashMap::new(),
            heap_start: 0x800, // 2KB reserved for stack/globals
        }
    }
    
    /// Register a Box type layout
    pub fn register_box_type(&mut self, type_name: String, fields: Vec<String>) {
        let mut layout = BoxLayout::new(&type_name);
        
        for field in fields {
            layout.add_field(field);
        }
        
        self.box_layouts.insert(type_name, layout);
    }
    
    /// Get Box layout by type name
    pub fn get_box_layout(&self, type_name: &str) -> Option<&BoxLayout> {
        self.box_layouts.get(type_name)
    }
    
    /// Generate WASM globals for heap management
    pub fn get_globals(&self) -> Vec<String> {
        vec![
            format!("(global $heap_ptr (mut i32) (i32.const {}))", self.heap_start),
        ]
    }
    
    /// Generate heap allocation function
    pub fn get_malloc_function(&self) -> String {
        format!(
            r#"(func $malloc (param $size i32) (result i32)
    (local $ptr i32)
    
    ;; Get current heap pointer
    global.get $heap_ptr
    local.set $ptr
    
    ;; Advance heap pointer
    global.get $heap_ptr
    local.get $size
    i32.add
    global.set $heap_ptr
    
    ;; Return allocated pointer
    local.get $ptr
  )"#
        )
    }
    
    /// Generate Box allocation function for specific type
    pub fn get_box_alloc_function(&self, type_name: &str) -> Result<String, WasmError> {
        let layout = self.get_box_layout(type_name)
            .ok_or_else(|| WasmError::MemoryError(format!("Unknown box type: {}", type_name)))?;
        
        Ok(format!(
            r#"(func $alloc_{} (result i32)
    (local $ptr i32)
    
    ;; Allocate memory for box
    i32.const {}
    call $malloc
    local.set $ptr
    
    ;; Initialize type_id
    local.get $ptr
    i32.const {}
    i32.store
    
    ;; Initialize field_count
    local.get $ptr
    i32.const 4
    i32.add
    i32.const {}
    i32.store
    
    ;; Return box pointer
    local.get $ptr
  )"#,
            type_name.to_lowercase(),
            layout.size,
            layout.type_id,
            layout.field_offsets.len()
        ))
    }
    
    /// Generate field getter function
    pub fn get_field_get_function(&self, type_name: &str, field_name: &str) -> Result<String, WasmError> {
        let layout = self.get_box_layout(type_name)
            .ok_or_else(|| WasmError::MemoryError(format!("Unknown box type: {}", type_name)))?;
        
        let offset = layout.get_field_offset(field_name)
            .ok_or_else(|| WasmError::MemoryError(format!("Unknown field: {}.{}", type_name, field_name)))?;
        
        Ok(format!(
            r#"(func $get_{}_{} (param $box_ptr i32) (result i32)
    local.get $box_ptr
    i32.const {}
    i32.add
    i32.load
  )"#,
            type_name.to_lowercase(),
            field_name,
            offset
        ))
    }
    
    /// Generate field setter function
    pub fn get_field_set_function(&self, type_name: &str, field_name: &str) -> Result<String, WasmError> {
        let layout = self.get_box_layout(type_name)
            .ok_or_else(|| WasmError::MemoryError(format!("Unknown box type: {}", type_name)))?;
        
        let offset = layout.get_field_offset(field_name)
            .ok_or_else(|| WasmError::MemoryError(format!("Unknown field: {}.{}", type_name, field_name)))?;
        
        Ok(format!(
            r#"(func $set_{}_{} (param $box_ptr i32) (param $value i32)
    local.get $box_ptr
    i32.const {}
    i32.add
    local.get $value
    i32.store
  )"#,
            type_name.to_lowercase(),
            field_name,
            offset
        ))
    }
    
    /// Get memory layout constants for documentation
    pub fn get_memory_layout_info(&self) -> String {
        format!(
            r#"
;; Memory Layout:
;; 0x000-0x3FF: Reserved/globals (1KB)
;; 0x400-0x7FF: Stack space (1KB)  
;; 0x800+:      Heap (bump allocator)
;;
;; Box Layout: [type_id:i32][field_count:i32][field0:i32][field1:i32]...
;; 
;; Heap start: 0x{:x}
"#,
            self.heap_start
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_box_layout_creation() {
        let layout = BoxLayout::new("TestBox");
        assert_eq!(layout.size, 8); // type_id + field_count
        assert!(layout.field_offsets.is_empty());
    }
    
    #[test]
    fn test_box_layout_field_addition() {
        let mut layout = BoxLayout::new("TestBox");
        layout.add_field("field1".to_string());
        layout.add_field("field2".to_string());
        
        assert_eq!(layout.size, 16); // 8 + 4 + 4
        assert_eq!(layout.get_field_offset("field1"), Some(8));
        assert_eq!(layout.get_field_offset("field2"), Some(12));
    }
    
    #[test]
    fn test_memory_manager_registration() {
        let mut manager = MemoryManager::new();
        manager.register_box_type("DataBox".to_string(), vec!["x".to_string(), "y".to_string()]);
        
        let layout = manager.get_box_layout("DataBox").unwrap();
        assert_eq!(layout.field_offsets.len(), 2);
        assert!(layout.get_field_offset("x").is_some());
        assert!(layout.get_field_offset("y").is_some());
    }
    
    #[test]
    fn test_malloc_function_generation() {
        let manager = MemoryManager::new();
        let malloc_func = manager.get_malloc_function();
        
        assert!(malloc_func.contains("$malloc"));
        assert!(malloc_func.contains("$heap_ptr"));
        assert!(malloc_func.contains("global.get"));
    }
    
    #[test]
    fn test_box_alloc_function_generation() {
        let mut manager = MemoryManager::new();
        manager.register_box_type("TestBox".to_string(), vec!["value".to_string()]);
        
        let alloc_func = manager.get_box_alloc_function("TestBox").unwrap();
        assert!(alloc_func.contains("$alloc_testbox"));
        assert!(alloc_func.contains("call $malloc"));
    }
}