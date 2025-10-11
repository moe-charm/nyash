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
        let a0 = self.reg_load(lhs)?;
        let b0 = self.reg_load(rhs)?;
        // ParserBox-specific safety (dev bring-up, behavior-preserving意図):
        // ParserBox.* 内で、数値系BinOpに BoxRef(ParserBox) が片側で混ざった場合、gpos を整数化して演算。
        // 代表例: j + 4 / j + 1 など。String連結等には影響しないよう、片側が数値のときのみ適用。
        let (a, b) = if let Some(cur) = &self.cur_fn {
            if cur.starts_with("ParserBox.") {
                let coerce_pb_num = |v: &VMValue| -> VMValue {
                    if let VMValue::BoxRef(bx) = v {
                        if let Some(inst) = bx.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                            if inst.class_name == "ParserBox" {
                                if let Some(ny) = inst.get_field_ng("gpos") {
                                    match ny {
                                        crate::value::NyashValue::Integer(i) => VMValue::Integer(i as i64),
                                        crate::value::NyashValue::Float(f) => VMValue::Float(f),
                                        crate::value::NyashValue::String(ref s) => {
                                            if let Ok(i) = s.parse::<i64>() { VMValue::Integer(i) } else { VMValue::Integer(0) }
                                        }
                                        _ => VMValue::Integer(0),
                                    }
                                } else { VMValue::Integer(0) }
                            } else { v.clone() }
                        } else { v.clone() }
                    } else { v.clone() }
                };
                use VMValue::*;
                match (&a0, &b0) {
                    (BoxRef(_), Integer(_)) | (BoxRef(_), Float(_)) => (coerce_pb_num(&a0), b0.clone()),
                    (Integer(_), BoxRef(_)) | (Float(_), BoxRef(_)) => (a0.clone(), coerce_pb_num(&b0)),
                    _ => (a0.clone(), b0.clone()),
                }
            } else { (a0, b0) }
        } else { (a0, b0) };
        let cfg = super::VmConfig::global();
        if cfg.general_trace || cfg.trace_exec {
            let ak = crate::backend::abi_util::tag_of_vm(&a);
            let bk = crate::backend::abi_util::tag_of_vm(&b);
            eprintln!("[vm-op] binop {:?} a_k={} b_k={}", op, ak, bk);
        }
        // String-concat fast path: if either side is String, perform string concat here (dev-friendly, avoids re-entry).
        if let BinaryOp::Add = op {
            let ak = crate::backend::abi_util::tag_of_vm(&a);
            let bk = crate::backend::abi_util::tag_of_vm(&b);
            let stringy = ak == "String" || bk == "String";
            if stringy {
                let s = format!("{}{}", crate::backend::abi_util::to_string_vm(&a), crate::backend::abi_util::to_string_vm(&b));
                self.regs.insert(dst, VMValue::String(s));
                return Ok(());
            }

            // Operator Box (Add) — adopt only when enabled and not inside operator implementation
            let in_guard = self
                .cur_fn
                .as_deref()
                .map(|n| n.starts_with("AddOperator.apply/"))
                .unwrap_or(false);
            if let Some(op_fn) = self.functions.get("AddOperator.apply/2").cloned() {
                if !in_guard && crate::config::env::operator_box_add_adopt() && stringy {
                    let out = self.exec_function_inner(&op_fn, Some(&[a.clone(), b.clone()]))?;
                    self.regs.insert(dst, out);
                    return Ok(());
                }
            }
        }
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
        if super::super::VmConfig::global().general_trace {
            eprintln!("[DEBUG-COMPARE] op={:?}, a={:?}, b={:?}", op, a, b);
        }
        // Operator Box (Compare) — adopt gated; do NOT execute when adopt is OFF
        if let Some(op_fn) = self.functions.get("CompareOperator.apply/3").cloned() {
            let in_guard = self
                .cur_fn
                .as_deref()
                .map(|n| n.starts_with("CompareOperator.apply/"))
                .unwrap_or(false);
            let opname = match op {
                CompareOp::Eq => "Eq",
                CompareOp::Ne => "Ne",
                CompareOp::Lt => "Lt",
                CompareOp::Le => "Le",
                CompareOp::Gt => "Gt",
                CompareOp::Ge => "Ge",
            };
            if !in_guard && crate::config::env::operator_box_compare_adopt() {
                let out = self.exec_function_inner(
                    &op_fn,
                    Some(&[VMValue::String(opname.to_string()), a.clone(), b.clone()]),
                )?;
                let res = match out {
                    VMValue::Bool(b) => b,
                    _ => self.eval_cmp(op, a.clone(), b.clone())?,
                };
                self.regs.insert(dst, VMValue::Bool(res));
                return Ok(());
            }
        }
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
