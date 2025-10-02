// Special call handlers (math, env, me direct calls)
use super::super::{Effect, EffectMask, MirBuilder, MirInstruction, MirType, ValueId};
use crate::ast::{ASTNode, LiteralValue};
use crate::mir::builder::calls::{extern_calls, special_handlers};
use crate::mir::TypeOpKind;

impl MirBuilder {
    /// Try handle math.* function in function-style (sin/cos/abs/min/max).
    /// Returns Some(result) if handled, otherwise None.
    pub(in super::super) fn try_handle_math_function(
        &mut self,
        name: &str,
        raw_args: Vec<ASTNode>,
    ) -> Option<Result<ValueId, String>> {
        if !special_handlers::is_math_function(name) {
            return None;
        }
        // Build numeric args directly for math.* to preserve f64 typing
        let mut math_args: Vec<ValueId> = Vec::new();
        for a in raw_args.into_iter() {
            match a {
                ASTNode::New { class, arguments, .. } if class == "FloatBox" && arguments.len() == 1 => {
                    match self.build_expression(arguments[0].clone()) { v @ Ok(_) => math_args.push(v.unwrap()), err @ Err(_) => return Some(err), }
                }
                ASTNode::New { class, arguments, .. } if class == "IntegerBox" && arguments.len() == 1 => {
                    let iv = match self.build_expression(arguments[0].clone()) { Ok(v) => v, Err(e) => return Some(Err(e)) };
                    let fv = self.value_gen.next();
                    if let Err(e) = self.emit_instruction(MirInstruction::TypeOp { dst: fv, op: TypeOpKind::Cast, value: iv, ty: MirType::Float }) { return Some(Err(e)); }
                    math_args.push(fv);
                }
                ASTNode::Literal { value: LiteralValue::Float(_), .. } => {
                    match self.build_expression(a) { v @ Ok(_) => math_args.push(v.unwrap()), err @ Err(_) => return Some(err), }
                }
                other => {
                    match self.build_expression(other) { v @ Ok(_) => math_args.push(v.unwrap()), err @ Err(_) => return Some(err), }
                }
            }
        }
        // new MathBox()
        let math_recv = self.value_gen.next();
        if let Err(e) = self.emit_constructor_call(math_recv, "MathBox".to_string(), vec![]) { return Some(Err(e)); }
        self.origin_register(math_recv, "MathBox".to_string());
        // birth()
        if let Err(e) = self.emit_method_call(None, math_recv, "birth".to_string(), vec![]) { return Some(Err(e)); }
        // call method
        let dst = self.value_gen.next();
        if let Err(e) = self.emit_method_call(Some(dst), math_recv, name.to_string(), math_args) { return Some(Err(e)); }
        Some(Ok(dst))
    }

    /// Try handle env.* extern methods like env.console.log via FieldAccess(object, field).
    pub(in super::super) fn try_handle_env_method(
        &mut self,
        object: &ASTNode,
        method: &str,
        arguments: &Vec<ASTNode>,
    ) -> Option<Result<ValueId, String>> {
        let ASTNode::FieldAccess { object: env_obj, field: env_field, .. } = object else { return None; };
        if let ASTNode::Variable { name: env_name, .. } = env_obj.as_ref() {
            if env_name != "env" && env_name != "nyrt" { return None; }
            // Build arguments once
            let mut arg_values = Vec::new();
            for arg in arguments {
                match self.build_expression(arg.clone()) { Ok(v) => arg_values.push(v), Err(e) => return Some(Err(e)) }
            }
            let iface = env_field.as_str();
            let m = method;
            let mut extern_call = |iface_name: &str, method_name: &str, effects: EffectMask, returns: bool| -> Result<ValueId, String> {
                let result_id = self.value_gen.next();
                self.emit_instruction(MirInstruction::ExternCall { dst: if returns { Some(result_id) } else { None }, iface_name: iface_name.to_string(), method_name: method_name.to_string(), args: arg_values.clone(), effects })?;
                if returns {
                    Ok(result_id)
                } else {
                    let void_id = crate::mir::builder::emission::constant::emit_void(self);
                    Ok(void_id)
                }
            };
            if env_name == "env" {
                if let Some((iface_name, method_name, effects, returns)) =
                    extern_calls::get_env_method_spec(iface, m)
                {
                    return Some(extern_call(&iface_name, &method_name, effects, returns));
                }
                return None;
            }
            if env_name == "nyrt" {
                match (iface, m) {
                    ("time", "now_ms") => {
                        return Some(extern_call(
                            "nyrt.time",
                            "now_ms",
                            EffectMask::READ,
                            true,
                        ));
                    }
                    _ => return None,
                }
            }
        }
        None
    }

    /// Try direct static call for `me` in static box
    pub(in super::super) fn try_handle_me_direct_call(
        &mut self,
        method: &str,
        arguments: &Vec<ASTNode>,
    ) -> Option<Result<ValueId, String>> {
        let Some(cls_name) = self.current_static_box.clone() else { return None; };
        // Build args
        let mut arg_values = Vec::new();
        for a in arguments {
            match self.build_expression(a.clone()) { Ok(v) => arg_values.push(v), Err(e) => return Some(Err(e)) }
        }
        let result_id = self.value_gen.next();
        let fun_name = format!("{}.{}{}", cls_name, method, format!("/{}", arg_values.len()));
        let fun_val = match crate::mir::builder::name_const::make_name_const_result(self, &fun_name) {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        };
        if let Err(e) = self.emit_instruction(MirInstruction::Call {
            dst: Some(result_id),
            func: fun_val,
            callee: None, // use legacy module resolution for static helper
            args: arg_values,
            effects: EffectMask::READ.add(Effect::ReadHeap)
        }) { return Some(Err(e)); }
        // Annotate from lowered function signature if present
        self.annotate_call_result_from_func_name(result_id, &fun_name);
        Some(Ok(result_id))
    }
}
