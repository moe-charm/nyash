//! プラグイン設定（nyash.toml）の読み込み
//! 
//! シンプルな実装から始める - 必要最小限の機能のみ

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// プラグイン設定
#[derive(Debug, Default)]
pub struct PluginConfig {
    /// Box名 → プラグイン名のマッピング
    /// 例: "FileBox" => "filebox"
    pub plugins: HashMap<String, String>,
}

impl PluginConfig {
    /// nyash.tomlを読み込む
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read nyash.toml: {}", e))?;
        
        Self::parse(&content)
    }
    
    /// 設定文字列をパース（シンプル版）
    /// 
    /// 対応フォーマット:
    /// ```toml
    /// [plugins]
    /// FileBox = "filebox"
    /// StringBox = "mystring"
    /// ```
    pub fn parse(content: &str) -> Result<Self, String> {
        let mut config = PluginConfig::default();
        let mut in_plugins_section = false;
        
        for line in content.lines() {
            let line = line.trim();
            
            // 空行やコメントはスキップ
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // セクション検出
            if line == "[plugins]" {
                in_plugins_section = true;
                continue;
            } else if line.starts_with('[') {
                in_plugins_section = false;
                continue;
            }
            
            // plugins セクション内の設定を読む
            if in_plugins_section {
                if let Some((key, value)) = line.split_once('=') {
                    let box_name = key.trim().to_string();
                    let plugin_name = value.trim().trim_matches('"').to_string();
                    config.plugins.insert(box_name, plugin_name);
                }
            }
        }
        
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_config() {
        let toml = r#"
[plugins]
FileBox = "filebox"
StringBox = "mystring"

[other]
something = "else"
"#;
        
        let config = PluginConfig::parse(toml).unwrap();
        assert_eq!(config.plugins.get("FileBox"), Some(&"filebox".to_string()));
        assert_eq!(config.plugins.get("StringBox"), Some(&"mystring".to_string()));
        assert_eq!(config.plugins.len(), 2);
    }
    
    #[test]
    fn test_parse_empty_config() {
        let toml = "";
        let config = PluginConfig::parse(toml).unwrap();
        assert!(config.plugins.is_empty());
    }
    
    #[test]
    fn test_parse_with_comments() {
        let toml = r#"
# This is a comment
[plugins]
# FileBox uses the plugin version
FileBox = "filebox"
# StringBox = "disabled"  # This is commented out
"#;
        
        let config = PluginConfig::parse(toml).unwrap();
        assert_eq!(config.plugins.get("FileBox"), Some(&"filebox".to_string()));
        assert_eq!(config.plugins.get("StringBox"), None);
        assert_eq!(config.plugins.len(), 1);
    }
}