use super::*;
use crate::box_trait::SharedNyashBox;

impl NyashInterpreter {
    /// コンストラクタを実行 - Constructor execution
    pub(super) fn execute_constructor(
        &mut self,
        instance: &SharedNyashBox,
        constructor: &ASTNode,
        arguments: &[ASTNode],
        box_decl: &BoxDeclaration,
    ) -> Result<(), RuntimeError> {
        if let ASTNode::FunctionDeclaration {
            name: _,
            params,
            body,
            ..
        } = constructor
        {
            let mut arg_values = Vec::new();
            for arg in arguments {
                arg_values.push(self.execute_expression(arg)?);
            }
            if params.len() != arg_values.len() {
                return Err(RuntimeError::InvalidOperation {
                    message: format!(
                        "Constructor expects {} arguments, got {}",
                        params.len(),
                        arg_values.len()
                    ),
                });
            }
            let saved_locals = self.save_local_vars();
            self.local_vars.clear();
            for (param, value) in params.iter().zip(arg_values.iter()) {
                self.declare_local_variable(param, value.clone_or_share());
            }
            self.declare_local_variable("me", instance.clone_or_share());
            let old_context = self.current_constructor_context.clone();
            self.current_constructor_context = Some(ConstructorContext {
                class_name: box_decl.name.clone(),
                parent_class: box_decl.extends.first().cloned(),
            });
            let mut result = Ok(());
            for statement in body.iter() {
                if let Err(e) = self.execute_statement(statement) {
                    result = Err(e);
                    break;
                }
            }
            self.restore_local_vars(saved_locals);
            self.current_constructor_context = old_context;
            result
        } else {
            Err(RuntimeError::InvalidOperation {
                message: "Invalid constructor node".to_string(),
            })
        }
    }

    /// 親コンストラクタを実行 - Parent constructor execution
    pub(crate) fn execute_parent_constructor(
        &mut self,
        parent_class: &str,
        arguments: &[ASTNode],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let parent_decl = {
            let box_decls = self.shared.box_declarations.read().unwrap();
            box_decls
                .get(parent_class)
                .ok_or(RuntimeError::UndefinedClass {
                    name: parent_class.to_string(),
                })?
                .clone()
        };
        let birth_key = format!("birth/{}", arguments.len());
        if let Some(parent_constructor) = parent_decl.constructors.get(&birth_key) {
            let this_instance =
                self.resolve_variable("me")
                    .map_err(|_| RuntimeError::InvalidOperation {
                        message: "'this' not available in parent constructor call".to_string(),
                    })?;
            self.execute_constructor(&this_instance, parent_constructor, arguments, &parent_decl)?;
            Ok(Box::new(VoidBox::new()))
        } else {
            Err(RuntimeError::InvalidOperation {
                message: format!(
                    "No constructor found for parent class {} with {} arguments",
                    parent_class,
                    arguments.len()
                ),
            })
        }
    }
}
