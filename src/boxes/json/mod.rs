//! JSONBox 📋 - JSON解析・生成
// Nyashの箱システムによるJSON解析・生成を提供します。
// 参考: 既存Boxの設計思想

use crate::box_trait::{NyashBox, StringBox, BoolBox, IntegerBox};
use crate::boxes::array::ArrayBox;
use crate::boxes::map_box::MapBox;
use std::any::Any;
use std::sync::{Arc, Mutex};
use serde_json::{Value, Error};

#[derive(Debug, Clone)]
pub struct JSONBox {
    value: Arc<Mutex<Value>>,
    id: u64,
}

impl JSONBox {
    pub fn from_str(s: &str) -> Result<Self, Error> {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        let value = serde_json::from_str(s)?;
        Ok(JSONBox { 
            value: Arc::new(Mutex::new(value)), 
            id 
        })
    }
    
    pub fn new(value: Value) -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        JSONBox { 
            value: Arc::new(Mutex::new(value)), 
            id 
        }
    }
    
    pub fn to_string(&self) -> String {
        let value = self.value.lock().unwrap();
        value.to_string()
    }
    
    /// JSONパース
    pub fn parse(data: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let json_str = data.to_string_box().value;
        match JSONBox::from_str(&json_str) {
            Ok(json_box) => Box::new(json_box),
            Err(e) => Box::new(StringBox::new(&format!("Error parsing JSON: {}", e))),
        }
    }
    
    /// JSON文字列化
    pub fn stringify(&self) -> Box<dyn NyashBox> {
        Box::new(StringBox::new(&self.to_string()))
    }
    
    /// 値取得
    pub fn get(&self, key: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let key_str = key.to_string_box().value;
        let value = self.value.lock().unwrap();
        
        if let Some(obj) = value.as_object() {
            if let Some(val) = obj.get(&key_str) {
                json_value_to_nyash_box(val)
            } else {
                Box::new(crate::boxes::null_box::NullBox::new())
            }
        } else if let Some(arr) = value.as_array() {
            if let Ok(index) = key_str.parse::<usize>() {
                if let Some(val) = arr.get(index) {
                    json_value_to_nyash_box(val)
                } else {
                    Box::new(crate::boxes::null_box::NullBox::new())
                }
            } else {
                Box::new(crate::boxes::null_box::NullBox::new())
            }
        } else {
            Box::new(crate::boxes::null_box::NullBox::new())
        }
    }
    
    /// 値設定
    pub fn set(&self, key: Box<dyn NyashBox>, new_value: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let key_str = key.to_string_box().value;
        let mut value = self.value.lock().unwrap();
        
        let json_value = nyash_box_to_json_value(new_value);
        
        if let Some(obj) = value.as_object_mut() {
            obj.insert(key_str, json_value);
            Box::new(StringBox::new("ok"))
        } else {
            Box::new(StringBox::new("Error: JSONBox is not an object"))
        }
    }
    
    /// キー存在チェック
    pub fn has(&self, key: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let key_str = key.to_string_box().value;
        let value = self.value.lock().unwrap();
        
        if let Some(obj) = value.as_object() {
            Box::new(BoolBox::new(obj.contains_key(&key_str)))
        } else {
            Box::new(BoolBox::new(false))
        }
    }
    
    /// すべてのキーを取得
    pub fn keys(&self) -> Box<dyn NyashBox> {
        let value = self.value.lock().unwrap();
        let array = ArrayBox::new();
        
        if let Some(obj) = value.as_object() {
            for key in obj.keys() {
                array.push(Box::new(StringBox::new(key)));
            }
        }
        
        Box::new(array)
    }
}

impl NyashBox for JSONBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    fn to_string_box(&self) -> StringBox {
        let value = self.value.lock().unwrap();
        StringBox::new(value.to_string())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        "JSONBox"
    }

    fn box_id(&self) -> u64 {
        self.id
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_json) = other.as_any().downcast_ref::<JSONBox>() {
            let self_value = self.value.lock().unwrap();
            let other_value = other_json.value.lock().unwrap();
            BoolBox::new(*self_value == *other_value)
        } else {
            BoolBox::new(false)
        }
    }
}

/// JSON Value を NyashBox に変換
fn json_value_to_nyash_box(value: &Value) -> Box<dyn NyashBox> {
    match value {
        Value::Null => Box::new(crate::boxes::null_box::NullBox::new()),
        Value::Bool(b) => Box::new(BoolBox::new(*b)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Box::new(IntegerBox::new(i))
            } else if let Some(f) = n.as_f64() {
                Box::new(crate::box_trait::FloatBox::new(f))
            } else {
                Box::new(StringBox::new(&n.to_string()))
            }
        }
        Value::String(s) => Box::new(StringBox::new(s)),
        Value::Array(arr) => {
            let array_box = ArrayBox::new();
            for item in arr {
                array_box.push(json_value_to_nyash_box(item));
            }
            Box::new(array_box)
        }
        Value::Object(obj) => {
            let map_box = MapBox::new();
            for (key, val) in obj {
                map_box.set(
                    Box::new(StringBox::new(key)),
                    json_value_to_nyash_box(val)
                );
            }
            Box::new(map_box)
        }
    }
}

/// NyashBox を JSON Value に変換
fn nyash_box_to_json_value(value: Box<dyn NyashBox>) -> Value {
    if value.as_any().downcast_ref::<crate::boxes::null_box::NullBox>().is_some() {
        Value::Null
    } else if let Some(bool_box) = value.as_any().downcast_ref::<BoolBox>() {
        Value::Bool(bool_box.value)
    } else if let Some(int_box) = value.as_any().downcast_ref::<IntegerBox>() {
        Value::Number(serde_json::Number::from(int_box.value))
    } else if let Some(float_box) = value.as_any().downcast_ref::<crate::box_trait::FloatBox>() {
        if let Some(n) = serde_json::Number::from_f64(float_box.value) {
            Value::Number(n)
        } else {
            Value::String(float_box.value.to_string())
        }
    } else if let Some(string_box) = value.as_any().downcast_ref::<StringBox>() {
        Value::String(string_box.value.clone())
    } else if let Some(array_box) = value.as_any().downcast_ref::<ArrayBox>() {
        let items = array_box.items.lock().unwrap();
        let arr: Vec<Value> = items.iter()
            .map(|item| nyash_box_to_json_value(item.clone_box()))
            .collect();
        Value::Array(arr)
    } else if let Some(map_box) = value.as_any().downcast_ref::<MapBox>() {
        let map = map_box.map.lock().unwrap();
        let mut obj = serde_json::Map::new();
        for (key, val) in map.iter() {
            obj.insert(key.clone(), nyash_box_to_json_value(val.clone_box()));
        }
        Value::Object(obj)
    } else {
        // その他の型は文字列に変換
        Value::String(value.to_string_box().value)
    }
}
