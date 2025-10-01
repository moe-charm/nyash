/*!
 * VM Engine Interface — Single entry to execute MIR via a chosen VM engine.
 *
 * Phase‑A: unify the call site behind a trait. Default engine is the existing
 * lightweight MIR interpreter (fallback). Full VM will be introduced behind
 * an env toggle in later phases.
 */

use crate::{backend::MirInterpreter, mir::MirModule};

/// Unified VM engine trait
pub trait VmEngine {
    fn execute(&mut self, module: &MirModule) -> Result<i32, String>;
}

/// Fallback VM engine (wraps the lightweight MIR interpreter)
pub struct FallbackVmEngine {
    interp: MirInterpreter,
}

impl FallbackVmEngine {
    pub fn new() -> Self {
        Self { interp: MirInterpreter::new() }
    }
}

impl VmEngine for FallbackVmEngine {
    fn execute(&mut self, module: &MirModule) -> Result<i32, String> {
        match self.interp.execute_module(module) {
            Ok(ret) => {
                // Map return value to process exit code (0..255).
                // Integer/Bool → code, others → 0.
                let code_i64 = crate::runtime::semantics::coerce_to_i64(ret.as_ref()).unwrap_or(0);
                let code = (code_i64 as i32) & 0xFF;
                Ok(code)
            }
            Err(e) => Err(format!("VM fallback error: {}", e)),
        }
    }
}

/// Full VM engine placeholder (Phase‑B/C will implement). Currently returns an error.
pub struct FullVmEngine;

impl FullVmEngine {
    #[allow(dead_code)]
    pub fn new() -> Self { Self }
}

impl VmEngine for FullVmEngine {
    fn execute(&mut self, _module: &MirModule) -> Result<i32, String> {
        Err("Full VM engine not implemented yet (NYASH_VM_ENGINE=full)".to_string())
    }
}

/// Factory: choose engine by env (default=fallback)
pub fn vm_engine_from_env() -> Box<dyn VmEngine> {
    match std::env::var("NYASH_VM_ENGINE").ok().as_deref() {
        Some("full") => Box::new(FullVmEngine::new()),
        _ => Box::new(FallbackVmEngine::new()),
    }
}
