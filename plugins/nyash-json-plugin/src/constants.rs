//! Constants and type definitions

// Result codes
pub const OK: i32 = 0;
pub const E_SHORT: i32 = -1;
pub const E_TYPE: i32 = -2;
pub const E_METHOD: i32 = -3;
pub const E_ARGS: i32 = -4;
pub const E_PLUGIN: i32 = -5;
pub const E_HANDLE: i32 = -8;

// Method IDs - JsonDocBox
pub const JD_BIRTH: u32 = 0;
pub const JD_PARSE: u32 = 1;
pub const JD_ROOT: u32 = 2;
pub const JD_ERROR: u32 = 3;
pub const JD_FINI: u32 = u32::MAX;

// Method IDs - JsonNodeBox
pub const JN_BIRTH: u32 = 0;
pub const JN_KIND: u32 = 1;
pub const JN_GET: u32 = 2;
pub const JN_SIZE: u32 = 3;
pub const JN_AT: u32 = 4;
pub const JN_STR: u32 = 5;
pub const JN_INT: u32 = 6;
pub const JN_BOOL: u32 = 7;
pub const JN_FINI: u32 = u32::MAX;

// Type IDs (for Handle TLV)
pub const T_JSON_DOC: u32 = 70;
pub const T_JSON_NODE: u32 = 71;
