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

// Phase 15.5: Unified plugin-based slot resolution (core box special handling removed)
// All boxes (including former "core" boxes) now use plugin-based slot assignment
// Static builtin slots are deprecated in favor of nyash.toml configuration
static BUILTIN_SLOTS: Lazy<HashMap<&'static str, Vec<(&'static str, MethodSlot)>>> =
    Lazy::new(|| {
        use std::iter::FromIterator;
        let mut m = HashMap::new();
        // Phase 15.5: Core boxes removed, all slots come from nyash.toml
        // Former core boxes (StringBox, IntegerBox, ArrayBox, MapBox) now use plugin slots

        // Common plugin boxes (reference examples only)
        m.insert(
            "FileBox",
            vec![("open", 4), ("read", 5), ("write", 6), ("close", 7)],
        );
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
        // Phase 15.5: Test unified plugin-based slot reservation
        let tid = get_or_assign_type_id("TestBox");
        reserve_method_slot(tid, "custom_method", 8);
        assert_eq!(resolve_slot(tid, "custom_method"), Some(8));
    }

    #[test]
    fn test_phase_15_5_unified_resolution() {
        // Phase 15.5: Former core boxes now use plugin-based resolution
        let string_tid = get_or_assign_type_id("StringBox");

        // Universal slots still work
        assert_eq!(resolve_slot(string_tid, "toString"), Some(0));
        assert_eq!(resolve_slot(string_tid, "type"), Some(1));

        // Former builtin slots (substring, concat) are no longer auto-assigned
        assert_eq!(resolve_slot(string_tid, "substring"), None);
        assert_eq!(resolve_slot(string_tid, "concat"), None);

        // Must be explicitly reserved (as plugins do)
        reserve_method_slot(string_tid, "get", 4);
        reserve_method_slot(string_tid, "set", 5);
        assert_eq!(resolve_slot(string_tid, "get"), Some(4));
        assert_eq!(resolve_slot(string_tid, "set"), Some(5));
    }
}
