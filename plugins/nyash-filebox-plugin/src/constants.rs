//! Constants and error codes for FileBox plugin

// ============ Error Codes (BID-1 alignment) ============
pub const NYB_SUCCESS: i32 = 0;
pub const NYB_E_SHORT_BUFFER: i32 = -1;
#[allow(dead_code)]
pub const NYB_E_INVALID_TYPE: i32 = -2;
pub const NYB_E_INVALID_METHOD: i32 = -3;
pub const NYB_E_INVALID_ARGS: i32 = -4;
pub const NYB_E_PLUGIN_ERROR: i32 = -5;
pub const NYB_E_INVALID_HANDLE: i32 = -8;

// ============ Method IDs ============
pub const METHOD_BIRTH: u32 = 0; // Constructor
pub const METHOD_OPEN: u32 = 1;
pub const METHOD_READ: u32 = 2;
pub const METHOD_WRITE: u32 = 3;
pub const METHOD_CLOSE: u32 = 4;
pub const METHOD_EXISTS: u32 = 5;
pub const METHOD_COPY_FROM: u32 = 7; // New: copyFrom(other: Handle)
pub const METHOD_CLONE_SELF: u32 = 8; // New: cloneSelf() -> Handle
pub const METHOD_FINI: u32 = u32::MAX; // Destructor

// ============ TLV Tags ============
pub const TLV_TAG_BOOL: u8 = 1;
pub const TLV_TAG_I32: u8 = 2;
#[allow(dead_code)]
pub const TLV_TAG_I64: u8 = 3;
pub const TLV_TAG_STRING: u8 = 6;
pub const TLV_TAG_BYTES: u8 = 7;
pub const TLV_TAG_HANDLE: u8 = 8;
pub const TLV_TAG_VOID: u8 = 9;

// ============ FileBox Type ID ============
#[allow(dead_code)]
pub const FILEBOX_TYPE_ID: u32 = 6;
