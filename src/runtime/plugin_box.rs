//! PluginBoxプロキシ - プラグインBoxの統一インターフェース
//! 
//! すべてのプラグインから提供されるBoxを、
//! 通常のNyashBoxとして扱えるようにするプロキシ実装

use crate::box_trait::{NyashBox, StringBox, BoolBox, BoxCore, BoxBase};
use crate::bid::BidHandle;
use std::any::Any;
use std::fmt;

/// プラグインから提供されるBoxのプロキシ
/// 
/// FFI境界を越えてプラグイン内のBoxインスタンスと通信する
#[derive(Debug)]
pub struct PluginBox {
    /// BoxCoreトレイト用の基本情報
    base: BoxBase,
    
    /// プラグイン名（例: "filebox"）
    plugin_name: String,
    
    /// プラグイン内のインスタンスハンドル
    handle: BidHandle,
}

impl PluginBox {
    /// 新しいPluginBoxプロキシを作成
    pub fn new(plugin_name: String, handle: BidHandle) -> Self {
        Self {
            base: BoxBase::new(),
            plugin_name,
            handle,
        }
    }
    
    /// プラグイン名を取得
    pub fn plugin_name(&self) -> &str {
        &self.plugin_name
    }
    
    /// ハンドルを取得
    pub fn handle(&self) -> BidHandle {
        self.handle
    }
    
    /// プラグインメソッド呼び出し（内部使用）
    fn call_plugin_method(&self, method_name: &str, args: &[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, String> {
        use crate::runtime::get_global_loader;
        let loader = get_global_loader();
        loader.invoke_plugin_method(&self.plugin_name, self.handle, method_name, args)
    }
}

impl BoxCore for PluginBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    
    fn fmt_box(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PluginBox({}, handle={:?})", self.plugin_name, self.handle)
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for PluginBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        // TODO: FFI経由でプラグインにcloneを依頼
        // 現在は同じハンドルを持つ新しいプロキシを返す（簡易実装）
        Box::new(PluginBox::new(self.plugin_name.clone(), self.handle))
    }
    
    fn share_box(&self) -> Box<dyn NyashBox> {
        // 現在はclone_boxと同じ実装
        self.clone_box()
    }
    
    fn to_string_box(&self) -> StringBox {
        // FFI経由でプラグインのtoStringメソッド呼び出し
        match self.call_plugin_method("toString", &[]) {
            Ok(result) => result.to_string_box(),
            Err(_) => {
                // エラー時はフォールバック
                StringBox::new(&format!("PluginBox({}, {:?})", self.plugin_name, self.handle))
            }
        }
    }
    
    fn type_name(&self) -> &'static str {
        // TODO: プラグインから実際の型名を取得
        "PluginBox"
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_plugin) = other.as_any().downcast_ref::<PluginBox>() {
            // 同じプラグイン＆同じハンドルなら等しい
            BoolBox::new(
                self.plugin_name == other_plugin.plugin_name &&
                self.handle == other_plugin.handle
            )
        } else {
            BoolBox::new(false)
        }
    }
}

impl fmt::Display for PluginBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_box(f)
    }
}

impl Clone for PluginBox {
    fn clone(&self) -> Self {
        Self {
            base: BoxBase::new(), // 新しいIDを生成
            plugin_name: self.plugin_name.clone(),
            handle: self.handle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bid::BoxTypeId;
    
    #[test]
    fn test_plugin_box_creation() {
        let handle = BidHandle::new(BoxTypeId::FileBox as u32, 123);
        let plugin_box = PluginBox::new("filebox".to_string(), handle);
        
        assert_eq!(plugin_box.plugin_name(), "filebox");
        assert_eq!(plugin_box.handle(), handle);
    }
    
    #[test]
    fn test_plugin_box_equality() {
        let handle1 = BidHandle::new(BoxTypeId::FileBox as u32, 123);
        let handle2 = BidHandle::new(BoxTypeId::FileBox as u32, 456);
        
        let box1 = PluginBox::new("filebox".to_string(), handle1);
        let box2 = PluginBox::new("filebox".to_string(), handle1);
        let box3 = PluginBox::new("filebox".to_string(), handle2);
        
        assert!(box1.equals(&box2).value);
        assert!(!box1.equals(&box3).value);
    }
}