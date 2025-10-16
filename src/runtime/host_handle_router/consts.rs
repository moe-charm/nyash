//! HostHandleRouter constants — slot IDs and canonical error codes
//! Keep this module free of heavy dependencies; used by routers and tests.

// Array slots
pub const ARRAY_GET: u64 = 100;
pub const ARRAY_SET: u64 = 101;
pub const ARRAY_SIZE: u64 = 102; // len/size/length

// Map slots
pub const MAP_SIZE: u64 = 200;
pub const MAP_HAS: u64 = 202;
pub const MAP_GET: u64 = 203;
pub const MAP_SET: u64 = 204;
pub const MAP_KEYS: u64 = 205;
pub const MAP_VALUES: u64 = 206;

// String slots
pub const STRING_LEN: u64 = 300;

// Return codes
pub const ERR_UNKNOWN_HANDLE: i32 = -1;   // unknown handle
pub const ERR_BUF_SMALL: i32     = -3;    // output buffer too small (retry)
pub const ERR_UNSUPPORTED: i32   = -11;   // known selector, wrong receiver type
pub const ERR_PLUGIN_FAILURE: i32= -12;   // plugin delegate failure
pub const ERR_BAD_ARGS: i32      = -13;   // TLV decode / arity / bad argument
pub const ERR_BAD_RETURN: i32    = -14;   // type/shape mismatch on return
