use super::LLVMCompiler;
use crate::box_trait::{BoolBox, IntegerBox, NyashBox, StringBox};
use crate::boxes::{math_box::FloatBox, null_box::NullBox};
use crate::mir::function::MirModule;
use crate::mir::instruction::{BinaryOp, ConstValue, MirInstruction as I};

impl LLVMCompiler {
    pub(crate) fn run_interpreter(
        &mut self,
        mir_module: &MirModule,
    ) -> Result<Box<dyn NyashBox>, String> {
        self.values.clear();
        let func = mir_module
            .functions
            .get("Main.main")
            .or_else(|| mir_module.functions.get("main"))
            .or_else(|| mir_module.functions.values().next())
            .ok_or_else(|| "Main.main function not found".to_string())?;

        for inst in &func.get_block(func.entry_block).unwrap().instructions {
            match inst {
                I::Const { dst, value } => {
                    let v: Box<dyn NyashBox> = match value {
                        ConstValue::Integer(i) => Box::new(IntegerBox::new(*i)),
                        ConstValue::Float(f) => Box::new(FloatBox::new(*f)),
                        ConstValue::String(s) => Box::new(StringBox::new(s.clone())),
                        ConstValue::Bool(b) => Box::new(BoolBox::new(*b)),
                        ConstValue::Null => Box::new(NullBox::new()),
                        ConstValue::Void => Box::new(IntegerBox::new(0)),
                    };
                    self.values.insert(*dst, v);
                }
                I::BinOp { dst, op, lhs, rhs } => {
                    let l = self
                        .values
                        .get(lhs)
                        .and_then(|b| b.as_any().downcast_ref::<IntegerBox>())
                        .ok_or_else(|| format!("binop lhs %{} not integer", lhs.0))?;
                    let r = self
                        .values
                        .get(rhs)
                        .and_then(|b| b.as_any().downcast_ref::<IntegerBox>())
                        .ok_or_else(|| format!("binop rhs %{} not integer", rhs.0))?;
                    let res = match op {
                        BinaryOp::Add => l.value + r.value,
                        BinaryOp::Sub => l.value - r.value,
                        BinaryOp::Mul => l.value * r.value,
                        BinaryOp::Div => {
                            if r.value == 0 {
                                return Err("division by zero".into());
                            }
                            l.value / r.value
                        }
                        BinaryOp::Mod => l.value % r.value,
                        BinaryOp::BitAnd => l.value & r.value,
                        BinaryOp::BitOr => l.value | r.value,
                        BinaryOp::BitXor => l.value ^ r.value,
                        BinaryOp::Shl => l.value << r.value,
                        BinaryOp::Shr => l.value >> r.value,
                        BinaryOp::And => {
                            if (l.value != 0) && (r.value != 0) {
                                1
                            } else {
                                0
                            }
                        }
                        BinaryOp::Or => {
                            if (l.value != 0) || (r.value != 0) {
                                1
                            } else {
                                0
                            }
                        }
                    };
                    self.values.insert(*dst, Box::new(IntegerBox::new(res)));
                }
                // ExternCall retired — unified Call(callee=Extern) is preferred
                I::Return { value } => {
                    if let Some(v) = value {
                        return self
                            .values
                            .get(v)
                            .map(|b| b.clone_box())
                            .ok_or_else(|| format!("return %{} missing", v.0));
                    }
                    return Ok(Box::new(IntegerBox::new(0)));
                }
                _ => {}
            }
        }
        Ok(Box::new(IntegerBox::new(0)))
    }
}
