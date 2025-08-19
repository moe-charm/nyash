/*!
 * Special Methods Module
 * 
 * Extracted from box_methods.rs
 * Contains specialized Box method implementations:
 * 
 * - execute_method_box_method (MethodBox) - イベントハンドラー/関数ポインタ機能
 * - execute_sound_method (SoundBox) - オーディオ機能
 * 
 * These are critical special-purpose Box implementations:
 * - MethodBox: Essential for event handling and callback functionality
 * - SoundBox: Essential for audio feedback and game sound effects
 */

use super::*;
use crate::boxes::SoundBox;
use crate::method_box::MethodBox;
use crate::instance_v2::InstanceBox;

impl NyashInterpreter {
    /// SoundBoxのメソッド呼び出しを実行
    /// 
    /// SoundBoxはオーディオ機能を提供する重要なBox:
    /// - beep(), beeps() - 基本的なビープ音
    /// - tone() - カスタム周波数/期間の音
    /// - alert(), success(), error() - UI音効果
    /// - pattern() - 音パターン再生
    /// - volumeTest() - 音量テスト
    /// - interval() - 間隔付き音再生
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

    /// MethodBoxのメソッド呼び出しを実行
    /// 
    /// MethodBoxはイベントハンドラー機能の核心:
    /// - invoke() - メソッド参照を実際に呼び出し
    /// - 関数ポインタ相当の機能を提供
    /// - GUI/イベント駆動プログラミングに必須
    pub(super) fn execute_method_box_method(&mut self, method_box: &MethodBox, method: &str, arguments: &[ASTNode]) 
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
    /// 
    /// この関数はMethodBoxの中核機能:
    /// 1. インスタンスとメソッド名からメソッドを取得
    /// 2. 引数数の検証
    /// 3. local変数スタック管理
    /// 4. 'me' 変数の設定
    /// 5. メソッド実行
    /// 6. 戻り値処理
    fn invoke_method_box(&mut self, method_box: &MethodBox, args: Vec<Box<dyn NyashBox>>) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // インスタンスを取得
        let instance_arc = method_box.get_instance();
        let instance = instance_arc.lock().unwrap();
        
        // InstanceBoxにダウンキャスト
        if let Some(instance_box) = instance.as_any().downcast_ref::<InstanceBox>() {
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
}