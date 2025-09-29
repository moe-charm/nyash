use super::*;

mod arithmetic;
mod boxes;
mod boxes_array;
mod boxes_string;
mod boxes_map;
mod boxes_fields;
mod boxes_instance;
mod calls;
mod externals;
mod memory;
mod misc;

impl MirInterpreter {
    pub(super) fn execute_instruction(&mut self, inst: &MirInstruction) -> Result<(), VMError> {
        match inst {
            MirInstruction::Const { dst, value } => self.handle_const(*dst, value)?,
            MirInstruction::NewBox {
                dst,
                box_type,
                args,
            } => self.handle_new_box(*dst, box_type, args)?,
            MirInstruction::PluginInvoke {
                dst,
                box_val,
                method,
                args,
                ..
            } => self.handle_plugin_invoke(*dst, *box_val, method, args)?,
            MirInstruction::BoxCall {
                dst,
                box_val,
                method,
                args,
                ..
            } => self.handle_box_call(*dst, *box_val, method, args)?,
            MirInstruction::ExternCall {
                dst,
                iface_name,
                method_name,
                args,
                ..
            } => self.handle_extern_call(*dst, iface_name, method_name, args)?,
            MirInstruction::RefSet {
                reference,
                field,
                value,
            } => self.handle_ref_set(*reference, field, *value)?,
            MirInstruction::RefGet {
                dst,
                reference,
                field,
            } => self.handle_ref_get(*dst, *reference, field)?,
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
