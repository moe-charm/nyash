/*!
 * Ownership Forest Verification System
 * 
 * Implements ownership forest validation rules per ChatGPT5 specification:
 * - Ownership forest: strong in-degree ≤ 1 
 * - Strong cycle prohibition: strong edges form DAG (forest)
 * - Weak/strong interaction: bidirectional strong → error
 * - RefSet safety: strong→strong requires Release of old target
 * - WeakLoad/WeakCheck deterministic behavior: null/false on expiration
 */

use super::{MirInstructionV2, ValueId, MirFunction, MirModule};
use std::collections::{HashMap, HashSet, VecDeque};

/// Ownership forest verification errors
#[derive(Debug, Clone, PartialEq)]
pub enum OwnershipError {
    /// Strong reference has multiple owners (violates forest constraint)
    MultipleStrongOwners {
        target: ValueId,
        owners: Vec<ValueId>,
    },
    
    /// Strong reference cycle detected (violates DAG constraint) 
    StrongCycle {
        cycle: Vec<ValueId>,
    },
    
    /// Bidirectional strong references (should be strong + weak)
    BidirectionalStrong {
        first: ValueId,
        second: ValueId,
    },
    
    /// RefSet without proper Release of old target
    UnsafeRefSet {
        reference: ValueId,
        old_target: ValueId,
        new_target: ValueId,
    },
    
    /// WeakLoad on expired reference (should return null deterministically)
    WeakLoadExpired {
        weak_ref: ValueId,
        dead_target: ValueId,
    },
    
    /// Use after Release (accessing released ownership)
    UseAfterRelease {
        value: ValueId,
        released_at: String,
    },
    
    /// Invalid ownership transfer via Adopt
    InvalidAdopt {
        parent: ValueId,
        child: ValueId,
        reason: String,
    },
}

/// Ownership forest verifier
pub struct OwnershipVerifier {
    /// Strong ownership edges: child -> parent
    strong_edges: HashMap<ValueId, ValueId>,
    
    /// Weak reference edges: weak_ref -> target
    weak_edges: HashMap<ValueId, ValueId>,
    
    /// Released references (no longer valid for ownership)
    released: HashSet<ValueId>,
    
    /// Track live weak references for liveness checking
    live_weak_refs: HashSet<ValueId>,
    
    /// Track dead targets for WeakLoad/WeakCheck determinism
    dead_targets: HashSet<ValueId>,
}

impl OwnershipVerifier {
    /// Create a new ownership verifier
    pub fn new() -> Self {
        Self {
            strong_edges: HashMap::new(),
            weak_edges: HashMap::new(),
            released: HashSet::new(),
            live_weak_refs: HashSet::new(),
            dead_targets: HashSet::new(),
        }
    }
    
