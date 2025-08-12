/*!
 * NyashValue - Revolutionary unified value representation system
 * 
 * Replaces Arc<Mutex<T>> overuse with direct value storage for primitives
 * and smart synchronization only where needed.
 * 
 * Inspired by Lua's TValue system for performance-critical language implementations.
 */

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::fmt::{self, Display, Debug};
use crate::box_trait::NyashBox;

/// Revolutionary unified value type - replaces individual Box allocations
#[derive(Clone)]
pub enum NyashValue {
    // Direct primitive values - no Arc<Mutex> overhead
    Integer(i64),
    Float(f64), 
    Bool(bool),
    String(String),
    
    // Collections - need synchronization
    Array(Arc<Mutex<Vec<NyashValue>>>),
    Map(Arc<Mutex<HashMap<String, NyashValue>>>),
    
    // Legacy Box compatibility and custom types
    Box(Arc<Mutex<dyn NyashBox>>),
    
    // Special values
    Null,
    Void,
}

impl NyashValue {
    /// Create a new NyashValue from a primitive value
    pub fn new_integer(value: i64) -> Self {
        NyashValue::Integer(value)
    }
    
    pub fn new_float(value: f64) -> Self {
        NyashValue::Float(value)
    }
    
    pub fn new_bool(value: bool) -> Self {
        NyashValue::Bool(value)
    }
    
    pub fn new_string(value: String) -> Self {
        NyashValue::String(value)
    }
    
    pub fn new_array() -> Self {
        NyashValue::Array(Arc::new(Mutex::new(Vec::new())))
    }
    
    pub fn new_map() -> Self {
        NyashValue::Map(Arc::new(Mutex::new(HashMap::new())))
    }
    
    pub fn new_null() -> Self {
        NyashValue::Null
    }
    
    pub fn new_void() -> Self {
        NyashValue::Void
    }
    
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            NyashValue::Integer(n) => n.to_string(),
            NyashValue::Float(f) => f.to_string(),
            NyashValue::Bool(b) => b.to_string(),
            NyashValue::String(s) => s.clone(),
            NyashValue::Array(arr) => {
                if let Ok(guard) = arr.try_lock() {
                    let elements: Vec<String> = guard.iter()
                        .map(|v| v.to_string())
                        .collect();
                    format!("[{}]", elements.join(", "))
                } else {
                    "[Array (locked)]".to_string()
                }
            },
            NyashValue::Map(map) => {
                if let Ok(guard) = map.try_lock() {
                    let pairs: Vec<String> = guard.iter()
                        .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                        .collect();
                    format!("{{{}}}", pairs.join(", "))
                } else {
                    "{Map (locked)}".to_string()
                }
            },
            NyashValue::Box(b) => {
                if let Ok(guard) = b.try_lock() {
                    guard.to_string_box().value
                } else {
                    "Box (locked)".to_string()
                }
            },
            NyashValue::Null => "null".to_string(),
            NyashValue::Void => "void".to_string(),
        }
    }
    
    /// Convert to integer (with type coercion)
    pub fn to_integer(&self) -> Result<i64, String> {
        match self {
            NyashValue::Integer(n) => Ok(*n),
            NyashValue::Float(f) => Ok(*f as i64),
            NyashValue::Bool(b) => Ok(if *b { 1 } else { 0 }),
            NyashValue::String(s) => {
                s.parse::<i64>()
                    .map_err(|_| format!("Cannot convert '{}' to integer", s))
            },
            _ => Err(format!("Cannot convert {:?} to integer", self.type_name())),
        }
    }
    
    /// Convert to float (with type coercion)
    pub fn to_float(&self) -> Result<f64, String> {
        match self {
            NyashValue::Integer(n) => Ok(*n as f64),
            NyashValue::Float(f) => Ok(*f),
            NyashValue::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
            NyashValue::String(s) => {
                s.parse::<f64>()
                    .map_err(|_| format!("Cannot convert '{}' to float", s))
            },
            _ => Err(format!("Cannot convert {:?} to float", self.type_name())),
        }
    }
    
    /// Convert to boolean (with type coercion)
    pub fn to_bool(&self) -> Result<bool, String> {
        match self {
            NyashValue::Bool(b) => Ok(*b),
            NyashValue::Integer(n) => Ok(*n != 0),
            NyashValue::Float(f) => Ok(*f != 0.0),
            NyashValue::String(s) => Ok(!s.is_empty()),
            NyashValue::Null => Ok(false),
            NyashValue::Void => Ok(false),
            _ => Ok(true), // Arrays, Maps, Boxes are truthy
        }
    }
    
    /// Get the type name for error messages
    pub fn type_name(&self) -> &'static str {
        match self {
            NyashValue::Integer(_) => "Integer",
            NyashValue::Float(_) => "Float", 
            NyashValue::Bool(_) => "Bool",
            NyashValue::String(_) => "String",
            NyashValue::Array(_) => "Array",
            NyashValue::Map(_) => "Map",
            NyashValue::Box(_) => "Box",
            NyashValue::Null => "Null",
            NyashValue::Void => "Void",
        }
    }
    
    /// Check if this value is numeric (Integer or Float)
    pub fn is_numeric(&self) -> bool {
        matches!(self, NyashValue::Integer(_) | NyashValue::Float(_))
    }
    
    /// Check if this value is falsy
    pub fn is_falsy(&self) -> bool {
        matches!(self, NyashValue::Null | NyashValue::Void) || 
        self.to_bool().unwrap_or(false) == false
    }
}

