//! FileBox Plugin - BID-FFI File Operations
//! 
//! Provides file I/O operations through the BID-FFI plugin interface.
//! Everything is Box philosophy applied to file operations!

use crate::bid::{BidHandle, BoxTypeId};
use crate::bid::{NyashPluginInfo, NyashMethodInfo, NyashHostVtable};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::os::raw::{c_char, c_void};
use std::sync::{Arc, Mutex};
use std::ffi::{CStr, CString};

/// FileBox handle management
pub struct FileBoxRegistry {
    files: HashMap<u32, Arc<Mutex<FileBoxState>>>,
    next_handle: u32,
}

/// State of an open file
struct FileBoxState {
    file: File,
    path: String,
    mode: FileMode,
}

#[derive(Debug, Clone, Copy)]
enum FileMode {
    Read,
    Write,
    Append,
    ReadWrite,
}

impl FileBoxRegistry {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            next_handle: 1,
        }
    }
    
    pub fn open(&mut self, path: &str, mode: FileMode) -> Result<BidHandle, std::io::Error> {
        let file = match mode {
            FileMode::Read => OpenOptions::new().read(true).open(path)?,
            FileMode::Write => OpenOptions::new().write(true).create(true).truncate(true).open(path)?,
            FileMode::Append => OpenOptions::new().append(true).create(true).open(path)?,
            FileMode::ReadWrite => OpenOptions::new().read(true).write(true).create(true).open(path)?,
        };
        
        let handle_id = self.next_handle;
        self.next_handle += 1;
        
        let state = FileBoxState {
            file,
            path: path.to_string(),
            mode,
        };
        
        self.files.insert(handle_id, Arc::new(Mutex::new(state)));
        
        Ok(BidHandle::new(BoxTypeId::FileBox as u32, handle_id))
    }
    
    pub fn read(&self, handle: BidHandle, size: usize) -> Result<Vec<u8>, std::io::Error> {
        let handle_id = handle.instance_id;
        let file_state = self.files.get(&handle_id)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Invalid file handle"))?;
        
        let mut state = file_state.lock().unwrap();
        let mut buffer = vec![0u8; size];
        let bytes_read = state.file.read(&mut buffer)?;
        buffer.truncate(bytes_read);
        
        Ok(buffer)
    }
    
    pub fn write(&self, handle: BidHandle, data: &[u8]) -> Result<usize, std::io::Error> {
        let handle_id = handle.instance_id;
        let file_state = self.files.get(&handle_id)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Invalid file handle"))?;
        
        let mut state = file_state.lock().unwrap();
        state.file.write(data)
    }
    
    pub fn close(&mut self, handle: BidHandle) -> Result<(), std::io::Error> {
        let handle_id = handle.instance_id;
        self.files.remove(&handle_id)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Invalid file handle"))?;
        Ok(())
    }
}

/// Global registry instance
static mut FILEBOX_REGISTRY: Option<Arc<Mutex<FileBoxRegistry>>> = None;

/// Get or create the global registry
fn get_registry() -> Arc<Mutex<FileBoxRegistry>> {
    unsafe {
        if FILEBOX_REGISTRY.is_none() {
            FILEBOX_REGISTRY = Some(Arc::new(Mutex::new(FileBoxRegistry::new())));
        }
        FILEBOX_REGISTRY.as_ref().unwrap().clone()
    }
}

/// FileBox plugin interface for Nyash
pub struct FileBoxPlugin {
    registry: Arc<Mutex<FileBoxRegistry>>,
}

impl FileBoxPlugin {
    pub fn new() -> Self {
        Self {
            registry: get_registry(),
        }
    }
    
    /// Open a file and return its handle
    pub fn open(&self, path: &str, mode: &str) -> Result<BidHandle, String> {
        let file_mode = match mode {
            "r" => FileMode::Read,
            "w" => FileMode::Write,
            "a" => FileMode::Append,
            "rw" | "r+" => FileMode::ReadWrite,
            _ => return Err(format!("Invalid file mode: {}", mode)),
        };
        
        let mut registry = self.registry.lock().unwrap();
        registry.open(path, file_mode)
            .map_err(|e| format!("Failed to open file: {}", e))
    }
    
    /// Read data from file
    pub fn read(&self, handle: BidHandle, size: usize) -> Result<Vec<u8>, String> {
        let registry = self.registry.lock().unwrap();
        registry.read(handle, size)
            .map_err(|e| format!("Failed to read file: {}", e))
    }
    
    /// Write data to file
    pub fn write(&self, handle: BidHandle, data: &[u8]) -> Result<usize, String> {
        let registry = self.registry.lock().unwrap();
        registry.write(handle, data)
            .map_err(|e| format!("Failed to write file: {}", e))
    }
    
    /// Close file
    pub fn close(&self, handle: BidHandle) -> Result<(), String> {
        let mut registry = self.registry.lock().unwrap();
        registry.close(handle)
            .map_err(|e| format!("Failed to close file: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_filebox_plugin() {
        let plugin = FileBoxPlugin::new();
        
        // Create a test file
        let test_path = "test_filebox_plugin.txt";
        let test_content = "Hello, FileBox Plugin!";
        fs::write(test_path, test_content).unwrap();
        
        // Test open
        let handle = plugin.open(test_path, "r").unwrap();
        assert_eq!(handle.type_id, BoxTypeId::FileBox as u32);
        
        // Test read
        let data = plugin.read(handle, 100).unwrap();
        assert_eq!(String::from_utf8(data).unwrap(), test_content);
        
        // Test close
        plugin.close(handle).unwrap();
        
        // Test write mode
        let write_handle = plugin.open(test_path, "w").unwrap();
        let new_content = b"New content!";
        let written = plugin.write(write_handle, new_content).unwrap();
        assert_eq!(written, new_content.len());
        plugin.close(write_handle).unwrap();
        
        // Verify new content
        let read_handle = plugin.open(test_path, "r").unwrap();
        let data = plugin.read(read_handle, 100).unwrap();
        assert_eq!(&data[..], new_content);
        plugin.close(read_handle).unwrap();
        
        // Cleanup
        fs::remove_file(test_path).unwrap();
    }
}