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
            let mut extern_call = |iface_name: &str, method_name: &str, _effects: EffectMask, returns: bool| -> Result<ValueId, String> {
                let result_id = if returns { Some(self.value_gen.next()) } else { None };
                let full = format!("{}.{}", iface_name, method_name);
                self.emit_unified_call(
                    result_id,
                    super::super::builder_calls::CallTarget::Extern(full),
                    arg_values.clone(),
                )?;
                Ok(result_id.unwrap_or_else(|| crate::mir::builder::emission::constant::emit_void(self)))
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
        // Canonicalize class for alias-alias form: JsonFragBox_JsonFragBox → JsonFragBox
        let canon_cls = if let Some((a,b)) = cls_name.split_once('_') { if a == b { a.to_string() } else { cls_name.clone() } } else { cls_name.clone() };
        // Build canonical function name strictly via normalizer to preserve underscores
        let fun_name = match crate::mir::resolve::call_name_resolver::CallNameResolverBox::static_name(&canon_cls, method, arg_values.len()) {
            Ok(n) => n,
            Err(e) => return Some(Err(e)),
        };
        // Prefer unified ModuleFunction callee to avoid legacy recursion
        let mut args_local = arg_values.clone();
        if let Err(e) = self.emit_call_with_guard(
            Some(result_id),
            super::super::ValueId::new(0),
            crate::mir::Callee::ModuleFunction(fun_name.clone()),
            args_local,
            EffectMask::READ.add(Effect::ReadHeap),
        ) {
            return Some(Err(e));
        }
        self.annotate_call_result_from_func_name(result_id, &fun_name);
        Some(Ok(result_id))
    }
}
