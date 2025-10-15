/*!
 * core_box_methods – known arities for core boxes (Array/Map/String) as a minimal, shared table.
 * This is a fallback helper (authoritative source may remain in runtime::type_registry).
 */

pub fn known_arities_for(box_name: &str, method: &str) -> Option<&'static [usize]> {
    match (box_name, method) {
        // ArrayBox
        ("ArrayBox", "get") => Some(&[1]),
        ("ArrayBox", "set") => Some(&[2]),
        ("ArrayBox", "len") | ("ArrayBox", "length") | ("ArrayBox", "size") => Some(&[0]),
        ("ArrayBox", "isEmpty") => Some(&[0]),
        // MapBox
        ("MapBox", "get") => Some(&[1]),
        ("MapBox", "set") => Some(&[2]),
        ("MapBox", "has") => Some(&[1]),
        ("MapBox", "len") | ("MapBox", "size") => Some(&[0]),
        ("MapBox", "isEmpty") => Some(&[0]),
        // StringBox
        ("StringBox", "len") | ("StringBox", "length") | ("StringBox", "size") => Some(&[0]),
        ("StringBox", "isEmpty") => Some(&[0]),
        ("StringBox", "indexOf") => Some(&[2]),
        ("StringBox", "substring") => Some(&[2]),
        ("StringBox", "charAt") => Some(&[1]),
        _ => None,
    }
}
