use super::Usize;

/// BID-1 Type System (ChatGPT Enhanced Edition)
#[derive(Clone, Debug, PartialEq)]
pub enum BidType {
    // === Primitives (pass by value across FFI) ===
    Bool,       // i32 (0=false, 1=true)
    I32,        // 32-bit signed integer
    I64,        // 64-bit signed integer
    F32,        // 32-bit floating point
    F64,        // 64-bit floating point
    
    // === Composite types (pass as ptr+len) ===
    String,     // UTF-8 string (ptr: usize, len: usize)
    Bytes,      // Binary data (ptr: usize, len: usize)
    
    // === Handle design (ChatGPT recommendation) ===
    Handle {
        type_id: u32,       // Box type ID (1=StringBox, 6=FileBox, etc.)
        instance_id: u32,   // Instance identifier
    },
    
    // === Meta types ===
    Void,       // No return value
    
    // === Phase 2 reserved (TLV tags reserved) ===
    #[allow(dead_code)]
    Option(Box<BidType>),         // TLV tag=21
    #[allow(dead_code)]
    Result(Box<BidType>, Box<BidType>), // TLV tag=20
    #[allow(dead_code)]
    Array(Box<BidType>),          // TLV tag=22
}

/// Handle representation for efficient Box references
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BidHandle {
    pub type_id: u32,
    pub instance_id: u32,
}

impl BidHandle {
    /// Create a new handle
    pub fn new(type_id: u32, instance_id: u32) -> Self {
        Self { type_id, instance_id }
    }
    
    /// Pack into single u64 (type_id << 32 | instance_id)
    pub fn to_u64(&self) -> u64 {
        ((self.type_id as u64) << 32) | (self.instance_id as u64)
    }
    
    /// Unpack from single u64
    pub fn from_u64(packed: u64) -> Self {
        Self {
            type_id: (packed >> 32) as u32,
            instance_id: (packed & 0xFFFFFFFF) as u32,
        }
    }
}

/// TLV (Type-Length-Value) tags for BID-1
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BidTag {
    Bool = 1,    // payload: 1 byte (0/1)
    I32 = 2,     // payload: 4 bytes (little-endian)
    I64 = 3,     // payload: 8 bytes (little-endian)
    F32 = 4,     // payload: 4 bytes (IEEE 754)
    F64 = 5,     // payload: 8 bytes (IEEE 754)
    String = 6,  // payload: UTF-8 bytes
    Bytes = 7,   // payload: binary data
    Handle = 8,  // payload: 8 bytes (type_id + instance_id)
    Void = 9,    // payload: 0 bytes
    
    // Phase 2 reserved
    Result = 20,
    Option = 21,
    Array = 22,
}

impl BidType {
    /// Get the TLV tag for this type
    pub fn tag(&self) -> BidTag {
        match self {
            BidType::Bool => BidTag::Bool,
            BidType::I32 => BidTag::I32,
            BidType::I64 => BidTag::I64,
            BidType::F32 => BidTag::F32,
            BidType::F64 => BidTag::F64,
            BidType::String => BidTag::String,
            BidType::Bytes => BidTag::Bytes,
            BidType::Handle { .. } => BidTag::Handle,
            BidType::Void => BidTag::Void,
            _ => panic!("Phase 2 types not yet implemented"),
        }
    }
    
    /// Get the expected payload size (None for variable-length types)
    pub fn payload_size(&self) -> Option<usize> {
        match self {
            BidType::Bool => Some(1),
            BidType::I32 => Some(4),
            BidType::I64 => Some(8),
            BidType::F32 => Some(4),
            BidType::F64 => Some(8),
            BidType::Handle { .. } => Some(8),
            BidType::Void => Some(0),
            BidType::String | BidType::Bytes => None, // Variable length
            _ => panic!("Phase 2 types not yet implemented"),
        }
    }
}

/// Box type IDs (matching existing Nyash boxes)
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxTypeId {
    StringBox = 1,
    IntegerBox = 2,
    BoolBox = 3,
    FloatBox = 4,
    ArrayBox = 5,
    FileBox = 6,      // Plugin example
    FutureBox = 7,    // Existing async support
    P2PBox = 8,       // Existing P2P support
    // ... more box types
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_handle_packing() {
        let handle = BidHandle::new(6, 12345);
        let packed = handle.to_u64();
        let unpacked = BidHandle::from_u64(packed);
        
        assert_eq!(handle, unpacked);
        assert_eq!(unpacked.type_id, 6);
        assert_eq!(unpacked.instance_id, 12345);
    }
    
    #[test]
    fn test_type_tags() {
        assert_eq!(BidType::Bool.tag(), BidTag::Bool);
        assert_eq!(BidType::String.tag(), BidTag::String);
        assert_eq!(BidType::Handle { type_id: 6, instance_id: 0 }.tag(), BidTag::Handle);
    }
}