use crate::mir::ValueId;
use std::collections::HashSet;

/// ValueIdAllocatorBox — Centralized ValueId allocation with collision avoidance.
///
/// **Purpose**: Unify 117 `value_gen.next()` call sites into a single allocation point.
///
/// **4-Layer Collision Check**:
/// 1. Function parameters (v%0-v%N) - never reuse parameter registers
/// 2. variable_map values - never reuse existing local variables
/// 3. value_types keys - never reuse defined ValueIds (prevents SSA violations)
/// 4. local_ssa_map values - never reuse cached local SSA copies
///
/// **ENV Toggle**: `HAKO_USE_VALUE_ALLOCATOR_BOX=1` to enable (default: disabled)
///
/// **Trace**: `HAKO_TRACE_VALUE_ALLOC=1` for detailed allocation logging
///
/// **Integration**: Use `MirBuilder::safe_next_value()` instead of `value_gen.next()`
pub struct ValueIdAllocatorBox {
    /// Next ValueId candidate
    next_id: u32,

    /// In-use ValueIds (synchronized from builder state)
    in_use: HashSet<ValueId>,

    /// Trace logging enabled
    trace_enabled: bool,
}

impl ValueIdAllocatorBox {
    /// Create a new ValueIdAllocatorBox.
    ///
    /// **param_count**: Number of function parameters (to start allocation at v%(N+1))
    pub fn new(param_count: usize) -> Self {
        let trace_enabled = crate::runtime::env_gate_box::bool_any(&[
            "HAKO_TRACE_VALUE_ALLOC",
            "NYASH_TRACE_VALUE_ALLOC",
        ]);

        Self {
            next_id: param_count as u32,
            in_use: HashSet::new(),
            trace_enabled,
        }
    }

    /// Update the parameter count (sets next_id to start after parameters).
    ///
    /// **param_count**: Number of function parameters (v%0-v%N reserved)
    pub fn set_param_count(&mut self, param_count: u32) {
        // Only update if new param_count is higher (don't regress allocation point)
        if param_count > self.next_id {
            if self.trace_enabled {
                eprintln!(
                    "[value-alloc] 🔧 set_param_count: {} -> {}",
                    self.next_id, param_count
                );
            }
            self.next_id = param_count;
        }
    }

    /// Allocate a safe ValueId with 4-layer collision avoidance.
    ///
    /// **Returns**: A fresh ValueId guaranteed not to collide with:
    /// - Function parameters
    /// - variable_map entries
    /// - value_types entries
    /// - local_ssa_map entries
    ///
    /// **Panics**: After 1000 failed allocation attempts (indicates infinite loop)
    pub fn allocate_safe(
        &mut self,
        current_function: Option<&crate::mir::MirFunction>,
        variable_map: &std::collections::HashMap<String, ValueId>,
        value_types: &std::collections::HashMap<ValueId, crate::mir::MirType>,
        local_ssa_map: &std::collections::HashMap<(crate::mir::BasicBlockId, ValueId, u8), ValueId>,
    ) -> ValueId {
        // Synchronize in-use set from builder state
        self.sync_all(current_function, variable_map, value_types, local_ssa_map);

        let mut attempts = 0;
        loop {
            let candidate = ValueId::new(self.next_id);
            self.next_id += 1;
            attempts += 1;

            if self.is_available(candidate) {
                self.in_use.insert(candidate);

                if self.trace_enabled {
                    eprintln!(
                        "[value-alloc] ✅ allocated v%{} (attempts={}, in_use={})",
                        candidate.0,
                        attempts,
                        self.in_use.len()
                    );
                }

                return candidate;
            }

            if attempts > 1000 {
                panic!(
                    "ValueId allocation failed after 1000 attempts (in_use={}, next_id={})",
                    self.in_use.len(),
                    self.next_id
                );
            }
        }
    }

