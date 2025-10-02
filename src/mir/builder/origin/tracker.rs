use crate::mir::ValueId;
use std::collections::HashMap;

/// 薄い Origin 追跡箱。NYASH_ORIGIN_TRACE=1 でトレース。
pub struct OriginTrackerBox<'a> {
    map: &'a mut HashMap<ValueId, String>,
    trace: bool,
}

impl<'a> OriginTrackerBox<'a> {
    pub fn new(map: &'a mut HashMap<ValueId, String>, trace: bool) -> Self {
        Self { map, trace }
    }

    pub fn register_newbox<S: Into<String>>(&mut self, value_id: ValueId, class_name: S) {
        let cls = class_name.into();
        if self.trace {
            eprintln!("[OriginTracker] register v%{} = {}", value_id.0, cls);
        }
        self.map.insert(value_id, cls);
    }

    pub fn propagate(&mut self, from: ValueId, to: ValueId) {
        if let Some(origin) = self.map.get(&from).cloned() {
            if self.trace {
                eprintln!("[OriginTracker] propagate v%{} → v%{} ({})", from.0, to.0, origin);
            }
            self.map.insert(to, origin);
        }
    }

    pub fn get(&self, value_id: ValueId) -> Option<&str> {
        self.map.get(&value_id).map(|s| s.as_str())
    }
}
