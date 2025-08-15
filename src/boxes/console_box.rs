/*! 📟 ConsoleBox - コンソール出力Box
 * 
 * ## 📝 概要
 * Webブラウザのコンソール機能を統合したBox。
 * WASM環境ではブラウザコンソール、ネイティブ環境では標準出力。
 * 
 * ## 🛠️ 利用可能メソッド
 * - `log(message)` - 通常のメッセージ出力
 * - `warn(message)` - 警告メッセージ出力
 * - `error(message)` - エラーメッセージ出力
 * - `clear()` - コンソール画面クリア
 * 
 * ## 💡 使用例
 * ```nyash
 * local console
 * console = new ConsoleBox()
 * 
 * console.log("Hello, Nyash!")           // 通常ログ
 * console.warn("This is a warning")      // 警告
 * console.error("Something went wrong")  // エラー
 * console.clear()                        // クリア
 * 
 * // デバッグ用途
 * local value
 * value = 42
 * console.log("Debug: value = " + value.toString())
 * ```
 * 
 * ## 🌐 環境別動作
 * - **WASM環境**: ブラウザの開発者ツールコンソールに出力
 * - **ネイティブ環境**: ターミナル標準出力にプレフィックス付きで出力
 * 
 * ## 🔍 デバッグ活用
 * ```nyash
 * // エラーハンドリング
 * if (error_condition) {
 *     console.error("Critical error occurred!")
 *     return null
 * }
 * 
 * // 実行トレース
 * console.log("Function start")
 * // 処理...
 * console.log("Function end")
 * ```
 */

use crate::box_trait::{NyashBox, StringBox, BoolBox, BoxCore, BoxBase};
use std::any::Any;
use std::fmt::Display;

// 🌐 Browser console access Box
#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub struct ConsoleBox {
    base: BoxBase,
}

#[cfg(target_arch = "wasm32")]
impl ConsoleBox {
    pub fn new() -> Self {
        Self { base: BoxBase::new() }
    }
    
    /// Log messages to browser console
    pub fn log(&self, message: &str) {
        web_sys::console::log_1(&message.into());
    }
    
    /// Log warning to browser console
    pub fn warn(&self, message: &str) {
        web_sys::console::warn_1(&message.into());
    }
    
    /// Log error to browser console  
    pub fn error(&self, message: &str) {
        web_sys::console::error_1(&message.into());
    }
    
    /// Clear browser console
    pub fn clear(&self) {
        web_sys::console::clear();
    }
}

#[cfg(target_arch = "wasm32")]
impl BoxCore for ConsoleBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[ConsoleBox - Browser Console Interface]")
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(target_arch = "wasm32")]
impl NyashBox for ConsoleBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new("[ConsoleBox - Browser Console Interface]")
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        BoolBox::new(other.as_any().is::<ConsoleBox>())
    }
    
    fn type_name(&self) -> &'static str {
        "ConsoleBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
    
}

// Non-WASM版 - モックアップ実装  
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct ConsoleBox {
    base: BoxBase,
}

#[cfg(not(target_arch = "wasm32"))]
impl ConsoleBox {
    pub fn new() -> Self {
        Self { base: BoxBase::new() }
    }
    
    /// Mock log method for non-WASM environments
    pub fn log(&self, message: &str) {
        println!("[Console LOG] {}", message);
    }
    
    pub fn warn(&self, message: &str) {
        println!("[Console WARN] {}", message);
    }
    
    pub fn error(&self, message: &str) {
        println!("[Console ERROR] {}", message);
    }
    
    pub fn clear(&self) {
        println!("[Console CLEAR]");
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl BoxCore for ConsoleBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[ConsoleBox - Mock Implementation]")
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl NyashBox for ConsoleBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new("[ConsoleBox - Mock Implementation]")
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        BoolBox::new(other.as_any().is::<ConsoleBox>())
    }
    
    fn type_name(&self) -> &'static str {
        "ConsoleBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
    
}


// Display implementations for both WASM and non-WASM versions
#[cfg(target_arch = "wasm32")]
impl Display for ConsoleBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Display for ConsoleBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}
