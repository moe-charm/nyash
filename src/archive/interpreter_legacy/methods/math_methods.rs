/*!
 * Math Methods Module
 * 
 * MathBox, RandomBox, TimeBox, DateTimeBoxのメソッド実装
 * Phase 9.75f-2: 動的ライブラリ化対応
 */

use crate::interpreter::{NyashInterpreter, RuntimeError};
use crate::ast::ASTNode;
use crate::box_trait::{NyashBox, IntegerBox, BoolBox, StringBox};
use crate::boxes::FloatBox;

#[cfg(feature = "dynamic-file")]
use crate::interpreter::plugin_loader::{MathBoxProxy, RandomBoxProxy, TimeBoxProxy, DateTimeBoxProxy};

#[cfg(not(feature = "dynamic-file"))]
use crate::boxes::{MathBox, RandomBox, TimeBox, DateTimeBox};

impl NyashInterpreter {
    /// MathBox用メソッドを実行（動的ライブラリ対応）
    #[cfg(feature = "dynamic-file")]
    pub fn execute_math_proxy_method(&mut self, 
        _math_box: &MathBoxProxy,
        method: &str,
        arguments: &[ASTNode]) -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        match method {
            "sqrt" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("sqrt() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let arg = self.execute_expression(&arguments[0])?;
                let value = self.to_float(&arg)?;
                
                let cache = crate::interpreter::plugin_loader::PLUGIN_CACHE.read().unwrap();
                if let Some(plugin) = cache.get("math") {
                    unsafe {
                        if let Ok(sqrt_fn) = plugin.library.get::<libloading::Symbol<unsafe extern "C" fn(f64) -> f64>>(b"nyash_math_sqrt\0") {
                            let result = sqrt_fn(value);
                            return Ok(Box::new(FloatBox::new(result)));
                        }
                    }
                }
                
                Err(RuntimeError::InvalidOperation {
                    message: "Failed to call sqrt".to_string(),
                })
            }
            "pow" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("pow() expects 2 arguments, got {}", arguments.len()),
                    });
                }
                let base_value = self.execute_expression(&arguments[0])?;
                let exp_value = self.execute_expression(&arguments[1])?;
                let base = self.to_float(&base_value)?;
                let exp = self.to_float(&exp_value)?;
                
                let cache = crate::interpreter::plugin_loader::PLUGIN_CACHE.read().unwrap();
                if let Some(plugin) = cache.get("math") {
                    unsafe {
                        if let Ok(pow_fn) = plugin.library.get::<libloading::Symbol<unsafe extern "C" fn(f64, f64) -> f64>>(b"nyash_math_pow\0") {
                            let result = pow_fn(base, exp);
                            return Ok(Box::new(FloatBox::new(result)));
                        }
                    }
                }
                
                Err(RuntimeError::InvalidOperation {
                    message: "Failed to call pow".to_string(),
                })
            }
            "sin" => self.call_unary_math_fn("nyash_math_sin", arguments),
            "cos" => self.call_unary_math_fn("nyash_math_cos", arguments),
            "tan" => self.call_unary_math_fn("nyash_math_tan", arguments),
            "abs" => self.call_unary_math_fn("nyash_math_abs", arguments),
            "floor" => self.call_unary_math_fn("nyash_math_floor", arguments),
            "ceil" => self.call_unary_math_fn("nyash_math_ceil", arguments),
            "round" => self.call_unary_math_fn("nyash_math_round", arguments),
            "log" => self.call_unary_math_fn("nyash_math_log", arguments),
            "log10" => self.call_unary_math_fn("nyash_math_log10", arguments),
            "exp" => self.call_unary_math_fn("nyash_math_exp", arguments),
            "min" => self.call_binary_math_fn("nyash_math_min", arguments),
            "max" => self.call_binary_math_fn("nyash_math_max", arguments),
            "toString" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toString() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(_math_box.to_string_box()))
            }
            "type_name" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("type_name() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(StringBox::new(_math_box.type_name())))
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Unknown MathBox method: {}", method),
            }),
        }
    }
    
    /// RandomBox用メソッドを実行（動的ライブラリ対応）
    #[cfg(feature = "dynamic-file")]
    pub fn execute_random_proxy_method(&mut self,
        random_box: &RandomBoxProxy,
        method: &str,
        arguments: &[ASTNode]) -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        match method {
            "next" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("next() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                random_box.next()
            }
            "range" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("range() expects 2 arguments, got {}", arguments.len()),
                    });
                }
                let min_value = self.execute_expression(&arguments[0])?;
                let max_value = self.execute_expression(&arguments[1])?;
                let min = self.to_float(&min_value)?;
                let max = self.to_float(&max_value)?;
                random_box.range(min, max)
            }
            "int" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("int() expects 2 arguments, got {}", arguments.len()),
                    });
                }
                let min_value = self.execute_expression(&arguments[0])?;
                let max_value = self.execute_expression(&arguments[1])?;
                let min = self.to_integer(&min_value)?;
                let max = self.to_integer(&max_value)?;
                random_box.int(min, max)
            }
            "toString" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toString() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(random_box.to_string_box()))
            }
            "type_name" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("type_name() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(StringBox::new(random_box.type_name())))
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Unknown RandomBox method: {}", method),
            }),
        }
    }
    
    /// TimeBox用メソッドを実行（動的ライブラリ対応）
    #[cfg(feature = "dynamic-file")]
    pub fn execute_time_proxy_method(&mut self,
        _time_box: &TimeBoxProxy,
        method: &str,
        arguments: &[ASTNode]) -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        match method {
            "now" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("now() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                crate::interpreter::plugin_loader::PluginLoader::create_datetime_now()
            }
            "parse" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("parse() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let time_str = self.execute_expression(&arguments[0])?.to_string_box().value;
                crate::interpreter::plugin_loader::PluginLoader::create_datetime_from_string(&time_str)
            }
            "toString" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toString() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(_time_box.to_string_box()))
            }
            "type_name" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("type_name() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(StringBox::new(_time_box.type_name())))
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Unknown TimeBox method: {}", method),
            }),
        }
    }
    
    /// DateTimeBox用メソッドを実行（動的ライブラリ対応）
    #[cfg(feature = "dynamic-file")]
    pub fn execute_datetime_proxy_method(&mut self,
        datetime_box: &DateTimeBoxProxy,
        method: &str,
        arguments: &[ASTNode]) -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        // type_name と toString は引数チェックを個別で行う
        if !arguments.is_empty() && method != "type_name" && method != "toString" {
            return Err(RuntimeError::InvalidOperation {
                message: format!("{}() expects 0 arguments, got {}", method, arguments.len()),
            });
        }
        
        let cache = crate::interpreter::plugin_loader::PLUGIN_CACHE.read().unwrap();
        if let Some(plugin) = cache.get("math") {
            unsafe {
                match method {
                    "year" => {
                        if let Ok(year_fn) = plugin.library.get::<libloading::Symbol<unsafe extern "C" fn(*mut std::ffi::c_void) -> i32>>(b"nyash_datetime_year\0") {
                            let year = year_fn(datetime_box.handle.ptr);
                            return Ok(Box::new(IntegerBox::new(year as i64)));
                        }
                    }
                    "month" => {
                        if let Ok(month_fn) = plugin.library.get::<libloading::Symbol<unsafe extern "C" fn(*mut std::ffi::c_void) -> u32>>(b"nyash_datetime_month\0") {
                            let month = month_fn(datetime_box.handle.ptr);
                            return Ok(Box::new(IntegerBox::new(month as i64)));
                        }
                    }
                    "day" => {
                        if let Ok(day_fn) = plugin.library.get::<libloading::Symbol<unsafe extern "C" fn(*mut std::ffi::c_void) -> u32>>(b"nyash_datetime_day\0") {
                            let day = day_fn(datetime_box.handle.ptr);
                            return Ok(Box::new(IntegerBox::new(day as i64)));
                        }
                    }
                    "hour" => {
                        if let Ok(hour_fn) = plugin.library.get::<libloading::Symbol<unsafe extern "C" fn(*mut std::ffi::c_void) -> u32>>(b"nyash_datetime_hour\0") {
                            let hour = hour_fn(datetime_box.handle.ptr);
                            return Ok(Box::new(IntegerBox::new(hour as i64)));
                        }
                    }
                    "minute" => {
                        if let Ok(minute_fn) = plugin.library.get::<libloading::Symbol<unsafe extern "C" fn(*mut std::ffi::c_void) -> u32>>(b"nyash_datetime_minute\0") {
                            let minute = minute_fn(datetime_box.handle.ptr);
                            return Ok(Box::new(IntegerBox::new(minute as i64)));
                        }
                    }
                    "second" => {
                        if let Ok(second_fn) = plugin.library.get::<libloading::Symbol<unsafe extern "C" fn(*mut std::ffi::c_void) -> u32>>(b"nyash_datetime_second\0") {
                            let second = second_fn(datetime_box.handle.ptr);
                            return Ok(Box::new(IntegerBox::new(second as i64)));
                        }
                    }
                    "timestamp" => {
                        if let Ok(timestamp_fn) = plugin.library.get::<libloading::Symbol<unsafe extern "C" fn(*mut std::ffi::c_void) -> i64>>(b"nyash_datetime_timestamp\0") {
                            let timestamp = timestamp_fn(datetime_box.handle.ptr);
                            return Ok(Box::new(IntegerBox::new(timestamp)));
                        }
                    }
                    "toString" => {
                        return Ok(Box::new(datetime_box.to_string_box()));
                    }
                    "type_name" => {
                        if !arguments.is_empty() {
                            return Err(RuntimeError::InvalidOperation {
                                message: format!("type_name() expects 0 arguments, got {}", arguments.len()),
                            });
                        }
                        return Ok(Box::new(StringBox::new(datetime_box.type_name())));
                    }
                    _ => {}
                }
            }
        }
        
        Err(RuntimeError::InvalidOperation {
            message: format!("Unknown DateTimeBox method: {}", method),
        })
    }
    
    // ヘルパーメソッド
    #[cfg(feature = "dynamic-file")]
    fn call_unary_math_fn(&mut self, fn_name: &str, arguments: &[ASTNode]) -> Result<Box<dyn NyashBox>, RuntimeError> {
        if arguments.len() != 1 {
            return Err(RuntimeError::InvalidOperation {
                message: format!("{}() expects 1 argument, got {}", fn_name.strip_prefix("nyash_math_").unwrap_or(fn_name), arguments.len()),
            });
        }
        let arg_value = self.execute_expression(&arguments[0])?;
        let value = self.to_float(&arg_value)?;
        
        let cache = crate::interpreter::plugin_loader::PLUGIN_CACHE.read().unwrap();
        if let Some(plugin) = cache.get("math") {
            unsafe {
                let fn_name_bytes = format!("{}\0", fn_name);
                if let Ok(math_fn) = plugin.library.get::<libloading::Symbol<unsafe extern "C" fn(f64) -> f64>>(fn_name_bytes.as_bytes()) {
                    let result = math_fn(value);
                    return Ok(Box::new(FloatBox::new(result)));
                }
            }
        }
        
        Err(RuntimeError::InvalidOperation {
            message: format!("Failed to call {}", fn_name),
        })
    }
    
    #[cfg(feature = "dynamic-file")]
    fn call_binary_math_fn(&mut self, fn_name: &str, arguments: &[ASTNode]) -> Result<Box<dyn NyashBox>, RuntimeError> {
        if arguments.len() != 2 {
            return Err(RuntimeError::InvalidOperation {
                message: format!("{}() expects 2 arguments, got {}", fn_name.strip_prefix("nyash_math_").unwrap_or(fn_name), arguments.len()),
            });
        }
        let a_value = self.execute_expression(&arguments[0])?;
        let b_value = self.execute_expression(&arguments[1])?;
        let a = self.to_float(&a_value)?;
        let b = self.to_float(&b_value)?;
        
        let cache = crate::interpreter::plugin_loader::PLUGIN_CACHE.read().unwrap();
        if let Some(plugin) = cache.get("math") {
            unsafe {
                let fn_name_bytes = format!("{}\0", fn_name);
                if let Ok(math_fn) = plugin.library.get::<libloading::Symbol<unsafe extern "C" fn(f64, f64) -> f64>>(fn_name_bytes.as_bytes()) {
                    let result = math_fn(a, b);
                    return Ok(Box::new(FloatBox::new(result)));
                }
            }
        }
        
        Err(RuntimeError::InvalidOperation {
            message: format!("Failed to call {}", fn_name),
        })
    }
    
    // 型変換ヘルパー
    fn to_float(&self, value: &Box<dyn NyashBox>) -> Result<f64, RuntimeError> {
        if let Some(float_box) = value.as_any().downcast_ref::<FloatBox>() {
            Ok(float_box.value)
        } else if let Some(int_box) = value.as_any().downcast_ref::<IntegerBox>() {
            Ok(int_box.value as f64)
        } else {
            Err(RuntimeError::TypeError {
                message: "Value must be a number".to_string(),
            })
        }
    }
    
    fn to_integer(&self, value: &Box<dyn NyashBox>) -> Result<i64, RuntimeError> {
        if let Some(int_box) = value.as_any().downcast_ref::<IntegerBox>() {
            Ok(int_box.value)
        } else if let Some(float_box) = value.as_any().downcast_ref::<FloatBox>() {
            Ok(float_box.value as i64)
        } else {
            Err(RuntimeError::TypeError {
                message: "Value must be a number".to_string(),
            })
        }
    }
}