    /// Synchronize in-use set from builder state (4-layer check).
    fn sync_all(
        &mut self,
        current_function: Option<&crate::mir::MirFunction>,
        variable_map: &std::collections::HashMap<String, ValueId>,
        value_types: &std::collections::HashMap<ValueId, crate::mir::MirType>,
        local_ssa_map: &std::collections::HashMap<(crate::mir::BasicBlockId, ValueId, u8), ValueId>,
    ) {
        self.in_use.clear();

        // Layer 1: Function parameters
        if let Some(fun) = current_function {
            for param in &fun.params {
                self.in_use.insert(*param);
            }
        }

        // Layer 2: variable_map values
        for vid in variable_map.values() {
            self.in_use.insert(*vid);
        }

        // Layer 3: value_types keys
        for vid in value_types.keys() {
            self.in_use.insert(*vid);
        }

        // Layer 4: local_ssa_map values
        for vid in local_ssa_map.values() {
            self.in_use.insert(*vid);
        }

        if self.trace_enabled {
            eprintln!(
                "[value-alloc] 🔄 sync_all: in_use={} (params={}, varmap={}, types={}, ssa={})",
                self.in_use.len(),
                current_function.map(|f| f.params.len()).unwrap_or(0),
                variable_map.len(),
                value_types.len(),
                local_ssa_map.len()
            );
        }
    }

    /// Check if a candidate ValueId is available (not in use).
    #[inline]
    fn is_available(&self, candidate: ValueId) -> bool {
        !self.in_use.contains(&candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::builder::MirBuilder;

    #[test]
    fn test_allocate_safe_basic() {
        let builder = MirBuilder::new();
        let mut allocator = ValueIdAllocatorBox::new(3);

        let v1 = allocator.allocate_safe(
            builder.current_function.as_ref(),
            &builder.variable_map,
            &builder.value_types,
            &builder.local_ssa_map,
        );
        assert_eq!(v1.0, 3, "Should skip params v%0-v%2");

        let v2 = allocator.allocate_safe(
            builder.current_function.as_ref(),
            &builder.variable_map,
            &builder.value_types,
            &builder.local_ssa_map,
        );
        assert_eq!(v2.0, 4);
    }

    #[test]
    fn test_collision_avoidance() {
        let mut builder = MirBuilder::new();

        // Simulate variable_map entry at v%5
        builder.variable_map.insert("i".to_string(), ValueId::new(5));

        let mut allocator = ValueIdAllocatorBox::new(3);

        // Allocate several values
        let v1 = allocator.allocate_safe(
            builder.current_function.as_ref(),
            &builder.variable_map,
            &builder.value_types,
            &builder.local_ssa_map,
        );
        assert_eq!(v1.0, 3);

        let v2 = allocator.allocate_safe(
            builder.current_function.as_ref(),
            &builder.variable_map,
            &builder.value_types,
            &builder.local_ssa_map,
        );
        assert_eq!(v2.0, 4);

        // Next should skip v%5 (occupied by variable_map)
        let v3 = allocator.allocate_safe(
            builder.current_function.as_ref(),
            &builder.variable_map,
            &builder.value_types,
            &builder.local_ssa_map,
        );
        assert_eq!(v3.0, 6, "Should skip v%5 (occupied by variable_map)");
    }

    #[test]
    fn test_sync_clears_previous_state() {
        let builder = MirBuilder::new();
        let mut allocator = ValueIdAllocatorBox::new(0);

        // First allocation
        let v1 = allocator.allocate_safe(
            builder.current_function.as_ref(),
            &builder.variable_map,
            &builder.value_types,
            &builder.local_ssa_map,
        );
        assert_eq!(v1.0, 0);

        // in_use should be cleared and re-synced
        let v2 = allocator.allocate_safe(
            builder.current_function.as_ref(),
            &builder.variable_map,
            &builder.value_types,
            &builder.local_ssa_map,
        );
        assert_eq!(v2.0, 1);
    }
}
