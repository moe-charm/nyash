/*!
 * WebConsoleBox - ブラウザHTML要素コンソール出力Box
 * 
 * WebAssembly環境でHTML要素へのコンソール風出力
 * F12コンソールの代わりに指定要素に出力
 */

use crate::box_trait::{NyashBox, StringBox, BoolBox, BoxCore, BoxBase};
use std::any::Any;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use web_sys::{Element, HtmlElement};

// 🌐 Browser HTML element console output Box
#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub struct WebConsoleBox {
    base: BoxBase,
    target_element_id: String,
}

#[cfg(target_arch = "wasm32")]
impl WebConsoleBox {
    pub fn new(element_id: String) -> Self {
        Self { 
            base: BoxBase::new(),
            target_element_id: element_id,
        }
    }
    
    /// 指定した要素IDのHTML要素を取得
    fn get_target_element(&self) -> Option<Element> {
        let window = web_sys::window()?;
        let document = window.document()?;
        document.get_element_by_id(&self.target_element_id)
    }
    
    /// コンソール出力を追加（改行付き）
    fn append_console_line(&self, message: &str, level: &str) {
        if let Some(element) = self.get_target_element() {
            let timestamp = js_sys::Date::new_0().to_iso_string().as_string().unwrap_or_default();
            let time_part = timestamp.split('T').nth(1).unwrap_or("00:00:00").split('.').nth(0).unwrap_or("00:00:00");
            
            let (level_prefix, color) = match level {
                "log" => ("📝", "white"),
                "warn" => ("⚠️", "yellow"),
                "error" => ("❌", "red"), 
                "info" => ("ℹ️", "cyan"),
                "debug" => ("🔍", "gray"),
                _ => ("📝", "white"),
            };
            
            let formatted_line = format!(
                "<span style='color: {}'>[{}] {} {}</span><br>", 
                color,
                time_part, 
                level_prefix, 
                message
            );
            
            let current_content = element.inner_html();
            let new_content = format!("{}{}", current_content, formatted_line);
            element.set_inner_html(&new_content);
            
            // 自動スクロール
            if let Some(html_element) = element.dyn_ref::<HtmlElement>() {
                html_element.set_scroll_top(html_element.scroll_height());
            }
        }
    }
    
    /// ログメッセージを出力
    pub fn log(&self, message: &str) {
        self.append_console_line(message, "log");
    }
    
    /// 警告メッセージを出力
    pub fn warn(&self, message: &str) {
        self.append_console_line(message, "warn");
    }
    
    /// エラーメッセージを出力
    pub fn error(&self, message: &str) {
        self.append_console_line(message, "error");
    }
    
    /// 情報メッセージを出力
    pub fn info(&self, message: &str) {
        self.append_console_line(message, "info");
    }
    
    /// デバッグメッセージを出力
    pub fn debug(&self, message: &str) {
        self.append_console_line(message, "debug");
    }
    
    /// コンソールをクリア
    pub fn clear(&self) {
        if let Some(element) = self.get_target_element() {
            element.set_inner_html("");
        }
    }
    
    /// 区切り線を追加
    pub fn separator(&self) {
        if let Some(element) = self.get_target_element() {
            let current_content = element.inner_html();
            let separator_line = "<hr style='border: 1px solid #333; margin: 5px 0;'>";
            let new_content = format!("{}{}", current_content, separator_line);
            element.set_inner_html(&new_content);
        }
    }
    
    /// グループ開始（見出し付き）
    pub fn group(&self, title: &str) {
        if let Some(element) = self.get_target_element() {
            let current_content = element.inner_html();
            let group_header = format!(
                "<div style='font-weight: bold; color: #4ecdc4; margin: 10px 0 5px 0;'>📂 {}</div><div style='margin-left: 20px; color: white;'>", 
                title
            );
            let new_content = format!("{}{}", current_content, group_header);
            element.set_inner_html(&new_content);
        }
    }
    
    /// グループ終了
    pub fn group_end(&self) {
        if let Some(element) = self.get_target_element() {
            let current_content = element.inner_html();
            let group_footer = "</div>";
            let new_content = format!("{}{}", current_content, group_footer);
            element.set_inner_html(&new_content);
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl BoxCore for WebConsoleBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "WebConsoleBox({})", self.target_element_id)
    }
    
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(target_arch = "wasm32")]
impl NyashBox for WebConsoleBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("WebConsoleBox({})", self.target_element_id))
    }


    fn type_name(&self) -> &'static str {
        "WebConsoleBox"
    }
    

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_console) = other.as_any().downcast_ref::<WebConsoleBox>() {
            BoolBox::new(self.base.id == other_console.base.id)
        } else {
            BoolBox::new(false)
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl std::fmt::Display for WebConsoleBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}