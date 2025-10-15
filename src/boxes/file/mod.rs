//! FileBox 📁 - ファイルI/O（PathBox/DirBoxとセット）
// Nyashの箱システムによるファイル入出力を提供します。
// 参考: 既存Boxの設計思想

use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox};
use std::any::Any;
use std::fs::{File, OpenOptions};
use std::io::{Read, Result, Write, Seek, SeekFrom};
use std::sync::RwLock;

#[derive(Debug)]
pub struct FileBox {
    file: RwLock<File>,
    path: RwLock<String>,
    base: BoxBase,
}

impl Clone for FileBox {
    fn clone(&self) -> Self {
        // File handles can't be easily cloned, so we'll reopen the file
        let path_now = { self.path.read().unwrap().clone() };
        match Self::open(&path_now) {
            Ok(new_file_box) => new_file_box,
            Err(_) => {
                // Fallback to default if reopening fails
                Self::new()
            }
        }
    }
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
                let file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .read(true)
                    .open("/dev/null")
                    .unwrap_or_else(|_| File::open("/dev/null").unwrap());
                FileBox {
                    file: RwLock::new(file),
                    path: RwLock::new(String::new()),
                    base: BoxBase::new(),
                }
            }
        }
    }

    pub fn open(path: &str) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        Ok(FileBox {
            file: RwLock::new(file),
            path: RwLock::new(path.to_string()),
            base: BoxBase::new(),
        })
    }

    pub fn read_to_string(&self) -> Result<String> {
        let mut file = self.file.write().unwrap();
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        Ok(s)
    }

    pub fn write_all(&self, buf: &[u8]) -> Result<()> {
        let mut file = self.file.write().unwrap();
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

    /// 末尾へ追記する
    pub fn append(&self, content: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let content_str = content.to_string_box().value;
        let mut file = self.file.write().unwrap();
        if let Err(e) = file.seek(SeekFrom::End(0)) { return Box::new(StringBox::new(&format!("Error seeking: {}", e))); }
        match file.write_all(content_str.as_bytes()) {
            Ok(()) => Box::new(StringBox::new("ok")),
            Err(e) => Box::new(StringBox::new(&format!("Error appending file: {}", e))),
        }
    }

    /// ファイルが存在するかチェック
    pub fn exists(&self) -> Box<dyn NyashBox> {
        use std::path::Path;
        let p = { self.path.read().unwrap().clone() };
        Box::new(BoolBox::new(Path::new(&p).exists()))
    }

    /// ファイルを削除
    pub fn delete(&self) -> Box<dyn NyashBox> {
        let p = { self.path.read().unwrap().clone() };
        match std::fs::remove_file(&p) {
            Ok(()) => Box::new(StringBox::new("ok")),
            Err(e) => Box::new(StringBox::new(&format!("Error deleting file: {}", e))),
        }
    }

    /// ファイルをコピー
    pub fn copy(&self, dest: &str) -> Box<dyn NyashBox> {
        let p = { self.path.read().unwrap().clone() };
        match std::fs::copy(&p, dest) {
            Ok(_) => Box::new(StringBox::new("ok")),
            Err(e) => Box::new(StringBox::new(&format!("Error copying file: {}", e))),
        }
    }

    /// Re-open underlying file in-place (read-only when mode starts with 'r')
    pub fn open_in_place(&self, path: &str, mode: &str) -> bool {
        // Minimal: support read-only ("r" prefix). If other modes, try rw.
        let mut opts = OpenOptions::new();
        if mode.starts_with('r') && !mode.contains('w') && !mode.contains('+') {
            opts.read(true).write(false).create(false);
        } else {
            // permissive fallback: read/write or append or append
            if mode.starts_with('a') { opts.read(true).append(true).create(true); }
            else { opts.read(true).write(true).create(true); }
        }
        match opts.open(path) {
            Ok(fh) => {
                // Swap file handle and update path
                {
                    let mut guard = self.file.write().unwrap();
                    *guard = fh;
                }
                {
                    let mut p = self.path.write().unwrap();
                    *p = path.to_string();
                }
                true
            }
            Err(_) => false,
        }
    }

    /// Close underlying file (reset to /dev/null). Returns true on success.
    pub fn close_in_place(&self) -> bool {
        match OpenOptions::new().read(true).open("/dev/null") {
            Ok(fh) => {
                {
                    let mut guard = self.file.write().unwrap();
                    *guard = fh;
                }
                true
            }
            Err(_) => false,
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
        let p = { self.path.read().unwrap().clone() };
        write!(f, "FileBox({})", p)
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
        let p = { self.path.read().unwrap().clone() };
        match FileBox::open(&p) {
            Ok(new_file) => Box::new(new_file),
            Err(_) => Box::new(crate::box_trait::VoidBox::new()), // Return void on error
        }
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }

    fn to_string_box(&self) -> StringBox {
        let p = { self.path.read().unwrap().clone() };
        StringBox::new(format!("FileBox({})", p))
    }

    fn type_name(&self) -> &'static str {
        "FileBox"
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_file) = other.as_any().downcast_ref::<FileBox>() {
            let p1 = { self.path.read().unwrap().clone() };
            let p2 = { other_file.path.read().unwrap().clone() };
            BoolBox::new(p1 == p2)
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