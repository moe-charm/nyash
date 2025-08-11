/*!
 * Object Processing Module
 * 
 * Extracted from core.rs - object creation, construction, and inheritance
 * Handles Box declarations, instantiation, constructors, and inheritance system
 * Core philosophy: "Everything is Box" with complete OOP support
 */

use super::*;
use crate::boxes::null_box::NullBox;
use crate::boxes::console_box::ConsoleBox;
// use crate::boxes::intent_box_wrapper::IntentBoxWrapper;
use std::sync::Arc;

impl NyashInterpreter {
    /// new式を実行 - Object creation engine  
    pub(super) fn execute_new(&mut self, class: &str, arguments: &[ASTNode], type_arguments: &[String]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 組み込みBox型のチェック
        match class {
            "ArrayBox" => {
                // ArrayBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("ArrayBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let array_box = Box::new(ArrayBox::new()) as Box<dyn NyashBox>;
                // 🌍 革命的実装：Environment tracking廃止
                return Ok(array_box);
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
                    return Ok(file_box);
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
                return Ok(result_box);
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
                    return Ok(error_box);
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
                return Ok(math_box);
            }
            "NullBox" => {
                // NullBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("NullBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let null_box = Box::new(NullBox::new()) as Box<dyn NyashBox>;
                return Ok(null_box);
            }
            "ConsoleBox" => {
                // ConsoleBoxは引数なしで作成（ブラウザconsole連携用）
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("ConsoleBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let console_box = Box::new(ConsoleBox::new()) as Box<dyn NyashBox>;
                return Ok(console_box);
            }
            // "IntentBox" => {
            //     // IntentBoxは引数なしで作成（メッセージバス）
            //     if !arguments.is_empty() {
            //         return Err(RuntimeError::InvalidOperation {
            //             message: format!("IntentBox constructor expects 0 arguments, got {}", arguments.len()),
            //         });
            //     }
            //     let intent_box = Arc::new(crate::boxes::IntentBox::new());
            //     let intent_box_wrapped = Box::new(IntentBoxWrapper {
            //         inner: intent_box
            //     }) as Box<dyn NyashBox>;
            //     return Ok(intent_box_wrapped);
            // }
            // "P2PBox" => {
            //     // P2PBoxは引数2個（node_id, intent_box）で作成
            //     if arguments.len() != 2 {
            //         return Err(RuntimeError::InvalidOperation {
            //             message: format!("P2PBox constructor expects 2 arguments (node_id, intent_box), got {}", arguments.len()),
            //         });
            //     }
            //     
            //     // node_id
            //     let node_id_value = self.execute_expression(&arguments[0])?;
            //     let node_id = if let Some(id_str) = node_id_value.as_any().downcast_ref::<StringBox>() {
            //         id_str.value.clone()
            //     } else {
            //         return Err(RuntimeError::TypeError {
            //             message: "P2PBox constructor requires string node_id as first argument".to_string(),
            //         });
            //     };
            //     
            //     // intent_box
            //     let intent_box_value = self.execute_expression(&arguments[1])?;
            //     let intent_box = if let Some(wrapper) = intent_box_value.as_any().downcast_ref::<IntentBoxWrapper>() {
            //         wrapper.inner.clone()
            //     } else {
            //         return Err(RuntimeError::TypeError {
            //             message: "P2PBox constructor requires IntentBox as second argument".to_string(),
            //         });
            //     };
            //     
            //     let p2p_box = Box::new(crate::boxes::P2PBox::new(node_id, intent_box)) as Box<dyn NyashBox>;
            //     return Ok(p2p_box);
            // }
            #[cfg(not(target_arch = "wasm32"))]
            "EguiBox" => {
                // EguiBoxは引数なしで作成（GUIアプリケーション用）
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("EguiBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let egui_box = Box::new(crate::boxes::EguiBox::new()) as Box<dyn NyashBox>;
                return Ok(egui_box);
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
                    return Ok(web_display_box);
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
                    return Ok(web_console_box);
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
                return Ok(web_canvas_box);
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
                    return Ok(float_box);
                } else if let Some(float_box) = value.as_any().downcast_ref::<FloatBox>() {
                    let new_float_box = Box::new(FloatBox::new(float_box.value)) as Box<dyn NyashBox>;
                    // 🌍 革命的実装：Environment tracking廃止
                    return Ok(new_float_box);
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
                    return Ok(range_box);
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
                return Ok(time_box);
            }
            "DateTimeBox" => {
                // DateTimeBoxは引数なしで現在時刻、または引数1個でタイムスタンプ
                match arguments.len() {
                    0 => {
                        let datetime_box = Box::new(DateTimeBox::now()) as Box<dyn NyashBox>;
                        // 🌍 革命的実装：Environment tracking廃止
                        return Ok(datetime_box);
                    }
                    1 => {
                        let timestamp_value = self.execute_expression(&arguments[0])?;
                        if let Some(int_box) = timestamp_value.as_any().downcast_ref::<IntegerBox>() {
                            let datetime_box = Box::new(DateTimeBox::from_timestamp(int_box.value)) as Box<dyn NyashBox>;
                            // 🌍 革命的実装：Environment tracking廃止
                            return Ok(datetime_box);
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
                return Ok(timer_box);
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
                return Ok(map_box);
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
                return Ok(random_box);
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
                return Ok(sound_box);
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
                return Ok(debug_box);
            }
            "BufferBox" => {
                // BufferBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("BufferBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let buffer_box = Box::new(crate::boxes::buffer::BufferBox::new()) as Box<dyn NyashBox>;
                return Ok(buffer_box);
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
                        Ok(regex_box) => return Ok(Box::new(regex_box)),
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
                        Ok(json_box) => return Ok(Box::new(json_box)),
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
                // IntentBoxは引数なしで作成（デフォルトローカル通信）
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("IntentBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let intent_box = crate::boxes::IntentBox::new();
                return Ok(Box::new(intent_box) as Box<dyn NyashBox>);
            }
            
            "P2PBox" => {
                // P2PBoxは引数2個（node_id, intent_box）で作成
                if arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: "P2PBox requires at least 1 argument (node_id)".to_string(),
                    });
                }
                
                // 引数を評価
                let mut arg_values = Vec::new();
                for arg in arguments {
                    arg_values.push(self.execute_expression(arg)?);
                }
                
                // 第1引数: ノードID
                let node_id = if let Some(str_box) = arg_values[0].as_any().downcast_ref::<StringBox>() {
                    str_box.value.clone()
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "P2PBox first argument must be a string (node_id)".to_string(),
                    });
                };
                
                // 第2引数: IntentBox（省略時はデフォルト）
                let intent_box = if arg_values.len() > 1 {
                    if let Some(intent) = arg_values[1].as_any().downcast_ref::<crate::boxes::IntentBox>() {
                        std::sync::Arc::new(intent.clone())
                    } else {
                        return Err(RuntimeError::TypeError {
                            message: "P2PBox second argument must be an IntentBox".to_string(),
                        });
                    }
                } else {
                    // デフォルトのIntentBoxを作成
                    std::sync::Arc::new(crate::boxes::IntentBox::new())
                };
                
                let p2p_box = crate::boxes::P2PBox::new(node_id, intent_box);
                return Ok(Box::new(p2p_box));
            }
            "StreamBox" => {
                // StreamBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("StreamBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let stream_box = Box::new(crate::boxes::stream::StreamBox::new()) as Box<dyn NyashBox>;
                return Ok(stream_box);
            }
            "HTTPClientBox" => {
                // HTTPClientBoxは引数なしで作成
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("HTTPClientBox constructor expects 0 arguments, got {}", arguments.len()),
                    });
                }
                let http_box = Box::new(crate::boxes::http::HttpClientBox::new()) as Box<dyn NyashBox>;
                return Ok(http_box);
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
                    return Ok(method_box);
                } else {
                    return Err(RuntimeError::TypeError {
                        message: "MethodBox constructor requires string method name as second argument".to_string(),
                    });
                }
            }
            _ => {}
        }
        
        // 🔥 Static Boxインスタンス化禁止チェック
        if self.is_static_box(class) {
            return Err(RuntimeError::InvalidOperation {
                message: format!("Cannot instantiate static box '{}'. Static boxes cannot be instantiated.", class),
            });
        }
        
        // ユーザー定義Box宣言を探す
        let box_decl = {
            let box_decls = self.shared.box_declarations.read().unwrap();
            box_decls.get(class)
                .ok_or(RuntimeError::UndefinedClass { name: class.to_string() })?
                .clone()
        };
        
        // 🔥 ジェネリクス型引数の検証
        if !box_decl.type_parameters.is_empty() || !type_arguments.is_empty() {
            self.validate_generic_arguments(&box_decl, type_arguments)?;
        }
        
        // インターフェースはインスタンス化できない
        if box_decl.is_interface {
            return Err(RuntimeError::InvalidOperation {
                message: format!("Cannot instantiate interface '{}'", class),
            });
        }
        
        // 🚀 ジェネリクス型の特殊化処理
        let (final_box_decl, actual_class_name) = if !type_arguments.is_empty() {
            // ジェネリクス型を特殊化
            let specialized = self.specialize_generic_class(&box_decl, type_arguments)?;
            let specialized_name = specialized.name.clone();
            (specialized, specialized_name)
        } else {
            (box_decl.clone(), class.to_string())
        };
        
        // 継承チェーンを解決してフィールドとメソッドを収集（init_fieldsも含む）
        let (all_fields, all_methods) = self.resolve_inheritance(&final_box_decl)?;
        
        // インスタンスを作成
        let instance = InstanceBox::new(
            actual_class_name.clone(),
            all_fields,
            all_methods
        );
        
        let instance_box = Box::new(instance) as Box<dyn NyashBox>;
        
        // 現在のスコープでBoxを追跡（自動解放のため）
        // 🌍 革命的実装：Environment tracking廃止
        
        // コンストラクタを呼び出す
        // "pack/引数数"、"init/引数数"、"Box名/引数数" の順で試す
        let pack_key = format!("pack/{}", arguments.len());
        let init_key = format!("init/{}", arguments.len());
        let box_name_key = format!("{}/{}", actual_class_name, arguments.len());
        
        if let Some(constructor) = final_box_decl.constructors.get(&pack_key)
            .or_else(|| final_box_decl.constructors.get(&init_key))
            .or_else(|| final_box_decl.constructors.get(&box_name_key)) {
            // コンストラクタを実行
            self.execute_constructor(&instance_box, constructor, arguments, &final_box_decl)?;
        } else if !arguments.is_empty() {
            return Err(RuntimeError::InvalidOperation {
                message: format!("No constructor found for {} with {} arguments", class, arguments.len()),
            });
        }
        
        Ok(instance_box)
    }
    
    /// コンストラクタを実行 - Constructor execution
    pub(super) fn execute_constructor(
        &mut self, 
        instance: &Box<dyn NyashBox>, 
        constructor: &ASTNode, 
        arguments: &[ASTNode],
        box_decl: &BoxDeclaration
    ) -> Result<(), RuntimeError> {
        if let ASTNode::FunctionDeclaration { name: _, params, body, .. } = constructor {
            // 引数を評価
            let mut arg_values = Vec::new();
            for arg in arguments {
                arg_values.push(self.execute_expression(arg)?);
            }
            
            // パラメータ数チェック
            if params.len() != arg_values.len() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("Constructor expects {} arguments, got {}", params.len(), arg_values.len()),
                });
            }
            
            // 🌍 革命的コンストラクタ実行：local変数スタックを使用
            let saved_locals = self.save_local_vars();
            self.local_vars.clear();
            
            // パラメータをlocal変数として設定
            for (param, value) in params.iter().zip(arg_values.iter()) {
                self.declare_local_variable(param, value.clone_box());
            }
            
            // this（me）をlocal変数として設定
            self.declare_local_variable("me", instance.clone_box());
            
            // コンストラクタコンテキストを設定
            let old_context = self.current_constructor_context.clone();
            self.current_constructor_context = Some(ConstructorContext {
                class_name: box_decl.name.clone(),
                parent_class: box_decl.extends.clone(),
            });
            
            // コンストラクタを実行
            let mut result = Ok(());
            for statement in body.iter() {
                if let Err(e) = self.execute_statement(statement) {
                    result = Err(e);
                    break;
                }
            }
            
            // local変数スタックとコンテキストを復元
            self.restore_local_vars(saved_locals);
            self.current_constructor_context = old_context;
            
            result
        } else {
            Err(RuntimeError::InvalidOperation {
                message: "Invalid constructor node".to_string(),
            })
        }
    }
    
    /// Box宣言を登録 - 🔥 コンストラクタオーバーロード禁止対応
    pub(super) fn register_box_declaration(
        &mut self, 
        name: String, 
        fields: Vec<String>, 
        methods: HashMap<String, ASTNode>,
        constructors: HashMap<String, ASTNode>,
        init_fields: Vec<String>,
        is_interface: bool,
        extends: Option<String>,
        implements: Vec<String>,
        type_parameters: Vec<String>  // 🔥 ジェネリクス型パラメータ追加
    ) -> Result<(), RuntimeError> {
        
        // 🚨 コンストラクタオーバーロード禁止：複数コンストラクタ検出
        if constructors.len() > 1 {
            let constructor_names: Vec<String> = constructors.keys().cloned().collect();
            return Err(RuntimeError::InvalidOperation {
                message: format!(
                    "🚨 CONSTRUCTOR OVERLOAD FORBIDDEN: Box '{}' has {} constructors: [{}].\n\
                    🌟 Nyash's explicit philosophy: One Box, One Constructor!\n\
                    💡 Use different Box classes for different initialization patterns.\n\
                    📖 Example: UserBox, AdminUserBox, GuestUserBox instead of User(type)",
                    name, 
                    constructors.len(),
                    constructor_names.join(", ")
                )
            });
        }
        let box_decl = super::BoxDeclaration { 
            name: name.clone(), 
            fields, 
            methods,
            constructors,
            init_fields,
            is_interface,
            extends,
            implements,
            type_parameters, // 🔥 ジェネリクス型パラメータを正しく使用
        };
        
        {
            let mut box_decls = self.shared.box_declarations.write().unwrap();
            box_decls.insert(name, box_decl);
        }
        
        Ok(()) // 🔥 正常終了
    }
    
    /// 🔥 ジェネリクス型引数の検証
    fn validate_generic_arguments(&self, box_decl: &BoxDeclaration, type_arguments: &[String]) 
        -> Result<(), RuntimeError> {
        // 型パラメータと型引数の数が一致するかチェック
        if box_decl.type_parameters.len() != type_arguments.len() {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Generic class '{}' expects {} type parameters, got {}. Expected: <{}>, Got: <{}>",
                    box_decl.name,
                    box_decl.type_parameters.len(),
                    type_arguments.len(),
                    box_decl.type_parameters.join(", "),
                    type_arguments.join(", ")
                ),
            });
        }
        
        // 型引数がジェネリクスでない場合、型パラメータがあってはならない
        if box_decl.type_parameters.is_empty() && !type_arguments.is_empty() {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Class '{}' is not generic, but got type arguments <{}>",
                    box_decl.name,
                    type_arguments.join(", ")
                ),
            });
        }
        
        // 各型引数が有効なBox型かチェック（基本型のみチェック）
        for type_arg in type_arguments {
            if !self.is_valid_type(type_arg) {
                return Err(RuntimeError::TypeError {
                    message: format!("Unknown type '{}'", type_arg),
                });
            }
        }
        
        Ok(())
    }
    
    /// 型が有効かどうかをチェック
    fn is_valid_type(&self, type_name: &str) -> bool {
        // 基本的なビルトイン型
        let is_builtin = matches!(type_name, 
            "IntegerBox" | "StringBox" | "BoolBox" | "ArrayBox" | "MapBox" | 
            "FileBox" | "ResultBox" | "FutureBox" | "ChannelBox" | "MathBox" | 
            "TimeBox" | "DateTimeBox" | "TimerBox" | "RandomBox" | "SoundBox" | 
            "DebugBox" | "MethodBox" | "NullBox" | "ConsoleBox" | "FloatBox" |
            "BufferBox" | "RegexBox" | "JSONBox" | "StreamBox" | "HTTPClientBox" |
            "IntentBox" | "P2PBox"
        );
        
        // Web専用Box（WASM環境のみ）
        #[cfg(target_arch = "wasm32")]
        let is_web_box = matches!(type_name, "WebDisplayBox" | "WebConsoleBox" | "WebCanvasBox");
        #[cfg(not(target_arch = "wasm32"))]
        let is_web_box = false;
        
        // GUI専用Box（非WASM環境のみ）
        #[cfg(not(target_arch = "wasm32"))]
        let is_gui_box = matches!(type_name, "EguiBox");
        #[cfg(target_arch = "wasm32")]
        let is_gui_box = false;
        
        is_builtin || is_web_box || is_gui_box ||
        // または登録済みのユーザー定義Box
        self.shared.box_declarations.read().unwrap().contains_key(type_name)
    }
    
    /// 親コンストラクタを実行 - Parent constructor execution
    pub(super) fn execute_parent_constructor(&mut self, parent_class: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 親クラスの宣言を取得
        let parent_decl = {
            let box_decls = self.shared.box_declarations.read().unwrap();
            box_decls.get(parent_class)
                .ok_or(RuntimeError::UndefinedClass { name: parent_class.to_string() })?
                .clone()
        };
            
        // 親コンストラクタを探す
        // まず "init/引数数" を試し、なければ "Box名/引数数" を試す
        let init_key = format!("init/{}", arguments.len());
        let box_name_key = format!("{}/{}", parent_class, arguments.len());
        
        if let Some(parent_constructor) = parent_decl.constructors.get(&init_key)
            .or_else(|| parent_decl.constructors.get(&box_name_key)) {
            // 現在のthis参照を取得
            // 🌍 革命的this取得：local変数から
            let this_instance = self.resolve_variable("me")
                .map_err(|_| RuntimeError::InvalidOperation {
                    message: "'this' not available in parent constructor call".to_string(),
                })?;
                
            // 親コンストラクタを実行
            self.execute_constructor(&this_instance, parent_constructor, arguments, &parent_decl)?;
            
            // VoidBoxを返す（コンストラクタ呼び出しは値を返さない）
            Ok(Box::new(VoidBox::new()))
        } else {
            Err(RuntimeError::InvalidOperation {
                message: format!("No constructor found for parent class {} with {} arguments", parent_class, arguments.len()),
            })
        }
    }
    
    /// 継承チェーンを解決してフィールドとメソッドを収集 - Inheritance resolution
    pub(super) fn resolve_inheritance(&self, box_decl: &BoxDeclaration) 
        -> Result<(Vec<String>, HashMap<String, ASTNode>), RuntimeError> {
        let mut all_fields = Vec::new();
        let mut all_methods = HashMap::new();
        
        // 親クラスの継承チェーンを再帰的に解決
        if let Some(parent_name) = &box_decl.extends {
            let parent_decl = {
                let box_decls = self.shared.box_declarations.read().unwrap();
                box_decls.get(parent_name)
                    .ok_or(RuntimeError::UndefinedClass { name: parent_name.clone() })?
                    .clone()
            };
            
            // インターフェースは継承できない
            if parent_decl.is_interface {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("Cannot extend interface '{}'. Use 'implements' instead.", parent_name),
                });
            }
            
            // 親クラスの継承チェーンを再帰的に解決
            let (parent_fields, parent_methods) = self.resolve_inheritance(&parent_decl)?;
            
            // 親のフィールドとメソッドを追加
            all_fields.extend(parent_fields);
            all_methods.extend(parent_methods);
        }
        
        // 現在のクラスのフィールドとメソッドを追加（オーバーライド可能）
        all_fields.extend(box_decl.fields.clone());
        
        // init_fieldsも追加（重複チェック）
        for init_field in &box_decl.init_fields {
            if !all_fields.contains(init_field) {
                all_fields.push(init_field.clone());
            }
        }
        
        for (method_name, method_ast) in &box_decl.methods {
            all_methods.insert(method_name.clone(), method_ast.clone());  // オーバーライド
        }
        
        // インターフェース実装の検証
        for interface_name in &box_decl.implements {
            let interface_decl = {
                let box_decls = self.shared.box_declarations.read().unwrap();
                box_decls.get(interface_name)
                    .ok_or(RuntimeError::UndefinedClass { name: interface_name.clone() })?
                    .clone()
            };
            
            if !interface_decl.is_interface {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("'{}' is not an interface", interface_name),
                });
            }
            
            // インターフェースの全メソッドが実装されているかチェック
            for (required_method, _) in &interface_decl.methods {
                if !all_methods.contains_key(required_method) {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("Class '{}' must implement method '{}' from interface '{}'", 
                                       box_decl.name, required_method, interface_name),
                    });
                }
            }
        }
        
        Ok((all_fields, all_methods))
    }
    
    /// 🚀 ジェネリクス型を特殊化してBoxDeclarationを生成
    fn specialize_generic_class(
        &self, 
        generic_decl: &BoxDeclaration, 
        type_arguments: &[String]
    ) -> Result<BoxDeclaration, RuntimeError> {
        use std::collections::HashMap;
        
        // 特殊化されたクラス名を生成
        let specialized_name = format!(
            "{}_{}",
            generic_decl.name,
            type_arguments.join("_")
        );
        
        // 型パラメータ → 具体型のマッピングを作成
        let mut type_mapping = HashMap::new();
        for (i, param) in generic_decl.type_parameters.iter().enumerate() {
            type_mapping.insert(param.clone(), type_arguments[i].clone());
        }
        
        // 特殊化されたBoxDeclarationを作成
        let mut specialized = generic_decl.clone();
        specialized.name = specialized_name.clone();
        specialized.type_parameters.clear(); // 特殊化後は型パラメータなし
        
        // 🔄 フィールドの型を置換
        specialized.init_fields = self.substitute_types_in_fields(
            &specialized.init_fields, 
            &type_mapping
        );
        
        // 🔧 コンストラクタキーを新しいクラス名で更新
        let mut updated_constructors = HashMap::new();
        for (old_key, constructor_node) in &generic_decl.constructors {
            // "Container/1" -> "Container_IntegerBox/1" に変更
            if let Some(args_count) = old_key.split('/').nth(1) {
                let new_key = format!("{}/{}", specialized_name, args_count);
                updated_constructors.insert(new_key, constructor_node.clone());
            }
        }
        specialized.constructors = updated_constructors;
        
        // 🔄 メソッドの型を置換（現在はプレースホルダー実装）
        // TODO: メソッド内部のコードも置換が必要
        
        Ok(specialized)
    }
    
    /// フィールドの型置換
    fn substitute_types_in_fields(
        &self,
        fields: &[String],
        _type_mapping: &HashMap<String, String>
    ) -> Vec<String> {
        // TODO: フィールド型の置換実装
        // 現在はシンプルにコピー
        fields.to_vec()
    }
}