/*!
 * Minimal MIR Interpreter
 *
 * Executes a subset of MIR instructions for fast iteration without LLVM/JIT.
 * Supported: Const, BinOp(Add/Sub/Mul/Div/Mod), Compare, Load/Store, Branch, Jump, Return,
 * Print/Debug (best-effort), Barrier/Safepoint (no-op).
 */

use std::collections::{HashMap, HashSet};

use crate::box_trait::NyashBox;
use crate::boxes::array::ArrayBox;

pub(super) use crate::backend::abi_util::{eq_vm, to_bool_vm};
pub(super) use crate::backend::vm::{VMError, VMValue};
pub(super) use crate::mir::{
    BasicBlockId, BinaryOp, Callee, CompareOp, ConstValue, MirFunction, MirInstruction, MirModule,
    ValueId,
};

mod exec;
mod handlers;
#[path = "helpers/mod.rs"]
mod helpers;
mod method_router;
mod extern_adapter;
mod operator_guard;
mod vm_config;
pub mod resolve;
pub mod contracts;

pub use vm_config::VmConfig;

/// Call mode for function execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallMode {
    /// Normal call with operator guard
    Normal,
    /// Skip operator guard to prevent recursion (used by op_eq)
    NoOperatorGuard,
}

pub struct MirInterpreter {
    pub(super) regs: HashMap<ValueId, VMValue>,
    pub(super) mem: HashMap<ValueId, VMValue>,
    // Object field storage keyed by stable object identity (Arc ptr addr fallback)
    pub(super) obj_fields: HashMap<u64, HashMap<String, VMValue>>,
    pub(super) functions: HashMap<String, MirFunction>,
    pub(super) cur_fn: Option<String>,
    // Trace context (dev-only; enabled with NYASH_VM_TRACE=1)
    pub(super) last_block: Option<BasicBlockId>,
    pub(super) last_inst: Option<MirInstruction>,
    // Contracts observation (dev-only; enabled with NYASH_CHECK_CONTRACTS=1)
    pub(super) contracts_new: HashSet<u64>,
    pub(super) contracts_new_argv: HashMap<u64, usize>,
    pub(super) contracts_born: HashSet<u64>,
    pub(super) contracts_in_birth: HashSet<u64>,
    // Instruction fuel (opt-in). When Some(limit), the interpreter aborts once
    // inst_count exceeds the limit. Defaults to None (unlimited) unless
    // NYASH_VM_MAX_INSTRUCTIONS or HAKO_VM_MAX_INSTRUCTIONS is set.
    pub(super) inst_count: usize,
    pub(super) max_inst: Option<usize>,
    // Basic block execution cap (opt-in): per-block execution counter.
    pub(super) block_exec_count: HashMap<BasicBlockId, usize>,
    pub(super) max_block_exec: Option<usize>,
    // Scope-based lifetime tracker (fini on scope exit)
    pub(super) scope: crate::scope_tracker::ScopeTracker,
    // Dev-only: naive reentrancy counter per callee (for quick cycle diagnosis)
    pub(super) reenter_count: std::collections::HashMap<String, usize>,
}

impl MirInterpreter {
    pub fn new() -> Self {
        Self {
            regs: HashMap::new(),
            mem: HashMap::new(),
            obj_fields: HashMap::new(),
            functions: HashMap::new(),
            cur_fn: None,
            last_block: None,
            last_inst: None,
            contracts_new: HashSet::new(),
            contracts_born: HashSet::new(),
            contracts_in_birth: HashSet::new(),
            contracts_new_argv: HashMap::new(),
            inst_count: 0,
            max_inst: {
                let primary = std::env::var("NYASH_VM_MAX_INSTRUCTIONS").ok();
                let compat = std::env::var("HAKO_VM_MAX_INSTRUCTIONS").ok();
                let raw = primary.or(compat);
                raw.and_then(|s| s.parse::<usize>().ok())
            },
            block_exec_count: HashMap::new(),
            max_block_exec: {
                let primary = std::env::var("NYASH_VM_MAX_BLOCK_EXEC").ok();
                let compat = std::env::var("HAKO_VM_MAX_BLOCK_EXEC").ok();
                let raw = primary.or(compat);
                raw.and_then(|s| s.parse::<usize>().ok())
            },
            scope: crate::scope_tracker::ScopeTracker::new(),
            reenter_count: HashMap::new(),
        }
    }

