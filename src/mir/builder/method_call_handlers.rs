//! Method call handlers for MIR builder
//!
//! This module contains specialized handlers for different types of method calls,
//! following the Single Responsibility Principle.

use crate::ast::ASTNode;
use crate::mir::builder::{MirBuilder, ValueId};
use crate::mir::builder::builder_calls::CallTarget;
use crate::mir::{MirInstruction, TypeOpKind, MirType};

impl MirBuilder {
    /// Handle static method calls: BoxName.method(args)
    pub(super) fn handle_static_method_call(
        &mut self,
        box_name: &str,
        method: &str,
        arguments: &[ASTNode],
    ) -> Result<ValueId, String> {
        // Special: unborn constructor — create instance without auto-birth
        if method == "unborn" {
            let mut arg_values = Vec::new();
            for arg in arguments { arg_values.push(self.build_expression(arg.clone())?); }
            let dst = self.value_gen.next();
            self.emit_instruction(MirInstruction::NewBox { dst, box_type: box_name.to_string(), args: arg_values , auto_birth: None})?;
            // Origin register for downstream routing/debug
            self.origin_register(dst, box_name.to_string());
            return Ok(dst);
        }
        // Build argument values
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.build_expression(arg.clone())?);
        }


        // Compose lowered function name: BoxName.method/N
        let func_name = match crate::mir::resolve::call_name_resolver::CallNameResolverBox::static_name(box_name, method, arg_values.len()) {
            Ok(n) => n,
            Err(e) => return Err(format!("static call name error: {}", e)),
        };
        let dst = self.value_gen.next();

        if std::env::var("NYASH_STATIC_CALL_TRACE").ok().as_deref() == Some("1") {
            eprintln!("[builder] static-call {}", func_name);
        }

        // Strict: if the exact canonical function is absent in this module,
        // fall back to the dotted base and let VM handle resolution (and optional legacy).
        // Use canonical function name produced by resolver (e.g., Box.method/Arity)
        let target_name = func_name.clone();

        // Prefer ModuleFunction emission; VM側の末尾一致フォールバックで alias/宣言名の揺れを吸収
        let name_val = crate::mir::builder::name_const::make_name_const_result(self, &target_name)?;
        self.emit_instruction(MirInstruction::Call {
            dst: Some(dst),
            func: name_val,
            callee: Some(crate::mir::Callee::ModuleFunction(target_name.clone())),
            args: arg_values,
            effects: crate::mir::EffectMask::READ.add(crate::mir::Effect::ReadHeap),
        })?;
        self.annotate_call_result_from_func_name(dst, &target_name);
        return Ok(dst)
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
    /// 📦 Kept for future use: type system extensions (runtime type checking/casting)
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

        // Core containers fast-path: when the receiver originated from a
        // builtin core box (ArrayBox/MapBox/StringBox), prefer emitting a
        // BoxCall so VM can dispatch to the builtin handlers. This avoids
        // accidental shadowing by user-defined boxes with the same name in
        // prelude files (e.g., hakorune std ArrayBox), while keeping behavior
        // consistent for user boxes (their NewBox origin will not be core).
        if method != "birth" {
            // Small helper: table-driven lowering entrypoint
            #[inline]
            fn try_lower_via_table(builder: &mut MirBuilder, recv_cls: Option<&str>, method: &str, obj: ValueId, args: &mut Vec<ValueId>) -> Option<ValueId> {
                if let Some(spec) = crate::mir::builder::lowering::lower_builtin_method(recv_cls, method, args.len()) {
                    let dst = builder.value_gen.next();
                    let full = spec.extern_name.to_string();
                    let name_const = crate::mir::builder::name_const::make_name_const_result(builder, &full).ok()?;
                    let call_args: Vec<ValueId> = if spec.prepend_recv {
                        let mut v = Vec::with_capacity(1 + args.len());
                        v.push(obj);
                        v.extend(args.drain(..));
                        v
                    } else {
                        args.drain(..).collect()
                    };
                    let _ = builder.emit_instruction(MirInstruction::Call {
                        dst: Some(dst),
                        func: name_const,
                        callee: Some(crate::mir::Callee::Extern(full)),
                        args: call_args,
                        effects: crate::mir::EffectMask::READ.add(crate::mir::Effect::ReadHeap),
                    });
                    builder.annotate_call_result_from_func_name(dst, method);
                    return Some(dst);
                }
                None
            }
            let origin_cls: Option<String> = self.origin_get(object_value).map(|s| s.to_string());
            // Also infer from value_types when origin is unknown
            let inferred_string = matches!(self.value_types.get(&object_value), Some(MirType::String))
                || matches!(self.value_types.get(&object_value), Some(MirType::Box(b)) if b == "StringBox");
            if let Some(cls) = origin_cls.as_deref() {
                if let Some(dst) = try_lower_via_table(self, Some(cls), &method, object_value, &mut arg_values) { return Ok(dst); }
                if cls == "ArrayBox" || cls == "MapBox" || cls == "StringBox" {
                    // Emit BoxCall only when we have a known builtin method_id
                    if let Some(mid_u32) = crate::runtime::type_registry::resolve_builtin_method_id(cls, &method) {
                        let dst = self.value_gen.next();
                        self.emit_instruction(MirInstruction::BoxCall {
                            dst: Some(dst),
                            box_val: object_value,
                            method: method.clone(),
                            method_id: Some(mid_u32 as u16),
                            args: arg_values,
                            effects: crate::mir::EffectMask::READ.add(crate::mir::Effect::ReadHeap),
                        })?;
                        // Conservatively annotate result type based on method name
                        self.annotate_call_result_from_func_name(dst, &method);
                        return Ok(dst);
                    }
                }
            }
            // Lower string methods for inferred String types (literals/expressions)
            if inferred_string {
                if let Some(dst) = try_lower_via_table(self, Some("StringBox"), &method, object_value, &mut arg_values) { return Ok(dst); }
            }
        }


        // Special-case: instance.birth(args) must call user-defined ModuleFunction(Class.birth/N)
        // to ensure contracts and user Box initializer run even when RouterPolicy prefers BoxCall.
        if method == "birth" {
            let dst = self.value_gen.next();
            // Delegate to legacy path which rewrites Method-birth → ModuleFunction(Class.birth/N)
            self.emit_legacy_call(
                Some(dst),
                CallTarget::Method { box_type: None, method, receiver: object_value },
                arg_values,
            )?;
            return Ok(dst);
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
