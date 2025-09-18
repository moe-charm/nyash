// Declarations lowering: static boxes and box declarations
use super::{ConstValue, MirInstruction, ValueId};
use crate::ast::ASTNode;
use crate::mir::slot_registry::{get_or_assign_type_id, reserve_method_slot};
use std::collections::HashSet;

impl super::MirBuilder {
    /// Build static box (e.g., Main) - extracts main() method body and converts to Program
    /// Also lowers other static methods into standalone MIR functions: BoxName.method/N
    pub(super) fn build_static_main_box(
        &mut self,
        box_name: String,
        methods: std::collections::HashMap<String, ASTNode>,
    ) -> Result<ValueId, String> {
        // Lower other static methods (except main) to standalone MIR functions so JIT can see them
        for (mname, mast) in methods.iter() {
            if mname == "main" {
                continue;
            }
            if let ASTNode::FunctionDeclaration { params, body, .. } = mast {
                let func_name = format!("{}.{}{}", box_name, mname, format!("/{}", params.len()));
                self.lower_static_method_as_function(func_name, params.clone(), body.clone())?;
            }
        }
        // Within this lowering, treat `me` receiver as this static box
        let saved_static = self.current_static_box.clone();
        self.current_static_box = Some(box_name.clone());
        // Look for the main() method
        let out = if let Some(main_method) = methods.get("main") {
            if let ASTNode::FunctionDeclaration { params, body, .. } = main_method {
                // Convert the method body to a Program AST node and lower it
                let program_ast = ASTNode::Program {
                    statements: body.clone(),
                    span: crate::ast::Span::unknown(),
                };
                // Bind default parameters if present (e.g., args=[])
                let saved_var_map = std::mem::take(&mut self.variable_map);
                for p in params.iter() {
                    let pid = self.value_gen.next();
                    if p == "args" {
                        // new ArrayBox() with no args
                        self.emit_instruction(MirInstruction::NewBox {
                            dst: pid,
                            box_type: "ArrayBox".to_string(),
                            args: vec![],
                        })?;
                    } else {
                        self.emit_instruction(MirInstruction::Const {
                            dst: pid,
                            value: ConstValue::Void,
                        })?;
                    }
                    self.variable_map.insert(p.clone(), pid);
                }
                let lowered = self.build_expression(program_ast);
                self.variable_map = saved_var_map;
                lowered
            } else {
                Err("main method in static box is not a FunctionDeclaration".to_string())
            }
        } else {
            Err("static box must contain a main() method".to_string())
        };
        // Restore static box context
        self.current_static_box = saved_static;
        out
    }

    /// Build box declaration: box Name { fields... methods... }
    pub(super) fn build_box_declaration(
        &mut self,
        name: String,
        methods: std::collections::HashMap<String, ASTNode>,
        fields: Vec<String>,
        weak_fields: Vec<String>,
    ) -> Result<(), String> {
        // Create a type registration constant (marker)
        let type_id = self.value_gen.next();
        self.emit_instruction(MirInstruction::Const {
            dst: type_id,
            value: ConstValue::String(format!("__box_type_{}", name)),
        })?;

        // Emit field metadata markers
        for field in fields {
            let field_id = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const {
                dst: field_id,
                value: ConstValue::String(format!("__field_{}_{}", name, field)),
            })?;
        }

        // Record weak fields for this box
        if !weak_fields.is_empty() {
            let set: HashSet<String> = weak_fields.into_iter().collect();
            self.weak_fields_by_box.insert(name.clone(), set);
        }

        // Reserve method slots for user-defined instance methods (deterministic, starts at 4)
        let mut instance_methods: Vec<String> = Vec::new();
        for (mname, mast) in &methods {
            if let ASTNode::FunctionDeclaration { is_static, .. } = mast {
                if !*is_static {
                    instance_methods.push(mname.clone());
                }
            }
        }
        instance_methods.sort();
        if !instance_methods.is_empty() {
            let tyid = get_or_assign_type_id(&name);
            for (i, m) in instance_methods.iter().enumerate() {
                let slot = 4u16.saturating_add(i as u16);
                reserve_method_slot(tyid, m, slot);
            }
        }

        // Emit markers for declared methods (kept as metadata hints)
        for (method_name, method_ast) in methods {
            if let ASTNode::FunctionDeclaration { .. } = method_ast {
                let method_id = self.value_gen.next();
                self.emit_instruction(MirInstruction::Const {
                    dst: method_id,
                    value: ConstValue::String(format!("__method_{}_{}", name, method_name)),
                })?;
                // Track unified member getters: __get_<prop> | __get_once_<prop> | __get_birth_<prop>
                let kind_and_prop: Option<(super::PropertyKind, String)> = if let Some(rest) = method_name.strip_prefix("__get_once_") {
                    Some((super::PropertyKind::Once, rest.to_string()))
                } else if let Some(rest) = method_name.strip_prefix("__get_birth_") {
                    Some((super::PropertyKind::BirthOnce, rest.to_string()))
                } else if let Some(rest) = method_name.strip_prefix("__get_") {
                    Some((super::PropertyKind::Computed, rest.to_string()))
                } else {
                    None
                };
                if let Some((k, prop)) = kind_and_prop {
                    use std::collections::HashMap;
                    let entry: &mut HashMap<String, super::PropertyKind> = self
                        .property_getters_by_box
                        .entry(name.clone())
                        .or_insert_with(HashMap::new);
                    entry.insert(prop, k);
                }
            }
        }

        Ok(())
    }
}
