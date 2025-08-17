/*!
 * WASM Host Functions - Implementation of host functions for WASM execution
 * 
 * Phase 4-3c: Provides actual implementations for env::print and other imports
 * Enables WASM modules to interact with the host environment
 */

use wasmtime::*;
use std::sync::{Arc, Mutex};

/// Host state for WASM execution
pub struct HostState {
    /// Output buffer for captured prints
    pub output: Arc<Mutex<String>>,
}

impl HostState {
    pub fn new() -> Self {
        Self {
            output: Arc::new(Mutex::new(String::new())),
        }
    }
}

/// Create host functions for WASM imports
pub fn create_host_functions(store: &mut Store<HostState>) -> Result<Vec<(String, String, Extern)>, Error> {
    let mut imports = Vec::new();
    
    // env::print - print a Box value (expecting a StringBox pointer)
    let print_func = Func::wrap(&mut *store, |mut caller: Caller<'_, HostState>, box_ptr: i32| {
        // Try to read StringBox content from WASM memory
        if let Some(mem) = caller.get_export("memory").and_then(|e| e.into_memory()) {
            let data = mem.data(&caller);
            let box_offset = box_ptr as usize;
            
            // StringBox layout: [type_id:4][ref_count:4][field_count:4][data_ptr:4][length:4]
            if box_offset + 20 <= data.len() {
                // Read data pointer (offset 12)
                let data_ptr = i32::from_le_bytes([
                    data[box_offset + 12],
                    data[box_offset + 13],
                    data[box_offset + 14],
                    data[box_offset + 15],
                ]);
                
                // Read length (offset 16)
                let length = i32::from_le_bytes([
                    data[box_offset + 16],
                    data[box_offset + 17],
                    data[box_offset + 18],
                    data[box_offset + 19],
                ]);
                
                // Read actual string content
                let str_start = data_ptr as usize;
                let str_end = str_start + length as usize;
                
                if str_end <= data.len() {
                    if let Ok(s) = std::str::from_utf8(&data[str_start..str_end]) {
                        println!("{}", s);
                        return;
                    }
                }
            }
        }
        
        // Fallback: print as pointer
        println!("Box[{}]", box_ptr);
    });
    imports.push(("env".to_string(), "print".to_string(), Extern::Func(print_func)));
    
    // env::print_str - print a string from memory (ptr, len)
    let print_str_func = Func::wrap(&mut *store, |mut caller: Caller<'_, HostState>, ptr: i32, len: i32| {
        if let Some(mem) = caller.get_export("memory").and_then(|e| e.into_memory()) {
            let data = mem.data(&caller);
            let start = ptr as usize;
            let end = start + len as usize;
            
            if end <= data.len() {
                if let Ok(s) = std::str::from_utf8(&data[start..end]) {
                    println!("{}", s);
                    // Note: Output capture removed for simplicity
                }
            }
        }
    });
    imports.push(("env".to_string(), "print_str".to_string(), Extern::Func(print_str_func)));
    
    // env::console_log - console logging (similar to print_str)
    let console_log_func = Func::wrap(&mut *store, |mut caller: Caller<'_, HostState>, ptr: i32, len: i32| {
        if let Some(mem) = caller.get_export("memory").and_then(|e| e.into_memory()) {
            let data = mem.data(&caller);
            let start = ptr as usize;
            let end = start + len as usize;
            
            if end <= data.len() {
                if let Ok(s) = std::str::from_utf8(&data[start..end]) {
                    println!("[console.log] {}", s);
                    // Note: Output capture removed for simplicity
                }
            }
        }
    });
    imports.push(("env".to_string(), "console_log".to_string(), Extern::Func(console_log_func)));
    
    // env::canvas_fillRect - stub implementation
    let canvas_fill_rect = Func::wrap(&mut *store, |_caller: Caller<'_, HostState>, _x: i32, _y: i32, _w: i32, _h: i32, _r: i32, _g: i32, _b: i32, _a: i32| {
        // Stub - in a real implementation, this would draw to a canvas
    });
    imports.push(("env".to_string(), "canvas_fillRect".to_string(), Extern::Func(canvas_fill_rect)));
    
    // env::canvas_fillText - stub implementation
    let canvas_fill_text = Func::wrap(&mut *store, |_caller: Caller<'_, HostState>, _ptr: i32, _len: i32, _x: i32, _y: i32, _size: i32, _r: i32, _g: i32, _b: i32, _a: i32, _align: i32| {
        // Stub - in a real implementation, this would draw text
    });
    imports.push(("env".to_string(), "canvas_fillText".to_string(), Extern::Func(canvas_fill_text)));
    
    // env::box_to_string - convert Box to string representation
    let box_to_string = Func::wrap(&mut *store, |_caller: Caller<'_, HostState>, box_ptr: i32| -> i32 {
        // For now, return the same pointer - in a real implementation,
        // this would convert the Box to its string representation
        box_ptr
    });
    imports.push(("env".to_string(), "box_to_string".to_string(), Extern::Func(box_to_string)));
    
    // env::box_print - print a Box value
    let box_print = Func::wrap(&mut *store, |mut caller: Caller<'_, HostState>, box_ptr: i32| {
        // Read Box type from memory
        if let Some(mem) = caller.get_export("memory").and_then(|e| e.into_memory()) {
            let data = mem.data(&caller);
            let type_id_offset = box_ptr as usize;
            
            if type_id_offset + 4 <= data.len() {
                let type_id = i32::from_le_bytes([
                    data[type_id_offset],
                    data[type_id_offset + 1],
                    data[type_id_offset + 2],
                    data[type_id_offset + 3],
                ]);
                
                match type_id {
                    0x1001 => { // StringBox
                        // Read string pointer and length from Box fields
                        let str_ptr_offset = type_id_offset + 12;
                        let str_len_offset = type_id_offset + 16;
                        
                        if str_len_offset + 4 <= data.len() {
                            let str_ptr = i32::from_le_bytes([
                                data[str_ptr_offset],
                                data[str_ptr_offset + 1],
                                data[str_ptr_offset + 2],
                                data[str_ptr_offset + 3],
                            ]);
                            
                            let str_len = i32::from_le_bytes([
                                data[str_len_offset],
                                data[str_len_offset + 1],
                                data[str_len_offset + 2],
                                data[str_len_offset + 3],
                            ]);
                            
                            // Read actual string content
                            let str_start = str_ptr as usize;
                            let str_end = str_start + str_len as usize;
                            
                            if str_end <= data.len() {
                                if let Ok(s) = std::str::from_utf8(&data[str_start..str_end]) {
                                    println!("{}", s);
                                    // Note: Output capture removed for simplicity
                                    return;
                                }
                            }
                        }
                    },
                    _ => {
                        println!("Box[type=0x{:x}]", type_id);
                    }
                }
            }
        }
        
        println!("Box[unknown]");
    });
    imports.push(("env".to_string(), "box_print".to_string(), Extern::Func(box_print)));
    
    // env::box_equals - check Box equality
    let box_equals = Func::wrap(&mut *store, |_caller: Caller<'_, HostState>, _box1: i32, _box2: i32| -> i32 {
        // Stub - for now always return 0 (false)
        0
    });
    imports.push(("env".to_string(), "box_equals".to_string(), Extern::Func(box_equals)));
    
    // env::box_clone - clone a Box
    let box_clone = Func::wrap(&mut *store, |_caller: Caller<'_, HostState>, box_ptr: i32| -> i32 {
        // Stub - for now return the same pointer
        box_ptr
    });
    imports.push(("env".to_string(), "box_clone".to_string(), Extern::Func(box_clone)));
    
    Ok(imports)
}