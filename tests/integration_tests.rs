/*! 
 * Nyash Rust Implementation - Integration Tests
 * 
 * 完全なNyash言語機能テストスイート
 * Everything is Box哲学の包括的検証
 */

use nyash_rust::*;
use std::collections::HashMap;

/// 統合テストヘルパー - コードを実行して結果を検証
fn execute_nyash_code(code: &str) -> Result<String, String> {
    match parser::NyashParser::parse_from_string(code) {
        Ok(ast) => {
            let mut interpreter = interpreter::NyashInterpreter::new();
            match interpreter.execute(ast) {
                Ok(result) => Ok(result.to_string_box().value),
                Err(e) => Err(format!("Runtime error: {}", e)),
            }
        }
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}

/// 変数値を取得するヘルパー
fn get_variable_value(code: &str, var_name: &str) -> Result<String, String> {
    match parser::NyashParser::parse_from_string(code) {
        Ok(ast) => {
            let mut interpreter = interpreter::NyashInterpreter::new();
            interpreter.execute(ast).map_err(|e| format!("Execution error: {}", e))?;
            
            match interpreter.get_variable(var_name) {
                Ok(value) => Ok(value.to_string_box().value),
                Err(_) => Err(format!("Variable '{}' not found", var_name)),
            }
        }
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        let code = r#"
        a = 10
        b = 32
        result = a + b
        "#;
        
        let result = get_variable_value(code, "result").unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_string_operations() {
        let code = r#"
        greeting = "Hello, "
        name = "Nyash!"
        message = greeting + name
        "#;
        
        let result = get_variable_value(code, "message").unwrap();
        assert_eq!(result, "Hello, Nyash!");
    }

    #[test]
    fn test_boolean_logic() {
        let code = r#"
        a = true
        b = false
        and_result = a && b
        or_result = a || b
        "#;
        
        let and_result = get_variable_value(code, "and_result").unwrap();
        let or_result = get_variable_value(code, "or_result").unwrap();
        
        assert_eq!(and_result, "false");
        assert_eq!(or_result, "true");
    }

    #[test]
    fn test_comparison_operators() {
        let code = r#"
        a = 10
        b = 20
        less = a < b
        greater = a > b
        equal = a == a
        not_equal = a != b
        "#;
        
        assert_eq!(get_variable_value(code, "less").unwrap(), "true");
        assert_eq!(get_variable_value(code, "greater").unwrap(), "false");
        assert_eq!(get_variable_value(code, "equal").unwrap(), "true");
        assert_eq!(get_variable_value(code, "not_equal").unwrap(), "true");
    }

    #[test]
    fn test_if_else_statements() {
        let code = r#"
        condition = true
        if condition {
            result = "success"
        } else {
            result = "failure"
        }
        "#;
        
        let result = get_variable_value(code, "result").unwrap();
        assert_eq!(result, "success");
    }

    #[test]
    fn test_loop_with_break() {
        let code = r#"
        counter = 0
        loop {
            counter = counter + 1
            if counter == 5 {
                break
            }
        }
        "#;
        
        let result = get_variable_value(code, "counter").unwrap();
        assert_eq!(result, "5");
    }

    #[test]
    fn test_function_declaration_and_call() {
        let code = r#"
        fn add(a, b) {
            return a + b
        }
        
        result = add(10, 32)
        "#;
        
        let result = get_variable_value(code, "result").unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_box_instance_creation() {
        let code = r#"
        box Point {
            init { x, y }
            
            getX() {
                return me.x
            }
        }
        
        p = new Point()
        p.x = 100
        result = p.getX()
        "#;
        
        let result = get_variable_value(code, "result").unwrap();
        assert_eq!(result, "100");
    }

    #[test]
    fn test_this_binding_in_methods() {
        let code = r#"
        box Calculator {
            value
            
            setValue(v) {
                me.value = v
            }
            
            getValue() {
                return me.value
            }
            
            add(amount) {
                me.value = me.value + amount
                return me.value
            }
        }
        
        calc = new Calculator()
        calc.setValue(10)
        calc.add(32)
        result = calc.getValue()
        "#;
        
        let result = get_variable_value(code, "result").unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_method_chaining_concept() {
        let code = r#"
        box Counter {
            init { count }
            
            birth() {
                me.count = 0
            }
            
            increment() {
                me.count = me.count + 1
                return me.count
            }
            
            getCount() {
                return me.count
            }
        }
        
        c = new Counter()
        c.increment()
        c.increment()
        c.increment()
        result = c.getCount()
        "#;
        
        let result = get_variable_value(code, "result").unwrap();
        assert_eq!(result, "3");
    }

    #[test]
    fn test_multiple_instances() {
        let code = r#"
        box Data {
            init { value }
            
            setValue(v) {
                me.value = v
            }
            
            getValue() {
                return me.value
            }
        }
        
        d1 = new Data()
        d2 = new Data()
        
        d1.setValue("first")
        d2.setValue("second")
        
        result1 = d1.getValue()
        result2 = d2.getValue()
        "#;
        
        let result1 = get_variable_value(code, "result1").unwrap();
        let result2 = get_variable_value(code, "result2").unwrap();
        
        assert_eq!(result1, "first");
        assert_eq!(result2, "second");
    }

    #[test]
    fn test_global_variables() {
        let code = r#"
        global config = "production"
        global version = 42
        
        fn getConfig() {
            return config + " v" + version
        }
        
        result = getConfig()
        "#;
        
        let result = get_variable_value(code, "result").unwrap();
        assert_eq!(result, "production v42");
    }

    #[test]
    fn test_complex_expression_evaluation() {
        let code = r#"
        a = 5
        b = 10
        c = 15
        result = (a + b) * c - a
        "#;
        
        let result = get_variable_value(code, "result").unwrap();
        // (5 + 10) * 15 - 5 = 15 * 15 - 5 = 225 - 5 = 220
        assert_eq!(result, "220");
    }

    #[test]
    fn test_nested_method_calls() {
        let code = r#"
        box Wrapper {
            init { inner }
            
            setInner(value) {
                me.inner = value
            }
            
            getInner() {
                return me.inner
            }
        }
        
        box Container {
            init { wrapper }
            
            createWrapper() {
                me.wrapper = new Wrapper()
                return me.wrapper
            }
            
            getWrapper() {
                return me.wrapper
            }
        }
        
        container = new Container()
        w = container.createWrapper()
        w.setInner("nested value")
        result = container.getWrapper().getInner()
        "#;
        
        let result = get_variable_value(code, "result").unwrap();
        assert_eq!(result, "nested value");
    }

    #[test]
    fn test_all_numeric_operations() {
        let code = r#"
        a = 20
        b = 5
        
        add_result = a + b
        sub_result = a - b
        mul_result = a * b
        
        less_result = b < a
        greater_result = a > b
        less_eq_result = b <= a
        greater_eq_result = a >= b
        "#;
        
        assert_eq!(get_variable_value(code, "add_result").unwrap(), "25");
        assert_eq!(get_variable_value(code, "sub_result").unwrap(), "15");
        assert_eq!(get_variable_value(code, "mul_result").unwrap(), "100");
        assert_eq!(get_variable_value(code, "less_result").unwrap(), "true");
        assert_eq!(get_variable_value(code, "greater_result").unwrap(), "true");
        assert_eq!(get_variable_value(code, "less_eq_result").unwrap(), "true");
        assert_eq!(get_variable_value(code, "greater_eq_result").unwrap(), "true");
    }

    #[test]
    fn test_the_debug_this_problem() {
        // 元のdebug_this_problem.nyashと同等のテスト
        let code = r#"
        box TestBox {
            init { value }
            
            getValue() {
                return me.value
            }
        }
        
        obj = new TestBox()
        obj.value = "test123"
        direct_access = obj.value
        method_result = obj.getValue()
        "#;
        
        let direct = get_variable_value(code, "direct_access").unwrap();
        let method = get_variable_value(code, "method_result").unwrap();
        
        assert_eq!(direct, "test123");
        assert_eq!(method, "test123");
        
        // thisが正しく動作している証明
        assert_eq!(direct, method);
    }
}