impl PartialEq for NyashValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Exact type matches
            (NyashValue::Integer(a), NyashValue::Integer(b)) => a == b,
            (NyashValue::Float(a), NyashValue::Float(b)) => (a - b).abs() < f64::EPSILON,
            (NyashValue::Bool(a), NyashValue::Bool(b)) => a == b,
            (NyashValue::String(a), NyashValue::String(b)) => a == b,
            (NyashValue::Null, NyashValue::Null) => true,
            (NyashValue::Void, NyashValue::Void) => true,
            
            // Cross-type numeric equality (42 == 42.0)
            (NyashValue::Integer(a), NyashValue::Float(b)) => (*a as f64 - b).abs() < f64::EPSILON,
            (NyashValue::Float(a), NyashValue::Integer(b)) => (a - *b as f64).abs() < f64::EPSILON,
            
            // Arrays and Maps require deep comparison (simplified for now)
            (NyashValue::Array(a), NyashValue::Array(b)) => Arc::ptr_eq(a, b),
            (NyashValue::Map(a), NyashValue::Map(b)) => Arc::ptr_eq(a, b),
            (NyashValue::Box(a), NyashValue::Box(b)) => Arc::ptr_eq(a, b),
            
            // Everything else is not equal
            _ => false,
        }
    }
}

impl Display for NyashValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Debug for NyashValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NyashValue::Integer(n) => write!(f, "Integer({})", n),
            NyashValue::Float(fl) => write!(f, "Float({})", fl),
            NyashValue::Bool(b) => write!(f, "Bool({})", b),
            NyashValue::String(s) => write!(f, "String(\"{}\")", s),
            NyashValue::Array(_) => write!(f, "Array([...])"),
            NyashValue::Map(_) => write!(f, "Map({{...}})"),
            NyashValue::Box(_) => write!(f, "Box(...)"),
            NyashValue::Null => write!(f, "Null"),
            NyashValue::Void => write!(f, "Void"),
        }
    }
}

/// Legacy Box compatibility - convert from existing Box types
impl NyashValue {
    /// Convert from a legacy NyashBox to NyashValue
    pub fn from_box(nyash_box: Arc<Mutex<dyn NyashBox>>) -> Self {
        // Try to extract primitive values for better performance
        if let Ok(guard) = nyash_box.try_lock() {
            let type_name = guard.type_name();
            let string_rep = guard.to_string_box().value;
            
            // Convert common types to direct values
            match type_name {
                "IntegerBox" => {
                    if let Ok(value) = string_rep.parse::<i64>() {
                        return NyashValue::Integer(value);
                    }
                },
                "FloatBox" => {
                    if let Ok(value) = string_rep.parse::<f64>() {
                        return NyashValue::Float(value);
                    }
                },
                "BoolBox" => {
                    if let Ok(value) = string_rep.parse::<bool>() {
                        return NyashValue::Bool(value);
                    }
                },
                "StringBox" => {
                    return NyashValue::String(string_rep);
                },
                "NullBox" => {
                    return NyashValue::Null;
                },
                "VoidBox" => {
                    return NyashValue::Void;
                },
                _ => {}
            }
        }
        
        // Fallback to Box wrapper
        NyashValue::Box(nyash_box)
    }
    
