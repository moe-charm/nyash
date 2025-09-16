#[cfg(test)]
mod array_state_sharing_tests {
    use nyash_rust::box_trait::{IntegerBox, NyashBox, StringBox};
    use nyash_rust::boxes::array::ArrayBox;
    use nyash_rust::interpreter::NyashInterpreter;
    use nyash_rust::parser::NyashParser;

    #[test]
    fn test_arraybox_state_sharing_bug_fix() {
        // 🚨 問題再現テスト
        let mut interpreter = NyashInterpreter::new();
        let program = r#"
            static box Main {
                init { result }
                main() {
                    local arr
                    arr = new ArrayBox()
                    arr.push("hello")
                    me.result = arr.length()
                    return me.result
                }
            }
        "#;

        let ast = NyashParser::parse_from_string(program).unwrap();
        let result = interpreter.execute(ast).unwrap();
        let int_result = result.as_any().downcast_ref::<IntegerBox>().unwrap();
        assert_eq!(int_result.value, 1); // 🎯 0ではなく1を返すべき
    }

    #[test]
    fn test_share_box_vs_clone_box_semantics() {
        let arr1 = ArrayBox::new();
        arr1.push(Box::new(StringBox::new("hello")));

        // share_box: 状態共有
        let arr2 = arr1.share_box();
        let arr2_array = arr2.as_any().downcast_ref::<ArrayBox>().unwrap();
        assert_eq!(arr2_array.len(), 1); // 共有されている

        // clone_box: 独立
        let arr3 = arr1.clone_box();
        let arr3_array = arr3.as_any().downcast_ref::<ArrayBox>().unwrap();
        arr1.push(Box::new(StringBox::new("world")));
        assert_eq!(arr3_array.len(), 1); // 影響を受けない
        assert_eq!(arr1.len(), 2); // 元は2要素
        assert_eq!(arr2_array.len(), 2); // 共有されているので2要素
    }

    #[test]
    fn test_multiple_operations_state_preservation() {
        let mut interpreter = NyashInterpreter::new();
        let program = r#"
            static box Main {
                init { result }
                main() {
                    local arr
                    arr = new ArrayBox()
                    arr.push("first")
                    arr.push("second")
                    arr.push("third")
                    me.result = arr.length()
                    return me.result
                }
            }
        "#;

        let ast = NyashParser::parse_from_string(program).unwrap();
        let result = interpreter.execute(ast).unwrap();
        let int_result = result.as_any().downcast_ref::<IntegerBox>().unwrap();
        assert_eq!(int_result.value, 3); // 3要素が正しく保持されるべき
    }
}
