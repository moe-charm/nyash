use super::{EffectMask, FunctionSignature, MirFunction, MirInstruction, MirModule, MirType, ValueId};
use crate::ast::ASTNode;

// Lifecycle routines extracted from builder.rs
impl super::MirBuilder {
    /// Unified declaration indexing (Phase A): collect symbols before lowering
    /// - user_defined_boxes: non-static Box names (for NewBox birth() skip)
    /// - static_method_index: name -> [(BoxName, arity)] (for bare-call fallback)
    fn index_declarations(&mut self, node: &ASTNode) {
        match node {
            ASTNode::Program { statements, .. } => {
                for st in statements {
                    self.index_declarations(st);
                }
            }
            ASTNode::BoxDeclaration { name, methods, is_static, .. } => {
                if !*is_static {
                    self.user_defined_boxes.insert(name.clone());
                } else {
                    for (mname, mast) in methods {
                        if let ASTNode::FunctionDeclaration { params, .. } = mast {
                            self.static_method_index
                                .entry(mname.clone())
                                .or_insert_with(Vec::new)
                                .push((name.clone(), params.len()));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    pub(super) fn prepare_module(&mut self) -> Result<(), String> {
        let module = MirModule::new("main".to_string());
        let main_signature = FunctionSignature {
            name: "main".to_string(),
            params: vec![],
            return_type: MirType::Void,
            effects: EffectMask::PURE,
        };

        let entry_block = self.block_gen.next();
        let mut main_function = MirFunction::new(main_signature, entry_block);
        main_function.metadata.is_entry_point = true;

        self.current_module = Some(module);
        self.current_function = Some(main_function);
        self.current_block = Some(entry_block);

        // Hint: scope enter at function entry (id=0 for main)
        self.hint_scope_enter(0);

        if std::env::var("NYASH_BUILDER_SAFEPOINT_ENTRY")
            .ok()
            .as_deref()
            == Some("1")
        {
            self.emit_instruction(MirInstruction::Safepoint)?;
        }

        Ok(())
    }

    pub(super) fn lower_root(&mut self, ast: ASTNode) -> Result<ValueId, String> {
        // Pre-index static methods to enable safe fallback for bare calls in using-prepended code
        let snapshot = ast.clone();
        // Phase A: collect declarations in one pass (symbols available to lowering)
        self.index_declarations(&snapshot);
        // Phase B: top-level program lowering with declaration-first pass
        match ast {
            ASTNode::Program { statements, .. } => {
                use crate::ast::ASTNode as N;
                // First pass: lower declarations (static boxes except Main, and instance boxes)
                let mut main_static: Option<(String, std::collections::HashMap<String, ASTNode>)> = None;
                for st in &statements {
                    if let N::BoxDeclaration {
                        name,
                        methods,
                        is_static,
                        fields,
                        constructors,
                        weak_fields,
                        ..
                    } = st
                    {
                        if *is_static {
                            if name == "Main" {
                                main_static = Some((name.clone(), methods.clone()));
                            } else {
                                // Lower all static methods into standalone functions: BoxName.method/Arity
                                for (mname, mast) in methods.iter() {
                                    if let N::FunctionDeclaration { params, body, .. } = mast {
                                        let func_name = format!("{}.{}{}", name, mname, format!("/{}", params.len()));
                                        self.lower_static_method_as_function(func_name, params.clone(), body.clone())?;
                                        self.static_method_index
                                            .entry(mname.clone())
                                            .or_insert_with(Vec::new)
                                            .push((name.clone(), params.len()));
                                    }
                                }
                            }
                        } else {
                            // Instance box: register type and lower instance methods/ctors as functions
                            self.user_defined_boxes.insert(name.clone());
                            self.build_box_declaration(
                                name.clone(),
                                methods.clone(),
                                fields.clone(),
                                weak_fields.clone(),
                            )?;
                            for (ctor_key, ctor_ast) in constructors.iter() {
                                if let N::FunctionDeclaration { params, body, .. } = ctor_ast {
                                    // Keep constructor function name as "Box.birth/N" where ctor_key already encodes arity.
                                    // ctor_key format comes from parser as "birth/<arity>".
                                    let func_name = format!("{}.{}", name, ctor_key);
                                    self.lower_method_as_function(
                                        func_name,
                                        name.clone(),
                                        params.clone(),
                                        body.clone(),
                                    )?;
                                }
                            }
                            for (mname, mast) in methods.iter() {
                                if let N::FunctionDeclaration { params, body, is_static, .. } = mast {
                                    if !*is_static {
                                        let func_name = format!("{}.{}{}", name, mname, format!("/{}", params.len()));
                                        self.lower_method_as_function(
                                            func_name,
                                            name.clone(),
                                            params.clone(),
                                            body.clone(),
                                        )?;
                                    }
                                }
                            }
                        }
                    }
                }

                // Second pass: lower Main.main body as Program (entry). If absent, fall back to sequential block.
                if let Some((box_name, methods)) = main_static {
                    self.build_static_main_box(box_name, methods)
                } else {
                    // Fallback: sequential lowering (keeps legacy behavior for scripts without Main)
                    self.cf_block(statements)
                }
            }
            other => self.build_expression(other),
        }
    }

    pub(super) fn finalize_module(
        &mut self,
        result_value: ValueId,
    ) -> Result<MirModule, String> {
        // Hint: scope leave at function end (id=0 for main)
        self.hint_scope_leave(0);
        if let Some(block_id) = self.current_block {
            if let Some(ref mut function) = self.current_function {
                if let Some(block) = function.get_block_mut(block_id) {
                    if !block.is_terminated() {
                        block.add_instruction(MirInstruction::Return {
                            value: Some(result_value),
                        });
                    }
                    if let Some(mt) = self.value_types.get(&result_value).cloned() {
                        function.signature.return_type = mt;
                    }
                }
            }
        }

        let mut module = self.current_module.take().unwrap();
        let mut function = self.current_function.take().unwrap();
        function.metadata.value_types = self.value_types.clone();
        if matches!(
            function.signature.return_type,
            super::MirType::Void | super::MirType::Unknown
        ) {
            let mut inferred: Option<super::MirType> = None;
            'outer: for (_bid, bb) in function.blocks.iter() {
                for inst in bb.instructions.iter() {
                    if let super::MirInstruction::Return { value: Some(v) } = inst {
                        if let Some(mt) = self.value_types.get(v).cloned() {
                            inferred = Some(mt);
                            break 'outer;
                        }
                        if let Some(mt) = crate::mir::phi_core::if_phi::infer_type_from_phi(
                            &function,
                            *v,
                            &self.value_types,
                        ) {
                            inferred = Some(mt);
                            break 'outer;
                        }
                    }
                }
                if let Some(super::MirInstruction::Return { value: Some(v) }) = &bb.terminator {
                    if let Some(mt) = self.value_types.get(v).cloned() {
                        inferred = Some(mt);
                        break;
                    }
                    if let Some(mt) = crate::mir::phi_core::if_phi::infer_type_from_phi(
                        &function,
                        *v,
                        &self.value_types,
                    ) {
                        inferred = Some(mt);
                        break;
                    }
                }
            }
            if let Some(mt) = inferred {
                function.signature.return_type = mt;
            }
        }
        module.add_function(function);

        Ok(module)
    }
}
