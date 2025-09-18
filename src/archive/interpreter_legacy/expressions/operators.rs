/*!
 * Binary and unary operator evaluation
 */

// Removed super::* import - specific imports below
use crate::ast::{ASTNode, BinaryOperator, UnaryOperator};
use crate::box_trait::{BoolBox, CompareBox, NyashBox};
use crate::box_trait::{IntegerBox, StringBox}; // 🔧 修正: box_trait::*に統一
use crate::boxes::FloatBox; // FloatBoxはboxesのみに存在
use crate::instance_v2::InstanceBox;
use crate::interpreter::{NyashInterpreter, RuntimeError};

// Local helper functions to bypass import issues

/// InstanceBoxでラップされている場合、内部のBoxを取得する
/// シンプルなヘルパー関数で型地獄を回避
fn unwrap_instance(boxed: &dyn NyashBox) -> &dyn NyashBox {
    eprintln!(
        "🔍 DEBUG unwrap_instance: input type = {}",
        boxed.type_name()
    );
    if let Some(instance) = boxed.as_any().downcast_ref::<InstanceBox>() {
        eprintln!("  ✅ Is InstanceBox");
        if let Some(ref inner) = instance.inner_content {
            eprintln!("  📦 Inner content type = {}", inner.type_name());
            return inner.as_ref();
        }
    }
    eprintln!("  ❌ Not InstanceBox, returning as is");
    boxed
}

fn best_effort_to_string(val: &dyn NyashBox) -> String {
    crate::runtime::semantics::coerce_to_string(val).unwrap_or_else(|| val.to_string_box().value)
}

fn best_effort_to_i64(val: &dyn NyashBox) -> Option<i64> {
    crate::runtime::semantics::coerce_to_i64(val)
}
pub(super) fn try_add_operation(
    left: &dyn NyashBox,
    right: &dyn NyashBox,
) -> Option<Box<dyn NyashBox>> {
    // 🎯 InstanceBoxのunwrap処理
    let left = unwrap_instance(left);
    let right = unwrap_instance(right);

    // IntegerBox + IntegerBox
    if let (Some(left_int), Some(right_int)) = (
        left.as_any().downcast_ref::<IntegerBox>(),
        right.as_any().downcast_ref::<IntegerBox>(),
    ) {
        return Some(Box::new(IntegerBox::new(left_int.value + right_int.value)));
    }

    // StringBox + anything -> concatenation
    if let Some(left_str) = left.as_any().downcast_ref::<StringBox>() {
        let right_str = right.to_string_box();
        return Some(Box::new(StringBox::new(format!(
            "{}{}",
            left_str.value, right_str.value
        ))));
    }

    // BoolBox + BoolBox -> IntegerBox
    if let (Some(left_bool), Some(right_bool)) = (
        left.as_any().downcast_ref::<BoolBox>(),
        right.as_any().downcast_ref::<BoolBox>(),
    ) {
        return Some(Box::new(IntegerBox::new(
            (left_bool.value as i64) + (right_bool.value as i64),
        )));
    }

    None
}

pub(super) fn try_sub_operation(
    left: &dyn NyashBox,
    right: &dyn NyashBox,
) -> Option<Box<dyn NyashBox>> {
    // 🎯 InstanceBoxのunwrap処理
    let left = unwrap_instance(left);
    let right = unwrap_instance(right);

    // IntegerBox - IntegerBox
    if let (Some(left_int), Some(right_int)) = (
        left.as_any().downcast_ref::<IntegerBox>(),
        right.as_any().downcast_ref::<IntegerBox>(),
    ) {
        return Some(Box::new(IntegerBox::new(left_int.value - right_int.value)));
    }
    None
}

