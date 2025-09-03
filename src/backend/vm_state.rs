/*!
 * VM State & Basics
 *
 * Contains constructor helpers, basic value storage, instruction accounting,
 * phi selection delegation, and small utilities that support the exec loop.
 */

use super::vm::{VM, VMError, VMValue};
use super::vm_phi::LoopExecutor;
use super::frame::ExecutionFrame;
use crate::mir::{BasicBlockId, ValueId};
use crate::runtime::NyashRuntime;
use crate::scope_tracker::ScopeTracker;
use std::collections::HashMap;
use std::time::Instant;

impl VM {
    fn jit_threshold_from_env() -> u32 {
        std::env::var("NYASH_JIT_THRESHOLD")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|&v| v > 0)
            .unwrap_or(64)
    }

    /// Helper: execute phi via LoopExecutor with previous_block-based selection
    pub(super) fn loop_execute_phi(
        &mut self,
        dst: ValueId,
        inputs: &[(BasicBlockId, ValueId)],
    ) -> Result<VMValue, VMError> {
        if inputs.is_empty() {
            return Err(VMError::InvalidInstruction("Phi node has no inputs".to_string()));
        }
        let debug_phi = std::env::var("NYASH_VM_DEBUG").ok().as_deref() == Some("1")
            || std::env::var("NYASH_VM_DEBUG_PHI").ok().as_deref() == Some("1");
        if debug_phi {
            eprintln!(
                "[VM] phi-select (delegated) prev={:?} inputs={:?}",
                self.previous_block, inputs
            );
        }
        let values_ref = &self.values;
        let res = self.loop_executor.execute_phi(dst, inputs, |val_id| {
            let index = val_id.to_usize();
            if index < values_ref.len() {
                if let Some(ref value) = values_ref[index] {
                    Ok(value.clone())
                } else {
                    Err(VMError::InvalidValue(format!("Value {} not set", val_id)))
                }
            } else {
                Err(VMError::InvalidValue(format!("Value {} out of bounds", val_id)))
            }
        });
        if debug_phi {
            match &res {
                Ok(v) => eprintln!("[VM] phi-result -> {:?}", v),
                Err(e) => eprintln!("[VM] phi-error -> {:?}", e),
            }
        }
        res
    }

    /// Create a new VM instance
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            current_function: None,
            frame: ExecutionFrame::new(),
            previous_block: None,
            object_fields: HashMap::new(),
            object_class: HashMap::new(),
            object_internal: std::collections::HashSet::new(),
            loop_executor: LoopExecutor::new(),
            runtime: NyashRuntime::new(),
            scope_tracker: ScopeTracker::new(),
            module: None,
            instr_counter: std::collections::HashMap::new(),
            exec_start: None,
            boxcall_hits_vtable: 0,
            boxcall_hits_poly_pic: 0,
            boxcall_hits_mono_pic: 0,
            boxcall_hits_generic: 0,
            boxcall_pic_hits: std::collections::HashMap::new(),
            boxcall_pic_funcname: std::collections::HashMap::new(),
            boxcall_poly_pic: std::collections::HashMap::new(),
            boxcall_vtable_funcname: std::collections::HashMap::new(),
            type_versions: std::collections::HashMap::new(),
            jit_manager: Some(crate::jit::manager::JitManager::new(Self::jit_threshold_from_env())),
        }
    }

    /// Create a VM with an external runtime (dependency injection)
    pub fn with_runtime(runtime: NyashRuntime) -> Self {
        Self {
            values: Vec::new(),
            current_function: None,
            frame: ExecutionFrame::new(),
            previous_block: None,
            object_fields: HashMap::new(),
            object_class: HashMap::new(),
            object_internal: std::collections::HashSet::new(),
            loop_executor: LoopExecutor::new(),
            runtime,
            scope_tracker: ScopeTracker::new(),
            module: None,
            instr_counter: std::collections::HashMap::new(),
            exec_start: None,
            boxcall_hits_vtable: 0,
            boxcall_hits_poly_pic: 0,
            boxcall_hits_mono_pic: 0,
            boxcall_hits_generic: 0,
            boxcall_pic_hits: std::collections::HashMap::new(),
            boxcall_pic_funcname: std::collections::HashMap::new(),
            boxcall_poly_pic: std::collections::HashMap::new(),
            boxcall_vtable_funcname: std::collections::HashMap::new(),
            type_versions: std::collections::HashMap::new(),
            jit_manager: Some(crate::jit::manager::JitManager::new(Self::jit_threshold_from_env())),
        }
    }

    /// Get a value from storage
    pub(super) fn get_value(&self, value_id: ValueId) -> Result<VMValue, VMError> {
        let index = value_id.to_usize();
        if index < self.values.len() {
            if let Some(ref value) = self.values[index] {
                Ok(value.clone())
            } else {
                Err(VMError::InvalidValue(format!("Value {} not set", value_id)))
            }
        } else {
            Err(VMError::InvalidValue(format!("Value {} out of bounds", value_id)))
        }
    }

    /// Set a value in the VM storage
    pub(super) fn set_value(&mut self, value_id: ValueId, value: VMValue) {
        let index = value_id.to_usize();
        if index >= self.values.len() {
            self.values.resize(index + 1, None);
        }
        self.values[index] = Some(value);
    }

    /// Record an instruction execution for statistics
    pub(super) fn record_instruction(&mut self, instruction: &crate::mir::MirInstruction) {
        let key: &'static str = match instruction {
            crate::mir::MirInstruction::Const { .. } => "Const",
            crate::mir::MirInstruction::BinOp { .. } => "BinOp",
            crate::mir::MirInstruction::UnaryOp { .. } => "UnaryOp",
            crate::mir::MirInstruction::Compare { .. } => "Compare",
            crate::mir::MirInstruction::Load { .. } => "Load",
            crate::mir::MirInstruction::Store { .. } => "Store",
            crate::mir::MirInstruction::Call { .. } => "Call",
            crate::mir::MirInstruction::FunctionNew { .. } => "FunctionNew",
            crate::mir::MirInstruction::BoxCall { .. } => "BoxCall",
            crate::mir::MirInstruction::Branch { .. } => "Branch",
            crate::mir::MirInstruction::Jump { .. } => "Jump",
            crate::mir::MirInstruction::Return { .. } => "Return",
            crate::mir::MirInstruction::Phi { .. } => "Phi",
            crate::mir::MirInstruction::NewBox { .. } => "NewBox",
            crate::mir::MirInstruction::TypeCheck { .. } => "TypeCheck",
            crate::mir::MirInstruction::Cast { .. } => "Cast",
            crate::mir::MirInstruction::TypeOp { .. } => "TypeOp",
            crate::mir::MirInstruction::ArrayGet { .. } => "ArrayGet",
            crate::mir::MirInstruction::ArraySet { .. } => "ArraySet",
            crate::mir::MirInstruction::Copy { .. } => "Copy",
            crate::mir::MirInstruction::Debug { .. } => "Debug",
            crate::mir::MirInstruction::Print { .. } => "Print",
            crate::mir::MirInstruction::Nop => "Nop",
            crate::mir::MirInstruction::Throw { .. } => "Throw",
            crate::mir::MirInstruction::Catch { .. } => "Catch",
            crate::mir::MirInstruction::Safepoint => "Safepoint",
            crate::mir::MirInstruction::RefNew { .. } => "RefNew",
            crate::mir::MirInstruction::RefGet { .. } => "RefGet",
            crate::mir::MirInstruction::RefSet { .. } => "RefSet",
            crate::mir::MirInstruction::WeakNew { .. } => "WeakNew",
            crate::mir::MirInstruction::WeakLoad { .. } => "WeakLoad",
            crate::mir::MirInstruction::BarrierRead { .. } => "BarrierRead",
            crate::mir::MirInstruction::BarrierWrite { .. } => "BarrierWrite",
            crate::mir::MirInstruction::WeakRef { .. } => "WeakRef",
            crate::mir::MirInstruction::Barrier { .. } => "Barrier",
            crate::mir::MirInstruction::FutureNew { .. } => "FutureNew",
            crate::mir::MirInstruction::FutureSet { .. } => "FutureSet",
            crate::mir::MirInstruction::Await { .. } => "Await",
            crate::mir::MirInstruction::ExternCall { .. } => "ExternCall",
            crate::mir::MirInstruction::PluginInvoke { .. } => "PluginInvoke",
        };
        *self.instr_counter.entry(key).or_insert(0) += 1;
    }
}

