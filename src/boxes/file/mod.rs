//! FileBox 📁 - ファイルI/O（PathBox/DirBoxとセット）
// Nyashの箱システムによるファイル入出力を提供します。
// 参考: 既存Boxの設計思想

use crate::box_trait::{NyashBox, StringBox, BoolBox};
use std::any::Any;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Result};

#[derive(Debug)]
pub struct FileBox {
    pub file: File,
    pub path: String,
    id: u64,
}

impl FileBox {
    pub fn open(path: &str) -> Result<Self> {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        let file = OpenOptions::new().read(true).write(true).create(true).open(path)?;
        Ok(FileBox { 
            file,
            path: path.to_string(),
            id,
        })
    }
    
    pub fn read_to_string(&mut self) -> Result<String> {
        let mut s = String::new();
        self.file.read_to_string(&mut s)?;
        Ok(s)
    }
    
    pub fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.file.write_all(buf)
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        "FileBox"
    }

    fn box_id(&self) -> u64 {
        self.id
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_file) = other.as_any().downcast_ref::<FileBox>() {
            BoolBox::new(self.path == other_file.path)
        } else {
            BoolBox::new(false)
        }
    }
}
