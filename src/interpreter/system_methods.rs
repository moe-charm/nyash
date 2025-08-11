/*!
 * System Methods Module
 * 
 * Extracted from box_methods.rs
 * Contains system-level Box method implementations:
 * - TimeBox methods (now, fromTimestamp, parse, sleep, format)
 * - DateTimeBox methods (year, month, day, hour, minute, second, timestamp, toISOString, format, addDays, addHours)
 * - TimerBox methods (elapsed, reset)
 * - DebugBox methods (startTracking, stopTracking, trackBox, dumpAll, saveToFile, watch, etc.)
 */

use super::*;
use crate::box_trait::StringBox;
use crate::boxes::{TimeBox, DateTimeBox};

impl NyashInterpreter {
    /// TimeBoxのメソッド呼び出しを実行
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

    /// DateTimeBoxのメソッド呼び出しを実行
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
            "toString" => {
                if !arg_values.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toString() expects 0 arguments, got {}", arg_values.len()),
                    });
                }
                Ok(Box::new(datetime_box.to_string_box()))
            }
            _ => {
                Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown DateTimeBox method: {}", method),
                })
            }
        }
    }

    /// TimerBoxのメソッド呼び出しを実行
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

    /// DebugBoxのメソッド呼び出しを実行
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
}