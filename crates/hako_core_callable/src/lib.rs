//! hako_core_callable — SSOT helpers for Callable surface
//!
//! Keep this crate free of runtime types; expose only minimal, pure helpers.

/// Whether async path should be considered enabled (dev convenience)
/// Delegates to env: HAKO_CALLABLE_ASYNC|NYASH_CALLABLE_ASYNC
pub fn async_enabled_from_env() -> bool {
    matches!(
        std::env::var("HAKO_CALLABLE_ASYNC").ok().as_deref(),
        Some("1") | Some("true") | Some("on")
    ) || matches!(
        std::env::var("NYASH_CALLABLE_ASYNC").ok().as_deref(),
        Some("1") | Some("true") | Some("on")
    )
}

/// Join policy hint: how many polls to attempt eagerly (tiny shim)
pub fn eager_poll_hint() -> usize {
    std::env::var("HAKO_CALLABLE_EAGER_POLL").ok().and_then(|s| s.parse().ok()).unwrap_or(0)
}


/// Generic argv flattener:
/// - If args has length 1 and `is_array` returns true for the first element,
///   produces a new Vec by calling `len` and `get` to expand elements.
/// - Otherwise, clones the original slice.
pub fn flatten_argv<U, IsArr, Len, Get>(args: &[U], is_array: IsArr, len: Len, get: Get) -> Vec<U>
where
    U: Clone,
    IsArr: Fn(&U) -> bool,
    Len: Fn(&U) -> usize,
    Get: Fn(&U, usize) -> U,
{
    if args.len() == 1 && is_array(&args[0]) {
        let a = &args[0];
        let n = len(a);
        let mut out = Vec::with_capacity(n);
        for i in 0..n { out.push(get(a, i)); }
        out
    } else {
        args.to_vec()
    }
}
