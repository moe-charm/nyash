//! TLV type tags and error codes

// TLV tags (synchronized with all implementations)
pub const TLV_TAG_BOOL: u8 = 1;
pub const TLV_TAG_I64: u8 = 3;
pub const TLV_TAG_STRING: u8 = 6;
pub const TLV_TAG_PLUGIN_HANDLE: u8 = 8;
pub const TLV_TAG_HOST_HANDLE: u8 = 9;
/// Void/empty value (tag=9, size=0) - Distinguished from HOST_HANDLE by size
pub const TLV_TAG_VOID: u8 = 9;

// Error codes
pub const HAKO_SUCCESS: i32 = 0;
pub const HAKO_E_SHORT_BUFFER: i32 = -1;
pub const HAKO_E_INVALID_ARGS: i32 = -2;
pub const HAKO_E_INVALID_HANDLE: i32 = -8;
