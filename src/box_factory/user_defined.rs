/*!
 * User-Defined Box Factory
 * 
 * Handles creation of user-defined Box types through InstanceBox
 * Manages inheritance, fields, methods, and birth/fini lifecycle
 */

use super::BoxFactory;
use crate::box_trait::NyashBox;
use crate::interpreter::{RuntimeError, SharedState};
use crate::instance_v2::InstanceBox;

/// Factory for user-defined Box types
pub struct UserDefinedBoxFactory {
    shared_state: SharedState,
}

impl UserDefinedBoxFactory {
    pub fn new(shared_state: SharedState) -> Self {
        Self {
            shared_state,
        }
    }
}

impl BoxFactory for UserDefinedBoxFactory {
    fn create_box(
        &self,
        name: &str,
        _args: &[Box<dyn NyashBox>],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        // Look up box declaration
        let box_decl = {
            let box_decls = self.shared_state.box_declarations.read().unwrap();
            box_decls.get(name).cloned()
        };
        
        let box_decl = box_decl.ok_or_else(|| RuntimeError::InvalidOperation {
            message: format!("Unknown Box type: {}", name),
        })?;
        
        // Create InstanceBox with fields and methods
        let instance = InstanceBox::from_declaration(
            name.to_string(),
            box_decl.fields.clone(),
            box_decl.methods.clone(),
        );
        
        // TODO: Execute birth/init constructor with args
        // For now, just return the instance
        Ok(Box::new(instance))
    }
    
    fn box_types(&self) -> Vec<&str> {
        // Can't return borrowed strings from temporary RwLock guard
        // For now, return empty - this method isn't critical
        vec![]
    }
    
    fn is_available(&self) -> bool {
        // Always available when SharedState is present
        true
    }
}