pub(super) fn try_mul_operation(
    left: &dyn NyashBox,
    right: &dyn NyashBox,
) -> Option<Box<dyn NyashBox>> {
    // 🎯 InstanceBoxのunwrap処理
    let left = unwrap_instance(left);
    let right = unwrap_instance(right);

    // デバッグ出力
    eprintln!(
        "🔍 DEBUG try_mul: left type = {}, right type = {}",
        left.type_name(),
        right.type_name()
    );

    // IntegerBox * IntegerBox
    if let (Some(left_int), Some(right_int)) = (
        left.as_any().downcast_ref::<IntegerBox>(),
        right.as_any().downcast_ref::<IntegerBox>(),
    ) {
        eprintln!(
            "✅ IntegerBox downcast success: {} * {}",
            left_int.value, right_int.value
        );
        return Some(Box::new(IntegerBox::new(left_int.value * right_int.value)));
    }

    // box_trait::IntegerBoxも試す
    eprintln!("❌ box_trait::IntegerBox downcast failed, trying boxes::integer_box::IntegerBox");

    // boxes::integer_box::IntegerBoxを試す
    use crate::boxes::integer_box::IntegerBox as BoxesIntegerBox;
    if let (Some(left_int), Some(right_int)) = (
        left.as_any().downcast_ref::<BoxesIntegerBox>(),
        right.as_any().downcast_ref::<BoxesIntegerBox>(),
    ) {
        eprintln!(
            "✅ boxes::IntegerBox downcast success: {} * {}",
            left_int.value, right_int.value
        );
        return Some(Box::new(IntegerBox::new(left_int.value * right_int.value)));
    }

    // StringBox * IntegerBox -> repetition
    if let (Some(str_box), Some(count_int)) = (
        left.as_any().downcast_ref::<StringBox>(),
        right.as_any().downcast_ref::<IntegerBox>(),
    ) {
        return Some(Box::new(StringBox::new(
            str_box.value.repeat(count_int.value as usize),
        )));
    }

    None
}

pub(super) fn try_div_operation(
    left: &dyn NyashBox,
    right: &dyn NyashBox,
) -> Result<Box<dyn NyashBox>, String> {
    // 🎯 InstanceBoxのunwrap処理
    let left = unwrap_instance(left);
    let right = unwrap_instance(right);

    // IntegerBox / IntegerBox
    if let (Some(left_int), Some(right_int)) = (
        left.as_any().downcast_ref::<IntegerBox>(),
        right.as_any().downcast_ref::<IntegerBox>(),
    ) {
        if right_int.value == 0 {
            return Err("Division by zero".to_string());
        }
        return Ok(Box::new(IntegerBox::new(left_int.value / right_int.value)));
    }

    Err(format!(
        "Division not supported between {} and {}",
        left.type_name(),
        right.type_name()
    ))
}

pub(super) fn try_mod_operation(
    left: &dyn NyashBox,
    right: &dyn NyashBox,
) -> Result<Box<dyn NyashBox>, String> {
    // IntegerBox % IntegerBox
    if let (Some(left_int), Some(right_int)) = (
        left.as_any().downcast_ref::<IntegerBox>(),
        right.as_any().downcast_ref::<IntegerBox>(),
    ) {
        if right_int.value == 0 {
            return Err("Modulo by zero".to_string());
        }
        return Ok(Box::new(IntegerBox::new(left_int.value % right_int.value)));
    }

    Err(format!(
        "Modulo not supported between {} and {}",
        left.type_name(),
        right.type_name()
    ))
}

