use super::*;

impl MirInterpreter {
    pub(super) fn handle_const(&mut self, dst: ValueId, value: &ConstValue) -> Result<(), VMError> {
        let v = match value {
            ConstValue::Integer(i) => VMValue::Integer(*i),
            ConstValue::Float(f) => VMValue::Float(*f),
            ConstValue::Bool(b) => VMValue::Bool(*b),
            ConstValue::String(s) => VMValue::String(s.clone()),
            ConstValue::Null | ConstValue::Void => VMValue::Void,
        };
        self.regs.insert(dst, v);
        Ok(())
    }

    pub(super) fn handle_binop(
        &mut self,
        dst: ValueId,
        op: BinaryOp,
        lhs: ValueId,
        rhs: ValueId,
    ) -> Result<(), VMError> {
        let a = self.reg_load(lhs)?;
        let b = self.reg_load(rhs)?;
        let v = self.eval_binop(op, a, b)?;
        self.regs.insert(dst, v);
        Ok(())
    }

    pub(super) fn handle_unary_op(
        &mut self,
        dst: ValueId,
        op: crate::mir::UnaryOp,
        operand: ValueId,
    ) -> Result<(), VMError> {
        let x = self.reg_load(operand)?;
        let v = match op {
            crate::mir::UnaryOp::Neg => match x {
                VMValue::Integer(i) => VMValue::Integer(-i),
                VMValue::Float(f) => VMValue::Float(-f),
                _ => {
                    return Err(VMError::TypeError(format!(
                        "neg expects number, got {:?}",
                        x
                    )))
                }
            },
            crate::mir::UnaryOp::Not => VMValue::Bool(!to_bool_vm(&x).map_err(VMError::TypeError)?),
            crate::mir::UnaryOp::BitNot => match x {
                VMValue::Integer(i) => VMValue::Integer(!i),
                _ => {
                    return Err(VMError::TypeError(format!(
                        "bitnot expects integer, got {:?}",
                        x
                    )))
                }
            },
        };
        self.regs.insert(dst, v);
        Ok(())
    }

    pub(super) fn handle_compare(
        &mut self,
        dst: ValueId,
        op: CompareOp,
        lhs: ValueId,
        rhs: ValueId,
    ) -> Result<(), VMError> {
        let a = self.reg_load(lhs)?;
        let b = self.reg_load(rhs)?;
        let res = self.eval_cmp(op, a, b)?;
        self.regs.insert(dst, VMValue::Bool(res));
        Ok(())
    }

    pub(super) fn handle_copy(&mut self, dst: ValueId, src: ValueId) -> Result<(), VMError> {
        let v = self.reg_load(src)?;
        self.regs.insert(dst, v);
        Ok(())
    }
}
