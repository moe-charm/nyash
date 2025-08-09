/*!
 * Math and Random Box Method Handlers Module
 * 
 * Extracted from box_methods.rs lines 148-632
 * Contains mathematical computation and random number generation method implementations:
 * 
 * MathBox methods:
 * - abs, max, min, pow, sqrt - Basic mathematical operations
 * - sin, cos, tan - Trigonometric functions
 * - log, log10, exp - Logarithmic and exponential functions
 * - floor, ceil, round - Rounding operations
 * - getPi, getE - Mathematical constants
 * 
 * RandomBox methods:
 * - seed, random, randInt, randBool - Basic random generation
 * - choice, shuffle, randString - Advanced random operations
 * - probability - Probability-based operations
 * 
 * All methods include comprehensive argument validation and error handling.
 */

use super::*;

impl NyashInterpreter {
    /// MathBoxのメソッド呼び出しを実行
    /// 包括的な数学計算機能を提供
    pub(super) fn execute_math_method(&mut self, math_box: &MathBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // メソッドを実行
        match method {
            // 基本数学演算
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
            
            // 数学定数
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
            
            // 三角関数
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
            
            // 対数・指数関数
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
            
            // 丸め関数
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

    /// RandomBoxのメソッド呼び出しを実行
    /// 乱数生成と確率的操作を提供
    pub(super) fn execute_random_method(&mut self, random_box: &RandomBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 引数を評価
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.execute_expression(arg)?);
        }
        
        // メソッドを実行
        match method {
            // 乱数シード設定
            "seed" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("seed() expects 1 argument, got {}", arg_values.len()),
                    });
                }
                Ok(random_box.seed(arg_values[0].clone_box()))
            }
            
            // 基本乱数生成
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
            
            // 配列・コレクション操作
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
            
            // 文字列・確率操作
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
}