/*!
 * VM GC Roots & Diagnostics (extracted from vm.rs)
 *
 * - Root region helpers: enter_root_region / pin_roots / leave_root_region
 * - Site info for GC logs
 * - Debug prints for roots snapshot and shallow reachability
 */

use super::vm::{VM, VMError, VMValue};

impl VM {
    /// Enter a GC root region and return a guard that leaves on drop
    pub(super) fn enter_root_region(&mut self) {
        self.scope_tracker.enter_root_region();
    }

    /// Pin a slice of VMValue as roots in the current region
    pub(super) fn pin_roots<'a>(&mut self, values: impl IntoIterator<Item = &'a VMValue>) {
        for v in values {
            self.scope_tracker.pin_root(v);
        }
    }

    /// Leave current GC root region
    pub(super) fn leave_root_region(&mut self) { self.scope_tracker.leave_root_region(); }

    /// Site info for GC logs: (func, bb, pc)
    pub(super) fn gc_site_info(&self) -> (String, i64, i64) {
        let func = self.current_function.as_deref().unwrap_or("<none>").to_string();
        let bb = self.frame.current_block.map(|b| b.0 as i64).unwrap_or(-1);
        let pc = self.frame.pc as i64;
        (func, bb, pc)
    }

    /// Print a simple breakdown of root VMValue kinds and top BoxRef types
    pub(super) fn gc_print_roots_breakdown(&self) {
        use std::collections::HashMap;
        let roots = self.scope_tracker.roots_snapshot();
        let mut kinds: HashMap<&'static str, u64> = HashMap::new();
        let mut box_types: HashMap<String, u64> = HashMap::new();
        for v in &roots {
            match v {
                VMValue::Integer(_) => *kinds.entry("Integer").or_insert(0) += 1,
                VMValue::Float(_) => *kinds.entry("Float").or_insert(0) += 1,
                VMValue::Bool(_) => *kinds.entry("Bool").or_insert(0) += 1,
                VMValue::String(_) => *kinds.entry("String").or_insert(0) += 1,
                VMValue::Future(_) => *kinds.entry("Future").or_insert(0) += 1,
                VMValue::Void => *kinds.entry("Void").or_insert(0) += 1,
                VMValue::BoxRef(b) => {
                    let tn = b.type_name().to_string();
                    *box_types.entry(tn).or_insert(0) += 1;
                }
            }
        }
        eprintln!("[GC] roots_breakdown: kinds={:?}", kinds);
        let mut top: Vec<(String, u64)> = box_types.into_iter().collect();
        top.sort_by(|a, b| b.1.cmp(&a.1));
        top.truncate(5);
        eprintln!("[GC] roots_boxref_top5: {:?}", top);
    }

    /// Print shallow reachability from current roots into ArrayBox/MapBox values
    pub(super) fn gc_print_reachability_depth2(&self) {
        use std::collections::HashMap;
        let roots = self.scope_tracker.roots_snapshot();
        let mut child_types: HashMap<String, u64> = HashMap::new();
        let mut child_count = 0u64;
        for v in &roots {
            if let VMValue::BoxRef(b) = v {
                if let Some(arr) = b.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                    if let Ok(items) = arr.items.read() {
                        for item in items.iter() {
                            let tn = item.type_name().to_string();
                            *child_types.entry(tn).or_insert(0) += 1;
                            child_count += 1;
                        }
                    }
                }
                if let Some(map) = b.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                    let vals = map.values();
                    if let Some(arr2) = vals.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                        if let Ok(items) = arr2.items.read() {
                            for item in items.iter() {
                                let tn = item.type_name().to_string();
                                *child_types.entry(tn).or_insert(0) += 1;
                                child_count += 1;
                            }
                        }
                    }
                }
            }
        }
        let mut top: Vec<(String, u64)> = child_types.into_iter().collect();
        top.sort_by(|a, b| b.1.cmp(&a.1));
        top.truncate(5);
        eprintln!("[GC] depth2_children: total={} top5={:?}", child_count, top);
    }
}

