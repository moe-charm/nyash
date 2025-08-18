//! Boxファクトリレジストリ - Box生成の中央管理
//! 
//! ビルトインBoxとプラグインBoxを統一的に管理し、
//! 透過的な置き換えを実現する

use crate::box_trait::NyashBox;
use crate::runtime::plugin_config::PluginConfig;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Box生成方法を表す列挙型
pub enum BoxProvider {
    /// ビルトイン実装（Rust関数）
    Builtin(BoxConstructor),
    
    /// プラグイン実装（プラグイン名を保持）
    Plugin(String),
}

/// ビルトインBoxのコンストラクタ関数型
pub type BoxConstructor = fn(&[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, String>;

/// Boxファクトリレジストリ
pub struct BoxFactoryRegistry {
    /// Box名 → プロバイダーのマッピング
    providers: RwLock<HashMap<String, BoxProvider>>,
}

impl BoxFactoryRegistry {
    /// 新しいレジストリを作成
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
        }
    }
    
    /// ビルトインBoxを登録
    pub fn register_builtin(&self, name: &str, constructor: BoxConstructor) {
        let mut providers = self.providers.write().unwrap();
        providers.insert(name.to_string(), BoxProvider::Builtin(constructor));
    }
    
    /// プラグイン設定を適用（既存のビルトインを上書き）
    pub fn apply_plugin_config(&self, config: &PluginConfig) {
        let mut providers = self.providers.write().unwrap();
        
        for (box_name, plugin_name) in &config.plugins {
            providers.insert(
                box_name.clone(),
                BoxProvider::Plugin(plugin_name.clone())
            );
        }
    }
    
    /// Box名からプロバイダーを取得
    pub fn get_provider(&self, name: &str) -> Option<BoxProvider> {
        let providers = self.providers.read().unwrap();
        providers.get(name).cloned()
    }
    
    /// Boxを生成
    pub fn create_box(&self, name: &str, args: &[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, String> {
        let provider = self.get_provider(name)
            .ok_or_else(|| format!("Unknown Box type: {}", name))?;
        
        match provider {
            BoxProvider::Builtin(constructor) => {
                // ビルトイン実装を直接呼び出し
                constructor(args)
            }
            BoxProvider::Plugin(plugin_name) => {
                // プラグインローダーと連携してプラグインBoxを生成
                self.create_plugin_box(&plugin_name, name, args)
            }
        }
    }
    
    /// プラグインBoxを生成（v2実装）
    fn create_plugin_box(&self, plugin_name: &str, box_name: &str, args: &[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, String> {
        use crate::runtime::get_global_loader_v2;
        
        // v2ローダーを取得
        let loader = get_global_loader_v2();
        let loader = loader.read().unwrap();
        
        // プラグインからBoxを生成
        loader.create_box(box_name, args)
            .map_err(|e| format!("Failed to create {} from plugin {}: {:?}", box_name, plugin_name, e))
    }
}

impl Clone for BoxProvider {
    fn clone(&self) -> Self {
        match self {
            BoxProvider::Builtin(f) => BoxProvider::Builtin(*f),
            BoxProvider::Plugin(name) => BoxProvider::Plugin(name.clone()),
        }
    }
}

// グローバルレジストリインスタンス
use once_cell::sync::Lazy;

static GLOBAL_REGISTRY: Lazy<Arc<BoxFactoryRegistry>> = 
    Lazy::new(|| Arc::new(BoxFactoryRegistry::new()));

/// グローバルレジストリを取得
pub fn get_global_registry() -> Arc<BoxFactoryRegistry> {
    GLOBAL_REGISTRY.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::box_trait::StringBox;
    
    fn test_string_constructor(args: &[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, String> {
        if args.is_empty() {
            Ok(Box::new(StringBox::new("")))
        } else {
            Ok(Box::new(StringBox::new(&args[0].to_string_box().value)))
        }
    }
    
    #[test]
    fn test_builtin_registration() {
        let registry = BoxFactoryRegistry::new();
        registry.register_builtin("StringBox", test_string_constructor);
        
        let result = registry.create_box("StringBox", &[]).unwrap();
        assert_eq!(result.to_string_box().value, "");
    }
    
    #[test]
    fn test_plugin_override() {
        let registry = BoxFactoryRegistry::new();
        registry.register_builtin("FileBox", test_string_constructor);
        
        // プラグイン設定で上書き
        let mut config = PluginConfig::default();
        config.plugins.insert("FileBox".to_string(), "filebox".to_string());
        registry.apply_plugin_config(&config);
        
        // プロバイダーがプラグインに変わっているか確認
        match registry.get_provider("FileBox").unwrap() {
            BoxProvider::Plugin(name) => assert_eq!(name, "filebox"),
            _ => panic!("Expected plugin provider"),
        }
    }
}