//! FileBox - File system operations in Nyash
//!
//! Implements the FileBox type with file I/O operations.

use crate::box_trait::{NyashBox, BoxCore, BoxBase, StringBox, BoolBox, VoidBox};
use std::any::Any;
use std::fmt::{Debug, Display};
use std::fs;
use std::path::Path;

/// File values in Nyash - file system operations
#[derive(Debug, Clone)]
pub struct FileBox {
    pub path: String,
    base: BoxBase,
}

impl FileBox {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            base: BoxBase::new(),
        }
    }

    // ===== File Methods for Nyash =====

    /// Read file contents as string
    pub fn read(&self) -> Box<dyn NyashBox> {
        match fs::read_to_string(&self.path) {
            Ok(content) => Box::new(StringBox::new(content)),
            Err(_) => Box::new(VoidBox::new()), // Return void on error for now
        }
    }

    /// Write content to file
    pub fn write(&self, content: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let content_str = content.to_string_box().value;
        match fs::write(&self.path, content_str) {
            Ok(_) => Box::new(BoolBox::new(true)),
            Err(_) => Box::new(BoolBox::new(false)),
        }
    }

    /// Check if file exists
    pub fn exists(&self) -> Box<dyn NyashBox> {
        Box::new(BoolBox::new(Path::new(&self.path).exists()))
    }

    /// Delete file
    pub fn delete(&self) -> Box<dyn NyashBox> {
        match fs::remove_file(&self.path) {
            Ok(_) => Box::new(BoolBox::new(true)),
            Err(_) => Box::new(BoolBox::new(false)),
        }
    }

    /// Copy file to destination
    pub fn copy(&self, dest_path: &str) -> Box<dyn NyashBox> {
        match fs::copy(&self.path, dest_path) {
            Ok(_) => Box::new(BoolBox::new(true)),
            Err(_) => Box::new(BoolBox::new(false)),
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
        write!(f, "<FileBox: {}>", self.path)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for FileBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("<FileBox: {}>", self.path))
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_file) = other.as_any().downcast_ref::<FileBox>() {
            BoolBox::new(self.path == other_file.path)
        } else {
            BoolBox::new(false)
        }
    }

    fn type_name(&self) -> &'static str {
        "FileBox"
    }

    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl Display for FileBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}