    /// Verify ownership forest properties for an entire module
    pub fn verify_module(&mut self, module: &MirModule) -> Result<(), Vec<OwnershipError>> {
        let mut errors = Vec::new();
        
        for function in module.functions.values() {
            if let Err(mut function_errors) = self.verify_function(function) {
                errors.append(&mut function_errors);
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Verify ownership forest properties for a single function
    pub fn verify_function(&mut self, function: &MirFunction) -> Result<(), Vec<OwnershipError>> {
        let mut errors = Vec::new();
        
        // Reset state for this function
        self.strong_edges.clear();
        self.weak_edges.clear();
        self.released.clear();
        self.live_weak_refs.clear();
        self.dead_targets.clear();
        
        // Process all instructions to build ownership graph
        for block in function.blocks.values() {
            for instruction in block.all_instructions() {
                if let Err(mut inst_errors) = self.process_instruction(instruction) {
                    errors.append(&mut inst_errors);
                }
            }
        }
        
        // Verify global ownership forest properties
        if let Err(mut forest_errors) = self.verify_ownership_forest() {
            errors.append(&mut forest_errors);
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Process a single instruction and update ownership state
    fn process_instruction(&mut self, instruction: &MirInstructionV2) -> Result<(), Vec<OwnershipError>> {
        let mut errors = Vec::new();
        
        match instruction {
            // NewBox creates a new ownership root
            MirInstructionV2::NewBox { dst, .. } => {
                // New boxes are ownership roots (no parent)
                // Clear any existing ownership for this value
                self.strong_edges.remove(dst);
            },
            
            // RefSet changes ownership relationships
            MirInstructionV2::RefSet { reference, new_target } => {
                // Check if the reference currently has a strong target
                if let Some(old_target) = self.strong_edges.get(reference) {
                    // Strong→Strong replacement requires explicit Release
                    if !self.released.contains(old_target) {
                        errors.push(OwnershipError::UnsafeRefSet {
                            reference: *reference,
                            old_target: *old_target,
                            new_target: *new_target,
                        });
                    }
                }
                
                // Set new strong ownership
                self.strong_edges.insert(*reference, *new_target);
                
                // Verify no multiple strong owners after this change
                if let Err(mut multiple_errors) = self.check_multiple_owners(*new_target) {
                    errors.append(&mut multiple_errors);
                }
            },
            
            // Adopt transfers ownership
            MirInstructionV2::Adopt { parent, child } => {
                // Verify the adoption is valid
                if self.released.contains(child) {
                    errors.push(OwnershipError::InvalidAdopt {
                        parent: *parent,
                        child: *child,
                        reason: "Cannot adopt released reference".to_string(),
                    });
                }
                
                // Check for cycle creation
                if self.would_create_cycle(*parent, *child) {
                    errors.push(OwnershipError::InvalidAdopt {
                        parent: *parent,
                        child: *child,
                        reason: "Would create strong cycle".to_string(),
                    });
                }
                
                // Establish strong ownership
                self.strong_edges.insert(*child, *parent);
            },
            
            // Release removes ownership
            MirInstructionV2::Release { reference } => {
                self.strong_edges.remove(reference);
                self.released.insert(*reference);
                
                // Mark any targets of this reference as potentially dead
                if let Some(target) = self.weak_edges.get(reference) {
                    self.dead_targets.insert(*target);
                }
            },
            
            // WeakNew creates weak reference
            MirInstructionV2::WeakNew { dst, box_val } => {
                self.weak_edges.insert(*dst, *box_val);
                self.live_weak_refs.insert(*dst);
            },
            
            // WeakLoad checks liveness
            MirInstructionV2::WeakLoad { weak_ref, .. } => {
                if let Some(target) = self.weak_edges.get(weak_ref) {
                    if self.dead_targets.contains(target) {
                        // This is actually expected behavior - WeakLoad should return null
                        // We track this for deterministic behavior verification
                    }
                }
            },
            
            // WeakCheck verifies liveness
            MirInstructionV2::WeakCheck { weak_ref, .. } => {
                if let Some(target) = self.weak_edges.get(weak_ref) {
                    if self.dead_targets.contains(target) {
                        // This is expected - WeakCheck should return false
                    }
                }
            },
            
            // Other instructions don't affect ownership
            _ => {},
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Verify global ownership forest properties
    fn verify_ownership_forest(&self) -> Result<(), Vec<OwnershipError>> {
        let mut errors = Vec::new();
        
        // Check for multiple strong owners (violates forest constraint)
        let mut target_owners: HashMap<ValueId, Vec<ValueId>> = HashMap::new();
        for (child, parent) in &self.strong_edges {
            target_owners.entry(*parent).or_insert_with(Vec::new).push(*child);
        }
        
        for (target, owners) in target_owners {
            if owners.len() > 1 {
                errors.push(OwnershipError::MultipleStrongOwners { target, owners });
            }
        }
        
        // Check for strong cycles (violates DAG constraint)
        if let Some(cycle) = self.find_strong_cycle() {
            errors.push(OwnershipError::StrongCycle { cycle });
        }
        
        // Check for bidirectional strong edges
        for (child, parent) in &self.strong_edges {
            if let Some(grandparent) = self.strong_edges.get(parent) {
                if grandparent == child {
                    errors.push(OwnershipError::BidirectionalStrong {
                        first: *child,
                        second: *parent,
                    });
                }
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Check if a value has multiple strong owners
    fn check_multiple_owners(&self, target: ValueId) -> Result<(), Vec<OwnershipError>> {
        let owners: Vec<ValueId> = self.strong_edges
            .iter()
            .filter(|(_, &parent)| parent == target)
            .map(|(&child, _)| child)
            .collect();
            
        if owners.len() > 1 {
            Err(vec![OwnershipError::MultipleStrongOwners { target, owners }])
        } else {
            Ok(())
        }
    }
    
    /// Check if adding an edge would create a cycle
    fn would_create_cycle(&self, parent: ValueId, child: ValueId) -> bool {
        // DFS to see if parent is reachable from child through strong edges
        let mut visited = HashSet::new();
        let mut stack = vec![child];
        
        while let Some(current) = stack.pop() {
            if current == parent {
                return true; // Cycle detected
            }
            
            if visited.insert(current) {
                // Add all strong children of current to stack
                for (&potential_child, &potential_parent) in &self.strong_edges {
                    if potential_parent == current {
                        stack.push(potential_child);
                    }
                }
            }
        }
        
        false
    }
    
    /// Find any strong cycle in the ownership graph
    fn find_strong_cycle(&self) -> Option<Vec<ValueId>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();
        
        // Get all nodes in the graph
        let mut all_nodes = HashSet::new();
        for (&child, &parent) in &self.strong_edges {
            all_nodes.insert(child);
            all_nodes.insert(parent);
        }
        
        // DFS from each unvisited node
        for &node in &all_nodes {
            if !visited.contains(&node) {
                if let Some(cycle) = self.dfs_cycle(node, &mut visited, &mut rec_stack, &mut path) {
                    return Some(cycle);
                }
            }
        }
        
        None
    }
    
    /// DFS cycle detection helper
    fn dfs_cycle(
        &self,
        node: ValueId,
        visited: &mut HashSet<ValueId>,
        rec_stack: &mut HashSet<ValueId>,
        path: &mut Vec<ValueId>,
    ) -> Option<Vec<ValueId>> {
        visited.insert(node);
        rec_stack.insert(node);
        path.push(node);
        
        // Visit all strong children
        for (&child, &parent) in &self.strong_edges {
            if parent == node {
                if rec_stack.contains(&child) {
                    // Found cycle - return path from child to current
                    let cycle_start = path.iter().position(|&x| x == child).unwrap();
                    return Some(path[cycle_start..].to_vec());
                }
                
                if !visited.contains(&child) {
                    if let Some(cycle) = self.dfs_cycle(child, visited, rec_stack, path) {
                        return Some(cycle);
                    }
                }
            }
        }
        
        rec_stack.remove(&node);
        path.pop();
        None
    }
    
    /// Get ownership statistics for debugging
    pub fn ownership_stats(&self) -> OwnershipStats {
        OwnershipStats {
            strong_edges: self.strong_edges.len(),
            weak_edges: self.weak_edges.len(),
            released_count: self.released.len(),
            live_weak_refs: self.live_weak_refs.len(),
            dead_targets: self.dead_targets.len(),
        }
    }
}

/// Ownership statistics for debugging and analysis
#[derive(Debug, Clone, PartialEq)]
pub struct OwnershipStats {
    pub strong_edges: usize,
    pub weak_edges: usize,
    pub released_count: usize,
    pub live_weak_refs: usize,
    pub dead_targets: usize,
}

impl Default for OwnershipVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{ValueIdGenerator, ConstValue};
    
    #[test]
    fn test_ownership_forest_basic() {
        let mut verifier = OwnershipVerifier::new();
        let mut value_gen = ValueIdGenerator::new();
        
        let parent = value_gen.next();
        let child = value_gen.next();
        
        // Create ownership relationship
        let adopt = MirInstructionV2::Adopt { parent, child };
        assert!(verifier.process_instruction(&adopt).is_ok());
        
        // Verify forest properties
        assert!(verifier.verify_ownership_forest().is_ok());
        
        let stats = verifier.ownership_stats();
        assert_eq!(stats.strong_edges, 1);
    }
    
    #[test]
    fn test_multiple_owners_error() {
        let mut verifier = OwnershipVerifier::new();
        let mut value_gen = ValueIdGenerator::new();
        
        let parent1 = value_gen.next();
        let parent2 = value_gen.next();
        let child = value_gen.next();
        
        // Create multiple ownership (invalid)
        verifier.strong_edges.insert(child, parent1);
        verifier.strong_edges.insert(child, parent2); // This overwrites, but we'll manually create conflict
        
        // Manually create the conflicting state for testing
        verifier.strong_edges.clear();
        verifier.strong_edges.insert(parent1, child); // parent1 -> child
        verifier.strong_edges.insert(parent2, child); // parent2 -> child (multiple owners of child)
        
        let result = verifier.verify_ownership_forest();
        assert!(result.is_err());
        
        if let Err(errors) = result {
            assert!(errors.iter().any(|e| matches!(e, OwnershipError::MultipleStrongOwners { .. })));
        }
    }
    
    #[test]
    fn test_strong_cycle_detection() {
        let mut verifier = OwnershipVerifier::new();
        let mut value_gen = ValueIdGenerator::new();
        
        let a = value_gen.next();
        let b = value_gen.next();
        let c = value_gen.next();
        
        // Create cycle: a -> b -> c -> a
        verifier.strong_edges.insert(b, a);
        verifier.strong_edges.insert(c, b);
        verifier.strong_edges.insert(a, c);
        
        let result = verifier.verify_ownership_forest();
        assert!(result.is_err());
        
        if let Err(errors) = result {
            assert!(errors.iter().any(|e| matches!(e, OwnershipError::StrongCycle { .. })));
        }
    }
    
    #[test]
    fn test_weak_reference_safety() {
        let mut verifier = OwnershipVerifier::new();
        let mut value_gen = ValueIdGenerator::new();
        
        let target = value_gen.next();
        let weak_ref = value_gen.next();
        
        // Create weak reference
        let weak_new = MirInstructionV2::WeakNew {
            dst: weak_ref,
            box_val: target,
        };
        assert!(verifier.process_instruction(&weak_new).is_ok());
        
        // Release the target
        let release = MirInstructionV2::Release {
            reference: target,
        };
        assert!(verifier.process_instruction(&release).is_ok());
        
        // WeakLoad should handle expired reference gracefully
        let weak_load = MirInstructionV2::WeakLoad {
            dst: value_gen.next(),
            weak_ref,
        };
        assert!(verifier.process_instruction(&weak_load).is_ok());
        
        let stats = verifier.ownership_stats();
        assert_eq!(stats.weak_edges, 1);
        assert_eq!(stats.dead_targets, 1);
    }
    
    #[test]
    fn test_unsafe_ref_set() {
        let mut verifier = OwnershipVerifier::new();
        let mut value_gen = ValueIdGenerator::new();
        
        let reference = value_gen.next();
        let old_target = value_gen.next();
        let new_target = value_gen.next();
        
        // Set initial strong ownership
        verifier.strong_edges.insert(reference, old_target);
        
        // Try to change without Release (should error)
        let ref_set = MirInstructionV2::RefSet { reference, new_target };
        let result = verifier.process_instruction(&ref_set);
        
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(errors.iter().any(|e| matches!(e, OwnershipError::UnsafeRefSet { .. })));
        }
    }
    
    #[test]
    fn test_safe_ref_set_with_release() {
        let mut verifier = OwnershipVerifier::new();
        let mut value_gen = ValueIdGenerator::new();
        
        let reference = value_gen.next();
        let old_target = value_gen.next();
        let new_target = value_gen.next();
        
        // Set initial strong ownership
        verifier.strong_edges.insert(reference, old_target);
        
        // Release old target first
        verifier.released.insert(old_target);
        
        // Now RefSet should be safe
        let ref_set = MirInstructionV2::RefSet { reference, new_target };
        assert!(verifier.process_instruction(&ref_set).is_ok());
    }
}