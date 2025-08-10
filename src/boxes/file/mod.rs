//! FileBox 📁 - ファイルI/O（PathBox/DirBoxとセット）
// Nyashの箱システムによるファイル入出力を提供します。
// 参考: 既存Boxの設計思想

use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Result};

pub struct FileBox {
    pub file: File,
}

impl FileBox {
    pub fn open(path: &str) -> Result<Self> {
        let file = OpenOptions::new().read(true).write(true).create(true).open(path)?;
        Ok(FileBox { file })
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
