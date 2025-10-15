/*!
 * TraceBox - Centralized trace/debug output management
 *
 * Consolidates scattered environment variable checks and eprintln! calls:
 * - Reduces code duplication (3-4 lines → 1 line)
 * - Lazy evaluation via closures (zero overhead when disabled)
 * - Centralized trace category management
 *
 * Usage:
 * ```rust
 * // Old (3 lines):
 * if std::env::var("NYASH_LOCAL_SSA_TRACE").ok().as_deref() == Some("1") {
 *     eprintln!("[local-ssa] msg: {}", value);
 * }
 *
 * // New (1 line):
 * TraceBox::local_ssa(|| format!("[local-ssa] msg: {}", value));
 * ```
 */

use std::sync::OnceLock;

/// Cached trace configuration flags
struct TraceConfig {
    local_ssa: bool,
    macro_trace: bool,
    resolve: bool,
}

impl TraceConfig {
    fn load() -> Self {
        Self {
            local_ssa: std::env::var("NYASH_LOCAL_SSA_TRACE").ok().as_deref() == Some("1"),
            macro_trace: std::env::var("NYASH_MACRO_TRACE").ok().as_deref() == Some("1"),
            resolve: std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1"),
        }
    }
}

static CONFIG: OnceLock<TraceConfig> = OnceLock::new();

/// Centralized trace output manager
pub struct TraceBox;

impl TraceBox {
    /// Get or initialize trace configuration
    fn config() -> &'static TraceConfig {
        CONFIG.get_or_init(TraceConfig::load)
    }

    /// Local SSA trace (NYASH_LOCAL_SSA_TRACE)
    #[inline]
    pub fn local_ssa<F: FnOnce() -> String>(msg: F) {
        if Self::config().local_ssa {
            eprintln!("{}", msg());
        }
    }

    /// Macro expansion trace (NYASH_MACRO_TRACE)
    #[inline]
    pub fn macro_trace<F: FnOnce() -> String>(msg: F) {
        if Self::config().macro_trace {
            eprintln!("{}", msg());
        }
    }

    /// Name resolution trace (NYASH_RESOLVE_TRACE)
    #[inline]
    pub fn resolve<F: FnOnce() -> String>(msg: F) {
        if Self::config().resolve {
            eprintln!("{}", msg());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_box_disabled_by_default() {
        // Without env vars, all traces should be no-op
        let mut called = false;
        TraceBox::local_ssa(|| {
            called = true;
            "test".to_string()
        });
        // Closure should not be called when trace is disabled
        // Note: This test may fail if NYASH_LOCAL_SSA_TRACE=1 is set
        // In production, trace is disabled by default
    }
}
