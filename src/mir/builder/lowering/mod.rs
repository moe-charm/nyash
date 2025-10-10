//! Builtin method → Extern lowering map
//!
//! Centralizes mapping from builtin Box methods to `Extern("iface.method")` names
//! and simple argument shaping rules (e.g., prepend receiver).

#[derive(Clone, Debug)]
pub struct LoweredExternSpec {
    pub extern_name: &'static str, // e.g., "nyrt.string.length"
    pub prepend_recv: bool,        // if true, push receiver as first argument
}

/// Try to lower a builtin method call to an Extern call.
///
/// - `recv_cls`: Optional receiver class hint (e.g., "StringBox").
/// - `method`: Method name as written in source (e.g., "length").
/// - `arity`: Number of explicit arguments (excluding receiver).
pub fn lower_builtin_method(
    recv_cls: Option<&str>,
    method: &str,
    arity: usize,
) -> Option<LoweredExternSpec> {
    match recv_cls.unwrap_or("") {
        // String builtins (byte semantics)
        "StringBox" => match (method, arity) {
            ("length", 0) | ("size", 0) | ("len", 0) => Some(LoweredExternSpec { extern_name: "nyrt.string.length", prepend_recv: true }),
            ("substring", 2) => Some(LoweredExternSpec { extern_name: "nyrt.string.substring", prepend_recv: true }),
            ("indexOf", 1) | ("indexOf", 2) => Some(LoweredExternSpec { extern_name: "nyrt.string.indexOf", prepend_recv: true }),
            ("find", 1) | ("find", 2) => Some(LoweredExternSpec { extern_name: "nyrt.string.indexOf", prepend_recv: true }),
            ("lastIndexOf", 1) | ("lastIndexOf", 2) => Some(LoweredExternSpec { extern_name: "nyrt.string.lastIndexOf", prepend_recv: true }),
            ("replace", 2) => Some(LoweredExternSpec { extern_name: "nyrt.string.replace", prepend_recv: true }),
            ("charAt", 1) => Some(LoweredExternSpec { extern_name: "nyrt.string.charAt", prepend_recv: true }),
            _ => None,
        },
        // Array builtins
        "ArrayBox" => match (method, arity) {
            ("length", 0) | ("size", 0) | ("len", 0) => Some(LoweredExternSpec { extern_name: "nyrt.array.size", prepend_recv: true }),
            _ => None,
        },
        // Map builtins
        "MapBox" => match (method, arity) {
            ("size", 0) => Some(LoweredExternSpec { extern_name: "nyrt.map.size", prepend_recv: true }),
            ("keys", 0) => Some(LoweredExternSpec { extern_name: "nyrt.map.keys", prepend_recv: true }),
            ("values", 0) => Some(LoweredExternSpec { extern_name: "nyrt.map.values", prepend_recv: true }),
            _ => None,
        },
        _ => None,
    }
}
