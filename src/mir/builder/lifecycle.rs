use super::{EffectMask, FunctionSignature, MirFunction, MirInstruction, MirModule, MirType, ValueId};
use crate::ast::ASTNode;

// Lifecycle routines extracted from builder.rs
impl super::MirBuilder {
    fn preindex_static_methods_from_ast(&mut self, node: &ASTNode) {
        match node {
            ASTNode::Program { statements, .. } => {
                for st in statements {
                    self.preindex_static_methods_from_ast(st);
                }
            }
            ASTNode::BoxDeclaration { name, methods, is_static, .. } => {
                if *is_static {
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
        self.preindex_static_methods_from_ast(&snapshot);
        self.build_expression(ast)
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
