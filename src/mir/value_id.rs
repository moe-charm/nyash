/*!
 * MIR Value ID System - SSA value tracking
 * 
 * Implements unique identifiers for SSA values with type safety
 */

use std::fmt;

/// Unique identifier for SSA values within a function
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ValueId(pub u32);

impl ValueId {
    /// Create a new ValueId
    pub fn new(id: u32) -> Self {
        ValueId(id)
    }
    
    /// Get the raw ID value
    pub fn as_u32(self) -> u32 {
        self.0
    }
    
    /// Create ValueId from usize (for array indexing)
    pub fn from_usize(id: usize) -> Self {
        ValueId(id as u32)
    }
    
    /// Convert to usize (for array indexing)
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for ValueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

/// Local variable identifier (before SSA conversion)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub u32);

impl LocalId {
    /// Create a new LocalId
    pub fn new(id: u32) -> Self {
        LocalId(id)
    }
    
    /// Get the raw ID value
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

impl fmt::Display for LocalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "local_{}", self.0)
    }
}

/// Value ID generator for unique SSA value creation
#[derive(Debug, Clone)]
pub struct ValueIdGenerator {
    next_id: u32,
}

impl ValueIdGenerator {
    /// Create a new generator starting from 0
    pub fn new() -> Self {
        Self { next_id: 0 }
    }
    
    /// Generate the next unique ValueId
    pub fn next(&mut self) -> ValueId {
        let id = ValueId(self.next_id);
        self.next_id += 1;
        id
    }
    
    /// Peek at the next ID without consuming it
    pub fn peek_next(&self) -> ValueId {
        ValueId(self.next_id)
    }
    
    /// Reset the generator (for testing)
    pub fn reset(&mut self) {
        self.next_id = 0;
    }
}

impl Default for ValueIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Local ID generator for variable naming
#[derive(Debug, Clone)]
pub struct LocalIdGenerator {
    next_id: u32,
}

impl LocalIdGenerator {
    /// Create a new generator starting from 0
    pub fn new() -> Self {
        Self { next_id: 0 }
    }
    
    /// Generate the next unique LocalId
    pub fn next(&mut self) -> LocalId {
        let id = LocalId(self.next_id);
        self.next_id += 1;
        id
    }
    
    /// Reset the generator (for testing)
    pub fn reset(&mut self) {
        self.next_id = 0;
    }
}

impl Default for LocalIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_value_id_creation() {
        let id1 = ValueId::new(0);
        let id2 = ValueId::new(1);
        
        assert_eq!(id1.as_u32(), 0);
        assert_eq!(id2.as_u32(), 1);
        assert_ne!(id1, id2);
        
        assert_eq!(format!("{}", id1), "%0");
        assert_eq!(format!("{}", id2), "%1");
    }
    
    #[test]
    fn test_value_id_generator() {
        let mut gen = ValueIdGenerator::new();
        
        let id1 = gen.next();
        let id2 = gen.next();
        let id3 = gen.next();
        
        assert_eq!(id1, ValueId(0));
        assert_eq!(id2, ValueId(1));
        assert_eq!(id3, ValueId(2));
        
        assert_eq!(gen.peek_next(), ValueId(3));
    }
    
    #[test]
    fn test_local_id_creation() {
        let local1 = LocalId::new(0);
        let local2 = LocalId::new(1);
        
        assert_eq!(format!("{}", local1), "local_0");
        assert_eq!(format!("{}", local2), "local_1");
    }
    
    #[test]
    fn test_local_id_generator() {
        let mut gen = LocalIdGenerator::new();
        
        let local1 = gen.next();
        let local2 = gen.next();
        
        assert_eq!(local1, LocalId(0));
        assert_eq!(local2, LocalId(1));
    }
    
    #[test]
    fn test_value_id_ordering() {
        let id1 = ValueId(1);
        let id2 = ValueId(2);
        let id3 = ValueId(3);
        
        assert!(id1 < id2);
        assert!(id2 < id3);
        assert!(id1 < id3);
        
        let mut ids = vec![id3, id1, id2];
        ids.sort();
        
        assert_eq!(ids, vec![id1, id2, id3]);
    }
}