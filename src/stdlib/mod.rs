/*!
 * Nyash Built-in Standard Library
 * 
 * 超簡単実装：ハードコード組み込み型標準ライブラリ
 * nyash.linkなしで動作する基本的な標準関数群
 */

use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox};
use crate::boxes::{ArrayBox, ConsoleBox};
use crate::interpreter::RuntimeError;
use std::collections::HashMap;

/// 組み込み標準ライブラリ
pub struct BuiltinStdlib {
    pub namespaces: HashMap<String, BuiltinNamespace>,
}

/// 組み込み名前空間
pub struct BuiltinNamespace {
    pub name: String,
    pub static_boxes: HashMap<String, BuiltinStaticBox>,
}

/// 組み込み静的Box
pub struct BuiltinStaticBox {
    pub name: String,
    pub methods: HashMap<String, BuiltinMethod>,
}

/// 組み込みメソッド関数型
pub type BuiltinMethod = fn(&[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, RuntimeError>;

impl BuiltinStdlib {
    /// 新しい組み込み標準ライブラリを作成
    pub fn new() -> Self {
        let mut stdlib = BuiltinStdlib {
            namespaces: HashMap::new(),
        };
        
        // nyashstd名前空間登録
        stdlib.register_nyashstd();
        
        stdlib
    }
    
    /// nyashstd名前空間を登録
    fn register_nyashstd(&mut self) {
        let mut nyashstd = BuiltinNamespace {
            name: "nyashstd".to_string(),
            static_boxes: HashMap::new(),
        };
        
        // string static box
        nyashstd.static_boxes.insert("string".to_string(), Self::create_string_box());
        
        // integer static box
        nyashstd.static_boxes.insert("integer".to_string(), Self::create_integer_box());
        
        // bool static box
        nyashstd.static_boxes.insert("bool".to_string(), Self::create_bool_box());
        
        // array static box
        nyashstd.static_boxes.insert("array".to_string(), Self::create_array_box());
        
        // console static box
        nyashstd.static_boxes.insert("console".to_string(), Self::create_console_box());
        
        self.namespaces.insert("nyashstd".to_string(), nyashstd);
    }
    
    /// string static boxを作成
    fn create_string_box() -> BuiltinStaticBox {
        let mut string_box = BuiltinStaticBox {
            name: "string".to_string(),
            methods: HashMap::new(),
        };
        
        // string.create(text) -> StringBox
        string_box.methods.insert("create".to_string(), |args| {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidOperation {
                    message: "string.create() takes exactly 1 argument".to_string()
                });
            }
            
            // StringBoxにダウンキャスト
            if let Some(string_arg) = args[0].as_any().downcast_ref::<StringBox>() {
                let result = StringBox::new(&string_arg.value);
                Ok(Box::new(result))
            } else {
                Err(RuntimeError::TypeError {
                    message: format!("string.create() expects string argument, got {:?}", args[0].type_name())
                })
            }
        });
        
        // string.upper(str) -> String
        string_box.methods.insert("upper".to_string(), |args| {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidOperation {
                    message: "string.upper() takes exactly 1 argument".to_string()
                });
            }
            
            // StringBoxにダウンキャスト
            if let Some(string_arg) = args[0].as_any().downcast_ref::<StringBox>() {
                let result = StringBox::new(&string_arg.value.to_uppercase());
                Ok(Box::new(result))
            } else {
                Err(RuntimeError::TypeError {
                    message: format!("string.upper() expects string argument, got {:?}", args[0].type_name())
                })
            }
        });
        
        string_box
    }
    
    /// integer static boxを作成
    fn create_integer_box() -> BuiltinStaticBox {
        let mut integer_box = BuiltinStaticBox {
            name: "integer".to_string(),
            methods: HashMap::new(),
        };
        
        // integer.create(value) -> IntegerBox
        integer_box.methods.insert("create".to_string(), |args| {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidOperation {
                    message: "integer.create() takes exactly 1 argument".to_string()
                });
            }
            
            // IntegerBoxにダウンキャスト
            if let Some(int_arg) = args[0].as_any().downcast_ref::<IntegerBox>() {
                let result = IntegerBox::new(int_arg.value);
                Ok(Box::new(result))
            } else {
                Err(RuntimeError::TypeError {
                    message: format!("integer.create() expects integer argument, got {:?}", args[0].type_name())
                })
            }
        });
        
        integer_box
    }
    
    /// bool static boxを作成
    fn create_bool_box() -> BuiltinStaticBox {
        let mut bool_box = BuiltinStaticBox {
            name: "bool".to_string(),
            methods: HashMap::new(),
        };
        
        // bool.create(value) -> BoolBox
        bool_box.methods.insert("create".to_string(), |args| {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidOperation {
                    message: "bool.create() takes exactly 1 argument".to_string()
                });
            }
            
            // BoolBoxにダウンキャスト
            if let Some(bool_arg) = args[0].as_any().downcast_ref::<BoolBox>() {
                let result = BoolBox::new(bool_arg.value);
                Ok(Box::new(result))
            } else {
                Err(RuntimeError::TypeError {
                    message: format!("bool.create() expects bool argument, got {:?}", args[0].type_name())
                })
            }
        });
        
        bool_box
    }
    
    /// array static boxを作成
    fn create_array_box() -> BuiltinStaticBox {
        let mut array_box = BuiltinStaticBox {
            name: "array".to_string(),
            methods: HashMap::new(),
        };
        
        // array.create() -> ArrayBox (引数なしで空配列作成)
        array_box.methods.insert("create".to_string(), |args| {
            if !args.is_empty() {
                return Err(RuntimeError::InvalidOperation {
                    message: "array.create() takes no arguments".to_string()
                });
            }
            
            let result = ArrayBox::new();
            Ok(Box::new(result))
        });
        
        array_box
    }
    
    /// console static boxを作成
    fn create_console_box() -> BuiltinStaticBox {
        let mut console_box = BuiltinStaticBox {
            name: "console".to_string(),
            methods: HashMap::new(),
        };
        
        // console.log(message) -> void
        console_box.methods.insert("log".to_string(), |args| {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidOperation {
                    message: "console.log() takes exactly 1 argument".to_string()
                });
            }
            
            // 任意のBoxを文字列として出力
            let message = args[0].to_string_box().value;
            println!("{}", message);
            
            // VoidBoxを返す
            use crate::box_trait::VoidBox;
            Ok(Box::new(VoidBox::new()))
        });
        
        console_box
    }
}