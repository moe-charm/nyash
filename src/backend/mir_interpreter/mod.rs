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
mod helpers;
mod method_router;

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
            contracts_new_argv: HashMap::new(),
        }
    }

    /// Execute module entry (main) and return boxed result
    pub fn execute_module(&mut self, module: &MirModule) -> Result<Box<dyn NyashBox>, VMError> {
        // Snapshot functions for call resolution
        self.functions = module.functions.clone();

        // Prefer static Main.main when present; otherwise fall back to top-level main
        let (entry_name, pass_argv) = if module.functions.contains_key("Main.main") {
            ("Main.main", true)
        } else if module.functions.contains_key("main") {
            ("main", true) // if main has params, provide empty argv; harmless if zero params
        } else {
            return Err(VMError::InvalidInstruction("missing main".into()));
        };

        let func = module
            .functions
            .get(entry_name)
            .ok_or_else(|| VMError::InvalidInstruction(format!("entry not found: {}", entry_name)))?;

        // If the entry expects at least one parameter, pass an empty ArrayBox as argv
        let ret = if pass_argv && !func.params.is_empty() {
            let argv = VMValue::from_nyash_box(Box::new(ArrayBox::new()));
            let args: [VMValue; 1] = [argv];
            self.exec_function_inner(func, Some(&args))?
        } else {
            self.execute_function(func)?
        };
        Ok(ret.to_nyash_box())
    }

    fn execute_function(&mut self, func: &MirFunction) -> Result<VMValue, VMError> {
        self.exec_function_inner(func, None)
    }
}
