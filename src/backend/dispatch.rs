/*!
 * VM Dispatch table (scaffolding)
 *
 * Purpose: Centralize mapping from MIR instruction kinds to handler fns.
 * Status: Initial skeleton; currently unused. Future: build static table for hot-path dispatch.
 */

use crate::mir::MirInstruction;
use super::vm::{VM, VMError};
use super::vm::ControlFlow;
use crate::mir::CompareOp;
use super::vm::VMValue;

/// Minimal dispatcher that routes a single instruction to the appropriate handler.
/// Keeps behavior identical to the big match in vm.rs but centralized here.
pub fn execute_instruction(vm: &mut VM, instruction: &MirInstruction, debug_global: bool) -> Result<ControlFlow, VMError> {
    match instruction {
        // Basic operations
        MirInstruction::Const { dst, value } => vm.execute_const(*dst, value),

        MirInstruction::BinOp { dst, op, lhs, rhs } => {
            if debug_global || std::env::var("NYASH_VM_DEBUG_ANDOR").ok().as_deref() == Some("1") {
                eprintln!("[VM] execute_instruction -> BinOp({:?})", op);
            }
            vm.execute_binop(*dst, op, *lhs, *rhs)
        },

        MirInstruction::UnaryOp { dst, op, operand } => vm.execute_unaryop(*dst, op, *operand),

        MirInstruction::Compare { dst, op, lhs, rhs } => {
            let debug_cmp = debug_global || std::env::var("NYASH_VM_DEBUG_CMP").ok().as_deref() == Some("1");
            if debug_cmp { eprintln!("[VM] dispatch Compare op={:?} lhs={:?} rhs={:?}", op, lhs, rhs); }
            if let (Ok(lv), Ok(rv)) = (vm.get_value(*lhs), vm.get_value(*rhs)) {
                if debug_cmp { eprintln!("[VM] values before fastpath: left={:?} right={:?}", lv, rv); }
                if let (VMValue::BoxRef(lb), VMValue::BoxRef(rb)) = (&lv, &rv) {
                    if debug_cmp { eprintln!("[VM] BoxRef types: lty={} rty={} lstr={} rstr={}", lb.type_name(), rb.type_name(), lb.to_string_box().value, rb.to_string_box().value); }
                    let li = lb.as_any().downcast_ref::<crate::box_trait::IntegerBox>().map(|x| x.value)
                        .or_else(|| lb.to_string_box().value.trim().parse::<i64>().ok());
                    let ri = rb.as_any().downcast_ref::<crate::box_trait::IntegerBox>().map(|x| x.value)
                        .or_else(|| rb.to_string_box().value.trim().parse::<i64>().ok());
                    if let (Some(li), Some(ri)) = (li, ri) {
                        let out = match op { CompareOp::Eq => li == ri, CompareOp::Ne => li != ri, CompareOp::Lt => li < ri, CompareOp::Le => li <= ri, CompareOp::Gt => li > ri, CompareOp::Ge => li >= ri };
                        vm.set_value(*dst, VMValue::Bool(out));
                        return Ok(ControlFlow::Continue);
                    }
                }
            }
            vm.execute_compare(*dst, op, *lhs, *rhs)
        },

        // I/O operations
        MirInstruction::Print { value, .. } => vm.execute_print(*value),

        // Type operations
        MirInstruction::TypeOp { dst, op, value, ty } => vm.execute_typeop(*dst, op, *value, ty),

        // Control flow
        MirInstruction::Return { value } => vm.execute_return(*value),
        MirInstruction::Jump { target } => vm.execute_jump(*target),
        MirInstruction::Branch { condition, then_bb, else_bb } => vm.execute_branch(*condition, *then_bb, *else_bb),
        MirInstruction::Phi { dst, inputs } => vm.execute_phi(*dst, inputs),

        // Memory operations
        MirInstruction::Load { dst, ptr } => vm.execute_load(*dst, *ptr),
        MirInstruction::Store { value, ptr } => vm.execute_store(*value, *ptr),
        MirInstruction::Copy { dst, src } => vm.execute_copy(*dst, *src),

        // Complex operations
        MirInstruction::Call { dst, func, args, effects: _ } => vm.execute_call(*dst, *func, args),
        MirInstruction::BoxCall { dst, box_val, method, method_id, args, effects: _ , .. } => vm.execute_boxcall(*dst, *box_val, method, *method_id, args),
        MirInstruction::NewBox { dst, box_type, args } => vm.execute_newbox(*dst, box_type, args),

        // Array operations
        MirInstruction::ArrayGet { dst, array, index } => vm.execute_array_get(*dst, *array, *index),
        MirInstruction::ArraySet { array, index, value } => vm.execute_array_set(*array, *index, *value),

        // Reference operations
        MirInstruction::RefNew { dst, box_val } => vm.execute_ref_new(*dst, *box_val),
        MirInstruction::RefGet { dst, reference, field } => vm.execute_ref_get(*dst, *reference, field),
        MirInstruction::RefSet { reference, field, value } => vm.execute_ref_set(*reference, field, *value),

        // Weak references
        MirInstruction::WeakNew { dst, box_val } => vm.execute_weak_new(*dst, *box_val),
        MirInstruction::WeakLoad { dst, weak_ref } => vm.execute_weak_load(*dst, *weak_ref),
        MirInstruction::WeakRef { dst, op, value } => match op {
            crate::mir::WeakRefOp::New => vm.execute_weak_new(*dst, *value),
            crate::mir::WeakRefOp::Load => vm.execute_weak_load(*dst, *value),
        },

        // Barriers
        MirInstruction::BarrierRead { .. } => Ok(ControlFlow::Continue),
        MirInstruction::BarrierWrite { .. } => Ok(ControlFlow::Continue),
        MirInstruction::Barrier { .. } => Ok(ControlFlow::Continue),

        // Exceptions
        MirInstruction::Throw { exception, .. } => vm.execute_throw(*exception),
        MirInstruction::Catch { exception_value, .. } => vm.execute_catch(*exception_value),

        // Futures
        MirInstruction::FutureNew { dst, value } => {
            let initial_value = vm.get_value(*value)?;
            let future = crate::boxes::future::FutureBox::new();
            let nyash_box = initial_value.to_nyash_box();
            future.set_result(nyash_box);
            vm.set_value(*dst, VMValue::Future(future));
            Ok(ControlFlow::Continue)
        }
        MirInstruction::FutureSet { future, value } => {
            let future_val = vm.get_value(*future)?;
            let new_value = vm.get_value(*value)?;
            if let VMValue::Future(ref future_box) = future_val {
                future_box.set_result(new_value.to_nyash_box());
                Ok(ControlFlow::Continue)
            } else {
                Err(VMError::TypeError(format!("Expected Future, got {:?}", future_val)))
            }
        }

        // Special
        MirInstruction::Await { dst, future } => vm.execute_await(*dst, *future),
        MirInstruction::ExternCall { dst, iface_name, method_name, args, .. } => vm.execute_extern_call(*dst, iface_name, method_name, args),
        MirInstruction::TypeCheck { dst, .. } => { vm.set_value(*dst, VMValue::Bool(true)); Ok(ControlFlow::Continue) }
        MirInstruction::Cast { dst, value, .. } => { let val = vm.get_value(*value)?; vm.set_value(*dst, val); Ok(ControlFlow::Continue) }
        MirInstruction::Debug { .. } => Ok(ControlFlow::Continue),
        MirInstruction::Nop => Ok(ControlFlow::Continue),
        MirInstruction::Safepoint => Ok(ControlFlow::Continue),
    }
}

/// Placeholder for an instruction dispatch entry
pub struct DispatchEntry;

/// Placeholder dispatch table
pub struct DispatchTable;

impl DispatchTable {
    pub fn new() -> Self { Self }

    /// Example API for future use: resolve a handler for an instruction
    pub fn resolve(&self, _instr: &MirInstruction) -> Option<DispatchEntry> { None }
}

/// Example execution of a dispatch entry
pub fn execute_entry(_entry: &DispatchEntry) -> Result<(), VMError> { Ok(()) }
