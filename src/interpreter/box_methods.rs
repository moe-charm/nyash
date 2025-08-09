/*!
 * Box Method Handlers Module
 * 
 * Extracted from interpreter.rs lines 1389-2515 (1,126 lines)  
 * Contains all Box type-specific method implementations:
 * - execute_string_method
 * - execute_array_method  
 * - execute_file_method
 * - execute_result_method
 * - execute_future_method
 * - execute_channel_method
 * - execute_math_method
 * - execute_time_method
 * - execute_datetime_method
 * - execute_timer_method
 * - execute_map_method
 * - execute_random_method
 * - execute_sound_method
 * - execute_debug_method
 * - execute_method_box_method
 */

use super::*;
use crate::box_trait::{StringBox, IntegerBox};
use crate::boxes::null_box::NullBox;

impl NyashInterpreter {
    /// StringBoxのメソッド呼び出しを実行
    pub(super) fn execute_string_method(&mut self, string_box: &StringBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // This will be filled with the actual implementation
        // Complete for now to test modular structure
        match method {
            "split" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("split() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let delimiter_value = self.execute_expression(&arguments[0])?;
                if let Some(delimiter_str) = delimiter_value.as_any().downcast_ref::<StringBox>() {
                    Ok(string_box.split(&delimiter_str.value))
                } else {
                    Err(RuntimeError::TypeError {
                        message: "split() requires string delimiter".to_string(),
                    })
                }
            }
            "toString" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toString() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                // StringBoxは自分自身を返す
                Ok(Box::new(string_box.clone()))
            }
            "length" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("length() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(string_box.length())
            }
            "get" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("get() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let index_value = self.execute_expression(&arguments[0])?;
                if let Some(index_int) = index_value.as_any().downcast_ref::<IntegerBox>() {
                    match string_box.get(index_int.value as usize) {
                        Some(char_box) => Ok(char_box),
                        None => Ok(Box::new(VoidBox::new())),
                    }
                } else {
                    Err(RuntimeError::TypeError {
                        message: "get() requires integer index".to_string(),
                    })
                }
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown method '{}' for StringBox", method),
                })
            }
        }
    }

    /// IntegerBoxのメソッド呼び出しを実行  
    pub(super) fn execute_integer_method(&mut self, integer_box: &IntegerBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "toString" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toString() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(StringBox::new(integer_box.value.to_string())))
            }
            "abs" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("abs() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(Box::new(IntegerBox::new(integer_box.value.abs())))
            }
            "max" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("max() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let other_value = self.execute_expression(&arguments[0])?;
                if let Some(other_int) = other_value.as_any().downcast_ref::<IntegerBox>() {
                    Ok(Box::new(IntegerBox::new(integer_box.value.max(other_int.value))))
                } else {
                    Err(RuntimeError::TypeError {
                        message: "max() requires integer argument".to_string(),
                    })
                }
            }
            "min" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("min() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let other_value = self.execute_expression(&arguments[0])?;
                if let Some(other_int) = other_value.as_any().downcast_ref::<IntegerBox>() {
                    Ok(Box::new(IntegerBox::new(integer_box.value.min(other_int.value))))
                } else {
                    Err(RuntimeError::TypeError {
                        message: "min() requires integer argument".to_string(),
                    })
                }
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown method '{}' for IntegerBox", method),
                })
            }
        }
    }

    /// ArrayBoxのメソッド呼び出しを実行  
    pub(super) fn execute_array_method(&mut self, array_box: &ArrayBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "push" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("push() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let element = self.execute_expression(&arguments[0])?;
                Ok(array_box.push(element))
            }
            "pop" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("pop() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(array_box.pop())
            }
            "length" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("length() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(array_box.length())
            }
            "get" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("get() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let index_value = self.execute_expression(&arguments[0])?;
                if let Some(index_int) = index_value.as_any().downcast_ref::<IntegerBox>() {
                    if let Some(element) = array_box.get(index_int.value as usize) {
                        Ok(element)
                    } else {
                        Ok(Box::new(StringBox::new("Index out of bounds")))
                    }
                } else {
                    Err(RuntimeError::TypeError {
                        message: "get() requires integer index".to_string(),
                    })
                }
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown method '{}' for ArrayBox", method),
                })
            }
        }
    }

    /// FileBoxのメソッド呼び出しを実行
    pub(super) fn execute_file_method(&mut self, file_box: &FileBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "read" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("read() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(file_box.read())
            }
            "write" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("write() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let content = self.execute_expression(&arguments[0])?;
                Ok(file_box.write(content))
            }
            "exists" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("exists() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(file_box.exists())
            }
            "delete" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("delete() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(file_box.delete())
            }
            "copy" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("copy() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let dest_value = self.execute_expression(&arguments[0])?;
                if let Some(dest_str) = dest_value.as_any().downcast_ref::<StringBox>() {
                    Ok(file_box.copy(&dest_str.value))
                } else {
                    Err(RuntimeError::TypeError {
                        message: "copy() requires string destination path".to_string(),
                    })
                }
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Unknown method '{}' for FileBox", method),
            })
        }
    }

    pub(super) fn execute_result_method(&mut self, result_box: &ResultBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "isOk" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("isOk() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(result_box.is_ok())
            }
            "getValue" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("getValue() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(result_box.get_value())
            }
            "getError" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("getError() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(result_box.get_error())
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Unknown method '{}' for ResultBox", method),
            })
        }
    }

    pub(super) fn execute_future_method(&mut self, future_box: &FutureBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "get" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("get() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(future_box.get())
            }
            "ready" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("ready() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(future_box.ready())
            }
            "equals" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("equals() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let other = self.execute_expression(&arguments[0])?;
                Ok(Box::new(future_box.equals(other.as_ref())))
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Unknown method '{}' for FutureBox", method),
            })
        }
    }

    pub(super) fn execute_channel_method(&mut self, channel_box: &ChannelBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // メソッドを実行
        match method {
            "sendMessage" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("sendMessage() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                // 簡易実装：メッセージを作成して返す
                let content = arg_values[0].to_string_box().value;
                let msg = MessageBox::new(&channel_box.sender_name, &content);
                Ok(Box::new(msg))
            }
            "announce" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("announce() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                let content = arg_values[0].to_string_box().value;
                Ok(Box::new(StringBox::new(&format!("Broadcast from {}: {}", channel_box.sender_name, content))))
            }
            "toString" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toString() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(Box::new(channel_box.to_string_box()))
            }
            "sender" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("sender() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(channel_box.sender())
            }
            "receiver" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("receiver() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(channel_box.receiver())
            }
            _ => {
                // その他のメソッドはChannelBoxに委譲
                Ok(channel_box.invoke(method, arg_values))
            }
        }
    }

    pub(super) fn execute_math_method(&mut self, math_box: &MathBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // メソッドを実行
        match method {
            "abs" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("abs() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(math_box.abs(arg_values[0].clone_box()))
            }
            "max" => {
                if arg_values.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("max() expects 2 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(math_box.max(arg_values[0].clone_box(), arg_values[1].clone_box()))
            }
            "min" => {
                if arg_values.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("min() expects 2 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(math_box.min(arg_values[0].clone_box(), arg_values[1].clone_box()))
            }
            "pow" => {
                if arg_values.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("pow() expects 2 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(math_box.pow(arg_values[0].clone_box(), arg_values[1].clone_box()))
            }
            "sqrt" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("sqrt() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(math_box.sqrt(arg_values[0].clone_box()))
            }
            "getPi" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("getPi() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(math_box.getPi())
            }
            "getE" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("getE() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(math_box.getE())
            }
            "sin" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("sin() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(math_box.sin(arg_values[0].clone_box()))
            }
            "cos" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("cos() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(math_box.cos(arg_values[0].clone_box()))
            }
            "tan" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("tan() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(math_box.tan(arg_values[0].clone_box()))
            }
            "log" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("log() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(math_box.log(arg_values[0].clone_box()))
            }
            "log10" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("log10() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(math_box.log10(arg_values[0].clone_box()))
            }
            "exp" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("exp() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(math_box.exp(arg_values[0].clone_box()))
            }
            "floor" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("floor() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(math_box.floor(arg_values[0].clone_box()))
            }
            "ceil" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("ceil() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(math_box.ceil(arg_values[0].clone_box()))
            }
            "round" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("round() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(math_box.round(arg_values[0].clone_box()))
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown MathBox method: {}", method),
                })
            }
        }
    }

    /// NullBoxのメソッド呼び出しを実行
    pub(super) fn execute_null_method(&mut self, _null_box: &NullBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // メソッドを実行
        match method {
            "is_null" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("is_null() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(Box::new(BoolBox::new(true)))
            }
            "is_not_null" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("is_not_null() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(Box::new(BoolBox::new(false)))
            }
            "equals" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("equals() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                let other = &arg_values[0];
                // NullBoxは他のNullBoxとのみ等しい
                let is_equal = other.as_any().downcast_ref::<NullBox>().is_some();
                Ok(Box::new(BoolBox::new(is_equal)))
            }
            "get_or_default" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("get_or_default() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                // nullの場合はデフォルト値を返す
                Ok(arg_values[0].clone_box())
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown NullBox method: {}", method),
                })
            }
        }
    }

    pub(super) fn execute_time_method(&mut self, time_box: &TimeBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // メソッドを実行
        match method {
            "now" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("now() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(time_box.now())
            }
            "fromTimestamp" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("fromTimestamp() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(time_box.fromTimestamp(arg_values[0].clone_box()))
            }
            "parse" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("parse() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(time_box.parse(arg_values[0].clone_box()))
            }
            "sleep" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("sleep() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(time_box.sleep(arg_values[0].clone_box()))
            }
            "format" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("format() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(time_box.format(arg_values[0].clone_box()))
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown TimeBox method: {}", method),
                })
            }
        }
    }

    pub(super) fn execute_datetime_method(&mut self, datetime_box: &DateTimeBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // メソッドを実行
        match method {
            "year" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("year() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(datetime_box.year())
            }
            "month" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("month() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(datetime_box.month())
            }
            "day" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("day() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(datetime_box.day())
            }
            "hour" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("hour() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(datetime_box.hour())
            }
            "minute" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("minute() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(datetime_box.minute())
            }
            "second" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("second() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(datetime_box.second())
            }
            "timestamp" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("timestamp() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(datetime_box.timestamp())
            }
            "toISOString" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toISOString() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(datetime_box.toISOString())
            }
            "format" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("format() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(datetime_box.format(arg_values[0].clone_box()))
            }
            "addDays" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("addDays() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(datetime_box.addDays(arg_values[0].clone_box()))
            }
            "addHours" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("addHours() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(datetime_box.addHours(arg_values[0].clone_box()))
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown DateTimeBox method: {}", method),
                })
            }
        }
    }

    pub(super) fn execute_timer_method(&mut self, timer_box: &TimerBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // メソッドを実行
        match method {
            "elapsed" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("elapsed() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(timer_box.elapsed())
            }
            "reset" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("reset() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                // NOTE: resetはmutableメソッドなので、ここでは新しいTimerBoxを作成
                let timer_box = Box::new(TimerBox::new()) as Box<dyn NyashBox>;
                // 🌍 革命的実装：Environment tracking廃止
                Ok(timer_box)
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown TimerBox method: {}", method),
                })
            }
        }
    }

    pub(super) fn execute_map_method(&mut self, map_box: &MapBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // メソッドを実行
        match method {
            "set" => {
                if arg_values.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("set() expects 2 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(map_box.set(arg_values[0].clone_box(), arg_values[1].clone_box()))
            }
            "get" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("get() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(map_box.get(arg_values[0].clone_box()))
            }
            "has" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("has() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(map_box.has(arg_values[0].clone_box()))
            }
            "delete" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("delete() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(map_box.delete(arg_values[0].clone_box()))
            }
            "keys" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("keys() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(map_box.keys())
            }
            "values" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("values() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(map_box.values())
            }
            "size" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("size() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(map_box.size())
            }
            "clear" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("clear() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(map_box.clear())
            }
            "forEach" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("forEach() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(map_box.forEach(arg_values[0].clone_box()))
            }
            "toJSON" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toJSON() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(map_box.toJSON())
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown MapBox method: {}", method),
                })
            }
        }
    }

    pub(super) fn execute_random_method(&mut self, random_box: &RandomBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // メソッドを実行
        match method {
            "seed" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("seed() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(random_box.seed(arg_values[0].clone_box()))
            }
            "random" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("random() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(random_box.random())
            }
            "randInt" => {
                if arg_values.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("randInt() expects 2 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(random_box.randInt(arg_values[0].clone_box(), arg_values[1].clone_box()))
            }
            "randBool" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("randBool() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(random_box.randBool())
            }
            "choice" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("choice() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(random_box.choice(arg_values[0].clone_box()))
            }
            "shuffle" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("shuffle() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(random_box.shuffle(arg_values[0].clone_box()))
            }
            "randString" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("randString() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(random_box.randString(arg_values[0].clone_box()))
            }
            "probability" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("probability() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(random_box.probability(arg_values[0].clone_box()))
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown RandomBox method: {}", method),
                })
            }
        }
    }

    pub(super) fn execute_sound_method(&mut self, sound_box: &SoundBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // メソッドを実行
        match method {
            "beep" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("beep() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(sound_box.beep())
            }
            "beeps" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("beeps() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(sound_box.beeps(arg_values[0].clone_box()))
            }
            "tone" => {
                if arg_values.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("tone() expects 2 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(sound_box.tone(arg_values[0].clone_box(), arg_values[1].clone_box()))
            }
            "alert" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("alert() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(sound_box.alert())
            }
            "success" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("success() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(sound_box.success())
            }
            "error" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("error() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(sound_box.error())
            }
            "pattern" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("pattern() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(sound_box.pattern(arg_values[0].clone_box()))
            }
            "volumeTest" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("volumeTest() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(sound_box.volumeTest())
            }
            "interval" => {
                if arg_values.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("interval() expects 2 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(sound_box.interval(arg_values[0].clone_box(), arg_values[1].clone_box()))
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown SoundBox method: {}", method),
                })
            }
        }
    }

    pub(super) fn execute_debug_method(&mut self, debug_box: &DebugBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // メソッドを実行
        match method {
            "startTracking" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("startTracking() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                debug_box.start_tracking()
            }
            "stopTracking" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("stopTracking() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                debug_box.stop_tracking()
            }
            "trackBox" => {
                if arg_values.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("trackBox() expects 2 arguments, got {}", arg_values.len()),
                    });
                }
                // 第2引数をStringBoxとして取得
                let name = if let Some(str_box) = arg_values[1].as_any().downcast_ref::<StringBox>() {
                    str_box.value.clone()
                } else {
                    return Err(RuntimeError::InvalidOperation {
                        message: "trackBox() second argument must be a string".to_string(),
                    });
                };
                debug_box.track_box(arg_values[0].as_ref(), &name)
            }
            "dumpAll" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("dumpAll() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                debug_box.dump_all()
            }
            "saveToFile" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("saveToFile() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                let filename = if let Some(str_box) = arg_values[0].as_any().downcast_ref::<StringBox>() {
                    str_box.value.clone()
                } else {
                    return Err(RuntimeError::InvalidOperation {
                        message: "saveToFile() argument must be a string".to_string(),
                    });
                };
                debug_box.save_to_file(&filename)
            }
            "watch" => {
                if arg_values.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("watch() expects 2 arguments, got {}", arg_values.len()),
                    });
                }
                let name = if let Some(str_box) = arg_values[1].as_any().downcast_ref::<StringBox>() {
                    str_box.value.clone()
                } else {
                    return Err(RuntimeError::InvalidOperation {
                        message: "watch() second argument must be a string".to_string(),
                    });
                };
                debug_box.watch(arg_values[0].as_ref(), &name)
            }
            "memoryReport" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("memoryReport() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                debug_box.memory_report()
            }
            "setBreakpoint" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("setBreakpoint() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                let func_name = if let Some(str_box) = arg_values[0].as_any().downcast_ref::<StringBox>() {
                    str_box.value.clone()
                } else {
                    return Err(RuntimeError::InvalidOperation {
                        message: "setBreakpoint() argument must be a string".to_string(),
                    });
                };
                debug_box.set_breakpoint(&func_name)
            }
            "traceCall" => {
                if arg_values.len() < 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("traceCall() expects at least 1 argument, got {}", arg_values.len()),
                    });
                }
                let func_name = if let Some(str_box) = arg_values[0].as_any().downcast_ref::<StringBox>() {
                    str_box.value.clone()
                } else {
                    return Err(RuntimeError::InvalidOperation {
                        message: "traceCall() first argument must be a string".to_string(),
                    });
                };
                // 残りの引数を文字列として収集
                let args: Vec<String> = arg_values[1..].iter()
                    .map(|v| v.to_string_box().value)
                    .collect();
                debug_box.trace_call(&func_name, args)
            }
            "showCallStack" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("showCallStack() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                debug_box.show_call_stack()
            }
            "clear" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("clear() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                debug_box.clear()
            }
            "isTracking" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("isTracking() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                debug_box.is_tracking()
            }
            "getTrackedCount" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("getTrackedCount() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                debug_box.get_tracked_count()
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown DebugBox method: {}", method),
                })
            }
        }
    }

    /// ConsoleBoxのメソッド呼び出しを実行
    pub(super) fn execute_console_method(&mut self, console_box: &crate::boxes::console_box::ConsoleBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // メソッドを実行
        match method {
            "log" => {
                if arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: "console.log() requires at least 1 argument".to_string(),
                    });
                }
                
                // 引数をすべて文字列に変換
                let messages: Vec<String> = arg_values.iter()
                    .map(|arg| arg.to_string_box().value)
                    .collect();
                
                let combined_message = messages.join(" ");
                console_box.log(&combined_message);
                
                Ok(Box::new(VoidBox::new()))
            }
            "warn" => {
                if arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: "console.warn() requires at least 1 argument".to_string(),
                    });
                }
                
                let messages: Vec<String> = arg_values.iter()
                    .map(|arg| arg.to_string_box().value)
                    .collect();
                
                let combined_message = messages.join(" ");
                console_box.warn(&combined_message);
                
                Ok(Box::new(VoidBox::new()))
            }
            "error" => {
                if arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: "console.error() requires at least 1 argument".to_string(),
                    });
                }
                
                let messages: Vec<String> = arg_values.iter()
                    .map(|arg| arg.to_string_box().value)
                    .collect();
                
                let combined_message = messages.join(" ");
                console_box.error(&combined_message);
                
                Ok(Box::new(VoidBox::new()))
            }
            "clear" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("console.clear() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                
                console_box.clear();
                Ok(Box::new(VoidBox::new()))
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown ConsoleBox method: {}", method),
                })
            }
        }
    }

    /// MethodBoxのメソッド呼び出しを実行
    pub(super) fn execute_method_box_method(&mut self, method_box: &crate::method_box::MethodBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "invoke" => {
                // 引数を評価
                let mut arg_values = Vec::new();
                for arg in arguments {
                    arg_values.push(self.execute_expression(arg)?);
                }
                
                // MethodBoxのinvokeを呼び出す
                self.invoke_method_box(method_box, arg_values)
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown MethodBox method: {}", method),
                })
            }
        }
    }

    /// MethodBoxでメソッドを実際に呼び出す
    fn invoke_method_box(&mut self, method_box: &crate::method_box::MethodBox, args: Vec<Box<dyn NyashBox>>) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // インスタンスを取得
        let instance_arc = method_box.get_instance();
        let instance = instance_arc.lock().unwrap();
        
        // InstanceBoxにダウンキャスト
        if let Some(instance_box) = instance.as_any().downcast_ref::<crate::instance::InstanceBox>() {
            // メソッドを取得
            let method_ast = instance_box.get_method(&method_box.method_name)
                .ok_or(RuntimeError::InvalidOperation {
                    message: format!("Method '{}' not found", method_box.method_name),
                })?
                .clone();
            
            // メソッド呼び出しを実行
            if let ASTNode::FunctionDeclaration { params, body, .. } = method_ast {
                // パラメータ数チェック
                if args.len() != params.len() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("Method {} expects {} arguments, got {}", 
                                       method_box.method_name, params.len(), args.len()),
                    });
                }
                
                // local変数スタックを保存
                let saved_locals = self.save_local_vars();
                self.local_vars.clear();
                
                // meをlocal変数として設定（インスタンス自体）
                self.declare_local_variable("me", instance.clone_box());
                
                // パラメータをlocal変数として設定
                for (param, arg) in params.iter().zip(args.iter()) {
                    self.declare_local_variable(param, arg.clone_box());
                }
                
                // メソッド本体を実行
                let mut result = Box::new(crate::box_trait::VoidBox::new()) as Box<dyn NyashBox>;
                for statement in &body {
                    result = self.execute_statement(statement)?;
                    
                    // return文チェック
                    if let super::ControlFlow::Return(ret_val) = &self.control_flow {
                        result = ret_val.clone_box();
                        self.control_flow = super::ControlFlow::None;
                        break;
                    }
                }
                
                // local変数スタックを復元
                self.restore_local_vars(saved_locals);
                
                Ok(result)
            } else {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Method '{}' is not a valid function declaration", method_box.method_name),
                })
            }
        } else {
            Err(RuntimeError::TypeError {
                message: "MethodBox instance is not an InstanceBox".to_string(),
            })
        }
    }
    
    /// WebDisplayBoxメソッド実行 (WASM環境のみ)
    #[cfg(target_arch = "wasm32")]
    pub(super) fn execute_web_display_method(&mut self, web_display_box: &crate::boxes::WebDisplayBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // メソッドを実行
        match method {
            "print" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("print() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                let message = arg_values[0].to_string_box().value;
                web_display_box.print(&message);
                Ok(Box::new(VoidBox::new()))
            }
            "println" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("println() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                let message = arg_values[0].to_string_box().value;
                web_display_box.println(&message);
                Ok(Box::new(VoidBox::new()))
            }
            "setHTML" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("setHTML() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                let html_content = arg_values[0].to_string_box().value;
                web_display_box.set_html(&html_content);
                Ok(Box::new(VoidBox::new()))
            }
            "appendHTML" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("appendHTML() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                let html_content = arg_values[0].to_string_box().value;
                web_display_box.append_html(&html_content);
                Ok(Box::new(VoidBox::new()))
            }
            "setCSS" => {
                if arg_values.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("setCSS() expects 2 arguments (property, value), got {}", arg_values.len()),
                    });
                }
                let property = arg_values[0].to_string_box().value;
                let value = arg_values[1].to_string_box().value;
                web_display_box.set_css(&property, &value);
                Ok(Box::new(VoidBox::new()))
            }
            "addClass" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("addClass() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                let class_name = arg_values[0].to_string_box().value;
                web_display_box.add_class(&class_name);
                Ok(Box::new(VoidBox::new()))
            }
            "removeClass" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("removeClass() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                let class_name = arg_values[0].to_string_box().value;
                web_display_box.remove_class(&class_name);
                Ok(Box::new(VoidBox::new()))
            }
            "clear" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("clear() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                web_display_box.clear();
                Ok(Box::new(VoidBox::new()))
            }
            "show" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("show() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                web_display_box.show();
                Ok(Box::new(VoidBox::new()))
            }
            "hide" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("hide() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                web_display_box.hide();
                Ok(Box::new(VoidBox::new()))
            }
            "scrollToBottom" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("scrollToBottom() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                web_display_box.scroll_to_bottom();
                Ok(Box::new(VoidBox::new()))
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown method '{}' for WebDisplayBox", method),
                })
            }
        }
    }
    
    /// WebConsoleBoxメソッド実行 (WASM環境のみ)
    #[cfg(target_arch = "wasm32")]
    pub(super) fn execute_web_console_method(&mut self, web_console_box: &crate::boxes::WebConsoleBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // メソッドを実行
        match method {
            "log" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("log() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                let message = arg_values[0].to_string_box().value;
                web_console_box.log(&message);
                Ok(Box::new(VoidBox::new()))
            }
            "warn" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("warn() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                let message = arg_values[0].to_string_box().value;
                web_console_box.warn(&message);
                Ok(Box::new(VoidBox::new()))
            }
            "error" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("error() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                let message = arg_values[0].to_string_box().value;
                web_console_box.error(&message);
                Ok(Box::new(VoidBox::new()))
            }
            "info" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("info() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                let message = arg_values[0].to_string_box().value;
                web_console_box.info(&message);
                Ok(Box::new(VoidBox::new()))
            }
            "debug" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("debug() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                let message = arg_values[0].to_string_box().value;
                web_console_box.debug(&message);
                Ok(Box::new(VoidBox::new()))
            }
            "clear" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("clear() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                web_console_box.clear();
                Ok(Box::new(VoidBox::new()))
            }
            "separator" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("separator() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                web_console_box.separator();
                Ok(Box::new(VoidBox::new()))
            }
            "group" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("group() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                let title = arg_values[0].to_string_box().value;
                web_console_box.group(&title);
                Ok(Box::new(VoidBox::new()))
            }
            "groupEnd" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("groupEnd() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                web_console_box.group_end();
                Ok(Box::new(VoidBox::new()))
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown method '{}' for WebConsoleBox", method),
                })
            }
        }
    }
    
    /// WebCanvasBoxメソッド実行 (WASM環境のみ)
    #[cfg(target_arch = "wasm32")]
    pub(super) fn execute_web_canvas_method(&mut self, web_canvas_box: &crate::boxes::WebCanvasBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // メソッドを実行
        match method {
            "clear" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("clear() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                web_canvas_box.clear();
                Ok(Box::new(VoidBox::new()))
            }
            "fillRect" => {
                if arg_values.len() != 5 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("fillRect() expects 5 arguments (x, y, width, height, color), got {}", arg_values.len()),
                    });
                }
                let x = if let Some(n) = arg_values[0].as_any().downcast_ref::<IntegerBox>() {
                    n.value as f64
                } else if let Some(n) = arg_values[0].as_any().downcast_ref::<FloatBox>() {
                    n.value
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "fillRect() x must be a number".to_string(),
                    });
                };
                let y = if let Some(n) = arg_values[1].as_any().downcast_ref::<IntegerBox>() {
                    n.value as f64
                } else if let Some(n) = arg_values[1].as_any().downcast_ref::<FloatBox>() {
                    n.value
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "fillRect() y must be a number".to_string(),
                    });
                };
                let width = if let Some(n) = arg_values[2].as_any().downcast_ref::<IntegerBox>() {
                    n.value as f64
                } else if let Some(n) = arg_values[2].as_any().downcast_ref::<FloatBox>() {
                    n.value
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "fillRect() width must be a number".to_string(),
                    });
                };
                let height = if let Some(n) = arg_values[3].as_any().downcast_ref::<IntegerBox>() {
                    n.value as f64
                } else if let Some(n) = arg_values[3].as_any().downcast_ref::<FloatBox>() {
                    n.value
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "fillRect() height must be a number".to_string(),
                    });
                };
                let color = arg_values[4].to_string_box().value;
                web_canvas_box.fill_rect(x, y, width, height, &color);
                Ok(Box::new(VoidBox::new()))
            }
            "strokeRect" => {
                if arg_values.len() != 6 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("strokeRect() expects 6 arguments (x, y, width, height, color, lineWidth), got {}", arg_values.len()),
                    });
                }
                let x = if let Some(n) = arg_values[0].as_any().downcast_ref::<IntegerBox>() {
                    n.value as f64
                } else if let Some(n) = arg_values[0].as_any().downcast_ref::<FloatBox>() {
                    n.value
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "strokeRect() x must be a number".to_string(),
                    });
                };
                let y = if let Some(n) = arg_values[1].as_any().downcast_ref::<IntegerBox>() {
                    n.value as f64
                } else if let Some(n) = arg_values[1].as_any().downcast_ref::<FloatBox>() {
                    n.value
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "strokeRect() y must be a number".to_string(),
                    });
                };
                let width = if let Some(n) = arg_values[2].as_any().downcast_ref::<IntegerBox>() {
                    n.value as f64
                } else if let Some(n) = arg_values[2].as_any().downcast_ref::<FloatBox>() {
                    n.value
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "strokeRect() width must be a number".to_string(),
                    });
                };
                let height = if let Some(n) = arg_values[3].as_any().downcast_ref::<IntegerBox>() {
                    n.value as f64
                } else if let Some(n) = arg_values[3].as_any().downcast_ref::<FloatBox>() {
                    n.value
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "strokeRect() height must be a number".to_string(),
                    });
                };
                let color = arg_values[4].to_string_box().value;
                let line_width = if let Some(n) = arg_values[5].as_any().downcast_ref::<IntegerBox>() {
                    n.value as f64
                } else if let Some(n) = arg_values[5].as_any().downcast_ref::<FloatBox>() {
                    n.value
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "strokeRect() lineWidth must be a number".to_string(),
                    });
                };
                web_canvas_box.stroke_rect(x, y, width, height, &color, line_width);
                Ok(Box::new(VoidBox::new()))
            }
            "fillCircle" => {
                if arg_values.len() != 4 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("fillCircle() expects 4 arguments (x, y, radius, color), got {}", arg_values.len()),
                    });
                }
                let x = if let Some(n) = arg_values[0].as_any().downcast_ref::<IntegerBox>() {
                    n.value as f64
                } else if let Some(n) = arg_values[0].as_any().downcast_ref::<FloatBox>() {
                    n.value
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "fillCircle() x must be a number".to_string(),
                    });
                };
                let y = if let Some(n) = arg_values[1].as_any().downcast_ref::<IntegerBox>() {
                    n.value as f64
                } else if let Some(n) = arg_values[1].as_any().downcast_ref::<FloatBox>() {
                    n.value
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "fillCircle() y must be a number".to_string(),
                    });
                };
                let radius = if let Some(n) = arg_values[2].as_any().downcast_ref::<IntegerBox>() {
                    n.value as f64
                } else if let Some(n) = arg_values[2].as_any().downcast_ref::<FloatBox>() {
                    n.value
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "fillCircle() radius must be a number".to_string(),
                    });
                };
                let color = arg_values[3].to_string_box().value;
                web_canvas_box.fill_circle(x, y, radius, &color);
                Ok(Box::new(VoidBox::new()))
            }
            "fillText" => {
                if arg_values.len() != 5 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("fillText() expects 5 arguments (text, x, y, font, color), got {}", arg_values.len()),
                    });
                }
                let text = arg_values[0].to_string_box().value;
                let x = if let Some(n) = arg_values[1].as_any().downcast_ref::<IntegerBox>() {
                    n.value as f64
                } else if let Some(n) = arg_values[1].as_any().downcast_ref::<FloatBox>() {
                    n.value
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "fillText() x must be a number".to_string(),
                    });
                };
                let y = if let Some(n) = arg_values[2].as_any().downcast_ref::<IntegerBox>() {
                    n.value as f64
                } else if let Some(n) = arg_values[2].as_any().downcast_ref::<FloatBox>() {
                    n.value
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "fillText() y must be a number".to_string(),
                    });
                };
                let font = arg_values[3].to_string_box().value;
                let color = arg_values[4].to_string_box().value;
                web_canvas_box.fill_text(&text, x, y, &font, &color);
                Ok(Box::new(VoidBox::new()))
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown method '{}' for WebCanvasBox", method),
                })
            }
        }
    }
}