use crate::box_trait::NyashBox;
use super::{BidHandle, BidType, BidError};
use std::sync::Arc;
use std::collections::HashMap;

/// BID-FFI Bridge for Nyash Box types
/// Provides conversion between Nyash runtime values and BID handles
pub trait BidBridge {
    /// Convert a Nyash Box to a BID handle
    fn to_bid_handle(&self, registry: &mut BoxRegistry) -> Result<BidHandle, BidError>;
    
    /// Get the BID type representation
    fn bid_type(&self) -> BidType;
}

/// Registry for managing Box instances and their handles
pub struct BoxRegistry {
    /// Maps handle to Arc<dyn NyashBox>
    handle_to_box: HashMap<BidHandle, Arc<dyn NyashBox>>,
    
    /// Next instance ID for each type
    next_instance_id: HashMap<u32, u32>,
    
    /// Reverse lookup: Arc pointer to handle
    box_to_handle: HashMap<usize, BidHandle>,
}

impl BoxRegistry {
    pub fn new() -> Self {
        Self {
            handle_to_box: HashMap::new(),
            next_instance_id: HashMap::new(),
            box_to_handle: HashMap::new(),
        }
    }
    
    /// Register a Box and get its handle
    pub fn register_box(&mut self, type_id: u32, boxed: Arc<dyn NyashBox>) -> BidHandle {
        // Check if already registered by comparing Arc pointers
        // We use the address of the Arc allocation itself as the key
        let arc_addr = &*boxed as *const dyn NyashBox as *const () as usize;
        if let Some(&handle) = self.box_to_handle.get(&arc_addr) {
            return handle;
        }
        
        // Generate new instance ID
        let instance_id = self.next_instance_id.entry(type_id).or_insert(1);
        let handle = BidHandle::new(type_id, *instance_id);
        *instance_id += 1;
        
        // Register bidirectionally
        self.handle_to_box.insert(handle, boxed.clone());
        self.box_to_handle.insert(arc_addr, handle);
        
        handle
    }
    
    /// Retrieve a Box by its handle
    pub fn get_box(&self, handle: BidHandle) -> Option<Arc<dyn NyashBox>> {
        self.handle_to_box.get(&handle).cloned()
    }
    
    /// Remove a Box from the registry
    pub fn unregister(&mut self, handle: BidHandle) -> Option<Arc<dyn NyashBox>> {
        if let Some(boxed) = self.handle_to_box.remove(&handle) {
            let arc_addr = &*boxed as *const dyn NyashBox as *const () as usize;
            self.box_to_handle.remove(&arc_addr);
            Some(boxed)
        } else {
            None
        }
    }
}

/// Convert Nyash Box to BID handle
pub fn box_to_bid_handle(
    arc_box: &Arc<dyn NyashBox>,
    registry: &mut BoxRegistry,
) -> Result<(BidType, BidHandle), BidError> {
    // Downcast to specific box types
    if let Some(_string_box) = arc_box.as_any().downcast_ref::<crate::boxes::string_box::StringBox>() {
        let handle = registry.register_box(
            crate::bid::types::BoxTypeId::StringBox as u32,
            arc_box.clone()
        );
        Ok((BidType::Handle { type_id: 1, instance_id: handle.instance_id }, handle))
    } else if let Some(_integer_box) = arc_box.as_any().downcast_ref::<crate::boxes::integer_box::IntegerBox>() {
        let handle = registry.register_box(
            crate::bid::types::BoxTypeId::IntegerBox as u32,
            arc_box.clone()
        );
        Ok((BidType::Handle { type_id: 2, instance_id: handle.instance_id }, handle))
    } else {
        Err(BidError::InvalidType)
    }
}

/// Convert BID handle back to Nyash Box
pub fn bid_handle_to_box(
    handle: BidHandle,
    registry: &BoxRegistry,
) -> Result<Arc<dyn NyashBox>, BidError> {
    registry.get_box(handle)
        .ok_or(BidError::InvalidHandle)
}

/// Extract string value from a Box for TLV encoding
pub fn extract_string_value(arc_box: &Arc<dyn NyashBox>) -> Result<String, BidError> {
    if let Some(string_box) = arc_box.as_any().downcast_ref::<crate::boxes::string_box::StringBox>() {
        Ok(string_box.value.clone())
    } else {
        Err(BidError::InvalidType)
    }
}

/// Extract integer value from a Box for TLV encoding  
pub fn extract_integer_value(arc_box: &Arc<dyn NyashBox>) -> Result<i64, BidError> {
    if let Some(integer_box) = arc_box.as_any().downcast_ref::<crate::boxes::integer_box::IntegerBox>() {
        Ok(integer_box.value)
    } else {
        Err(BidError::InvalidType)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_box_registry() {
        let mut registry = BoxRegistry::new();
        
        // Create a mock box
        let string_box = crate::boxes::string_box::StringBox::new("Hello");
        let arc_box: Arc<dyn NyashBox> = Arc::new(string_box);
        
        // Register it
        let handle = registry.register_box(1, arc_box.clone());
        assert_eq!(handle.type_id, 1);
        assert_eq!(handle.instance_id, 1);
        
        // Retrieve it
        let retrieved = registry.get_box(handle).unwrap();
        assert_eq!(Arc::as_ptr(&retrieved), Arc::as_ptr(&arc_box));
        
        // Register same box again should return same handle
        let handle2 = registry.register_box(1, arc_box.clone());
        assert_eq!(handle, handle2);
    }
    
    #[test]
    fn test_string_box_bid_conversion() {
        let mut registry = BoxRegistry::new();
        
        // Create StringBox
        let string_box = crate::boxes::string_box::StringBox::new("Test String");
        let arc_box: Arc<dyn NyashBox> = Arc::new(string_box);
        
        // Convert to BID handle
        let (bid_type, handle) = box_to_bid_handle(&arc_box, &mut registry).unwrap();
        assert_eq!(handle.type_id, 1); // StringBox type ID
        match bid_type {
            BidType::Handle { type_id, .. } => assert_eq!(type_id, 1),
            _ => panic!("Expected Handle type"),
        }
        
        // Extract string value
        let value = extract_string_value(&arc_box).unwrap();
        assert_eq!(value, "Test String");
        
        // Round-trip test
        let retrieved = bid_handle_to_box(handle, &registry).unwrap();
        let retrieved_value = extract_string_value(&retrieved).unwrap();
        assert_eq!(retrieved_value, "Test String");
    }
    
    #[test]
    fn test_integer_box_bid_conversion() {
        let mut registry = BoxRegistry::new();
        
        // Create IntegerBox
        let integer_box = crate::boxes::integer_box::IntegerBox::new(42);
        let arc_box: Arc<dyn NyashBox> = Arc::new(integer_box);
        
        // Convert to BID handle
        let (bid_type, handle) = box_to_bid_handle(&arc_box, &mut registry).unwrap();
        assert_eq!(handle.type_id, 2); // IntegerBox type ID
        match bid_type {
            BidType::Handle { type_id, .. } => assert_eq!(type_id, 2),
            _ => panic!("Expected Handle type"),
        }
        
        // Extract integer value
        let value = extract_integer_value(&arc_box).unwrap();
        assert_eq!(value, 42);
        
        // Round-trip test
        let retrieved = bid_handle_to_box(handle, &registry).unwrap();
        let retrieved_value = extract_integer_value(&retrieved).unwrap();
        assert_eq!(retrieved_value, 42);
    }
}