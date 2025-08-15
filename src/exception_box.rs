/*!
 * Exception Boxes for Nyash try/catch/throw system
 * 
 * Everything is Box哲学に基づく例外システム
 */

use crate::box_trait::{NyashBox, StringBox, BoolBox, BoxCore, BoxBase};
use std::any::Any;
use std::collections::HashMap;

/// 基底例外Box
#[derive(Debug, Clone)]
pub struct ErrorBox {
    pub message: String,
    pub stack_trace: Vec<String>,
    pub cause: Option<Box<ErrorBox>>,
    base: BoxBase,
}

impl ErrorBox {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            stack_trace: Vec::new(),
            cause: None,
            base: BoxBase::new(),
        }
    }
    
    pub fn with_cause(message: &str, cause: ErrorBox) -> Self {
        Self {
            message: message.to_string(),
            stack_trace: Vec::new(),
            cause: Some(Box::new(cause)),
            base: BoxBase::new(),
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
    
    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl BoxCore for ErrorBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ErrorBox({})", self.message)
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl std::fmt::Display for ErrorBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
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