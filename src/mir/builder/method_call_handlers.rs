//! Method call handlers for MIR builder
//!
//! This module contains specialized handlers for different types of method calls,
//! following the Single Responsibility Principle.

use crate::ast::ASTNode;
use crate::mir::builder::{MirBuilder, ValueId};
use crate::mir::builder::builder_calls::CallTarget;
use crate::mir::{MirInstruction, TypeOpKind};

impl MirBuilder {
    /// Handle static method calls: BoxName.method(args)
    pub(super) fn handle_static_method_call(
        &mut self,
        box_name: &str,
        method: &str,
        arguments: &[ASTNode],
    ) -> Result<ValueId, String> {
        // Build argument values
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.build_expression(arg.clone())?);
        }

        // Compose lowered function name: BoxName.method/N
        let func_name = format!("{}.{}/{}", box_name, method, arg_values.len());
        let dst = self.value_gen.next();

        if std::env::var("NYASH_STATIC_CALL_TRACE").ok().as_deref() == Some("1") {
            eprintln!("[builder] static-call {}", func_name);
        }

        // If exact name is not present (aliased prelude renamed to Alias_Top),
        // try unique tail-match fallback (…<Box>.<method>/<arity>)
        let target_name = if let Some(ref module) = self.current_module {
            if module.functions.contains_key(&func_name) {
                func_name.clone()
            } else {
                let idx = crate::mir::indexes::functions::FunctionIndex::new(module);
                match idx.tail_unique(Some(box_name), method, arg_values.len()) {
                    crate::mir::indexes::functions::TailQueryResult::Unique(n) => n,
                    crate::mir::indexes::functions::TailQueryResult::Ambiguous(mut cands) => {
                        if std::env::var("NYASH_MIR_CALL_MODULE_FN_STRICT").ok().as_deref() == Some("1") {
                            cands.sort();
                            return Err(format!(
                                "Ambiguous static method resolution for '{}.{}', arity={} ({} candidates): {}",
                                box_name, method, arg_values.len(), cands.len(), cands.join(", ")
                            ));
                        }
                        // method-only unique fallback
                        match idx.tail_unique(None, method, arg_values.len()) {
                            crate::mir::indexes::functions::TailQueryResult::Unique(n2) => n2,
                            crate::mir::indexes::functions::TailQueryResult::Ambiguous(c2) => {
                                if std::env::var("NYASH_MIR_CALL_MODULE_FN_STRICT").ok().as_deref() == Some("1") {
                                    let mut c2s = c2; let shown = c2s.len().min(10); c2s.sort();
                                    return Err(format!(
                                        "Ambiguous static method resolution (method-only) for '{}', arity={} ({} candidates, showing {}): {}",
                                        method, arg_values.len(), c2s.len(), shown, c2s.iter().take(shown).cloned().collect::<Vec<_>>().join(", ")
                                    ));
                                }
                                func_name.clone()
                            }
                            crate::mir::indexes::functions::TailQueryResult::None => func_name.clone(),
                        }
                    }
                    crate::mir::indexes::functions::TailQueryResult::None => {
                        // method-only unique fallback
                        match idx.tail_unique(None, method, arg_values.len()) {
                            crate::mir::indexes::functions::TailQueryResult::Unique(n2) => n2,
                            _ => func_name.clone(),
                        }
                    }
                }
            }
        } else {
            func_name.clone()
        };

        // Prefer ModuleFunction under env gate when the function exists in the module
        let use_modfn = std::env::var("NYASH_MIR_CALL_MODULE_FN").ok().as_deref() == Some("1");
        if use_modfn {
            if let Some(ref module) = self.current_module {
                if module.functions.contains_key(&target_name) {
                    let name_val = crate::mir::builder::name_const::make_name_const_result(self, &target_name)?;
                    self.emit_instruction(MirInstruction::Call {
                        dst: Some(dst),
                        func: name_val,
                        callee: Some(crate::mir::Callee::ModuleFunction(target_name.clone())),
                        args: arg_values,
                        effects: crate::mir::EffectMask::READ.add(crate::mir::Effect::ReadHeap),
                    })?;
                    self.annotate_call_result_from_func_name(dst, &target_name);
                    return Ok(dst);
                }
            }
        }

        // Fallback: legacy global-call emission to keep behavior identical
        self.emit_legacy_call(Some(dst), CallTarget::Global(target_name), arg_values)?;
        Ok(dst)
    }

    /// Handle TypeOp method calls: value.is("Type") and value.as("Type")
    pub(super) fn handle_typeop_method(
        &mut self,
        object_value: ValueId,
        method: &str,
        type_name: &str,
    ) -> Result<ValueId, String> {
        let mir_ty = Self::parse_type_name_to_mir(type_name);
        let dst = self.value_gen.next();
        let op = if method == "is" {
            TypeOpKind::Check
        } else {
            TypeOpKind::Cast
        };

        self.emit_instruction(MirInstruction::TypeOp {
            dst,
            op,
            value: object_value,
            ty: mir_ty,
        })?;

        Ok(dst)
    }

    /// Check if this is a TypeOp method call
    pub(super) fn is_typeop_method(method: &str, arguments: &[ASTNode]) -> Option<String> {
        if (method == "is" || method == "as") && arguments.len() == 1 {
            Self::extract_string_literal(&arguments[0])
        } else {
            None
        }
    }

    /// Handle me.method() calls within static box context
    pub(super) fn handle_me_method_call(
        &mut self,
        method: &str,
        arguments: &[ASTNode],
    ) -> Result<Option<ValueId>, String> {
        // Convert slice to Vec for compatibility
        let args_vec = arguments.to_vec();
        // Delegate to existing try_handle_me_direct_call
        match self.try_handle_me_direct_call(method, &args_vec) {
            Some(result) => result.map(Some),
            None => Ok(None),
        }
    }

    /// Handle standard Box/Plugin method calls (fallback)
    pub(super) fn handle_standard_method_call(
        &mut self,
        object_value: ValueId,
        method: String,
        arguments: &[ASTNode],
    ) -> Result<ValueId, String> {
        // Build argument values
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.build_expression(arg.clone())?);
        }

        // Receiver class hintは emit_unified_call 側で起源/型から判断する（重複回避）
        // 統一経路: emit_unified_call に委譲（RouterPolicy と rewrite::* で安定化）
        let dst = self.value_gen.next();
        self.emit_unified_call(
            Some(dst),
            CallTarget::Method { box_type: None, method, receiver: object_value },
            arg_values,
        )?;
        Ok(dst)
    }
}
