// Function lowering for method bodies
use super::super::{MirBuilder, MirFunction, MirInstruction, MirType};
use crate::ast::ASTNode;
use crate::mir::builder::calls::function_lowering;

impl MirBuilder {
    // Lower a box method into a standalone MIR function (with `me` parameter)
    pub(in super::super) fn lower_method_as_function(
        &mut self,
        func_name: String,
        box_name: String,
        params: Vec<String>,
        body: Vec<ASTNode>,
    ) -> Result<(), String> {
        let signature = function_lowering::prepare_method_signature(
            func_name,
            &box_name,
            &params,
            &body,
        );
        let returns_value = !matches!(signature.return_type, MirType::Void);
        let entry = self.block_gen.next();
        let function = MirFunction::new(signature, entry);
        let saved_function = self.current_function.take();
        let saved_block = self.current_block.take();
        let saved_var_map = std::mem::take(&mut self.variable_map);
        // Per-function metadata scope: ValueId は関数ごとにリセットされるため、
        // value_types / value_origin_newbox は関数単位で分離する（交差汚染防止）。
        let saved_types = std::mem::take(&mut self.value_types);
        let saved_origin = std::mem::take(&mut self.value_origin_newbox);
        let saved_value_gen = self.value_gen.clone();
        self.value_gen.reset();
        self.current_function = Some(function);
        self.current_block = Some(entry);
        self.ensure_block_exists(entry)?;
        if let Some(ref mut f) = self.current_function {
            let me_id = self.value_gen.next();
            f.params.push(me_id);
            self.variable_map.insert("me".to_string(), me_id);
            self.value_origin_newbox.insert(me_id, box_name.clone());
            for p in &params {
                let pid = self.value_gen.next();
                f.params.push(pid);
                self.variable_map.insert(p.clone(), pid);
            }
        }
        let program_ast = function_lowering::wrap_in_program(body);
        let _last = self.build_expression(program_ast)?;
        if !returns_value && !self.is_current_block_terminated() {
            let void_val = crate::mir::builder::emission::constant::emit_void(self);
            self.emit_instruction(MirInstruction::Return {
                value: Some(void_val),
            })?;
        }
        if let Some(ref mut f) = self.current_function {
            if returns_value
                && matches!(f.signature.return_type, MirType::Void | MirType::Unknown)
            {
                let mut inferred: Option<MirType> = None;
                'search: for (_bid, bb) in f.blocks.iter() {
                    for inst in bb.instructions.iter() {
                        if let MirInstruction::Return { value: Some(v) } = inst {
                            if let Some(mt) = self.value_types.get(v).cloned() {
                                inferred = Some(mt);
                                break 'search;
                            }
                        }
                    }
                    if let Some(MirInstruction::Return { value: Some(v) }) = &bb.terminator {
                        if let Some(mt) = self.value_types.get(v).cloned() {
                            inferred = Some(mt);
                            break;
                        }
                    }
                }
                if let Some(mt) = inferred {
                    f.signature.return_type = mt;
                }
            }
        }
        let finalized_function = self.current_function.take().unwrap();
        if let Some(ref mut module) = self.current_module {
            module.add_function(finalized_function);
        }
        self.current_function = saved_function;
        self.current_block = saved_block;
        self.variable_map = saved_var_map;
        // Drop per-function metadata and restore outer scope
        self.value_types = saved_types;
        self.value_origin_newbox = saved_origin;
        self.value_gen = saved_value_gen;
        Ok(())
    }

    // Lower a static method body into a standalone MIR function (no `me` parameter)
    pub(in super::super) fn lower_static_method_as_function(
        &mut self,
        func_name: String,
        params: Vec<String>,
        body: Vec<ASTNode>,
    ) -> Result<(), String> {
        // Derive static box context from function name prefix, e.g., "BoxName.method/N"
        let saved_static_ctx = self.current_static_box.clone();
        if let Some(pos) = func_name.find('.') {
            let box_name = &func_name[..pos];
            if !box_name.is_empty() {
                self.current_static_box = Some(box_name.to_string());
            }
        }
        let signature = function_lowering::prepare_static_method_signature(
            func_name,
            &params,
            &body,
        );
        let returns_value = !matches!(signature.return_type, MirType::Void);
        let entry = self.block_gen.next();
        let function = MirFunction::new(signature, entry);
        let saved_function = self.current_function.take();
        let saved_block = self.current_block.take();
        let saved_var_map = std::mem::take(&mut self.variable_map);
        let saved_types = std::mem::take(&mut self.value_types);
        let saved_origin = std::mem::take(&mut self.value_origin_newbox);
        let saved_value_gen = self.value_gen.clone();
        self.value_gen.reset();
        self.current_function = Some(function);
        self.current_block = Some(entry);
        self.ensure_block_exists(entry)?;
        if let Some(ref mut f) = self.current_function {
            for p in &params {
                let pid = self.value_gen.next();
                f.params.push(pid);
                self.variable_map.insert(p.clone(), pid);
            }
        }
        let program_ast = function_lowering::wrap_in_program(body);
        let _last = self.build_expression(program_ast)?;
        if !returns_value {
            if let Some(ref mut f) = self.current_function {
                if let Some(block) = f.get_block(self.current_block.unwrap()) {
                    if !block.is_terminated() {
                        let void_val = crate::mir::builder::emission::constant::emit_void(self);
                        self.emit_instruction(MirInstruction::Return {
                            value: Some(void_val),
                        })?;
                    }
                }
            }
        }
        if let Some(ref mut f) = self.current_function {
            if returns_value
                && matches!(f.signature.return_type, MirType::Void | MirType::Unknown)
            {
                let mut inferred: Option<MirType> = None;
                'search: for (_bid, bb) in f.blocks.iter() {
                    for inst in bb.instructions.iter() {
                        if let MirInstruction::Return { value: Some(v) } = inst {
                            if let Some(mt) = self.value_types.get(v).cloned() {
                                inferred = Some(mt);
                                break 'search;
                            }
                        }
                    }
                    if let Some(MirInstruction::Return { value: Some(v) }) = &bb.terminator {
                        if let Some(mt) = self.value_types.get(v).cloned() {
                            inferred = Some(mt);
                            break;
                        }
                    }
                }
                if let Some(mt) = inferred {
                    f.signature.return_type = mt;
                }
            }
        }
        let finalized = self.current_function.take().unwrap();
        if let Some(ref mut module) = self.current_module {
            module.add_function(finalized);
        }
        self.current_function = saved_function;
        self.current_block = saved_block;
        self.variable_map = saved_var_map;
        self.value_types = saved_types;
        self.value_origin_newbox = saved_origin;
        self.value_gen = saved_value_gen;
        // Restore static box context
        self.current_static_box = saved_static_ctx;
        Ok(())
    }
}