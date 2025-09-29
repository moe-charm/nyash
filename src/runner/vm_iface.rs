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
                // The program itself prints via ConsoleBox etc.; here we only return exit code.
                // Map bool/integer to exit code if possible when needed; default 0.
                // Keep minimal: return 0 for success.
                let _ = ret; // result object ignored here; prints already happened.
                Ok(0)
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

