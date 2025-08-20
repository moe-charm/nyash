//! Core model definitions for Nyash
//!
//! This module contains pure data models that are shared between
//! the interpreter and the VM. Keep these types free of execution
//! strategy details so they can be reused across backends.

use std::collections::HashMap;

use crate::ast::ASTNode;

/// Declaration of a user-defined Box type (class) in Nyash
///
/// Pure model data used by both the interpreter and VM layers.
#[derive(Debug, Clone)]
pub struct BoxDeclaration {
    pub name: String,
    pub fields: Vec<String>,
    pub public_fields: Vec<String>,
    pub private_fields: Vec<String>,
    pub methods: HashMap<String, ASTNode>,
    pub constructors: HashMap<String, ASTNode>,
    pub init_fields: Vec<String>,
    pub weak_fields: Vec<String>,
    pub is_interface: bool,
    /// Supports multi-delegation: list of parent types
    pub extends: Vec<String>,
    pub implements: Vec<String>,
    /// Generic type parameters
    pub type_parameters: Vec<String>,
}
