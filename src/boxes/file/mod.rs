//! FileBox 📁 - ファイルI/O（PathBox/DirBoxとセット）
// Nyashの箱システムによるファイル入出力を提供します。
// 参考: 既存Boxの設計思想

use crate::box_trait::{NyashBox, StringBox, BoolBox, BoxCore, BoxBase};
use std::any::Any;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Result};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct FileBox {
    file: Arc<Mutex<File>>,
    path: Arc<String>,
    base: BoxBase,
}

impl FileBox {
    pub fn new() -> Self {
        // Create a default FileBox for delegation dispatch
        // Uses a temporary file for built-in Box inheritance dispatch
        let temp_path = "/tmp/nyash_temp_file";
        match Self::open(temp_path) {
            Ok(file_box) => file_box,
            Err(_) => {
                // Fallback: create with empty file handle - only for dispatch
                use std::fs::OpenOptions;
                let file = OpenOptions::new().create(true).write(true).read(true)
                    .open("/dev/null").unwrap_or_else(|_| File::open("/dev/null").unwrap());
                FileBox {
                    file: Arc::new(Mutex::new(file)),
                    path: Arc::new(String::new()),
                    base: BoxBase::new(),
                }
            }
        }
    }
    
    pub fn open(path: &str) -> Result<Self> {
        let file = OpenOptions::new().read(true).write(true).create(true).open(path)?;
        Ok(FileBox { 
            file: Arc::new(Mutex::new(file)),
            path: Arc::new(path.to_string()),
            base: BoxBase::new(),
        })
    }
    
    pub fn read_to_string(&self) -> Result<String> {
        let mut file = self.file.lock().unwrap();
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        Ok(s)
    }
    
    pub fn write_all(&self, buf: &[u8]) -> Result<()> {
        let mut file = self.file.lock().unwrap();
        file.write_all(buf)
    }
    
    /// ファイルの内容を読み取る
    pub fn read(&self) -> Box<dyn NyashBox> {
        match self.read_to_string() {
            Ok(content) => Box::new(StringBox::new(&content)),
            Err(e) => Box::new(StringBox::new(&format!("Error reading file: {}", e))),
        }
    }
    
    /// ファイルに内容を書き込む
    pub fn write(&self, content: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let content_str = content.to_string_box().value;
        match self.write_all(content_str.as_bytes()) {
            Ok(()) => Box::new(StringBox::new("ok")),
            Err(e) => Box::new(StringBox::new(&format!("Error writing file: {}", e))),
        }
    }
    
    /// ファイルが存在するかチェック
    pub fn exists(&self) -> Box<dyn NyashBox> {
        use std::path::Path;
        Box::new(BoolBox::new(Path::new(&**self.path).exists()))
    }
    
    /// ファイルを削除
    pub fn delete(&self) -> Box<dyn NyashBox> {
        match std::fs::remove_file(&**self.path) {
            Ok(()) => Box::new(StringBox::new("ok")),
            Err(e) => Box::new(StringBox::new(&format!("Error deleting file: {}", e))),
        }
    }
    
    /// ファイルをコピー
    pub fn copy(&self, dest: &str) -> Box<dyn NyashBox> {
        match std::fs::copy(&**self.path, dest) {
            Ok(_) => Box::new(StringBox::new("ok")),
            Err(e) => Box::new(StringBox::new(&format!("Error copying file: {}", e))),
        }
    }
}

impl BoxCore for FileBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "FileBox({})", self.path)
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for FileBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        // Note: Cannot truly clone a File handle, so create a new one to the same path
        match FileBox::open(&self.path) {
            Ok(new_file) => Box::new(new_file),
            Err(_) => Box::new(crate::box_trait::VoidBox::new())  // Return void on error
        }
    }

    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("FileBox({})", self.path))
    }


    fn type_name(&self) -> &'static str {
        "FileBox"
    }


    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_file) = other.as_any().downcast_ref::<FileBox>() {
            BoolBox::new(*self.path == *other_file.path)
        } else {
            BoolBox::new(false)
        }
    }
}

impl std::fmt::Display for FileBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}
