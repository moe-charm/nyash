/*!
 * Builtin Box Factory
 * 
 * Handles creation of all built-in Box types (StringBox, IntegerBox, etc.)
 * Replaces the 600+ line match statement with clean factory pattern
 */

use super::BoxFactory;
use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox};
use crate::interpreter::RuntimeError;
use crate::boxes::*;
use crate::method_box::MethodBox;
use crate::boxes::p2p_box::TransportKind;
use crate::boxes::math_box::RangeBox;
use std::collections::HashMap;

/// Group switches to control which builtin types are registered
#[derive(Debug, Clone, Copy)]
pub struct BuiltinGroups {
    pub basic: bool,      // String, Integer, Bool, Float, Null
    pub container: bool,  // Array, Map, Result, Buffer
    pub utility: bool,    // Math, Random, Time, Debug
    pub io: bool,         // Console, Sound
    pub network: bool,    // Socket, HTTP*
    pub text: bool,       // Regex, JSON
    pub misc: bool,       // Stream, Range, Method, Intent, Error
    pub native: bool,     // DateTime, Timer, Egui (cfg-gated)
    pub wasm: bool,       // Web* (cfg-gated)
}

impl Default for BuiltinGroups {
    fn default() -> Self {
        Self {
            basic: true,
            container: true,
            utility: true,
            io: true,
            network: true,
            text: true,
            misc: true,
            native: true,
            wasm: true,
        }
    }
}

impl BuiltinGroups {
    /// Native full preset (default): all groups enabled
    pub fn native_full() -> Self { Self::default() }

    /// Native minimal preset: disable network-related boxes
    pub fn native_minimal() -> Self {
        Self { network: false, ..Self::default() }
    }

    /// WASM playground preset: enable core features, disable native/network/io
    /// - native: false (no DateTimeBox/TimerBox/Egui)
    /// - io: false (no ConsoleBox/SoundBox)
    /// - network: false (no Socket/HTTP/P2P)
    /// - wasm: true (enable Web* boxes)
    pub fn wasm_playground() -> Self {
        Self {
            native: false,
            io: false,
            network: false,
            wasm: true,
            ..Self::default()
        }
    }
}