impl NyashInterpreter {
    /// 二項演算を実行 - Binary operation processing
    pub(super) fn execute_binary_op(
        &mut self,
        op: &BinaryOperator,
        left: &ASTNode,
        right: &ASTNode,
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let left_val = self.execute_expression(left)?;
        let right_val = self.execute_expression(right)?;
        // Binary operation execution

        match op {
            BinaryOperator::Add => {
                // Optional: enforce grammar rule for add (behind env)
                if std::env::var("NYASH_GRAMMAR_ENFORCE_ADD").ok().as_deref() == Some("1") {
                    let lty = if crate::runtime::semantics::coerce_to_string(left_val.as_ref())
                        .is_some()
                    {
                        "String"
                    } else if crate::runtime::semantics::coerce_to_i64(left_val.as_ref()).is_some()
                    {
                        "Integer"
                    } else {
                        "Other"
                    };
                    let rty = if crate::runtime::semantics::coerce_to_string(right_val.as_ref())
                        .is_some()
                    {
                        "String"
                    } else if crate::runtime::semantics::coerce_to_i64(right_val.as_ref()).is_some()
                    {
                        "Integer"
                    } else {
                        "Other"
                    };
                    if let Some((res, _act)) =
                        crate::grammar::engine::get().decide_add_result(lty, rty)
                    {
                        match res {
                            "String" => {
                                let ls =
                                    crate::runtime::semantics::coerce_to_string(left_val.as_ref())
                                        .unwrap_or_else(|| left_val.to_string_box().value);
                                let rs =
                                    crate::runtime::semantics::coerce_to_string(right_val.as_ref())
                                        .unwrap_or_else(|| right_val.to_string_box().value);
                                return Ok(Box::new(StringBox::new(format!("{}{}", ls, rs))));
                            }
                            "Integer" => {
                                if let (Some(li), Some(ri)) = (
                                    crate::runtime::semantics::coerce_to_i64(left_val.as_ref()),
                                    crate::runtime::semantics::coerce_to_i64(right_val.as_ref()),
                                ) {
                                    return Ok(Box::new(IntegerBox::new(li + ri)));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                let (strat, lty, rty, expect) =
                    if std::env::var("NYASH_GRAMMAR_DIFF").ok().as_deref() == Some("1") {
                        let strat = crate::grammar::engine::get().add_coercion_strategy();
                        let lty = if crate::runtime::semantics::coerce_to_string(left_val.as_ref())
                            .is_some()
                        {
                            "String"
                        } else if crate::runtime::semantics::coerce_to_i64(left_val.as_ref())
                            .is_some()
                        {
                            "Integer"
                        } else {
                            "Other"
                        };
                        let rty = if crate::runtime::semantics::coerce_to_string(right_val.as_ref())
                            .is_some()
                        {
                            "String"
                        } else if crate::runtime::semantics::coerce_to_i64(right_val.as_ref())
                            .is_some()
                        {
                            "Integer"
                        } else {
                            "Other"
                        };
                        let rule = crate::grammar::engine::get().decide_add_result(lty, rty);
                        (
                            Some(strat.to_string()),
                            Some(lty.to_string()),
                            Some(rty.to_string()),
                            rule.map(|(res, act)| (res.to_string(), act.to_string())),
                        )
                    } else {
                        (None, None, None, None)
                    };
                // 1) Intrinsic fast-paths (Integer+Integer, String+*, Bool+Bool)
                if let Some(result) = try_add_operation(left_val.as_ref(), right_val.as_ref()) {
                    if let (Some(s), Some(l), Some(r)) =
                        (strat.as_ref(), lty.as_ref(), rty.as_ref())
                    {
                        let actual = if result.as_any().downcast_ref::<StringBox>().is_some() {
                            "String"
                        } else if result.as_any().downcast_ref::<IntegerBox>().is_some() {
                            "Integer"
                        } else {
                            "Other"
                        };
                        eprintln!("[GRAMMAR-DIFF][Interp] add strat={} lty={} rty={} expect={:?} actual={} match={}", s, l, r, expect, actual, expect.as_ref().map(|(res,_)| res.as_str())==Some(actual));
                    }
                    return Ok(result);
                }
                // 2) Concatenation if either side is string-like (semantics)
                let ls_opt = crate::runtime::semantics::coerce_to_string(left_val.as_ref());
                let rs_opt = crate::runtime::semantics::coerce_to_string(right_val.as_ref());
                if ls_opt.is_some() || rs_opt.is_some() {
                    let ls = ls_opt.unwrap_or_else(|| left_val.to_string_box().value);
                    let rs = rs_opt.unwrap_or_else(|| right_val.to_string_box().value);
                    if let (Some(s), Some(l), Some(r)) =
                        (strat.as_ref(), lty.as_ref(), rty.as_ref())
                    {
                        eprintln!("[GRAMMAR-DIFF][Interp] add strat={} lty={} rty={} expect={:?} actual=String match={}", s, l, r, expect, expect.as_ref().map(|(res,_)| res=="String").unwrap_or(false));
                    }
                    return Ok(Box::new(StringBox::new(format!("{}{}", ls, rs))));
                }
                // 3) Numeric fallback via coerce_to_i64
                if let (Some(li), Some(ri)) = (
                    crate::runtime::semantics::coerce_to_i64(left_val.as_ref()),
                    crate::runtime::semantics::coerce_to_i64(right_val.as_ref()),
                ) {
                    if let (Some(s), Some(l), Some(r)) =
                        (strat.as_ref(), lty.as_ref(), rty.as_ref())
                    {
                        eprintln!("[GRAMMAR-DIFF][Interp] add strat={} lty={} rty={} expect={:?} actual=Integer match={}", s, l, r, expect, expect.as_ref().map(|(res,_)| res=="Integer").unwrap_or(false));
                    }
                    return Ok(Box::new(IntegerBox::new(li + ri)));
                }
                // 4) Final error
                if let (Some(s), Some(l), Some(r)) = (strat.as_ref(), lty.as_ref(), rty.as_ref()) {
                    eprintln!("[GRAMMAR-DIFF][Interp] add strat={} lty={} rty={} expect={:?} actual=Error", s, l, r, expect);
                }
                Err(RuntimeError::InvalidOperation {
                    message: format!(
                        "Addition not supported between {} and {}",
                        left_val.type_name(),
                        right_val.type_name()
                    ),
                })
            }

            BinaryOperator::Equal => {
                let result = left_val.equals(right_val.as_ref());
                Ok(Box::new(result))
            }

            BinaryOperator::NotEqual => {
                let result = left_val.equals(right_val.as_ref());
                Ok(Box::new(BoolBox::new(!result.value)))
            }

            BinaryOperator::And => {
                let left_bool = self.is_truthy(&left_val);
                if !left_bool {
                    Ok(Box::new(BoolBox::new(false)))
                } else {
                    let right_bool = self.is_truthy(&right_val);
                    Ok(Box::new(BoolBox::new(right_bool)))
                }
            }

            BinaryOperator::Or => {
                let left_bool = self.is_truthy(&left_val);
                if left_bool {
                    Ok(Box::new(BoolBox::new(true)))
                } else {
                    let right_bool = self.is_truthy(&right_val);
                    Ok(Box::new(BoolBox::new(right_bool)))
                }
            }

            BinaryOperator::Subtract => {
                if std::env::var("NYASH_GRAMMAR_DIFF").ok().as_deref() == Some("1") {
                    let strat = crate::grammar::engine::get().sub_coercion_strategy();
                    let lty = if crate::runtime::semantics::coerce_to_string(left_val.as_ref())
                        .is_some()
                    {
                        "String"
                    } else if crate::runtime::semantics::coerce_to_i64(left_val.as_ref()).is_some()
                    {
                        "Integer"
                    } else {
                        "Other"
                    };
                    let rty = if crate::runtime::semantics::coerce_to_string(right_val.as_ref())
                        .is_some()
                    {
                        "String"
                    } else if crate::runtime::semantics::coerce_to_i64(right_val.as_ref()).is_some()
                    {
                        "Integer"
                    } else {
                        "Other"
                    };
                    let rule = crate::grammar::engine::get().decide_sub_result(lty, rty);
                    eprintln!(
                        "[GRAMMAR-DIFF][Interp] sub strat={} lty={} rty={} expect={:?}",
                        strat, lty, rty, rule
                    );
                }
                // Use helper function instead of trait methods
                if let Some(result) = try_sub_operation(left_val.as_ref(), right_val.as_ref()) {
                    return Ok(result);
                }

                Err(RuntimeError::InvalidOperation {
                    message: format!(
                        "Subtraction not supported between {} and {}",
                        left_val.type_name(),
                        right_val.type_name()
                    ),
                })
            }

            BinaryOperator::Multiply => {
                if std::env::var("NYASH_GRAMMAR_DIFF").ok().as_deref() == Some("1") {
                    let strat = crate::grammar::engine::get().mul_coercion_strategy();
                    let lty = if crate::runtime::semantics::coerce_to_string(left_val.as_ref())
                        .is_some()
                    {
                        "String"
                    } else if crate::runtime::semantics::coerce_to_i64(left_val.as_ref()).is_some()
                    {
                        "Integer"
                    } else {
                        "Other"
                    };
                    let rty = if crate::runtime::semantics::coerce_to_string(right_val.as_ref())
                        .is_some()
                    {
                        "String"
                    } else if crate::runtime::semantics::coerce_to_i64(right_val.as_ref()).is_some()
                    {
                        "Integer"
                    } else {
                        "Other"
                    };
                    let rule = crate::grammar::engine::get().decide_mul_result(lty, rty);
                    eprintln!(
                        "[GRAMMAR-DIFF][Interp] mul strat={} lty={} rty={} expect={:?}",
                        strat, lty, rty, rule
                    );
                }
                // Use helper function instead of trait methods
                if let Some(result) = try_mul_operation(left_val.as_ref(), right_val.as_ref()) {
                    return Ok(result);
                }

                Err(RuntimeError::InvalidOperation {
                    message: format!(
                        "Multiplication not supported between {} and {}",
                        left_val.type_name(),
                        right_val.type_name()
                    ),
                })
            }

            BinaryOperator::Divide => {
                if std::env::var("NYASH_GRAMMAR_DIFF").ok().as_deref() == Some("1") {
                    let strat = crate::grammar::engine::get().div_coercion_strategy();
                    let lty = if crate::runtime::semantics::coerce_to_string(left_val.as_ref())
                        .is_some()
                    {
                        "String"
                    } else if crate::runtime::semantics::coerce_to_i64(left_val.as_ref()).is_some()
                    {
                        "Integer"
                    } else {
                        "Other"
                    };
                    let rty = if crate::runtime::semantics::coerce_to_string(right_val.as_ref())
                        .is_some()
                    {
                        "String"
                    } else if crate::runtime::semantics::coerce_to_i64(right_val.as_ref()).is_some()
                    {
                        "Integer"
                    } else {
                        "Other"
                    };
                    let rule = crate::grammar::engine::get().decide_div_result(lty, rty);
                    eprintln!(
                        "[GRAMMAR-DIFF][Interp] div strat={} lty={} rty={} expect={:?}",
                        strat, lty, rty, rule
                    );
                }
                // Use helper function instead of trait methods
                match try_div_operation(left_val.as_ref(), right_val.as_ref()) {
                    Ok(result) => Ok(result),
                    Err(error_msg) => Err(RuntimeError::InvalidOperation { message: error_msg }),
                }
            }

            BinaryOperator::Modulo => {
                // Use helper function for modulo operation
                match try_mod_operation(left_val.as_ref(), right_val.as_ref()) {
                    Ok(result) => Ok(result),
                    Err(error_msg) => Err(RuntimeError::InvalidOperation { message: error_msg }),
                }
            }

            BinaryOperator::Shl => {
                // Integer-only left shift
                if let (Some(li), Some(ri)) = (
                    crate::runtime::semantics::coerce_to_i64(left_val.as_ref()),
                    crate::runtime::semantics::coerce_to_i64(right_val.as_ref()),
                ) {
                    let sh = (ri as u32) & 63;
                    return Ok(Box::new(IntegerBox::new(li.wrapping_shl(sh))));
                }
                Err(RuntimeError::TypeError {
                    message: format!(
                        "Shift-left '<<' requires integers (got {} and {})",
                        left_val.type_name(),
                        right_val.type_name()
                    ),
                })
            }
            BinaryOperator::Shr => {
                if let (Some(li), Some(ri)) = (
                    crate::runtime::semantics::coerce_to_i64(left_val.as_ref()),
                    crate::runtime::semantics::coerce_to_i64(right_val.as_ref()),
                ) {
                    let sh = (ri as u32) & 63;
                    return Ok(Box::new(IntegerBox::new(((li as u64) >> sh) as i64)));
                }
                Err(RuntimeError::TypeError {
                    message: format!(
                        "Shift-right '>>' requires integers (got {} and {})",
                        left_val.type_name(),
                        right_val.type_name()
                    ),
                })
            }
            BinaryOperator::BitAnd => {
                if let (Some(li), Some(ri)) = (
                    crate::runtime::semantics::coerce_to_i64(left_val.as_ref()),
                    crate::runtime::semantics::coerce_to_i64(right_val.as_ref()),
                ) {
                    return Ok(Box::new(IntegerBox::new(li & ri)));
                }
                Err(RuntimeError::TypeError {
                    message: format!(
                        "Bitwise '&' requires integers (got {} and {})",
                        left_val.type_name(),
                        right_val.type_name()
                    ),
                })
            }
            BinaryOperator::BitOr => {
                if let (Some(li), Some(ri)) = (
                    crate::runtime::semantics::coerce_to_i64(left_val.as_ref()),
                    crate::runtime::semantics::coerce_to_i64(right_val.as_ref()),
                ) {
                    return Ok(Box::new(IntegerBox::new(li | ri)));
                }
                Err(RuntimeError::TypeError {
                    message: format!(
                        "Bitwise '|' requires integers (got {} and {})",
                        left_val.type_name(),
                        right_val.type_name()
                    ),
                })
            }
            BinaryOperator::BitXor => {
                if let (Some(li), Some(ri)) = (
                    crate::runtime::semantics::coerce_to_i64(left_val.as_ref()),
                    crate::runtime::semantics::coerce_to_i64(right_val.as_ref()),
                ) {
                    return Ok(Box::new(IntegerBox::new(li ^ ri)));
                }
                Err(RuntimeError::TypeError {
                    message: format!(
                        "Bitwise '^' requires integers (got {} and {})",
                        left_val.type_name(),
                        right_val.type_name()
                    ),
                })
            }

            BinaryOperator::Less => {
                let result = CompareBox::less(left_val.as_ref(), right_val.as_ref());
                Ok(Box::new(result))
            }

            BinaryOperator::Greater => {
                let result = CompareBox::greater(left_val.as_ref(), right_val.as_ref());
                Ok(Box::new(result))
            }

            BinaryOperator::LessEqual => {
                let result = CompareBox::less_equal(left_val.as_ref(), right_val.as_ref());
                Ok(Box::new(result))
            }

            BinaryOperator::GreaterEqual => {
                let result = CompareBox::greater_equal(left_val.as_ref(), right_val.as_ref());
                Ok(Box::new(result))
            }
        }
    }

    /// 単項演算を実行 - Unary operation processing
    pub(super) fn execute_unary_op(
        &mut self,
        operator: &UnaryOperator,
        operand: &ASTNode,
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let operand_val = self.execute_expression(operand)?;

        match operator {
            UnaryOperator::Minus => {
                // 数値の符号反転
                if let Some(int_box) = operand_val.as_any().downcast_ref::<IntegerBox>() {
                    Ok(Box::new(IntegerBox::new(-int_box.value)))
                } else if let Some(float_box) = operand_val.as_any().downcast_ref::<FloatBox>() {
                    Ok(Box::new(FloatBox::new(-float_box.value)))
                } else {
                    Err(RuntimeError::TypeError {
                        message: "Unary minus can only be applied to Integer or Float".to_string(),
                    })
                }
            }
            UnaryOperator::Not => {
                // 論理否定
                if let Some(bool_box) = operand_val.as_any().downcast_ref::<BoolBox>() {
                    Ok(Box::new(BoolBox::new(!bool_box.value)))
                } else {
                    // どんな値でもtruthyness判定してnot演算を適用
                    let is_truthy = self.is_truthy(&operand_val);
                    Ok(Box::new(BoolBox::new(!is_truthy)))
                }
            }
        }
    }
}
