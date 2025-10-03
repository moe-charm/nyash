//! Boxファクトリレジストリ - Box生成の中央管理
//!
//! プラグインBoxを中心にBox生成を管理する（Plugin-First）。
//! 旧ビルトイン経路は互換目的のAPIとして最小限に保持（テスト用途）。

use crate::box_trait::NyashBox;
use crate::runtime::plugin_config::PluginConfig;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Box生成方法を表す列挙型
pub enum BoxProvider {
    /// 互換用ビルトイン実装（Rust関数、現在は原則未使用）
    Builtin(BoxConstructor),

    /// プラグイン実装（プラグイン名を保持）
    Plugin(String),
}

/// 互換用ビルトインBoxのコンストラクタ関数型
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

    /// 互換用ビルトインBoxを登録（通常は使用しない）
    pub fn register_builtin(&self, name: &str, constructor: BoxConstructor) {
        let mut providers = self.providers.write().unwrap();
        providers.insert(name.to_string(), BoxProvider::Builtin(constructor));
    }

    /// プラグイン設定を適用（既存のビルトインを上書き）
    pub fn apply_plugin_config(&self, config: &PluginConfig) {
        let mut providers = self.providers.write().unwrap();

        for (box_name, plugin_name) in &config.plugins {
            providers.insert(box_name.clone(), BoxProvider::Plugin(plugin_name.clone()));
        }
    }

    /// Box名からプロバイダーを取得
    pub fn get_provider(&self, name: &str) -> Option<BoxProvider> {
        let providers = self.providers.read().unwrap();
        providers.get(name).cloned()
    }

    /// Boxを生成
    pub fn create_box(
        &self,
        name: &str,
        args: &[Box<dyn NyashBox>],
    ) -> Result<Box<dyn NyashBox>, String> {
        let provider = self
            .get_provider(name)
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

    /// プラグインBoxを生成（unified facade→v2）
    fn create_plugin_box(
        &self,
        plugin_name: &str,
        box_name: &str,
        args: &[Box<dyn NyashBox>],
    ) -> Result<Box<dyn NyashBox>, String> {
        use crate::runtime::get_global_plugin_host;
        let host = get_global_plugin_host();
        let host = host.read().unwrap();
        if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
            eprintln!(
                "[BoxFactoryRegistry] create_plugin_box: plugin={} box_type={}",
                plugin_name, box_name
            );
        }
        host.create_box(box_name, args).map_err(|e| {
            format!(
                "Failed to create {} from plugin {}: {:?}",
                box_name, plugin_name, e
            )
        })
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

// ---- Core builtins (minimal, plugin-overrideable) ----
fn register_core_builtins(reg: &BoxFactoryRegistry) {
    // TimerBox: provide builtin constructor so `new TimerBox()` works without plugins.
    // Plugin 設定が適用されると上書きされる（Plugin-First 原則を維持）。
    reg.register_builtin("TimerBox", |_args: &[Box<dyn NyashBox>]| {
        Ok(Box::new(crate::boxes::time_box::TimerBox::new()))
    });
}

static GLOBAL_REGISTRY: Lazy<Arc<BoxFactoryRegistry>> = Lazy::new(|| {
    let reg = Arc::new(BoxFactoryRegistry::new());
    // Register minimal core builtins (can be overridden by plugins later)
    register_core_builtins(&reg);
    reg
});

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
        config
            .plugins
            .insert("FileBox".to_string(), "filebox".to_string());
        registry.apply_plugin_config(&config);

        // プロバイダーがプラグインに変わっているか確認
        match registry.get_provider("FileBox").unwrap() {
            BoxProvider::Plugin(name) => assert_eq!(name, "filebox"),
            _ => panic!("Expected plugin provider"),
        }
    }
}
