/*!
 * Builtin Box Constructors Module
 * 
 * Extracted from objects.rs - handles instantiation of all builtin Box types
 * Core responsibility: Creating instances of StringBox, IntegerBox, ArrayBox, etc.
 * Part of "Everything is Box" philosophy with unified constructor interface
 */

use super::*;
use crate::NullBox;
use crate::boxes::console_box::ConsoleBox;
use crate::boxes::{SocketBox, HTTPServerBox, HTTPRequestBox, HTTPResponseBox};

impl NyashInterpreter {
    /// Create builtin box instance - Extracted from execute_new
    pub(super) fn create_builtin_box_instance(&mut self, class: &str, arguments: &[ASTNode]) 
        -> Result<Option<Box<dyn NyashBox>>, RuntimeError> {
        
        match class {
            // Basic Box constructors (CRITICAL - these were missing!)
            "StringBox" => {
                // StringBoxは引数1個（文字列値）で作成
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("StringBox constructor expects 1 argument, got {}", arguments.len()),
                    });
                }
                let value = self.execute_expression(&arguments[0])?;
                let string_value = value.to_string_box().value;
                let string_box = Box::new(StringBox::new(string_value)) as Box<dyn NyashBox>;
                return Ok(Some(string_box));
            }
            "IntegerBox" => {
                // IntegerBoxは引数1個（整数値）で作成
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("IntegerBox constructor expects 1 argument, got {}", arguments.len()),
                    });
                }
                let value = self.execute_expression(&arguments[0])?;
                if let Some(int_box) = value.as_any().downcast_ref::<IntegerBox>() {
                    let integer_box = Box::new(IntegerBox::new(int_box.value)) as Box<dyn NyashBox>;
                    return Ok(Some(integer_box));
                } else {
                    // Try to parse from string or other types
                    let int_value = value.to_string_box().value.parse::<i64>()
                        .map_err(|_| RuntimeError::TypeError {
                            message: format!("Cannot convert '{}' to integer", value.to_string_box().value),
                        })?;
                    let integer_box = Box::new(IntegerBox::new(int_value)) as Box<dyn NyashBox>;
                    return Ok(Some(integer_box));
                }
            }
            "BoolBox" => {
                // BoolBoxは引数1個（真偽値）で作成
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("BoolBox constructor expects 1 argument, got {}", arguments.len()),
                    });
                }
                let value = self.execute_expression(&arguments[0])?;
                if let Some(bool_box) = value.as_any().downcast_ref::<BoolBox>() {
                    let bool_box_new = Box::new(BoolBox::new(bool_box.value)) as Box<dyn NyashBox>;
                    return Ok(Some(bool_box_new));
                } else {
                    // Try to convert from string or other types
                    let bool_value = match value.to_string_box().value.to_lowercase().as_str() {
                        "true" => true,
                        "false" => false,
                        _ => return Err(RuntimeError::TypeError {
                            message: format!("Cannot convert '{}' to boolean", value.to_string_box().value),
                        }),
                    };
                    let bool_box_new = Box::new(BoolBox::new(bool_value)) as Box<dyn NyashBox>;
                    return Ok(Some(bool_box_new));
                }
            }
            "ArrayBox" => {
                // ArrayBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("ArrayBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let array_box = Box::new(ArrayBox::new()) as Box<dyn NyashBox>;
                // 🌍 革命的実装：Environment tracking廃止
                return Ok(Some(array_box));
            }
            "FileBox" => {
                // FileBoxは引数1個（ファイルパス）で作成
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("FileBox constructor expects 1 argument, got {}", arguments.len()),
                    });
                }
                let path_value = self.execute_expression(&arguments[0])?;
                if let Some(path_str) = path_value.as_any().downcast_ref::<StringBox>() {
                    let file_box = Box::new(FileBox::new(&path_str.value)) as Box<dyn NyashBox>;
                    // 🌍 革命的実装：Environment tracking廃止
                    return Ok(Some(file_box));
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "FileBox constructor requires string path argument".to_string(),
                    });
                }
            }
            "ResultBox" => {
                // ResultBoxは引数1個（成功値）で作成
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("ResultBox constructor expects 1 argument, got {}", arguments.len()),
                    });
                }
                let value = self.execute_expression(&arguments[0])?;
                let result_box = Box::new(ResultBox::new_success(value)) as Box<dyn NyashBox>;
                // 🌍 革命的実装：Environment tracking廃止
                return Ok(Some(result_box));
            }
            "ErrorBox" => {
                // ErrorBoxは引数2個（エラータイプ、メッセージ）で作成
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("ErrorBox constructor expects 2 arguments, got {}", arguments.len()),
                    });
                }
                let error_type_value = self.execute_expression(&arguments[0])?;
                let message_value = self.execute_expression(&arguments[1])?;
                
                if let (Some(error_type_str), Some(message_str)) = (
                    error_type_value.as_any().downcast_ref::<StringBox>(),
                    message_value.as_any().downcast_ref::<StringBox>()
                ) {
                    let error_box = Box::new(ErrorBox::new(&error_type_str.value, &message_str.value)) as Box<dyn NyashBox>;
                    // 🌍 革命的実装：Environment tracking廃止
                    return Ok(Some(error_box));
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "ErrorBox constructor requires two string arguments".to_string(),
                    });
                }
            }
            "MathBox" => {
                // MathBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("MathBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let math_box = Box::new(MathBox::new()) as Box<dyn NyashBox>;
                // 🌍 革命的実装：Environment tracking廃止
                return Ok(Some(math_box));
            }
            "NullBox" => {
                // NullBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("NullBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let null_box = Box::new(NullBox::new()) as Box<dyn NyashBox>;
                return Ok(Some(null_box));
            }
            "ConsoleBox" => {
                // ConsoleBoxは引数なしで作成（ブラウザconsole連携用）
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("ConsoleBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let console_box = Box::new(ConsoleBox::new()) as Box<dyn NyashBox>;
                return Ok(Some(console_box));
            }
            #[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
            "EguiBox" => {
                // EguiBoxは引数なしで作成（GUIアプリケーション用）
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("EguiBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let egui_box = Box::new(crate::boxes::EguiBox::new()) as Box<dyn NyashBox>;
                return Ok(Some(egui_box));
            }
            #[cfg(target_arch = "wasm32")]
            "WebDisplayBox" => {
                // WebDisplayBoxは引数1個（要素ID）で作成（ブラウザHTML操作用）
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("WebDisplayBox constructor expects 1 argument (element_id), got {}", arguments.len()),
                    });
                }
                let element_id_value = self.execute_expression(&arguments[0])?;
                if let Some(id_str) = element_id_value.as_any().downcast_ref::<StringBox>() {
                    let web_display_box = Box::new(crate::boxes::WebDisplayBox::new(id_str.value.clone())) as Box<dyn NyashBox>;
                    return Ok(Some(web_display_box));
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "WebDisplayBox constructor requires string element_id argument".to_string(),
                    });
                }
            }
            #[cfg(target_arch = "wasm32")]
            "WebConsoleBox" => {
                // WebConsoleBoxは引数1個（要素ID）で作成（ブラウザコンソール風出力用）
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("WebConsoleBox constructor expects 1 argument (element_id), got {}", arguments.len()),
                    });
                }
                let element_id_value = self.execute_expression(&arguments[0])?;
                if let Some(id_str) = element_id_value.as_any().downcast_ref::<StringBox>() {
                    let web_console_box = Box::new(crate::boxes::WebConsoleBox::new(id_str.value.clone())) as Box<dyn NyashBox>;
                    return Ok(Some(web_console_box));
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "WebConsoleBox constructor requires string element_id argument".to_string(),
                    });
                }
            }
            #[cfg(target_arch = "wasm32")]
            "WebCanvasBox" => {
                // WebCanvasBoxは引数3個（canvas ID、幅、高さ）で作成
                if arguments.len() != 3 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("WebCanvasBox constructor expects 3 arguments (canvas_id, width, height), got {}", arguments.len()),
                    });
                }
                
                // Canvas ID
                let canvas_id_value = self.execute_expression(&arguments[0])?;
                let canvas_id = if let Some(id_str) = canvas_id_value.as_any().downcast_ref::<StringBox>() {
                    id_str.value.clone()
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "WebCanvasBox constructor requires string canvas_id as first argument".to_string(),
                    });
                };
                
                // Width
                let width_value = self.execute_expression(&arguments[1])?;
                let width = if let Some(int_box) = width_value.as_any().downcast_ref::<IntegerBox>() {
                    int_box.value as u32
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "WebCanvasBox constructor requires integer width as second argument".to_string(),
                    });
                };
                
                // Height
                let height_value = self.execute_expression(&arguments[2])?;
                let height = if let Some(int_box) = height_value.as_any().downcast_ref::<IntegerBox>() {
                    int_box.value as u32
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "WebCanvasBox constructor requires integer height as third argument".to_string(),
                    });
                };
                
                let web_canvas_box = Box::new(crate::boxes::WebCanvasBox::new(canvas_id, width, height)) as Box<dyn NyashBox>;
                return Ok(Some(web_canvas_box));
            }
            "FloatBox" => {
                // FloatBoxは引数1個（数値）で作成
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("FloatBox constructor expects 1 argument, got {}", arguments.len()),
                    });
                }
                let value = self.execute_expression(&arguments[0])?;
                if let Some(int_box) = value.as_any().downcast_ref::<IntegerBox>() {
                    let float_box = Box::new(FloatBox::new(int_box.value as f64)) as Box<dyn NyashBox>;
                    // 🌍 革命的実装：Environment tracking廃止
                    return Ok(Some(float_box));
                } else if let Some(float_box) = value.as_any().downcast_ref::<FloatBox>() {
                    let new_float_box = Box::new(FloatBox::new(float_box.value)) as Box<dyn NyashBox>;
                    // 🌍 革命的実装：Environment tracking廃止
                    return Ok(Some(new_float_box));
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "FloatBox constructor requires numeric argument".to_string(),
                    });
                }
            }
            "RangeBox" => {
                // RangeBoxは引数2-3個（start, end, [step]）で作成
                if arguments.len() < 2 || arguments.len() > 3 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("RangeBox constructor expects 2-3 arguments, got {}", arguments.len()),
                    });
                }
                let start_value = self.execute_expression(&arguments[0])?;
                let end_value = self.execute_expression(&arguments[1])?;
                let step_value = if arguments.len() == 3 {
                    self.execute_expression(&arguments[2])?
                } else {
                    Box::new(IntegerBox::new(1))
                };
                
                if let (Some(start_int), Some(end_int), Some(step_int)) = (
                    start_value.as_any().downcast_ref::<IntegerBox>(),
                    end_value.as_any().downcast_ref::<IntegerBox>(),
                    step_value.as_any().downcast_ref::<IntegerBox>()
                ) {
                    let range_box = Box::new(RangeBox::new(start_int.value, end_int.value, step_int.value)) as Box<dyn NyashBox>;
                    // 🌍 革命的実装：Environment tracking廃止
                    return Ok(Some(range_box));
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "RangeBox constructor requires integer arguments".to_string(),
                    });
                }
            }
            "TimeBox" => {
                // TimeBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("TimeBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let time_box = Box::new(TimeBox::new()) as Box<dyn NyashBox>;
                // 🌍 革命的実装：Environment tracking廃止
                return Ok(Some(time_box));
            }
            "DateTimeBox" => {
                // DateTimeBoxは引数なしで現在時刻、または引数1個でタイムスタンプ
                match arguments.len() {
                    0 => {
                        let datetime_box = Box::new(DateTimeBox::now()) as Box<dyn NyashBox>;
                        // 🌍 革命的実装：Environment tracking廃止
                        return Ok(Some(datetime_box));
                    }
                    1 => {
                        let timestamp_value = self.execute_expression(&arguments[0])?;
                        if let Some(int_box) = timestamp_value.as_any().downcast_ref::<IntegerBox>() {
                            let datetime_box = Box::new(DateTimeBox::from_timestamp(int_box.value)) as Box<dyn NyashBox>;
                            // 🌍 革命的実装：Environment tracking廃止
                            return Ok(Some(datetime_box));
                        } else {
                            return Err(RuntimeError::TypeError {
                                message: "DateTimeBox constructor requires integer timestamp".to_string(),
                            });
                        }
                    }
                    _ => {
                        return Err(RuntimeError::InvalidOperation {
                            message: format!("DateTimeBox constructor expects 0-1 arguments, got {}", arguments.len()),
                        });
                    }
                }
            }
            "TimerBox" => {
                // TimerBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("TimerBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let timer_box = Box::new(TimerBox::new()) as Box<dyn NyashBox>;
                // 🌍 革命的実装：Environment tracking廃止
                return Ok(Some(timer_box));
            }
            "MapBox" => {
                // MapBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("MapBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let map_box = Box::new(MapBox::new()) as Box<dyn NyashBox>;
                // 🌍 革命的実装：Environment tracking廃止
                return Ok(Some(map_box));
            }
            "RandomBox" => {
                // RandomBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("RandomBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let random_box = Box::new(RandomBox::new()) as Box<dyn NyashBox>;
                // 🌍 革命的実装：Environment tracking廃止
                return Ok(Some(random_box));
            }
            "SoundBox" => {
                // SoundBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("SoundBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let sound_box = Box::new(SoundBox::new()) as Box<dyn NyashBox>;
                // 🌍 革命的実装：Environment tracking廃止
                return Ok(Some(sound_box));
            }
            "DebugBox" => {
                // DebugBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("DebugBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let debug_box = Box::new(DebugBox::new()) as Box<dyn NyashBox>;
                // 🌍 革命的実装：Environment tracking廃止
                return Ok(Some(debug_box));
            }
            "BufferBox" => {
                // BufferBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("BufferBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let buffer_box = Box::new(crate::boxes::buffer::BufferBox::new()) as Box<dyn NyashBox>;
                return Ok(Some(buffer_box));
            }
            "RegexBox" => {
                // RegexBoxは引数1個（パターン）で作成
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("RegexBox constructor expects 1 argument, got {}", arguments.len()),
                    });
                }
                let pattern_value = self.execute_expression(&arguments[0])?;
                if let Some(pattern_str) = pattern_value.as_any().downcast_ref::<StringBox>() {
                    match crate::boxes::regex::RegexBox::new(&pattern_str.value) {
                        Ok(regex_box) => return Ok(Some(Box::new(regex_box))),
                        Err(e) => return Err(RuntimeError::InvalidOperation {
                            message: format!("Invalid regex pattern: {}", e),
                        }),
                    }
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "RegexBox constructor requires string pattern argument".to_string(),
                    });
                }
            }
            "JSONBox" => {
                // JSONBoxは引数1個（JSON文字列）で作成
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("JSONBox constructor expects 1 argument, got {}", arguments.len()),
                    });
                }
                let json_value = self.execute_expression(&arguments[0])?;
                if let Some(json_str) = json_value.as_any().downcast_ref::<StringBox>() {
                    match crate::boxes::json::JSONBox::from_str(&json_str.value) {
                        Ok(json_box) => return Ok(Some(Box::new(json_box))),
                        Err(e) => return Err(RuntimeError::InvalidOperation {
                            message: format!("Invalid JSON: {}", e),
                        }),
                    }
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "JSONBox constructor requires string JSON argument".to_string(),
                    });
                }
            }
            
            "IntentBox" => {
                // IntentBoxは引数2個（name, payload）で作成
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("IntentBox constructor expects 2 arguments (name, payload), got {}", arguments.len()),
                    });
                }
                
                // メッセージ名
                let name_value = self.execute_expression(&arguments[0])?;
                let name = if let Some(name_str) = name_value.as_any().downcast_ref::<StringBox>() {
                    name_str.value.clone()
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "IntentBox constructor requires string name as first argument".to_string(),
                    });
                };
                
                // ペイロード（JSON形式）
                let payload_value = self.execute_expression(&arguments[1])?;
                let payload = match payload_value.to_string_box().value.parse::<serde_json::Value>() {
                    Ok(json) => json,
                    Err(_) => {
                        // 文字列として保存
                        serde_json::Value::String(payload_value.to_string_box().value)
                    }
                };
                
                let intent_box = crate::boxes::intent_box::IntentBox::new(name, payload);
                return Ok(Some(Box::new(intent_box) as Box<dyn NyashBox>));
            }
            
            "P2PBox" => {
                // P2PBoxは引数2個（node_id, transport_type）で作成
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("P2PBox constructor expects 2 arguments (node_id, transport_type), got {}", arguments.len()),
                    });
                }
                
                // ノードID
                let node_id_value = self.execute_expression(&arguments[0])?;
                let node_id = if let Some(id_str) = node_id_value.as_any().downcast_ref::<StringBox>() {
                    id_str.value.clone()
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "P2PBox constructor requires string node_id as first argument".to_string(),
                    });
                };
                
                // トランスポート種類
                let transport_value = self.execute_expression(&arguments[1])?;
                let transport_str = if let Some(t_str) = transport_value.as_any().downcast_ref::<StringBox>() {
                    t_str.value.clone()
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "P2PBox constructor requires string transport_type as second argument".to_string(),
                    });
                };
                
                // TODO: Re-enable P2PBox after fixing transport/messaging imports
                return Err(RuntimeError::TypeError {
                    message: "P2PBox temporarily disabled due to import issues".to_string(),
                });
            }
            "StreamBox" => {
                // StreamBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("StreamBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let stream_box = Box::new(crate::boxes::stream::StreamBox::new()) as Box<dyn NyashBox>;
                return Ok(Some(stream_box));
            }
            "HTTPClientBox" => {
                // HTTPClientBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("HTTPClientBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let http_box = Box::new(crate::boxes::http::HttpClientBox::new()) as Box<dyn NyashBox>;
                return Ok(Some(http_box));
            }
            "MethodBox" => {
                // MethodBoxは引数2個（インスタンス、メソッド名）で作成
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("MethodBox constructor expects 2 arguments (instance, method_name), got {}", arguments.len()),
                    });
                }
                
                // インスタンスを評価
                let instance = self.execute_expression(&arguments[0])?;
                
                // メソッド名を評価
                let method_name_value = self.execute_expression(&arguments[1])?;
                if let Some(method_name_str) = method_name_value.as_any().downcast_ref::<StringBox>() {
                    let method_box = Box::new(MethodBox::new(instance, method_name_str.value.clone())) as Box<dyn NyashBox>;
                    return Ok(Some(method_box));
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "MethodBox constructor requires string method name as second argument".to_string(),
                    });
                }
            }
            "SocketBox" => {
                // SocketBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("SocketBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let socket_box = Box::new(SocketBox::new()) as Box<dyn NyashBox>;
                return Ok(Some(socket_box));
            }
            "HTTPServerBox" => {
                // HTTPServerBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("HTTPServerBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let http_server_box = Box::new(HTTPServerBox::new()) as Box<dyn NyashBox>;
                return Ok(Some(http_server_box));
            }
            "HTTPRequestBox" => {
                // HTTPRequestBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("HTTPRequestBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let http_request_box = Box::new(HTTPRequestBox::new()) as Box<dyn NyashBox>;
                return Ok(Some(http_request_box));
            }
            "HTTPResponseBox" => {
                // HTTPResponseBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("HTTPResponseBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let http_response_box = Box::new(HTTPResponseBox::new()) as Box<dyn NyashBox>;
                return Ok(Some(http_response_box));
            }
            _ => {
                // Not a builtin box, return None to indicate this should be handled by user-defined box logic
                return Ok(None);
            }
        }
    }
}