/*!
 * Provider Lock (skeleton)
 *
 * Phase 15.5 受け口: 型→Provider のロック状態を保持するための最小スケルトン。
 * 既定では挙動を変えず、環境変数により警告/エラー化のみ可能にする。
 */

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

static LOCKED: AtomicBool = AtomicBool::new(false);
static WARN_ONCE: OnceLock<()> = OnceLock::new();

/// Return true when providers are locked
pub fn is_locked() -> bool { LOCKED.load(Ordering::Relaxed) }

/// Lock providers (idempotent)
pub fn lock_providers() { LOCKED.store(true, Ordering::Relaxed); }

/// Guard called before creating a new box instance.
/// Default: no-op. When NYASH_PROVIDER_LOCK_STRICT=1, returns Err if not locked.
/// When NYASH_PROVIDER_LOCK_WARN=1, prints a warning once.
pub fn guard_before_new_box(box_type: &str) -> Result<(), String> {
    if is_locked() { return Ok(()); }
    let strict = std::env::var("NYASH_PROVIDER_LOCK_STRICT").ok().as_deref() == Some("1");
    let warn = std::env::var("NYASH_PROVIDER_LOCK_WARN").ok().as_deref() == Some("1");
    if strict {
        return Err(format!("E_PROVIDER_NOT_LOCKED: attempted to create '{}' before Provider Lock", box_type));
    }
    if warn {
        // Print once per process
        let _ = WARN_ONCE.get_or_init(|| {
            if !crate::config::env::cli_quiet() {
                eprintln!("{}", crate::common::diagnostics::provider_lock_warn());
            }
        });
    }
    Ok(())
}
