/*!
 * MIR Slot Registry (Phase 9.79b.1)
 *
 * Provides numeric BoxTypeId assignment and per-type method slot resolution.
 * - Low slots [0..3] are universally reserved: 0=toString, 1=type, 2=equals, 3=clone
 * - Exposes minimal APIs for the MIR builder to resolve method slots when
 *   the receiver type is known at build time.
 */

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

pub type BoxTypeId = u32;
pub type MethodSlot = u16;

// Global maps (scoped to compiler process)
static TYPE_IDS: Lazy<Mutex<HashMap<String, BoxTypeId>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static NEXT_TYPE_ID: Lazy<Mutex<BoxTypeId>> = Lazy::new(|| Mutex::new(100)); // start after small reserved area

// Per-type explicit slot reservations: (type_id, method) -> slot
static EXPLICIT_SLOTS: Lazy<Mutex<HashMap<(BoxTypeId, String), MethodSlot>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// Builtin type -> (method, slot) static table (slots start at 4; 0..3 are universal)
static BUILTIN_SLOTS: Lazy<HashMap<&'static str, Vec<(&'static str, MethodSlot)>>> = Lazy::new(|| {
    use std::iter::FromIterator;
    let mut m = HashMap::new();
    m.insert("ArrayBox", vec![
        ("push", 4), ("pop", 5), ("length", 6), ("len", 6), ("get", 7), ("set", 8),
        ("remove", 9), ("contains", 10), ("indexOf", 11), ("clear", 12), ("join", 13),
        ("sort", 14), ("reverse", 15), ("slice", 16)
    ]);
    m.insert("MapBox", vec![
        ("set", 4), ("get", 5), ("has", 6), ("delete", 7), ("remove", 7), ("keys", 8),
        ("values", 9), ("size", 10), ("clear", 11)
    ]);
    m.insert("IntegerBox", vec![("abs", 4)]);
    m.insert("StringBox", vec![("substring", 4), ("concat", 5)]);
    // Common plugin boxes (minimal seed)
    m.insert("FileBox", vec![ ("open", 4), ("read", 5), ("write", 6), ("close", 7) ]);
    HashMap::from_iter(m)
});

// Universal slots mapping for quick checks
fn universal_slot(method: &str) -> Option<MethodSlot> {
    match method {
        "toString" => Some(0),
        "type" => Some(1),
        "equals" => Some(2),
        "clone" => Some(3),
        _ => None,
    }
}

/// Get or assign a numeric BoxTypeId for a given type name.
pub fn get_or_assign_type_id(type_name: &str) -> BoxTypeId {
    let mut map = TYPE_IDS.lock().unwrap();
    if let Some(&id) = map.get(type_name) {
        return id;
    }
    let mut next = NEXT_TYPE_ID.lock().unwrap();
    let id = *next;
    *next += 1;
    map.insert(type_name.to_string(), id);
    id
}

/// Reserve a method slot for a given (type_id, method) pair.
/// If the method is one of the universal methods, the reservation is ignored
/// as universal slots are implicitly enforced for all types.
pub fn reserve_method_slot(type_id: BoxTypeId, method: &str, slot: MethodSlot) {
    if universal_slot(method).is_some() {
        return; // universal slots are global invariants
    }
    let mut table = EXPLICIT_SLOTS.lock().unwrap();
    table.insert((type_id, method.to_string()), slot);
}

/// Resolve a method slot given numeric type id and method name.
pub fn resolve_slot(type_id: BoxTypeId, method: &str) -> Option<MethodSlot> {
    // Universal first
    if let Some(s) = universal_slot(method) {
        return Some(s);
    }
    let table = EXPLICIT_SLOTS.lock().unwrap();
    table.get(&(type_id, method.to_string())).copied()
}

/// Resolve a method slot given a type name and method name.
pub fn resolve_slot_by_type_name(type_name: &str, method: &str) -> Option<MethodSlot> {
    let ty = get_or_assign_type_id(type_name);
    // Seed builtin slots lazily
    seed_builtin_slots(ty, type_name);
    resolve_slot(ty, method)
}

/// Minimal MIR Debug Info scaffold to map IDs back to names (off by default).
#[derive(Default, Debug, Clone)]
pub struct MIRDebugInfo {
    // Optionally carry reverse maps when enabled in the future.
}

/// Seed builtin slots for a type name if present in the builtin table
fn seed_builtin_slots(type_id: BoxTypeId, type_name: &str) {
    if let Some(entries) = BUILTIN_SLOTS.get(type_name) {
        let mut table = EXPLICIT_SLOTS.lock().unwrap();
        for (name, slot) in entries.iter() {
            table.entry((type_id, (*name).to_string())).or_insert(*slot);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universal_slots_reserved() {
        let tid = get_or_assign_type_id("StringBox");
        assert_eq!(resolve_slot(tid, "toString"), Some(0));
        assert_eq!(resolve_slot(tid, "type"), Some(1));
        assert_eq!(resolve_slot(tid, "equals"), Some(2));
        assert_eq!(resolve_slot(tid, "clone"), Some(3));
    }

    #[test]
    fn test_explicit_slot_reservation() {
        let tid = get_or_assign_type_id("ArrayBox");
        reserve_method_slot(tid, "push", 8);
        assert_eq!(resolve_slot(tid, "push"), Some(8));
    }
}
