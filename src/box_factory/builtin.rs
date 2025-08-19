/*!
 * Builtin Box Factory
 * 
 * Handles creation of all built-in Box types (StringBox, IntegerBox, etc.)
 * Replaces the 600+ line match statement with clean factory pattern
 */

use super::BoxFactory;
use crate::box_trait::NyashBox;
use crate::interpreter::RuntimeError;
use crate::boxes::*;
use std::collections::HashMap;

type BoxCreator = Box<dyn Fn(&[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, RuntimeError> + Send + Sync>;

/// Factory for all built-in Box types
pub struct BuiltinBoxFactory {
    /// Map of Box type names to their creation functions
    creators: HashMap<String, BoxCreator>,
}

impl BuiltinBoxFactory {
    /// Create a new factory with all built-in types registered
    pub fn new() -> Self {
        let mut factory = Self {
            creators: HashMap::new(),
        };
        
        // Register all built-in Box types
        factory.register_basic_types();
        factory.register_container_types();
        factory.register_utility_types();
        factory.register_io_types();
        #[cfg(not(target_arch = "wasm32"))]
        factory.register_native_types();
        #[cfg(target_arch = "wasm32")]
        factory.register_wasm_types();
        
        factory
    }
    
    /// Register basic data types
    fn register_basic_types(&mut self) {
        // StringBox
        self.register("StringBox", |args| {
            let value = match args.get(0) {
                Some(arg) => arg.to_string_box().value,
                None => String::new(),
            };
            Ok(Box::new(StringBox::new(value)))
        });
        
        // IntegerBox
        self.register("IntegerBox", |args| {
            let value = match args.get(0) {
                Some(arg) => {
                    // Try direct downcast first
                    if let Some(int_box) = arg.as_any().downcast_ref::<IntegerBox>() {
                        int_box.value
                    } else {
                        // Parse from string
                        arg.to_string_box().value.parse::<i64>()
                            .map_err(|_| RuntimeError::TypeError {
                                message: format!("Cannot convert '{}' to integer", arg.to_string_box().value),
                            })?
                    }
                },
                None => 0,
            };
            Ok(Box::new(IntegerBox::new(value)))
        });
        
        // BoolBox
        self.register("BoolBox", |args| {
            let value = match args.get(0) {
                Some(arg) => {
                    if let Some(bool_box) = arg.as_any().downcast_ref::<BoolBox>() {
                        bool_box.value
                    } else {
                        match arg.to_string_box().value.to_lowercase().as_str() {
                            "true" => true,
                            "false" => false,
                            _ => return Err(RuntimeError::TypeError {
                                message: format!("Cannot convert '{}' to boolean", arg.to_string_box().value),
                            }),
                        }
                    }
                },
                None => false,
            };
            Ok(Box::new(BoolBox::new(value)))
        });
        
        // FloatBox
        self.register("FloatBox", |args| {
            let value = match args.get(0) {
                Some(arg) => {
                    if let Some(float_box) = arg.as_any().downcast_ref::<FloatBox>() {
                        float_box.value
                    } else if let Some(int_box) = arg.as_any().downcast_ref::<IntegerBox>() {
                        int_box.value as f64
                    } else {
                        arg.to_string_box().value.parse::<f64>()
                            .map_err(|_| RuntimeError::TypeError {
                                message: format!("Cannot convert '{}' to float", arg.to_string_box().value),
                            })?
                    }
                },
                None => 0.0,
            };
            Ok(Box::new(FloatBox::new(value)))
        });
        
        // NullBox
        self.register("NullBox", |_args| {
            Ok(Box::new(NullBox::new()))
        });
    }
    
    /// Register container types
    fn register_container_types(&mut self) {
        // ArrayBox
        self.register("ArrayBox", |args| {
            if !args.is_empty() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("ArrayBox constructor expects 0 arguments, got {}", args.len()),
                });
            }
            Ok(Box::new(ArrayBox::new()))
        });
        
        // MapBox
        self.register("MapBox", |args| {
            if !args.is_empty() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("MapBox constructor expects 0 arguments, got {}", args.len()),
                });
            }
            Ok(Box::new(MapBox::new()))
        });
        
        // ResultBox
        self.register("ResultBox", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("ResultBox constructor expects 1 argument, got {}", args.len()),
                });
            }
            let value = args[0].clone_box();
            Ok(Box::new(crate::boxes::result::NyashResultBox::new_ok(value)))
        });
    }
    
    /// Register utility types
    fn register_utility_types(&mut self) {
        // MathBox
        self.register("MathBox", |args| {
            if !args.is_empty() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("MathBox constructor expects 0 arguments, got {}", args.len()),
                });
            }
            Ok(Box::new(MathBox::new()))
        });
        
        // RandomBox
        self.register("RandomBox", |args| {
            if !args.is_empty() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("RandomBox constructor expects 0 arguments, got {}", args.len()),
                });
            }
            Ok(Box::new(RandomBox::new()))
        });
        
        // TimeBox
        self.register("TimeBox", |args| {
            if !args.is_empty() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("TimeBox constructor expects 0 arguments, got {}", args.len()),
                });
            }
            Ok(Box::new(TimeBox::new()))
        });
        
        // DebugBox
        self.register("DebugBox", |args| {
            if !args.is_empty() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("DebugBox constructor expects 0 arguments, got {}", args.len()),
                });
            }
            Ok(Box::new(DebugBox::new()))
        });
    }
    
    /// Register I/O types
    fn register_io_types(&mut self) {
        // ConsoleBox
        self.register("ConsoleBox", |args| {
            if !args.is_empty() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("ConsoleBox constructor expects 0 arguments, got {}", args.len()),
                });
            }
            Ok(Box::new(ConsoleBox::new()))
        });
        
        // SoundBox
        self.register("SoundBox", |args| {
            if !args.is_empty() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("SoundBox constructor expects 0 arguments, got {}", args.len()),
                });
            }
            Ok(Box::new(SoundBox::new()))
        });
    }
    
    /// Register native-only types
    #[cfg(not(target_arch = "wasm32"))]
    fn register_native_types(&mut self) {
        // DateTimeBox
        self.register("DateTimeBox", |args| {
            if !args.is_empty() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("DateTimeBox constructor expects 0 arguments, got {}", args.len()),
                });
            }
            Ok(Box::new(DateTimeBox::now()))
        });
        
        // Additional native types can be registered here
        #[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
        {
            self.register("EguiBox", |args| {
                if !args.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("EguiBox constructor expects 0 arguments, got {}", args.len()),
                    });
                }
                Ok(Box::new(crate::boxes::EguiBox::new()))
            });
        }
    }
    
    /// Register WASM-specific types
    #[cfg(target_arch = "wasm32")]
    fn register_wasm_types(&mut self) {
        // WebDisplayBox
        self.register("WebDisplayBox", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("WebDisplayBox constructor expects 1 argument (element_id), got {}", args.len()),
                });
            }
            
            if let Some(id_str) = args[0].as_any().downcast_ref::<StringBox>() {
                Ok(Box::new(crate::boxes::WebDisplayBox::new(id_str.value.clone())))
            } else {
                Err(RuntimeError::TypeError {
                    message: "WebDisplayBox constructor requires string element_id argument".to_string(),
                })
            }
        });
        
        // Additional WASM types can be registered here
    }
    
    /// Register a Box creator function
    fn register<F>(&mut self, name: &str, creator: F)
    where
        F: Fn(&[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, RuntimeError> + Send + Sync + 'static,
    {
        self.creators.insert(name.to_string(), Box::new(creator));
    }
}

impl BoxFactory for BuiltinBoxFactory {
    fn create_box(
        &self,
        name: &str,
        args: &[Box<dyn NyashBox>],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        if let Some(creator) = self.creators.get(name) {
            creator(args)
        } else {
            Err(RuntimeError::InvalidOperation {
                message: format!("Unknown built-in Box type: {}", name),
            })
        }
    }
    
    fn box_types(&self) -> Vec<&str> {
        self.creators.keys().map(|s| s.as_str()).collect()
    }
}

/// Declarative macro for registering multiple Box types at once
#[macro_export]
macro_rules! register_builtins {
    ($factory:expr, $($box_name:literal => $creator_fn:expr),* $(,)?) => {
        $(
            $factory.register($box_name, $creator_fn);
        )*
    };
}