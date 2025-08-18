use crate::bid::{BidError, BidResult, LoadedPlugin, MethodTypeInfo, ArgTypeMapping};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use once_cell::sync::OnceCell;

/// Registry mapping Box names and type IDs to loaded plugins
pub struct PluginRegistry {
    by_name: HashMap<String, LoadedPlugin>,
    by_type_id: HashMap<u32, String>,
    /// 型情報: Box名 -> メソッド名 -> MethodTypeInfo
    type_info: HashMap<String, HashMap<String, MethodTypeInfo>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { 
            by_name: HashMap::new(), 
            by_type_id: HashMap::new(),
            type_info: HashMap::new(),
        }
    }

    pub fn get_by_name(&self, name: &str) -> Option<&LoadedPlugin> {
        self.by_name.get(name)
    }

    pub fn get_by_type_id(&self, type_id: u32) -> Option<&LoadedPlugin> {
        self.by_type_id.get(&type_id).and_then(|name| self.by_name.get(name))
    }
    
    /// 指定されたBox・メソッドの型情報を取得
    pub fn get_method_type_info(&self, box_name: &str, method_name: &str) -> Option<&MethodTypeInfo> {
        self.type_info.get(box_name)?.get(method_name)
    }

    /// Load plugins based on nyash.toml minimal parsing
    pub fn load_from_config(path: &str) -> BidResult<Self> {
        eprintln!("🔍 DEBUG: load_from_config called with path: {}", path);
        let content = fs::read_to_string(path).map_err(|e| {
            eprintln!("🔍 DEBUG: Failed to read file {}: {}", path, e);
            BidError::PluginError
        })?;

        // Very small parser: look for lines like `FileBox = "nyash-filebox-plugin"`
        let mut mappings: HashMap<String, String> = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() { continue; }
            if let Some((k, v)) = trimmed.split_once('=') {
                let key = k.trim().trim_matches(' ').to_string();
                let val = v.trim().trim_matches('"').to_string();
                if key.chars().all(|c| c.is_alphanumeric() || c == '_' ) && !val.is_empty() {
                    mappings.insert(key, val);
                }
            }
        }

        // Candidate directories
        let mut candidates: Vec<PathBuf> = vec![
            PathBuf::from("./plugins/nyash-filebox-plugin/target/release"),
            PathBuf::from("./plugins/nyash-filebox-plugin/target/debug"),
        ];
        // Also parse plugin_paths.search_paths if present
        if let Some(sp_start) = content.find("search_paths") {
            if let Some(open) = content[sp_start..].find('[') {
                if let Some(close) = content[sp_start + open..].find(']') {
                    let list = &content[sp_start + open + 1.. sp_start + open + close];
                    for item in list.split(',') {
                        let p = item.trim().trim_matches('"');
                        if !p.is_empty() { candidates.push(PathBuf::from(p)); }
                    }
                }
            }
        }

        let mut reg = Self::new();

        for (box_name, plugin_name) in mappings.into_iter() {
            // Find dynamic library path
            if let Some(path) = super::loader::resolve_plugin_path(&plugin_name, &candidates) {
                let loaded = super::loader::LoadedPlugin::load_from_file(&path)?;
                reg.by_type_id.insert(loaded.type_id, box_name.clone());
                reg.by_name.insert(box_name, loaded);
            }
        }

        // 型情報をパース（ベストエフォート）
        eprintln!("🔍 DEBUG: About to call parse_type_info");
        reg.parse_type_info(&content);
        eprintln!("🔍 DEBUG: parse_type_info completed");
        
        // デバッグ出力：型情報の読み込み状況
        eprintln!("🔍 Type info loaded:");
        for (box_name, methods) in &reg.type_info {
            eprintln!("  📦 {}: {} methods", box_name, methods.len());
            for (method_name, type_info) in methods {
                eprintln!("    - {}: {} args", method_name, type_info.args.len());
            }
        }
        
        Ok(reg)
    }
    
    /// 型情報をパース（簡易実装）
    /// [plugins.FileBox.methods] セクションを探してパース
    fn parse_type_info(&mut self, content: &str) {
        eprintln!("🔍 DEBUG: parse_type_info called!");
        // 安全に文字列をトリミング（文字境界考慮）
        let preview = if content.len() <= 500 {
            content
        } else {
            // 文字境界を考慮して安全にトリミング
            content.char_indices()
                .take_while(|(idx, _)| *idx < 500)
                .last()
                .map(|(idx, ch)| &content[..idx + ch.len_utf8()])
                .unwrap_or("")
        };
        eprintln!("📄 TOML content preview:\n{}", preview);
        
        // FileBoxの型情報を探す（簡易実装、後で汎用化）
        if let Some(methods_start) = content.find("[plugins.FileBox.methods]") {
            println!("✅ Found [plugins.FileBox.methods] section at position {}", methods_start);
            let methods_section = &content[methods_start..];
            
            // 🔄 動的にメソッド名を抽出（決め打ちなし！）
            let method_names = self.extract_method_names_from_toml(methods_section);
            
            // 抽出されたメソッドそれぞれを処理
            for method_name in method_names {
                self.parse_method_type_info("FileBox", &method_name, methods_section);
            }
        } else {
            eprintln!("❌ [plugins.FileBox.methods] section not found in TOML!");
            // TOMLの全内容をダンプ
            eprintln!("📄 Full TOML content:\n{}", content);
        }
    }
    
    /// TOMLセクションからメソッド名を動的に抽出
    fn extract_method_names_from_toml(&self, section: &str) -> Vec<String> {
        let mut method_names = Vec::new();
        
        println!("🔍 DEBUG: Extracting methods from TOML section:");
        println!("📄 Section content:\n{}", section);
        
        for line in section.lines() {
            let line = line.trim();
            println!("🔍 Processing line: '{}'", line);
            
            // "method_name = { ... }" の形式を探す
            if let Some(eq_pos) = line.find(" = {") {
                let method_name = line[..eq_pos].trim();
                
                // セクション名やコメントは除外
                if !method_name.starts_with('[') && !method_name.starts_with('#') && !method_name.is_empty() {
                    println!("✅ Found method: '{}'", method_name);
                    method_names.push(method_name.to_string());
                } else {
                    println!("❌ Skipped line (section/comment): '{}'", method_name);
                }
            } else {
                println!("❌ Line doesn't match pattern: '{}'", line);
            }
        }
        
        println!("🎯 Total extracted methods: {:?}", method_names);
        method_names
    }
    
    /// 特定メソッドの型情報をパース
    fn parse_method_type_info(&mut self, box_name: &str, method_name: &str, section: &str) {
        // メソッド定義を探す
        if let Some(method_start) = section.find(&format!("{} = ", method_name)) {
            let method_line_start = section[..method_start].rfind('\n').unwrap_or(0);
            let method_line_end = section[method_start..].find('\n').map(|p| method_start + p).unwrap_or(section.len());
            let method_def = &section[method_line_start..method_line_end];
            
            // args = [] をパース
            if method_def.contains("args = []") {
                // 引数なし
                let type_info = MethodTypeInfo {
                    args: vec![],
                    returns: None,
                };
                self.type_info.entry(box_name.to_string())
                    .or_insert_with(HashMap::new)
                    .insert(method_name.to_string(), type_info);
            } else if method_def.contains("args = [{") {
                // 引数あり（簡易パース）
                let mut args = Vec::new();
                
                // writeメソッドの特殊処理
                if method_name == "write" && method_def.contains("from = \"string\"") && method_def.contains("to = \"bytes\"") {
                    args.push(ArgTypeMapping::new("string".to_string(), "bytes".to_string()));
                }
                // openメソッドの特殊処理  
                else if method_name == "open" {
                    args.push(ArgTypeMapping::with_name("path".to_string(), "string".to_string(), "string".to_string()));
                    args.push(ArgTypeMapping::with_name("mode".to_string(), "string".to_string(), "string".to_string()));
                }
                
                let type_info = MethodTypeInfo {
                    args,
                    returns: None,
                };
                self.type_info.entry(box_name.to_string())
                    .or_insert_with(HashMap::new)
                    .insert(method_name.to_string(), type_info);
            }
        }
    }
}

// ===== Global registry (for interpreter access) =====
static PLUGIN_REGISTRY: OnceCell<PluginRegistry> = OnceCell::new();

/// Initialize global plugin registry from config
pub fn init_global_from_config(path: &str) -> BidResult<()> {
    eprintln!("🔍 DEBUG: init_global_from_config called with path: {}", path);
    let reg = PluginRegistry::load_from_config(path)?;
    let _ = PLUGIN_REGISTRY.set(reg);
    Ok(())
}

/// Get global plugin registry if initialized
pub fn global() -> Option<&'static PluginRegistry> {
    PLUGIN_REGISTRY.get()
}
