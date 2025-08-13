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
        // Assign consistent type IDs for standard Box types
        let type_id = match type_name {
            "StringBox" => 0x1001,
            "IntegerBox" => 0x1002,
            "BoolBox" => 0x1003,
            "ArrayBox" => 0x1004,
            "DataBox" => 0x1005,  // For testing
            _ => {
                // Generate ID from hash for custom types
                type_name.chars().map(|c| c as u32).sum::<u32>() % 65536 + 0x2000
            }
        };
        
        Self {
            type_id,
            size: 12, // Header: type_id + ref_count + field_count
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
        let mut manager = Self {
            box_layouts: HashMap::new(),
            heap_start: 0x800, // 2KB reserved for stack/globals
        };
        
        // Register standard Box types
        manager.register_standard_box_types();
        manager
    }
    
    /// Register standard built-in Box types
    fn register_standard_box_types(&mut self) {
        // StringBox: [type_id][ref_count][field_count][ptr_to_chars][length]
        self.register_box_type("StringBox".to_string(), vec!["data_ptr".to_string(), "length".to_string()]);
        
        // IntegerBox: [type_id][ref_count][field_count][value]
        self.register_box_type("IntegerBox".to_string(), vec!["value".to_string()]);
        
        // BoolBox: [type_id][ref_count][field_count][value]
        self.register_box_type("BoolBox".to_string(), vec!["value".to_string()]);
        
        // DataBox: [type_id][ref_count][field_count][value] - for testing
        self.register_box_type("DataBox".to_string(), vec!["value".to_string()]);
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
    
    /// Generate heap allocation function with 4-byte alignment
    pub fn get_malloc_function(&self) -> String {
        format!(
            r#"(func $malloc (param $size i32) (result i32)
    (local $ptr i32)
    (local $aligned_size i32)
    
    ;; Align size to 4-byte boundary
    local.get $size
    i32.const 3
    i32.add
    i32.const -4
    i32.and
    local.set $aligned_size
    
    ;; Get current heap pointer
    global.get $heap_ptr
    local.set $ptr
    
    ;; Advance heap pointer by aligned size
    global.get $heap_ptr
    local.get $aligned_size
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
    
    ;; Initialize ref_count to 1
    local.get $ptr
    i32.const 4
    i32.add
    i32.const 1
    i32.store
    
    ;; Initialize field_count
    local.get $ptr
    i32.const 8
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
    ;; Verify type_id (optional safety check)
    local.get $box_ptr
    i32.load
    i32.const {}
    i32.ne
    if
        i32.const 0
        return
    end
    
    ;; Load field value
    local.get $box_ptr
    i32.const {}
    i32.add
    i32.load
  )"#,
            type_name.to_lowercase(),
            field_name,
            layout.type_id,
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
    ;; Verify type_id (optional safety check)
    local.get $box_ptr
    i32.load
    i32.const {}
    i32.ne
    if
        return
    end
    
    ;; Store field value
    local.get $box_ptr
    i32.const {}
    i32.add
    local.get $value
    i32.store
  )"#,
            type_name.to_lowercase(),
            field_name,
            layout.type_id,
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
;; Box Layout: [type_id:i32][ref_count:i32][field_count:i32][field0:i32][field1:i32]...
;; 
;; Standard Type IDs:
;; StringBox:  0x1001, IntegerBox: 0x1002, BoolBox: 0x1003
;; ArrayBox:   0x1004, DataBox:    0x1005
;; Custom:     0x2000+
;;
;; Heap start: 0x{:x}
"#,
            self.heap_start
        )
    }
    
    /// Get type ID for a Box type
    pub fn get_type_id(&self, type_name: &str) -> Option<u32> {
        self.box_layouts.get(type_name).map(|layout| layout.type_id)
    }
    
    /// Generate generic Box creation helper
    pub fn get_generic_box_alloc_function(&self) -> String {
        format!(
            r#"(func $box_alloc (param $type_id i32) (param $field_count i32) (result i32)
    (local $ptr i32)
    (local $total_size i32)
    
    ;; Calculate total size: header (12) + fields (field_count * 4)
    local.get $field_count
    i32.const 4
    i32.mul
    i32.const 12
    i32.add
    local.set $total_size
    
    ;; Allocate memory
    local.get $total_size
    call $malloc
    local.set $ptr
    
    ;; Initialize type_id
    local.get $ptr
    local.get $type_id
    i32.store
    
    ;; Initialize ref_count to 1
    local.get $ptr
    i32.const 4
    i32.add
    i32.const 1
    i32.store
    
    ;; Initialize field_count
    local.get $ptr
    i32.const 8
    i32.add
    local.get $field_count
    i32.store
    
    ;; Return box pointer
    local.get $ptr
  )"#
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_box_layout_creation() {
        let layout = BoxLayout::new("DataBox");
        assert_eq!(layout.size, 12); // type_id + ref_count + field_count
        assert_eq!(layout.type_id, 0x1005); // DataBox has specific ID
        assert!(layout.field_offsets.is_empty());
    }
    
    #[test]
    fn test_box_layout_field_addition() {
        let mut layout = BoxLayout::new("DataBox");
        layout.add_field("field1".to_string());
        layout.add_field("field2".to_string());
        
        assert_eq!(layout.size, 20); // 12 + 4 + 4
        assert_eq!(layout.get_field_offset("field1"), Some(12));
        assert_eq!(layout.get_field_offset("field2"), Some(16));
    }
    
    #[test]
    fn test_memory_manager_standard_types() {
        let manager = MemoryManager::new();
        
        // Verify standard types are registered
        assert!(manager.get_box_layout("StringBox").is_some());
        assert!(manager.get_box_layout("IntegerBox").is_some());
        assert!(manager.get_box_layout("BoolBox").is_some());
        assert!(manager.get_box_layout("DataBox").is_some());
        
        // Verify type IDs
        assert_eq!(manager.get_type_id("StringBox"), Some(0x1001));
        assert_eq!(manager.get_type_id("IntegerBox"), Some(0x1002));
        assert_eq!(manager.get_type_id("DataBox"), Some(0x1005));
    }
    
    #[test]
    fn test_memory_manager_registration() {
        let mut manager = MemoryManager::new();
        manager.register_box_type("CustomBox".to_string(), vec!["x".to_string(), "y".to_string()]);
        
        let layout = manager.get_box_layout("CustomBox").unwrap();
        assert_eq!(layout.field_offsets.len(), 2);
        assert!(layout.get_field_offset("x").is_some());
        assert!(layout.get_field_offset("y").is_some());
        assert!(layout.type_id >= 0x2000); // Custom types start at 0x2000
    }
    
    #[test]
    fn test_malloc_function_generation() {
        let manager = MemoryManager::new();
        let malloc_func = manager.get_malloc_function();
        
        assert!(malloc_func.contains("$malloc"));
        assert!(malloc_func.contains("$heap_ptr"));
        assert!(malloc_func.contains("global.get"));
        assert!(malloc_func.contains("i32.and")); // Alignment check
    }
    
    #[test]
    fn test_box_alloc_function_generation() {
        let manager = MemoryManager::new();
        let alloc_func = manager.get_box_alloc_function("DataBox").unwrap();
        
        assert!(alloc_func.contains("$alloc_databox"));
        assert!(alloc_func.contains("call $malloc"));
        assert!(alloc_func.contains("4101")); // 0x1005 type ID for DataBox
        assert!(alloc_func.contains("i32.const 1")); // ref_count initialization
    }
    
    #[test]
    fn test_generic_box_alloc_function() {
        let manager = MemoryManager::new();
        let generic_func = manager.get_generic_box_alloc_function();
        
        assert!(generic_func.contains("$box_alloc"));
        assert!(generic_func.contains("$type_id"));
        assert!(generic_func.contains("$field_count"));
        assert!(generic_func.contains("i32.const 12")); // Header size
    }
}