    /// Execute module entry (main) and return boxed result
    pub fn execute_module(&mut self, module: &MirModule) -> Result<Box<dyn NyashBox>, VMError> {
        // Snapshot functions for call resolution
        self.functions = module.functions.clone();

        // Prefer static Main.main when present; otherwise consider a unique <Box>.main,
        // then fall back to top-level main when allowed.
        let allow_top = crate::config::env::entry_allow_toplevel_main();
        let prefer_static = crate::config::env::entry_prefer_static_main();
        let (entry_name, pass_argv) = if module.functions.contains_key("Main.main") {
            ("Main.main", true)
        } else if prefer_static {
            // Collect unique candidates matching "*.main" or "*.main/0"
            let mut cands: Vec<&str> = Vec::new();
            for k in module.functions.keys() {
                if k.ends_with(".main") || k.ends_with(".main/0") {
                    cands.push(k.as_str());
                }
            }
            if cands.len() == 1 {
                (cands[0], true)
            } else if allow_top && module.functions.contains_key("main") {
                ("main", true)
            } else if module.functions.contains_key("main") {
                // Use top-level main but warn (unless quiet)
                if !crate::config::env::cli_quiet() {
                    eprintln!("[entry] Warning: using top-level 'main' without explicit allow; set NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 to silence.");
                }
                ("main", true)
            } else {
                return Err(VMError::InvalidInstruction("missing main".into()));
            }
        } else if allow_top && module.functions.contains_key("main") {
            ("main", true)
        } else if module.functions.contains_key("main") {
            if !crate::config::env::cli_quiet() {
                eprintln!("[entry] Warning: using top-level 'main' without explicit allow; set NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 to silence.");
            }
            ("main", true)
        } else {
            return Err(VMError::InvalidInstruction("missing main".into()));
        };

        let func = module
            .functions
            .get(entry_name)
            .ok_or_else(|| VMError::InvalidInstruction(format!("entry not found: {}", entry_name)))?;

        // If the entry expects at least one parameter, pass argv from CLI script args (JSON),
        // falling back to an empty ArrayBox when unavailable.
        let ret = if pass_argv && !func.params.is_empty() {
            let argv = {
                // Build argv from NYASH_SCRIPT_ARGS_JSON when present (array of strings)
                if let Ok(raw) = std::env::var("NYASH_SCRIPT_ARGS_JSON") {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                        if let Some(arr) = v.as_array() {
                            let ab = ArrayBox::new();
                            for item in arr {
                                if let Some(s) = item.as_str() {
                                    let sb = crate::box_trait::StringBox::new(s.to_string());
                                    let _ = ab.push(Box::new(sb));
                                } else {
                                    // Non-string items are stringified conservatively
                                    let sb = crate::box_trait::StringBox::new(item.to_string());
                                    let _ = ab.push(Box::new(sb));
                                }
                            }
                            VMValue::from_nyash_box(Box::new(ab))
                        } else {
                            VMValue::from_nyash_box(Box::new(ArrayBox::new()))
                        }
                    } else {
                        VMValue::from_nyash_box(Box::new(ArrayBox::new()))
                    }
                } else {
                    VMValue::from_nyash_box(Box::new(ArrayBox::new()))
                }
            };
            let args: [VMValue; 1] = [argv];
            self.exec_function_inner(func, Some(&args))?
        } else {
            self.execute_function(func)?
        };
        Ok(ret.to_nyash_box())
    }

    

    pub fn execute_entry_by_name(&mut self, module: &MirModule, entry_name: &str) -> Result<Box<dyn NyashBox>, VMError> {
        self.functions = module.functions.clone();
        let func = module
            .functions
            .get(entry_name)
            .ok_or_else(|| VMError::InvalidInstruction(format!("entry not found: {}", entry_name)))?;
        if !func.params.is_empty() {
            let argv = VMValue::from_nyash_box(Box::new(ArrayBox::new()));
            let args: [VMValue; 1] = [argv];
            let ret = self.exec_function_inner(func, Some(&args))?;
            Ok(ret.to_nyash_box())
        } else {
            let ret = self.execute_function(func)?;
            Ok(ret.to_nyash_box())
        }
    }
fn execute_function(&mut self, func: &MirFunction) -> Result<VMValue, VMError> {
        self.exec_function_inner(func, None)
    }
}
