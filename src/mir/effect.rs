/*!
 * MIR Effect System - Track side effects for optimization
 * 
 * Based on ChatGPT5's design for parallel execution and optimization safety
 */

use std::fmt;

/// Effect flags for tracking side effects and enabling optimizations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectMask(u16);

/// Individual effect types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// Pure computation with no side effects
    Pure = 0x0001,
    /// Reads from heap/memory (but doesn't modify)
    ReadHeap = 0x0002,
    /// Writes to heap/memory
    WriteHeap = 0x0004,
    /// Performs I/O operations (file, network, console)
    IO = 0x0008,
    /// P2P/network communication
    P2P = 0x0010,
    /// Foreign Function Interface calls
    FFI = 0x0020,
    /// May panic or throw exceptions
    Panic = 0x0040,
    /// Allocates memory
    Alloc = 0x0080,
    /// Accesses global state
    Global = 0x0100,
    /// Thread/async operations
    Async = 0x0200,
    /// Unsafe operations
    Unsafe = 0x0400,
    /// Debug/logging operations
    Debug = 0x0800,
    /// Memory barrier operations
    Barrier = 0x1000,
}

impl EffectMask {
    /// No effects - pure computation
    pub const PURE: Self = Self(Effect::Pure as u16);
    
    /// Memory read effects
    pub const READ: Self = Self(Effect::ReadHeap as u16);
    
    /// Memory write effects (includes read)
    pub const WRITE: Self = Self((Effect::WriteHeap as u16) | (Effect::ReadHeap as u16));
    
    /// I/O effects
    pub const IO: Self = Self(Effect::IO as u16);
    
    /// P2P communication effects
    pub const P2P: Self = Self(Effect::P2P as u16);
    
    /// Panic/exception effects
    pub const PANIC: Self = Self(Effect::Panic as u16);
    
    /// All effects - maximum side effects
    pub const ALL: Self = Self(0xFFFF);
    
    /// Create an empty effect mask
    pub fn new() -> Self {
        Self(0)
    }
    
    /// Create effect mask from raw bits
    pub fn from_bits(bits: u16) -> Self {
        Self(bits)
    }
    
    /// Get raw bits
    pub fn bits(self) -> u16 {
        self.0
    }
    
    /// Add an effect to the mask
    pub fn add(self, effect: Effect) -> Self {
        Self(self.0 | (effect as u16))
    }
    
    /// Remove an effect from the mask
    pub fn remove(self, effect: Effect) -> Self {
        Self(self.0 & !(effect as u16))
    }
    
    /// Check if mask contains an effect
    pub fn contains(self, effect: Effect) -> bool {
        (self.0 & (effect as u16)) != 0
    }
    
    /// Check if mask contains any of the given effects
    pub fn contains_any(self, mask: EffectMask) -> bool {
        (self.0 & mask.0) != 0
    }
    
    /// Check if mask contains all of the given effects
    pub fn contains_all(self, mask: EffectMask) -> bool {
        (self.0 & mask.0) == mask.0
    }
    
    /// Combine two effect masks
    pub fn union(self, other: EffectMask) -> Self {
        Self(self.0 | other.0)
    }
    
    /// Get intersection of two effect masks
    pub fn intersection(self, other: EffectMask) -> Self {
        Self(self.0 & other.0)
    }
    
    /// Check if the computation is pure (no side effects)
    pub fn is_pure(self) -> bool {
        self.0 == 0 || self.0 == (Effect::Pure as u16)
    }
    
    /// Check if the computation only reads (doesn't modify state)
    pub fn is_read_only(self) -> bool {
        !self.contains(Effect::WriteHeap) && 
        !self.contains(Effect::IO) && 
        !self.contains(Effect::P2P) &&
        !self.contains(Effect::Global)
    }
    
    /// Check if parallel execution is safe
    pub fn is_parallel_safe(self) -> bool {
        !self.contains(Effect::WriteHeap) &&
        !self.contains(Effect::Global) &&
        !self.contains(Effect::Unsafe)
    }
    
    /// Check if operation can be moved across other operations
    pub fn is_moveable(self) -> bool {
        self.is_pure() || self.is_read_only()
    }
    