    /// Convert back to a legacy NyashBox for compatibility
    pub fn to_box(&self) -> Result<Arc<Mutex<dyn NyashBox>>, String> {
        use crate::box_trait::{StringBox, IntegerBox, BoolBox, VoidBox};
        use crate::boxes::null_box::NullBox;
        
        match self {
            NyashValue::Integer(n) => {
                Ok(Arc::new(Mutex::new(IntegerBox::new(*n))))
            },
            NyashValue::Float(f) => {
                // Note: Need FloatBox implementation - for now convert to string
                Ok(Arc::new(Mutex::new(StringBox::new(f.to_string()))))
            },
            NyashValue::Bool(b) => {
                Ok(Arc::new(Mutex::new(BoolBox::new(*b))))
            },
            NyashValue::String(s) => {
                Ok(Arc::new(Mutex::new(StringBox::new(s.clone()))))
            },
            NyashValue::Null => {
                Ok(Arc::new(Mutex::new(NullBox::new())))
            },
            NyashValue::Void => {
                Ok(Arc::new(Mutex::new(VoidBox::new())))
            },
            NyashValue::Box(b) => {
                Ok(b.clone())
            },
            _ => Err(format!("Cannot convert {} to legacy Box", self.type_name())),
        }
    }
}

/// Unified object creation system
impl NyashValue {
    /// Create objects of different types with unified interface
    pub fn create_object(type_name: &str, args: Vec<NyashValue>) -> Result<NyashValue, String> {
        match type_name {
            "StringBox" => {
                let value = match args.get(0) {
                    Some(arg) => arg.to_string(),
                    None => String::new(),
                };
                Ok(NyashValue::String(value))
            },
            "IntegerBox" => {
                let value = match args.get(0) {
                    Some(arg) => arg.to_integer()?,
                    None => 0,
                };
                Ok(NyashValue::Integer(value))
            },
            "FloatBox" => {
                let value = match args.get(0) {
                    Some(arg) => arg.to_float()?,
                    None => 0.0,
                };
                Ok(NyashValue::Float(value))
            },
            "BoolBox" => {
                let value = match args.get(0) {
                    Some(arg) => arg.to_bool()?,
                    None => false,
                };
                Ok(NyashValue::Bool(value))
            },
            "ArrayBox" => {
                Ok(NyashValue::Array(Arc::new(Mutex::new(Vec::new()))))
            },
            "MapBox" => {
                Ok(NyashValue::Map(Arc::new(Mutex::new(HashMap::new()))))
            },
            _ => {
                Err(format!("Unknown object type: {}", type_name))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_creation() {
        let int_val = NyashValue::new_integer(42);
        let float_val = NyashValue::new_float(3.14);
        let bool_val = NyashValue::new_bool(true);
        let string_val = NyashValue::new_string("hello".to_string());
        
        assert_eq!(int_val.to_string(), "42");
        assert_eq!(float_val.to_string(), "3.14");
        assert_eq!(bool_val.to_string(), "true");
        assert_eq!(string_val.to_string(), "hello");
    }
    
    #[test]
    fn test_type_conversion() {
        let int_val = NyashValue::new_integer(42);
        assert_eq!(int_val.to_float().unwrap(), 42.0);
        assert_eq!(int_val.to_bool().unwrap(), true);
        
        let float_val = NyashValue::new_float(3.14);
        assert_eq!(float_val.to_integer().unwrap(), 3);
        
        let zero_val = NyashValue::new_integer(0);
        assert_eq!(zero_val.to_bool().unwrap(), false);
    }
    
    #[test]
    fn test_cross_type_equality() {
        let int_val = NyashValue::new_integer(42);
        let float_val = NyashValue::new_float(42.0);
        
        assert_eq!(int_val, float_val);
        assert_eq!(float_val, int_val);
    }
    
    #[test]
    fn test_object_creation() {
        let string_obj = NyashValue::create_object("StringBox", vec![
            NyashValue::new_string("test".to_string())
        ]).unwrap();
        
        assert_eq!(string_obj.to_string(), "test");
        
        let int_obj = NyashValue::create_object("IntegerBox", vec![
            NyashValue::new_integer(100)
        ]).unwrap();
        
        assert_eq!(int_obj.to_integer().unwrap(), 100);
    }
    
    #[test]
    fn test_type_names() {
        assert_eq!(NyashValue::new_integer(1).type_name(), "Integer");
        assert_eq!(NyashValue::new_float(1.0).type_name(), "Float");
        assert_eq!(NyashValue::new_bool(true).type_name(), "Bool");
        assert_eq!(NyashValue::new_string("".to_string()).type_name(), "String");
        assert_eq!(NyashValue::new_null().type_name(), "Null");
        assert_eq!(NyashValue::new_void().type_name(), "Void");
    }
}