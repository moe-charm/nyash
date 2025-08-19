/*!
 * Web Box Methods Module
 * 
 * Extracted from box_methods.rs
 * Contains WASM/browser-specific Box type method implementations:
 * 
 * - execute_web_display_method (WebDisplayBox) - HTML DOM manipulation
 * - execute_web_console_method (WebConsoleBox) - Browser console logging  
 * - execute_web_canvas_method (WebCanvasBox) - Canvas drawing operations
 * 
 * All methods are conditionally compiled for WASM target architecture only.
 */

#[cfg(target_arch = "wasm32")]
use super::*;
#[cfg(target_arch = "wasm32")]
use crate::boxes::web::{WebDisplayBox, WebConsoleBox, WebCanvasBox};
#[cfg(target_arch = "wasm32")]
use crate::boxes::FloatBox;

#[cfg(target_arch = "wasm32")]
impl NyashInterpreter {
    /// WebDisplayBoxメソッド実行 (WASM環境のみ)
    /// HTML DOM操作、CSS スタイル設定、クラス管理などの包括的なWeb表示機能
    pub(super) fn execute_web_display_method(&mut self, web_display_box: &WebDisplayBox, method: &str, arguments: &[ASTNode]) 
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
    /// ブラウザーコンソールへの多彩なログ出力、グループ化、区切り表示機能
    pub(super) fn execute_web_console_method(&mut self, web_console_box: &WebConsoleBox, method: &str, arguments: &[ASTNode]) 
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
    /// HTML5 Canvas描画操作 - 矩形、円、テキスト描画の包括的な2D描画機能
    pub(super) fn execute_web_canvas_method(&mut self, web_canvas_box: &WebCanvasBox, method: &str, arguments: &[ASTNode]) 
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