//! プラグインシステム統合テスト
//! 
//! プラグインBoxの透過的切り替えをテスト

#[cfg(test)]
mod tests {
    use super::super::{PluginConfig, BoxFactoryRegistry, PluginBox};
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
        
        // プラグインBoxが生成されることを確認
        let result = registry.create_box("FileBox", &[]).unwrap();
        
        // PluginBoxかどうかを確認
        assert!(result.as_any().downcast_ref::<PluginBox>().is_some());
        let plugin_box = result.as_any().downcast_ref::<PluginBox>().unwrap();
        assert_eq!(plugin_box.plugin_name(), "filebox");
    }
    
    #[test]
    fn test_plugin_box_creation() {
        let handle = BidHandle::new(BoxTypeId::FileBox as u32, 123);
        let plugin_box = PluginBox::new("filebox".to_string(), handle);
        
        assert_eq!(plugin_box.plugin_name(), "filebox");
        assert_eq!(plugin_box.handle().type_id, BoxTypeId::FileBox as u32);
        assert_eq!(plugin_box.handle().instance_id, 123);
    }
    
    #[test]
    fn test_plugin_box_equality() {
        let handle1 = BidHandle::new(BoxTypeId::FileBox as u32, 123);
        let handle2 = BidHandle::new(BoxTypeId::FileBox as u32, 456);
        
        let box1 = PluginBox::new("filebox".to_string(), handle1);
        let box2 = PluginBox::new("filebox".to_string(), handle1);
        let box3 = PluginBox::new("filebox".to_string(), handle2);
        let box4 = PluginBox::new("otherbox".to_string(), handle1);
        
        // 同じプラグイン・同じハンドル
        assert!(box1.equals(&box2).value);
        
        // 異なるハンドル
        assert!(!box1.equals(&box3).value);
        
        // 異なるプラグイン
        assert!(!box1.equals(&box4).value);
    }
    
    #[test]
    fn test_plugin_box_type_name() {
        let handle = BidHandle::new(BoxTypeId::FileBox as u32, 123);
        let plugin_box = PluginBox::new("filebox".to_string(), handle);
        
        // 現在の実装では"PluginBox"を返す
        assert_eq!(plugin_box.type_name(), "PluginBox");
    }
    
    #[test]
    fn test_plugin_box_to_string() {
        let handle = BidHandle::new(BoxTypeId::FileBox as u32, 123);
        let plugin_box = PluginBox::new("filebox".to_string(), handle);
        
        let string_result = plugin_box.to_string_box();
        
        // FFI呼び出しが失敗した場合のフォールバック文字列をチェック
        assert!(string_result.value.contains("PluginBox"));
        assert!(string_result.value.contains("filebox"));
    }
    
    #[test]
    fn test_transparent_box_switching() {
        let registry = BoxFactoryRegistry::new();
        
        // 1. ビルトイン版を登録
        registry.register_builtin("FileBox", dummy_filebox_constructor);
        
        // 2. ビルトイン版で作成
        let builtin_box = registry.create_box("FileBox", &[]).unwrap();
        assert_eq!(builtin_box.to_string_box().value, "DummyFileBox");
        
        // 3. プラグイン設定を適用
        let mut config = PluginConfig::default();
        config.plugins.insert("FileBox".to_string(), "filebox".to_string());
        registry.apply_plugin_config(&config);
        
        // 4. 同じコードでプラグイン版が作成される
        let plugin_box = registry.create_box("FileBox", &[]).unwrap();
        
        // 透過的にプラグイン版に切り替わっている
        assert!(plugin_box.as_any().downcast_ref::<PluginBox>().is_some());
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