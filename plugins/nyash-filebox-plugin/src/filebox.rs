//! FileBox実装
//! 
//! ファイル操作を提供するBox実装

use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::collections::HashMap;
use std::path::PathBuf;

/// FileBoxインスタンス
pub struct FileBoxInstance {
    path: Option<PathBuf>,
    file: Option<File>,
    content: String,
}

impl FileBoxInstance {
    /// 新しいFileBoxインスタンス作成
    pub fn new() -> Self {
        Self {
            path: None,
            file: None,
            content: String::new(),
        }
    }
    
    /// ファイルオープン
    pub fn open(&mut self, path: &str) -> Result<(), std::io::Error> {
        let path_buf = PathBuf::from(path);
        
        // ファイルを読み書きモードでオープン
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path_buf)?;
        
        // 既存内容を読み込み
        let mut content = String::new();
        if let Err(_) = file.read_to_string(&mut content) {
            // 読み込めない場合は空文字列
            content.clear();
        }
        
        // ファイルポインタを先頭に戻す
        file.seek(SeekFrom::Start(0))?;
        
        self.path = Some(path_buf);
        self.file = Some(file);
        self.content = content;
        
        Ok(())
    }
    
    /// ファイル読み取り
    pub fn read(&mut self) -> Result<String, std::io::Error> {
        if let Some(ref mut file) = self.file {
            let mut content = String::new();
            file.seek(SeekFrom::Start(0))?;
            file.read_to_string(&mut content)?;
            self.content = content.clone();
            Ok(content)
        } else {
            Ok(self.content.clone())
        }
    }
    
    /// ファイル書き込み
    pub fn write(&mut self, data: &str) -> Result<(), std::io::Error> {
        self.content = data.to_string();
        
        if let Some(ref mut file) = self.file {
            file.seek(SeekFrom::Start(0))?;
            file.set_len(0)?; // ファイル内容をクリア
            file.write_all(data.as_bytes())?;
            file.flush()?;
        }
        
        Ok(())
    }
    
    /// ファイルパス取得
    pub fn path(&self) -> Option<&str> {
        self.path.as_ref().and_then(|p| p.to_str())
    }
    
    /// 内容取得
    pub fn content(&self) -> &str {
        &self.content
    }
}

/// FileBoxレジストリ
pub struct FileBoxRegistry {
    instances: HashMap<u32, FileBoxInstance>,
}

impl FileBoxRegistry {
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
        }
    }
    
    pub fn register(&mut self, handle: u32, instance: FileBoxInstance) {
        self.instances.insert(handle, instance);
    }
    
    pub fn get(&self, handle: u32) -> Option<&FileBoxInstance> {
        self.instances.get(&handle)
    }
    
    pub fn get_mut(&mut self, handle: u32) -> Option<&mut FileBoxInstance> {
        self.instances.get_mut(&handle)
    }
    
    pub fn remove(&mut self, handle: u32) -> Option<FileBoxInstance> {
        self.instances.remove(&handle)
    }
    
    pub fn count(&self) -> usize {
        self.instances.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_filebox_basic_operations() {
        let mut filebox = FileBoxInstance::new();
        
        // テスト用ファイル
        let test_file = "test_plugin_file.txt";
        
        // ファイルオープン
        assert!(filebox.open(test_file).is_ok());
        assert_eq!(filebox.path(), Some(test_file));
        
        // 書き込み
        assert!(filebox.write("Hello from plugin!").is_ok());
        assert_eq!(filebox.content(), "Hello from plugin!");
        
        // 読み込み
        let content = filebox.read().unwrap();
        assert_eq!(content, "Hello from plugin!");
        
        // クリーンアップ
        let _ = fs::remove_file(test_file);
    }
    
    #[test]
    fn test_filebox_registry() {
        let mut registry = FileBoxRegistry::new();
        
        let filebox1 = FileBoxInstance::new();
        let filebox2 = FileBoxInstance::new();
        
        registry.register(1, filebox1);
        registry.register(2, filebox2);
        
        assert_eq!(registry.count(), 2);
        assert!(registry.get(1).is_some());
        assert!(registry.get(2).is_some());
        assert!(registry.get(3).is_none());
        
        registry.remove(1);
        assert_eq!(registry.count(), 1);
        assert!(registry.get(1).is_none());
    }
}