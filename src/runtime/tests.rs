//! プラグインシステム統合テスト
//! 
//! プラグインBoxの透過的切り替えをテスト

#[cfg(test)]
mod tests {
    use super::super::{PluginConfig, BoxFactoryRegistry};
    use crate::runtime::box_registry::BoxProvider;
    use crate::box_trait::{NyashBox, StringBox};
    use crate::bid::{BidHandle, BoxTypeId};
    
    fn dummy_filebox_constructor(args: &[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, String> {
        // ダミーFileBox作成（ビルトイン版シミュレーション）
        if args.is_empty() {
            Ok(Box::new(StringBox::new("DummyFileBox")))
        } else {
            Ok(Box::new(StringBox::new(&format!("DummyFileBox({})", args[0].to_string_box().value))))
        }
    }
    
    #[test]
    fn test_plugin_config_parsing() {
        let toml = r#"
[plugins]
FileBox = "filebox"
StringBox = "custom_string"
"#;
        
        let config = PluginConfig::parse(toml).unwrap();
        assert_eq!(config.plugins.get("FileBox"), Some(&"filebox".to_string()));
        assert_eq!(config.plugins.get("StringBox"), Some(&"custom_string".to_string()));
    }
    
    #[test]
    fn test_box_registry_builtin() {
        let registry = BoxFactoryRegistry::new();
        registry.register_builtin("FileBox", dummy_filebox_constructor);
        
        let result = registry.create_box("FileBox", &[]).unwrap();
        assert_eq!(result.to_string_box().value, "DummyFileBox");
    }
    
    #[test]
    fn test_box_registry_plugin_override() {
        let registry = BoxFactoryRegistry::new();
        registry.register_builtin("FileBox", dummy_filebox_constructor);
        
        // プラグイン設定でビルトインを上書き
        let mut config = PluginConfig::default();
        config.plugins.insert("FileBox".to_string(), "filebox".to_string());
        registry.apply_plugin_config(&config);

        // 生成までは行わず、プロバイダーがプラグインに切り替わったことを確認
        match registry.get_provider("FileBox").unwrap() {
            BoxProvider::Plugin(name) => assert_eq!(name, "filebox"),
            _ => panic!("Expected plugin provider for FileBox"),
        }
    }
    
    // TODO: PluginBox型が削除されたためこのテストはコメントアウト
    // #[test]
    // fn test_plugin_box_creation() {
    //     let handle = BidHandle::new(BoxTypeId::FileBox as u32, 123);
    //     let plugin_box = PluginBox::new("filebox".to_string(), handle);
    //     
    //     assert_eq!(plugin_box.plugin_name(), "filebox");
    //     assert_eq!(plugin_box.handle().type_id, BoxTypeId::FileBox as u32);
    //     assert_eq!(plugin_box.handle().instance_id, 123);
    // }
    
    // 旧PluginBox直接生成テストは削除（v2統合により不要）
    
    #[test]
    fn test_transparent_box_switching() {
        let registry = BoxFactoryRegistry::new();
        
        // 1. ビルトイン版を登録
        registry.register_builtin("FileBox", dummy_filebox_constructor);
        
        // 2. 現在のプロバイダーはビルトイン
        match registry.get_provider("FileBox").unwrap() {
            BoxProvider::Builtin(_) => {}
            _ => panic!("Expected builtin provider before plugin override"),
        }
        
        // 3. プラグイン設定を適用
        let mut config = PluginConfig::default();
        config.plugins.insert("FileBox".to_string(), "filebox".to_string());
        registry.apply_plugin_config(&config);
        
        // 4. プロバイダーがプラグインに切り替わっている
        match registry.get_provider("FileBox").unwrap() {
            BoxProvider::Plugin(name) => assert_eq!(name, "filebox"),
            _ => panic!("Expected plugin provider after override"),
        }
    }
    
    #[test]
    fn test_multiple_plugin_types() {
        let mut config = PluginConfig::default();
        config.plugins.insert("FileBox".to_string(), "filebox".to_string());
        config.plugins.insert("StringBox".to_string(), "custom_string".to_string());
        config.plugins.insert("MathBox".to_string(), "advanced_math".to_string());
        
        assert_eq!(config.plugins.len(), 3);
        assert_eq!(config.plugins.get("FileBox"), Some(&"filebox".to_string()));
        assert_eq!(config.plugins.get("StringBox"), Some(&"custom_string".to_string()));
        assert_eq!(config.plugins.get("MathBox"), Some(&"advanced_math".to_string()));
    }
}
