use super::*;

mod arithmetic;
mod boxes;
mod boxes_fields;
mod boxes_instance;
mod calls;
mod externals;
mod memory;
mod misc;
pub(crate) mod op_handlers;

impl MirInterpreter {
    pub(super) fn execute_instruction(&mut self, inst: &MirInstruction) -> Result<(), VMError> {
        match inst {
            MirInstruction::Const { dst, value } => self.handle_const(*dst, value)?,
            MirInstruction::NewBox {
                dst,
                box_type,
                args,
                auto_birth,
            } => {
                self.handle_new_box(*dst, box_type, args)?;
                if let Some(name) = auto_birth.as_ref() {
                    let mut bargs: Vec<super::ValueId> = Vec::with_capacity(1 + args.len());
                    bargs.push(*dst);
                    bargs.extend(args.iter().copied());
                    // Call only when the function exists; otherwise treat as no-op
                    if self.functions.contains_key(name) {
                        let _ = self.handle_callee_module_function(name, &bargs);
                    }
                }
                // Fallback safety: ensure birth() runs for user/builtin boxes.
                // Guarded by env so we can bisect issues (NYASH_VM_BIRTH_AFTER_NEW=0 disables).
                if crate::config::env::vm_birth_after_new_fallback() {
                    let _ = self.handle_box_call(None, *dst, "birth", args);
                }
            }
            
            MirInstruction::BoxCall { .. } => {
                if let MirInstruction::BoxCall { dst, box_val, method, args, .. } = inst {
                    self.handle_box_call(*dst, *box_val, method, args)?
                } else { unreachable!() }
            }
            // ExternCall retired
            
            MirInstruction::BinOp { dst, op, lhs, rhs } => {
                self.handle_binop(*dst, *op, *lhs, *rhs)?
            }
            MirInstruction::UnaryOp { dst, op, operand } => {
                self.handle_unary_op(*dst, *op, *operand)?
            }
            MirInstruction::Compare { dst, op, lhs, rhs } => {
                self.handle_compare(*dst, *op, *lhs, *rhs)?
            }
            MirInstruction::Copy { dst, src } => self.handle_copy(*dst, *src)?,
            MirInstruction::Load { dst, ptr } => self.handle_load(*dst, *ptr)?,
            MirInstruction::Store { ptr, value } => self.handle_store(*ptr, *value)?,
            MirInstruction::Call {
                dst,
                func,
                callee,
                args,
                ..
            } => self.handle_call(*dst, *func, callee.as_ref(), args)?,
            MirInstruction::Debug { message, value } => {
                self.handle_debug(message, *value)?;
            }
            MirInstruction::Print { value, .. } => self.handle_print(*value)?,
            MirInstruction::BarrierRead { .. }
            | MirInstruction::BarrierWrite { .. }
            | MirInstruction::Barrier { .. }
            | MirInstruction::Safepoint
            | MirInstruction::Nop => {}
            other => {
                return Err(VMError::InvalidInstruction(format!(
                    "MIR interp: unimplemented instruction: {:?}",
                    other
                )))
            }
        }
        Ok(())
    }
}
