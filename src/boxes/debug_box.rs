use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::Local;
use crate::box_trait::{NyashBox, StringBox, BoolBox, VoidBox};
use crate::interpreter::RuntimeError;
use crate::instance::InstanceBox;
use std::any::Any;

#[derive(Debug, Clone)]
pub struct DebugBox {
    tracking_enabled: Arc<Mutex<bool>>,
    tracked_boxes: Arc<Mutex<HashMap<String, TrackedBoxInfo>>>,
    breakpoints: Arc<Mutex<Vec<String>>>,
    call_stack: Arc<Mutex<Vec<CallInfo>>>,
    id: u64,
}

#[derive(Debug, Clone)]
struct TrackedBoxInfo {
    box_type: String,
    created_at: String,
    fields: String,
    value_repr: String,
}

#[derive(Debug, Clone)]
struct CallInfo {
    function_name: String,
    args: Vec<String>,
    timestamp: String,
}

impl DebugBox {
    pub fn new() -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        DebugBox {
            tracking_enabled: Arc::new(Mutex::new(false)),
            tracked_boxes: Arc::new(Mutex::new(HashMap::new())),
            breakpoints: Arc::new(Mutex::new(Vec::new())),
            call_stack: Arc::new(Mutex::new(Vec::new())),
            id,
        }
    }

    pub fn start_tracking(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let mut enabled = self.tracking_enabled.lock().unwrap();
        *enabled = true;
        println!("[DEBUG] Tracking started");
        Ok(Box::new(VoidBox::new()))
    }

    pub fn stop_tracking(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let mut enabled = self.tracking_enabled.lock().unwrap();
        *enabled = false;
        println!("[DEBUG] Tracking stopped");
        Ok(Box::new(VoidBox::new()))
    }

    pub fn track_box(&self, box_value: &dyn NyashBox, name: &str) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let enabled = self.tracking_enabled.lock().unwrap();
        if !*enabled {
            return Ok(Box::new(VoidBox::new()));
        }

        let mut tracked = self.tracked_boxes.lock().unwrap();
        
        let info = TrackedBoxInfo {
            box_type: box_value.type_name().to_string(),
            created_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            fields: self.get_box_fields(box_value),
            value_repr: box_value.to_string_box().value,
        };
        
        tracked.insert(name.to_string(), info);
        
        Ok(Box::new(VoidBox::new()))
    }

    fn get_box_fields(&self, box_value: &dyn NyashBox) -> String {
        // Try to downcast to InstanceBox to get fields
        if let Some(instance) = box_value.as_any().downcast_ref::<InstanceBox>() {
            let fields = instance.fields.lock().unwrap();
            let field_names: Vec<String> = fields.keys().cloned().collect();
            field_names.join(", ")
        } else {
            "N/A".to_string()
        }
    }

    pub fn dump_all(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let tracked = self.tracked_boxes.lock().unwrap();
        let mut output = String::from("=== Box State Dump ===\n");
        output.push_str(&format!("Time: {}\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
        output.push_str(&format!("Total tracked boxes: {}\n\n", tracked.len()));
        
        for (name, info) in tracked.iter() {
            output.push_str(&format!("Box: {}\n", name));
            output.push_str(&format!("  Type: {}\n", info.box_type));
            output.push_str(&format!("  Created: {}\n", info.created_at));
            output.push_str(&format!("  Fields: {}\n", info.fields));
            output.push_str(&format!("  Value: {}\n", info.value_repr));
            output.push_str("\n");
        }
        
        Ok(Box::new(StringBox::new(output)))
    }

    pub fn save_to_file(&self, filename: &str) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let dump_result = self.dump_all()?;
        let content = dump_result.to_string_box().value;
        
        // Write to file using std::fs
        std::fs::write(filename, content)
            .map_err(|e| RuntimeError::InvalidOperation {
                message: format!("Failed to write debug file: {}", e),
            })?;
        
        println!("[DEBUG] Saved debug info to {}", filename);
        Ok(Box::new(VoidBox::new()))
    }

    pub fn watch(&self, box_value: &dyn NyashBox, name: &str) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let value_str = box_value.to_string_box().value;
        let type_name = box_value.type_name();
        
        println!("[DEBUG] Watching {} ({}): {}", name, type_name, value_str);
        Ok(Box::new(VoidBox::new()))
    }

    pub fn memory_report(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let tracked = self.tracked_boxes.lock().unwrap();
        let mut report = String::from("=== Memory Report ===\n");
        report.push_str(&format!("Tracked boxes: {}\n", tracked.len()));
        
        // Count by type
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        for info in tracked.values() {
            *type_counts.entry(info.box_type.clone()).or_insert(0) += 1;
        }
        
        report.push_str("\nBoxes by type:\n");
        for (box_type, count) in type_counts.iter() {
            report.push_str(&format!("  {}: {}\n", box_type, count));
        }
        
        Ok(Box::new(StringBox::new(report)))
    }

    // Advanced features
    pub fn set_breakpoint(&self, function_name: &str) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let mut breakpoints = self.breakpoints.lock().unwrap();
        breakpoints.push(function_name.to_string());
        println!("[DEBUG] Breakpoint set at function: {}", function_name);
        Ok(Box::new(VoidBox::new()))
    }

    pub fn trace_call(&self, function_name: &str, args: Vec<String>) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let mut stack = self.call_stack.lock().unwrap();
        stack.push(CallInfo {
            function_name: function_name.to_string(),
            args,
            timestamp: Local::now().format("%H:%M:%S.%3f").to_string(),
        });
        
        // Keep only last 100 calls to prevent memory issues
        if stack.len() > 100 {
            stack.remove(0);
        }
        
        Ok(Box::new(VoidBox::new()))
    }

    pub fn show_call_stack(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let stack = self.call_stack.lock().unwrap();
        let mut output = String::from("=== Call Stack ===\n");
        
        for (i, call) in stack.iter().enumerate() {
            output.push_str(&format!("{}: [{}] {}({})\n", 
                i, 
                call.timestamp, 
                call.function_name,
                call.args.join(", ")
            ));
        }
        
        Ok(Box::new(StringBox::new(output)))
    }

    pub fn clear(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let mut tracked = self.tracked_boxes.lock().unwrap();
        tracked.clear();
        
        let mut stack = self.call_stack.lock().unwrap();
        stack.clear();
        
        println!("[DEBUG] Cleared all debug information");
        Ok(Box::new(VoidBox::new()))
    }

    pub fn is_tracking(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let enabled = self.tracking_enabled.lock().unwrap();
        Ok(Box::new(BoolBox::new(*enabled)))
    }

    pub fn get_tracked_count(&self) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let tracked = self.tracked_boxes.lock().unwrap();
        Ok(Box::new(crate::box_trait::IntegerBox::new(tracked.len() as i64)))
    }
}

// Implement NyashBox trait for DebugBox
impl NyashBox for DebugBox {
    fn to_string_box(&self) -> StringBox {
        let tracked = self.tracked_boxes.lock().unwrap();
        StringBox::new(format!("DebugBox[{} tracked]", tracked.len()))
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_debug) = other.as_any().downcast_ref::<DebugBox>() {
            BoolBox::new(self.id == other_debug.id)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn type_name(&self) -> &'static str {
        "DebugBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn box_id(&self) -> u64 {
        self.id
    }
}