    /// Get a human-readable list of effects
    pub fn effect_names(self) -> Vec<&'static str> {
        let mut names = Vec::new();
        
        if self.is_pure() {
            names.push("pure");
            return names;
        }
        
        if self.contains(Effect::ReadHeap) { names.push("read"); }
        if self.contains(Effect::WriteHeap) { names.push("write"); }
        if self.contains(Effect::IO) { names.push("io"); }
        if self.contains(Effect::P2P) { names.push("p2p"); }
        if self.contains(Effect::FFI) { names.push("ffi"); }
        if self.contains(Effect::Panic) { names.push("panic"); }
        if self.contains(Effect::Alloc) { names.push("alloc"); }
        if self.contains(Effect::Global) { names.push("global"); }
        if self.contains(Effect::Async) { names.push("async"); }
        if self.contains(Effect::Unsafe) { names.push("unsafe"); }
        if self.contains(Effect::Debug) { names.push("debug"); }
        if self.contains(Effect::Barrier) { names.push("barrier"); }
        
        names
    }
}

impl Default for EffectMask {
    fn default() -> Self {
        Self::PURE
    }
}

impl fmt::Display for EffectMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names = self.effect_names();
        if names.is_empty() {
            write!(f, "none")
        } else {
            write!(f, "{}", names.join("|"))
        }
    }
}

impl std::ops::BitOr for EffectMask {
    type Output = Self;
    
    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl std::ops::BitOrAssign for EffectMask {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl std::ops::BitAnd for EffectMask {
    type Output = Self;
    
    fn bitand(self, rhs: Self) -> Self {
        self.intersection(rhs)
    }
}

impl std::ops::BitAndAssign for EffectMask {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_effect_mask_creation() {
        let pure = EffectMask::PURE;
        let read = EffectMask::READ;
        let write = EffectMask::WRITE;
        
        assert!(pure.is_pure());
        assert!(!read.is_pure());
        assert!(!write.is_pure());
        
        assert!(read.is_read_only());
        assert!(!write.is_read_only());
    }
    
    #[test]
    fn test_effect_combination() {
        let mut effects = EffectMask::new();
        assert!(effects.is_pure());
        
        effects = effects.add(Effect::ReadHeap);
        assert!(effects.contains(Effect::ReadHeap));
        assert!(effects.is_read_only());
        
        effects = effects.add(Effect::WriteHeap);
        assert!(effects.contains(Effect::WriteHeap));
        assert!(!effects.is_read_only());
        
        effects = effects.add(Effect::IO);
        assert!(effects.contains(Effect::IO));
        assert!(!effects.is_parallel_safe());
    }
    
    #[test]
    fn test_effect_union() {
        let read_effect = EffectMask::READ;
        let io_effect = EffectMask::IO;
        
        let combined = read_effect | io_effect;
        
        assert!(combined.contains(Effect::ReadHeap));
        assert!(combined.contains(Effect::IO));
        assert!(!combined.is_pure());
        assert!(!combined.is_parallel_safe());
    }
    
    #[test]
    fn test_parallel_safety() {
        let pure = EffectMask::PURE;
        let read = EffectMask::READ;
        let write = EffectMask::WRITE;
        let io = EffectMask::IO;
        
        assert!(pure.is_parallel_safe());
        assert!(read.is_parallel_safe());
        assert!(!write.is_parallel_safe());
        assert!(io.is_parallel_safe()); // I/O can be parallel if properly synchronized
    }
    
    #[test]
    fn test_effect_names() {
        let pure = EffectMask::PURE;
        assert_eq!(pure.effect_names(), vec!["pure"]);
        
        let read_write = EffectMask::READ.add(Effect::WriteHeap);
        let names = read_write.effect_names();
        assert!(names.contains(&"read"));
        assert!(names.contains(&"write"));
    }
    
    #[test]
    fn test_effect_display() {
        let pure = EffectMask::PURE;
        assert_eq!(format!("{}", pure), "pure");
        
        let read_io = EffectMask::READ | EffectMask::IO;
        let display = format!("{}", read_io);
        assert!(display.contains("read"));
        assert!(display.contains("io"));
    }
}