type BoxCreator = Box<dyn Fn(&[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, RuntimeError> + Send + Sync>;

/// Factory for all built-in Box types
pub struct BuiltinBoxFactory {
    /// Map of Box type names to their creation functions
    creators: HashMap<String, BoxCreator>,
}

impl BuiltinBoxFactory {
    /// Create a new factory with default (all) groups registered
    pub fn new() -> Self {
        Self::new_with_groups(BuiltinGroups::default())
    }

    /// Create a new factory with group-based registration control
    pub fn new_with_groups(groups: BuiltinGroups) -> Self {
        let mut factory = Self { creators: HashMap::new() };

        if groups.basic { factory.register_basic_types(); }
        if groups.container { factory.register_container_types(); }
        if groups.utility { factory.register_utility_types(); }
        if groups.io { factory.register_io_types(); }
        if groups.network { factory.register_network_types(); }
        if groups.text { factory.register_text_types(); }
        if groups.misc { factory.register_misc_types(); }

        // Platform-specific sets
        #[cfg(not(target_arch = "wasm32"))]
        {
            if groups.native { factory.register_native_types(); }
        }
        #[cfg(target_arch = "wasm32")]
        {
            if groups.wasm { factory.register_wasm_types(); }
        }

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
            // Return StringBox directly without InstanceBox wrapper
            Ok(Box::new(StringBox::new(value)) as Box<dyn NyashBox>)
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
        
        // BufferBox
        self.register("BufferBox", |args| {
            if !args.is_empty() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("BufferBox constructor expects 0 arguments, got {}", args.len()),
                });
            }
            Ok(Box::new(BufferBox::new()))
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

    /// Register networking-related types (sockets, HTTP)
    fn register_network_types(&mut self) {
        // SocketBox
        self.register("SocketBox", |args| {
            if !args.is_empty() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("SocketBox constructor expects 0 arguments, got {}", args.len()),
                });
            }
            Ok(Box::new(SocketBox::new()))
        });
        
        // HTTPClientBox
        self.register("HTTPClientBox", |args| {
            if !args.is_empty() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("HTTPClientBox constructor expects 0 arguments, got {}", args.len()),
                });
            }
            Ok(Box::new(HttpClientBox::new()))
        });
        
        // HTTPServerBox
        self.register("HTTPServerBox", |args| {
            if !args.is_empty() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("HTTPServerBox constructor expects 0 arguments, got {}", args.len()),
                });
            }
            Ok(Box::new(HTTPServerBox::new()))
        });
        
        // HTTPRequestBox
        self.register("HTTPRequestBox", |args| {
            if !args.is_empty() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("HTTPRequestBox constructor expects 0 arguments, got {}", args.len()),
                });
            }
            Ok(Box::new(HTTPRequestBox::new()))
        });
        
        // HTTPResponseBox
        self.register("HTTPResponseBox", |args| {
            if !args.is_empty() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("HTTPResponseBox constructor expects 0 arguments, got {}", args.len()),
                });
            }
            Ok(Box::new(HTTPResponseBox::new()))
        });

        // P2PBox
        self.register("P2PBox", |args| {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("P2PBox constructor expects 2 arguments (node_id, transport_type), got {}", args.len()),
                });
            }
            let node_id = args[0].to_string_box().value;
            let transport_str = args[1].to_string_box().value;
            let transport_kind = transport_str.parse::<TransportKind>()
                .map_err(|e| RuntimeError::InvalidOperation { message: e })?;
            Ok(Box::new(P2PBox::new(node_id, transport_kind)))
        });
    }

    /// Register text/format related types (Regex, JSON)
    fn register_text_types(&mut self) {
        // RegexBox
        self.register("RegexBox", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("RegexBox constructor expects 1 argument, got {}", args.len()),
                });
            }
            let pattern = args[0].to_string_box().value;
            match RegexBox::new(&pattern) {
                Ok(regex_box) => Ok(Box::new(regex_box)),
                Err(e) => Err(RuntimeError::InvalidOperation { message: format!("Invalid regex pattern: {}", e) }),
            }
        });
        
        // JSONBox
        self.register("JSONBox", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("JSONBox constructor expects 1 argument, got {}", args.len()),
                });
            }
            let json_str = args[0].to_string_box().value;
            match JSONBox::from_str(&json_str) {
                Ok(json_box) => Ok(Box::new(json_box)),
                Err(e) => Err(RuntimeError::InvalidOperation { message: format!("Invalid JSON: {}", e) }),
            }
        });
    }

    /// Register various utility types not covered elsewhere
    fn register_misc_types(&mut self) {
        // StreamBox
        self.register("StreamBox", |args| {
            if !args.is_empty() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("StreamBox constructor expects 0 arguments, got {}", args.len()),
                });
            }
            Ok(Box::new(StreamBox::new()))
        });

        // TimerBox (native only)
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.register("TimerBox", |args| {
                if !args.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("TimerBox constructor expects 0 arguments, got {}", args.len()),
                    });
                }
                Ok(Box::new(TimerBox::new()))
            });
        }

        // RangeBox
        self.register("RangeBox", |args| {
            if args.len() < 2 || args.len() > 3 {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("RangeBox constructor expects 2-3 arguments, got {}", args.len()),
                });
            }
            let start = args[0].to_string_box().value.parse::<i64>().map_err(|_| RuntimeError::TypeError { message: "RangeBox constructor requires integer arguments".to_string() })?;
            let end = args[1].to_string_box().value.parse::<i64>().map_err(|_| RuntimeError::TypeError { message: "RangeBox constructor requires integer arguments".to_string() })?;
            let step = if args.len() == 3 {
                args[2].to_string_box().value.parse::<i64>().map_err(|_| RuntimeError::TypeError { message: "RangeBox constructor requires integer arguments".to_string() })?
            } else { 1 };
            Ok(Box::new(RangeBox::new(start, end, step)))
        });

        // MethodBox
        self.register("MethodBox", |args| {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("MethodBox constructor expects 2 arguments (instance, method_name), got {}", args.len()),
                });
            }
            let instance = args[0].clone_box();
            let method_name = args[1].to_string_box().value;
            Ok(Box::new(MethodBox::new(instance, method_name)))
        });

        // IntentBox
        self.register("IntentBox", |args| {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("IntentBox constructor expects 2 arguments (name, payload), got {}", args.len()),
                });
            }
            let name = args[0].to_string_box().value;
            // Try parse payload as JSON, fallback to string
            let payload_str = args[1].to_string_box().value;
            let payload = match serde_json::from_str::<serde_json::Value>(&payload_str) {
                Ok(json) => json,
                Err(_) => serde_json::Value::String(payload_str),
            };
            Ok(Box::new(IntentBox::new(name, payload)))
        });

        // ErrorBox (Exception)
        self.register("ErrorBox", |args| {
            let message = match args.get(0) {
                Some(arg) => arg.to_string_box().value,
                None => String::new(),
            };
            Ok(Box::new(crate::exception_box::ErrorBox::new(&message)))
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

        // WebConsoleBox
        self.register("WebConsoleBox", |args| {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("WebConsoleBox constructor expects 1 argument (element_id), got {}", args.len()),
                });
            }
            if let Some(id_str) = args[0].as_any().downcast_ref::<StringBox>() {
                Ok(Box::new(crate::boxes::WebConsoleBox::new(id_str.value.clone())))
            } else {
                Err(RuntimeError::TypeError {
                    message: "WebConsoleBox constructor requires string element_id argument".to_string(),
                })
            }
        });

        // WebCanvasBox
        self.register("WebCanvasBox", |args| {
            if args.len() != 3 {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("WebCanvasBox constructor expects 3 arguments (canvas_id, width, height), got {}", args.len()),
                });
            }

            let canvas_id = args[0].to_string_box().value;
            let width = args[1].to_string_box().value.parse::<u32>()
                .map_err(|_| RuntimeError::TypeError { message: "WebCanvasBox width must be integer".to_string() })?;
            let height = args[2].to_string_box().value.parse::<u32>()
                .map_err(|_| RuntimeError::TypeError { message: "WebCanvasBox height must be integer".to_string() })?;
            Ok(Box::new(crate::boxes::WebCanvasBox::new(canvas_id, width, height)))
        });
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

    fn is_builtin_factory(&self) -> bool { true }
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
