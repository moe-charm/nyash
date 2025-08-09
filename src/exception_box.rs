/*!
 * Exception Boxes for Nyash try/catch/throw system
 * 
 * Everything is Box哲学に基づく例外システム
 */

use crate::box_trait::{NyashBox, StringBox, BoolBox};
use std::any::Any;
use std::collections::HashMap;

/// 基底例外Box
#[derive(Debug, Clone)]
pub struct ErrorBox {
    pub message: String,
    pub stack_trace: Vec<String>,
    pub cause: Option<Box<ErrorBox>>,
    id: u64,
}

impl ErrorBox {
    pub fn new(message: &str) -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        Self {
            message: message.to_string(),
            stack_trace: Vec::new(),
            cause: None,
            id,
        }
    }
    
    pub fn with_cause(message: &str, cause: ErrorBox) -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        Self {
            message: message.to_string(),
            stack_trace: Vec::new(),
            cause: Some(Box::new(cause)),
            id,
        }
    }
    
    pub fn add_stack_frame(&mut self, frame: String) {
        self.stack_trace.push(frame);
    }
    
    pub fn get_full_message(&self) -> String {
        let mut msg = self.message.clone();
        if let Some(ref cause) = self.cause {
            msg.push_str(&format!("\nCaused by: {}", cause.get_full_message()));
        }
        msg
    }
}

impl NyashBox for ErrorBox {
    fn type_name(&self) -> &'static str {
        "ErrorBox"
    }
    
    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("ErrorBox({})", self.message))
    }
    
    fn box_id(&self) -> u64 {
        self.id
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_error) = other.as_any().downcast_ref::<ErrorBox>() {
            BoolBox::new(self.message == other_error.message)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// 例外タイプの判定ユーティリティ
pub fn is_exception_type(exception: &dyn NyashBox, type_name: &str) -> bool {
    match type_name {
        "Error" | "ErrorBox" => exception.as_any().downcast_ref::<ErrorBox>().is_some(),
        _ => false,
    }
}

/// 例外の作成ヘルパー
pub fn create_exception(_type_name: &str, message: &str, _extra_info: &HashMap<String, String>) -> Box<dyn NyashBox> {
    // 現在はErrorBoxのみサポート（シンプル実装）
    Box::new(ErrorBox::new(message))
}