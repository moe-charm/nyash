/*!
 * ExternBox - External API proxy for Phase 9.7 ExternCall
 */

use crate::box_trait::{NyashBox, StringBox, VoidBox, IntegerBox, BoxCore, BoxBase};
use std::any::Any;

/// External API proxy box for external calls
pub struct ExternBox {
    id: u64,
    api_name: String,
}

impl ExternBox {
    pub fn new_console() -> Box<dyn NyashBox> {
        Box::new(ExternBox {
            id: BoxBase::generate_box_id(),
            api_name: "console".to_string(),
        })
    }
    
    pub fn new_canvas() -> Box<dyn NyashBox> {
        Box::new(ExternBox {
            id: BoxBase::generate_box_id(),
            api_name: "canvas".to_string(),
        })
    }
}

impl BoxCore for ExternBox {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn box_clone(&self) -> Box<dyn NyashBox> {
        Box::new(ExternBox { 
            id: self.id,
            api_name: self.api_name.clone(),
        })
    }

    fn box_eq(&self, other: &dyn NyashBox) -> bool {
        if let Some(other_extern) = other.as_any().downcast_ref::<ExternBox>() {
            self.id == other_extern.id
        } else {
            false
        }
    }
}

impl NyashBox for ExternBox {
    fn get_type_name(&self) -> &str {
        "ExternBox"
    }

    fn to_string(&self) -> String {
        format!("ExternBox({})", self.api_name)
    }

    fn call_method(&mut self, method: &str, args: Vec<Box<dyn NyashBox>>) -> Box<dyn NyashBox> {
        println!("ExternBox({})::{} called with {} args", self.api_name, method, args.len());
        
        match (self.api_name.as_str(), method) {
            ("console", "log") => {
                print!("Console: ");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { print!(" "); }
                    print!("{}", arg.to_string());
                }
                println!();
                VoidBox::new()
            },
            ("canvas", "fillRect") => {
                if args.len() >= 6 {
                    println!("Canvas fillRect: canvas={}, x={}, y={}, w={}, h={}, color={}", 
                             args[0].to_string(),
                             args[1].to_string(), 
                             args[2].to_string(),
                             args[3].to_string(),
                             args[4].to_string(),
                             args[5].to_string());
                } else {
                    println!("Canvas fillRect called with {} args (expected 6)", args.len());
                }
                VoidBox::new()
            },
            ("canvas", "fillText") => {
                if args.len() >= 6 {
                    println!("Canvas fillText: canvas={}, text={}, x={}, y={}, font={}, color={}", 
                             args[0].to_string(),
                             args[1].to_string(),
                             args[2].to_string(),
                             args[3].to_string(),
                             args[4].to_string(),
                             args[5].to_string());
                } else {
                    println!("Canvas fillText called with {} args (expected 6)", args.len());
                }
                VoidBox::new()
            },
            _ => {
                println!("Unknown external method: {}.{}", self.api_name, method);
                VoidBox::new()
            }
        }
    }

    fn get_field(&self, _field: &str) -> Option<Box<dyn NyashBox>> {
        None
    }

    fn set_field(&mut self, _field: &str, _value: Box<dyn NyashBox>) -> bool {
        false
    }

    fn list_methods(&self) -> Vec<String> {
        match self.api_name.as_str() {
            "console" => vec!["log".to_string()],
            "canvas" => vec!["fillRect".to_string(), "fillText".to_string()],
            _ => vec![],
        }
    }

    fn list_fields(&self) -> Vec<String> {
        vec![]
    }
}