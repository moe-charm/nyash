/*!
 * Type Validation Module
 * 
 * Extracted from objects.rs - handles type checking and validation
 * Core responsibility: Validating generic arguments and type system integrity
 * Part of robust type safety in "Everything is Box" philosophy
 */

use super::*;

impl NyashInterpreter {
    /// 🔥 ジェネリクス型引数の検証
    pub(super) fn validate_generic_arguments(&self, box_decl: &BoxDeclaration, type_arguments: &[String]) 
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
    pub(super) fn is_valid_type(&self, type_name: &str) -> bool {
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
        #[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
        let is_gui_box = matches!(type_name, "EguiBox");
        #[cfg(not(all(feature = "gui", not(target_arch = "wasm32"))))]
        let is_gui_box = false;
        
        is_builtin || is_web_box || is_gui_box ||
        // または登録済みのユーザー定義Box
        self.shared.box_declarations.read().unwrap().contains_key(type_name)
    }
}