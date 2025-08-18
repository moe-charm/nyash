/// BID-1 Standard Error Codes (Runtime FFI)
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BidRuntimeError {
    /// Operation successful
    Success = 0,
    
    /// Buffer too small (need to call again with larger buffer)
    ShortBuffer = -1,
    
    /// Invalid type ID
    InvalidType = -2,
    
    /// Invalid method ID
    InvalidMethod = -3,
    
    /// Invalid arguments
    InvalidArgs = -4,
    
    /// Plugin internal error
    PluginError = -5,
    
    /// Memory allocation failed
    OutOfMemory = -6,
    
    /// UTF-8 encoding error
    InvalidUtf8 = -7,
    
    /// Handle not found
    InvalidHandle = -8,
    
    /// Version mismatch
    VersionMismatch = -9,
}

/// BID Schema and Code Generation Errors
#[derive(Debug, Clone)]
pub enum BidError {
    /// Runtime FFI error
    Runtime(BidRuntimeError),
    
    /// IO error (file not found, permission denied, etc.)
    IoError(String),
    
    /// Parse error (invalid YAML/JSON)
    ParseError(String),
    
    /// Unsupported BID version
    UnsupportedVersion(u32),
    
    /// Duplicate interface name
    DuplicateInterface(String),
    
    /// Duplicate method name
    DuplicateMethod(String),
    
    /// Duplicate parameter name
    DuplicateParameter(String),
    
    /// Unsupported target
    UnsupportedTarget(String),
    
    /// Template error
    TemplateError(String),
    
    /// Code generation error
    CodeGenError(String),
}

impl BidRuntimeError {
    /// Convert from raw i32
    pub fn from_raw(code: i32) -> Self {
        match code {
            0 => BidRuntimeError::Success,
            -1 => BidRuntimeError::ShortBuffer,
            -2 => BidRuntimeError::InvalidType,
            -3 => BidRuntimeError::InvalidMethod,
            -4 => BidRuntimeError::InvalidArgs,
            -5 => BidRuntimeError::PluginError,
            -6 => BidRuntimeError::OutOfMemory,
            -7 => BidRuntimeError::InvalidUtf8,
            -8 => BidRuntimeError::InvalidHandle,
            -9 => BidRuntimeError::VersionMismatch,
            _ => BidRuntimeError::PluginError, // Unknown errors map to plugin error
        }
    }
    
    /// Get human-readable error message
    pub fn message(&self) -> &'static str {
        match self {
            BidRuntimeError::Success => "Operation successful",
            BidRuntimeError::ShortBuffer => "Buffer too small, call again with larger buffer",
            BidRuntimeError::InvalidType => "Invalid type ID",
            BidRuntimeError::InvalidMethod => "Invalid method ID",
            BidRuntimeError::InvalidArgs => "Invalid arguments",
            BidRuntimeError::PluginError => "Plugin internal error",
            BidRuntimeError::OutOfMemory => "Memory allocation failed",
            BidRuntimeError::InvalidUtf8 => "Invalid UTF-8 encoding",
            BidRuntimeError::InvalidHandle => "Handle not found",
            BidRuntimeError::VersionMismatch => "BID version mismatch",
        }
    }
}

impl std::fmt::Display for BidRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (code: {})", self.message(), *self as i32)
    }
}

impl std::error::Error for BidRuntimeError {}

impl std::fmt::Display for BidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BidError::Runtime(err) => write!(f, "Runtime error: {}", err),
            BidError::IoError(msg) => write!(f, "IO error: {}", msg),
            BidError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            BidError::UnsupportedVersion(v) => write!(f, "Unsupported BID version: {}", v),
            BidError::DuplicateInterface(name) => write!(f, "Duplicate interface: {}", name),
            BidError::DuplicateMethod(name) => write!(f, "Duplicate method: {}", name),
            BidError::DuplicateParameter(name) => write!(f, "Duplicate parameter: {}", name),
            BidError::UnsupportedTarget(target) => write!(f, "Unsupported target: {}", target),
            BidError::TemplateError(msg) => write!(f, "Template error: {}", msg),
            BidError::CodeGenError(msg) => write!(f, "Code generation error: {}", msg),
        }
    }
}

impl std::error::Error for BidError {}

impl BidError {
    /// For compatibility with existing code
    pub fn invalid_args() -> Self {
        BidError::Runtime(BidRuntimeError::InvalidArgs)
    }
    
    pub fn invalid_type() -> Self {
        BidError::Runtime(BidRuntimeError::InvalidType)
    }
    
    pub fn plugin_error() -> Self {
        BidError::Runtime(BidRuntimeError::PluginError)
    }
    
    pub fn invalid_method() -> Self {
        BidError::Runtime(BidRuntimeError::InvalidMethod)
    }
    
    pub fn out_of_memory() -> Self {
        BidError::Runtime(BidRuntimeError::OutOfMemory)
    }
    
    pub fn short_buffer() -> Self {
        BidError::Runtime(BidRuntimeError::ShortBuffer)
    }
    
    pub fn invalid_handle() -> Self {
        BidError::Runtime(BidRuntimeError::InvalidHandle)
    }
    
    pub fn invalid_utf8() -> Self {
        BidError::Runtime(BidRuntimeError::InvalidUtf8)
    }
    
    pub fn version_mismatch() -> Self {
        BidError::Runtime(BidRuntimeError::VersionMismatch)
    }
    
    /// Convert from raw i32 (for compatibility with existing plugin API)
    pub fn from_raw(code: i32) -> Self {
        BidError::Runtime(BidRuntimeError::from_raw(code))
    }
}

/// Result type for BID operations
pub type BidResult<T> = Result<T, BidError>;

/// Result type for BID runtime operations  
pub type BidRuntimeResult<T> = Result<T, BidRuntimeError>;