/*!
 * WebDisplayBox - ブラウザHTML要素表示制御Box
 * 
 * WebAssembly環境でHTML要素への直接出力・スタイル制御
 * プレイグラウンドの出力パネル等を完全制御
 */

use crate::box_trait::{NyashBox, StringBox, BoolBox, BoxCore, BoxBase};
use std::any::Any;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use web_sys::{Element, HtmlElement};

// 🌐 Browser HTML element display control Box
#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub struct WebDisplayBox {
    base: BoxBase,
    target_element_id: String,
}

#[cfg(target_arch = "wasm32")]
impl WebDisplayBox {
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
    
    /// テキストを追加出力
    pub fn print(&self, message: &str) {
        if let Some(element) = self.get_target_element() {
            let current_content = element.inner_html();
            let new_content = if current_content.is_empty() {
                message.to_string()
            } else {
                format!("{}{}", current_content, message)
            };
            element.set_inner_html(&new_content);
        }
    }
    
    /// テキストを改行付きで追加出力
    pub fn println(&self, message: &str) {
        if let Some(element) = self.get_target_element() {
            let current_content = element.inner_html();
            let new_content = if current_content.is_empty() {
                message.to_string()
            } else {
                format!("{}<br>{}", current_content, message)
            };
            element.set_inner_html(&new_content);
        }
    }
    
    /// HTMLコンテンツを完全置換
    pub fn set_html(&self, html_content: &str) {
        if let Some(element) = self.get_target_element() {
            element.set_inner_html(html_content);
        }
    }
    
    /// HTMLコンテンツを追加
    pub fn append_html(&self, html_content: &str) {
        if let Some(element) = self.get_target_element() {
            let current_content = element.inner_html();
            let new_content = format!("{}{}", current_content, html_content);
            element.set_inner_html(&new_content);
        }
    }
    
    /// CSSスタイルを設定
    pub fn set_css(&self, property: &str, value: &str) {
        if let Some(element) = self.get_target_element() {
            if let Some(html_element) = element.dyn_ref::<HtmlElement>() {
                // HTMLElement の style プロパティへアクセス
                let _ = html_element.style().set_property(property, value);
            }
        }
    }
    
    /// CSSクラスを追加
    pub fn add_class(&self, class_name: &str) {
        if let Some(element) = self.get_target_element() {
            let _ = element.class_list().add_1(class_name);
        }
    }
    
    /// CSSクラスを削除  
    pub fn remove_class(&self, class_name: &str) {
        if let Some(element) = self.get_target_element() {
            let _ = element.class_list().remove_1(class_name);
        }
    }
    
    /// 内容をクリア
    pub fn clear(&self) {
        if let Some(element) = self.get_target_element() {
            element.set_inner_html("");
        }
    }
    
    /// 要素を表示
    pub fn show(&self) {
        self.set_css("display", "block");
    }
    
    /// 要素を非表示
    pub fn hide(&self) {
        self.set_css("display", "none");
    }
    
    /// スクロールを最下部に移動
    pub fn scroll_to_bottom(&self) {
        if let Some(element) = self.get_target_element() {
            if let Some(html_element) = element.dyn_ref::<HtmlElement>() {
                html_element.set_scroll_top(html_element.scroll_height());
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl BoxCore for WebDisplayBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "WebDisplayBox({})", self.target_element_id)
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(target_arch = "wasm32")]
impl NyashBox for WebDisplayBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }

    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("WebDisplayBox({})", self.target_element_id))
    }


    fn type_name(&self) -> &'static str {
        "WebDisplayBox"
    }
    

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_display) = other.as_any().downcast_ref::<WebDisplayBox>() {
            BoolBox::new(self.base.id == other_display.base.id)
        } else {
            BoolBox::new(false)
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl std::fmt::Display for WebDisplayBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}