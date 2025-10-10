//! hako_core_map — shared helpers for Map semantics

/// Return size as i64
pub fn size(len: usize) -> i64 { len as i64 }


use std::collections::HashMap;

/// Merge i64 and String keys into a single Vec<String> and sort lexicographically (dictionary order).
/// This is the canonical ordering used by Map.keys() and Map.values() alignment.
pub fn keys_sorted_from_maps_i64_str<V1, V2>(
    m_i64: &HashMap<i64, V1>,
    m_str: &HashMap<String, V2>,
) -> Vec<String> {
    let mut keys: Vec<String> = Vec::with_capacity(m_i64.len() + m_str.len());
    for k in m_i64.keys() { keys.push(k.to_string()); }
    for k in m_str.keys() { keys.push(k.clone()); }
    keys.sort();
    keys
}


/// Return values aligned to the given keys. For each key in `keys`, look up
/// the corresponding entry in `map_i64` (when the key parses as i64) or
/// `map_str` (otherwise). Missing entries yield None. This is purely a
/// structural helper (no stringification or formatting).
pub fn values_for_keys<'a, V>(
    keys: &[String],
    map_i64: &'a HashMap<i64, V>,
    map_str: &'a HashMap<String, V>,
) -> Vec<Option<&'a V>> {
    let mut out = Vec::with_capacity(keys.len());
    for k in keys.iter() {
        if let Ok(ik) = k.parse::<i64>() {
            out.push(map_i64.get(&ik));
        } else {
            out.push(map_str.get(k));
        }
    }
    out
}



/// Return whether `map` contains the given string key.
pub fn has_key_str<V>(map: &std::collections::HashMap<String, V>, key: &str) -> bool {
    map.contains_key(key)
}

/// Return map size (string-keyed) as i64, consistent with Map.size() policy.
pub fn size_of_str_map<V>(map: &std::collections::HashMap<String, V>) -> i64 {
    map.len() as i64
}
/// Variant for builtin-style single String-keyed map: return sorted keys.
pub fn keys_sorted_from_map_str<V>(m: &std::collections::HashMap<String, V>) -> Vec<String> {
    let mut keys: Vec<String> = m.keys().cloned().collect();
    keys.sort();
    keys
}

/// Variant for builtin-style single String-keyed map: align values by keys.
pub fn values_for_keys_str<'a, V>(
    keys: &[String],
    map: &'a std::collections::HashMap<String, V>,
) -> Vec<Option<&'a V>> {
    let mut out = Vec::with_capacity(keys.len());
    for k in keys.iter() {
        out.push(map.get(k));
    }
    out
}
