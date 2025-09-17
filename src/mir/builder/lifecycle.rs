use super::{EffectMask, FunctionSignature, MirFunction, MirInstruction, MirModule, MirType, ValueId};
use crate::ast::ASTNode;

// Lifecycle routines extracted from builder.rs
impl super::MirBuilder {
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
        self.build_expression(ast)
    }

    pub(super) fn finalize_module(
        &mut self,
        result_value: ValueId,
    ) -> Result<MirModule, String> {
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
                        if let Some(mt) = super::phi::infer_type_from_phi(
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
                    if let Some(mt) = super::phi::infer_type_from_phi(
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
