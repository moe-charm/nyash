use super::*;
use crate::box_trait::SharedNyashBox;
use std::sync::Arc;

impl NyashInterpreter {
    /// Evaluate `new` expression arguments to NyashBox values
    pub(super) fn new_eval_args(&mut self, arguments: &[ASTNode]) -> Result<Vec<Box<dyn NyashBox>>, RuntimeError> {
        arguments.iter().map(|arg| self.execute_expression(arg)).collect()
    }

    /// If user-defined and type args provided, validate/specialize and register declaration
    pub(super) fn new_specialize_if_needed(&self, class: &str, type_arguments: &[String]) -> Result<String, RuntimeError> {
        let mut target_class = class.to_string();
        let user_defined_exists = {
            let box_decls = self.shared.box_declarations.read().unwrap();
            box_decls.contains_key(class)
        };
        if user_defined_exists && !type_arguments.is_empty() {
            let generic_decl = {
                let box_decls = self.shared.box_declarations.read().unwrap();
                box_decls.get(class).cloned()
            };
            if let Some(generic_decl) = generic_decl {
                self.validate_generic_arguments(&generic_decl, type_arguments)?;
                let specialized = self.specialize_generic_class(&generic_decl, type_arguments)?;
                target_class = specialized.name.clone();
                // Insert specialized declaration so registry can create it
                let mut box_decls = self.shared.box_declarations.write().unwrap();
                box_decls.insert(target_class.clone(), specialized);
            }
        }
        Ok(target_class)
    }

    /// Create box via registry and optionally run user-defined constructor (birth/arity)
    pub(super) fn new_create_via_registry_and_maybe_ctor(
        &mut self,
        target_class: &str,
        args: Vec<Box<dyn NyashBox>>,
        arguments: &[ASTNode],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        // Try unified registry (use interpreter's runtime registry to include user-defined boxes)
        let registry = self.runtime.box_registry.clone();
        let registry_lock = registry.lock().unwrap();
        match registry_lock.create_box(target_class, &args) {
            Ok(box_instance) => {
                // Check if this is a user-defined box that needs constructor execution
                if let Some(_instance_box) = box_instance.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                    // Check if we have a box declaration for this class
                    let (box_decl_opt, constructor_opt) = {
                        let box_decls = self.shared.box_declarations.read().unwrap();
                        if let Some(box_decl) = box_decls.get(target_class) {
                            // Find the birth constructor (unified constructor system)
                            let birth_key = format!("birth/{}", arguments.len());
                            let constructor = box_decl.constructors.get(&birth_key).cloned();
                            (Some(box_decl.clone()), constructor)
                        } else { (None, None) }
                    };
                    if let Some(box_decl) = box_decl_opt {
                        if let Some(constructor) = constructor_opt {
                            // Execute the constructor
                            let instance_arc: SharedNyashBox = Arc::from(box_instance);
                            drop(registry_lock); // Release lock before executing constructor
                            self.execute_constructor(&instance_arc, &constructor, arguments, &box_decl)?;
                            return Ok((*instance_arc).clone_box());
                        } else if arguments.is_empty() {
                            // No constructor needed for zero arguments
                            return Ok(box_instance);
                        } else {
                            return Err(RuntimeError::InvalidOperation {
                                message: format!("No constructor found for {} with {} arguments", target_class, arguments.len()),
                            });
                        }
                    }
                }
                // Not a user-defined box or no constructor needed
                Ok(box_instance)
            },
            Err(e) => {
                // Fallback: handle basic built-in boxes directly (e.g., FutureBox)
                // This keeps interpreter usability when registry has no provider.
                drop(registry_lock);
                match self.create_basic_box(target_class, arguments) {
                    Ok(b) => Ok(b),
                    Err(_) => Err(e),
                }
            },
        }
    }

    /// new式を実行 - Object creation engine
    pub(crate) fn execute_new(&mut self, class: &str, arguments: &[ASTNode], type_arguments: &[String])
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 80/20 path: unified registry + constructor
        let args = self.new_eval_args(arguments)?;
        let target_class = self.new_specialize_if_needed(class, type_arguments)?;
        self.new_create_via_registry_and_maybe_ctor(&target_class, args, arguments)
    }
}
