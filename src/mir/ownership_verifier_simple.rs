/*!
 * Ownership Forest Verification System (Simplified for Current MIR)
 * 
 * Basic implementation working with current MirInstruction enum
 * Will be expanded when MirInstructionV2 is integrated
 */

use super::{MirInstruction, ValueId, MirFunction, MirModule};
use std::collections::{HashMap, HashSet};

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
    
    /// RefSet without proper Release of old target
    UnsafeRefSet {
        reference: ValueId,
        old_target: ValueId,
        new_target: ValueId,
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
    pub fn process_instruction(&mut self, instruction: &MirInstruction) -> Result<(), Vec<OwnershipError>> {
        let mut errors = Vec::new();
        
        match instruction {
            // NewBox creates a new ownership root
            MirInstruction::NewBox { dst, .. } => {
                // New boxes are ownership roots (no parent)
                self.strong_edges.remove(dst);
            },
            
            // RefSet changes ownership relationships  
            MirInstruction::RefSet { reference, field: _, value } => {
                // Check if the reference currently has a strong target
                if let Some(old_target) = self.strong_edges.get(reference) {
                    // Strong→Strong replacement requires explicit Release
                    if !self.released.contains(old_target) {
                        errors.push(OwnershipError::UnsafeRefSet {
                            reference: *reference,
                            old_target: *old_target,
                            new_target: *value,
                        });
                    }
                }
                
                // Set new strong ownership
                self.strong_edges.insert(*reference, *value);
                
                // Verify no multiple owners after this change
                if let Err(mut multiple_errors) = self.check_multiple_owners(*value) {
                    errors.append(&mut multiple_errors);
                }
            },
            
            // WeakNew creates weak reference
            MirInstruction::WeakNew { dst, box_val } => {
                self.weak_edges.insert(*dst, *box_val);
                self.live_weak_refs.insert(*dst);
            },
            
            // WeakLoad checks liveness
            MirInstruction::WeakLoad { weak_ref, .. } => {
                if let Some(target) = self.weak_edges.get(weak_ref) {
                    if self.dead_targets.contains(target) {
                        // This is expected behavior - WeakLoad should return null deterministically
                    }
                }
            },
            
            // Other instructions don't affect ownership in current implementation
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
    use crate::mir::ValueIdGenerator;
    
    #[test]
    fn test_ownership_forest_basic() {
        let mut verifier = OwnershipVerifier::new();
        let mut value_gen = ValueIdGenerator::new();
        
        let parent = value_gen.next();
        let child = value_gen.next();
        
        // Create NewBox instruction (current MirInstruction)
        let new_box = MirInstruction::NewBox { 
            dst: parent, 
            box_type: "TestBox".to_string(), 
            args: vec![] 
        };
        assert!(verifier.process_instruction(&new_box).is_ok());
        
        // Verify forest properties
        assert!(verifier.verify_ownership_forest().is_ok());
        
        let stats = verifier.ownership_stats();
        assert_eq!(stats.strong_edges, 0); // NewBox doesn't create edges, just roots
    }
    
    #[test]
    fn test_weak_reference_tracking() {
        let mut verifier = OwnershipVerifier::new();
        let mut value_gen = ValueIdGenerator::new();
        
        let target = value_gen.next();
        let weak_ref = value_gen.next();
        
        // Create weak reference
        let weak_new = MirInstruction::WeakNew { dst: weak_ref, box_val: target };
        assert!(verifier.process_instruction(&weak_new).is_ok(), "Weak reference creation should succeed");
        
        let stats = verifier.ownership_stats();
        assert_eq!(stats.weak_edges, 1, "Should have one weak edge");
        assert_eq!(stats.live_weak_refs, 1, "Should have one live weak reference");
        
        // WeakLoad should handle gracefully
        let weak_load = MirInstruction::WeakLoad { dst: value_gen.next(), weak_ref };
        assert!(verifier.process_instruction(&weak_load).is_ok(), "WeakLoad should succeed");
    }
    
    #[test]
    fn test_basic_ref_set() {
        let mut verifier = OwnershipVerifier::new();
        let mut value_gen = ValueIdGenerator::new();
        
        let reference = value_gen.next();
        let target = value_gen.next();
        
        // Basic RefSet without prior ownership (should succeed)
        let ref_set = MirInstruction::RefSet { 
            reference, 
            field: "test".to_string(), 
            value: target 
        };
        assert!(verifier.process_instruction(&ref_set).is_ok(), "Basic RefSet should succeed");
        
        let stats = verifier.ownership_stats();
        assert_eq!(stats.strong_edges, 1, "Should have one strong edge");
    }
}