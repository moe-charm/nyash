/// BID-1 Standard Error Codes
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BidError {
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

impl BidError {
    /// Convert from raw i32
    pub fn from_raw(code: i32) -> Self {
        match code {
            0 => BidError::Success,
            -1 => BidError::ShortBuffer,
            -2 => BidError::InvalidType,
            -3 => BidError::InvalidMethod,
            -4 => BidError::InvalidArgs,
            -5 => BidError::PluginError,
            -6 => BidError::OutOfMemory,
            -7 => BidError::InvalidUtf8,
            -8 => BidError::InvalidHandle,
            -9 => BidError::VersionMismatch,
            _ => BidError::PluginError, // Unknown errors map to plugin error
        }
    }
    
    /// Get human-readable error message
    pub fn message(&self) -> &'static str {
        match self {
            BidError::Success => "Operation successful",
            BidError::ShortBuffer => "Buffer too small, call again with larger buffer",
            BidError::InvalidType => "Invalid type ID",
            BidError::InvalidMethod => "Invalid method ID",
            BidError::InvalidArgs => "Invalid arguments",
            BidError::PluginError => "Plugin internal error",
            BidError::OutOfMemory => "Memory allocation failed",
            BidError::InvalidUtf8 => "Invalid UTF-8 encoding",
            BidError::InvalidHandle => "Handle not found",
            BidError::VersionMismatch => "BID version mismatch",
        }
    }
}

impl std::fmt::Display for BidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (code: {})", self.message(), *self as i32)
    }
}

impl std::error::Error for BidError {}

/// Result type for BID operations
pub type BidResult<T> = Result<T, BidError>;