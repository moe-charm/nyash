use super::*;

impl NyashInterpreter {
    /// Box宣言を登録 - 🔥 コンストラクタオーバーロード禁止対応
    pub(crate) fn register_box_declaration(
        &mut self,
        name: String,
        fields: Vec<String>,
        public_fields: Vec<String>,
        private_fields: Vec<String>,
        methods: HashMap<String, ASTNode>,
        constructors: HashMap<String, ASTNode>,
        init_fields: Vec<String>,
        weak_fields: Vec<String>,
        is_interface: bool,
        extends: Vec<String>,
        implements: Vec<String>,
        type_parameters: Vec<String>
    ) -> Result<(), RuntimeError> {
        if !constructors.is_empty() {
            eprintln!("🐛 DEBUG: Registering Box '{}' with constructors: {:?}", name, constructors.keys().collect::<Vec<_>>());
        }
        if constructors.len() > 1 {
            let constructor_names: Vec<String> = constructors.keys().cloned().collect();
            return Err(RuntimeError::InvalidOperation {
                message: format!(
                    "🚨 CONSTRUCTOR OVERLOAD FORBIDDEN: Box '{}' has {} constructors: [{}].\n\
                    🌟 Nyash's explicit philosophy: One Box, One Constructor!\n\
                    💡 Use different Box classes for different initialization patterns.\n\
                    📖 Example: UserBox, AdminUserBox, GuestUserBox instead of User(type)",
                    name,
                    constructors.len(),
                    constructor_names.join(", ")
                )
            });
        }
        let box_decl = super::BoxDeclaration {
            name: name.clone(),
            fields,
            public_fields,
            private_fields,
            methods,
            constructors,
            init_fields,
            weak_fields,
            is_interface,
            extends,
            implements,
            type_parameters,
        };
        {
            let mut box_decls = self.shared.box_declarations.write().unwrap();
            box_decls.insert(name, box_decl);
        }
        Ok(())
    }

    /// 🔥 ジェネリクス型引数の検証
    pub(super) fn validate_generic_arguments(&self, box_decl: &BoxDeclaration, type_arguments: &[String])
        -> Result<(), RuntimeError> {
        if box_decl.type_parameters.len() != type_arguments.len() {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Generic class '{}' expects {} type parameters, got {}. Expected: <{}>, Got: <{}>",
                    box_decl.name,
                    box_decl.type_parameters.len(),
                    type_arguments.len(),
                    box_decl.type_parameters.join(", "),
                    type_arguments.join(", ")
                ),
            });
        }
        if box_decl.type_parameters.is_empty() && !type_arguments.is_empty() {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Class '{}' is not generic, but got type arguments <{}>",
                    box_decl.name,
                    type_arguments.join(", ")
                ),
            });
        }
        for type_arg in type_arguments {
            if !self.is_valid_type(type_arg) {
                return Err(RuntimeError::TypeError { message: format!("Unknown type '{}'", type_arg) });
            }
        }
        Ok(())
    }

    /// 型が有効かどうかをチェック
    fn is_valid_type(&self, type_name: &str) -> bool {
        if let Ok(reg) = self.runtime.box_registry.lock() {
            if reg.has_type(type_name) { return true; }
        }
        self.shared.box_declarations.read().unwrap().contains_key(type_name)
    }

    /// 継承チェーンを解決してフィールドとメソッドを収集 - Inheritance resolution
    pub(crate) fn resolve_inheritance(&self, box_decl: &BoxDeclaration)
        -> Result<(Vec<String>, HashMap<String, ASTNode>), RuntimeError> {
        let mut all_fields = Vec::new();
        let mut all_methods = HashMap::new();
        for parent_name in &box_decl.extends {
            use crate::box_trait::is_builtin_box;
            let mut is_builtin = is_builtin_box(parent_name);
            #[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
            {
                if parent_name == "EguiBox" { is_builtin = true; }
            }
            if is_builtin {
                // skip builtin inheritance
            } else {
                let parent_decl = {
                    let box_decls = self.shared.box_declarations.read().unwrap();
                    box_decls.get(parent_name)
                        .ok_or(RuntimeError::UndefinedClass { name: parent_name.to_string() })?
                        .clone()
                };
                if parent_decl.is_interface {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("Cannot extend interface '{}'. Use 'implements' instead.", parent_name),
                    });
                }
                let (parent_fields, parent_methods) = self.resolve_inheritance(&parent_decl)?;
                all_fields.extend(parent_fields);
                all_methods.extend(parent_methods);
            }
        }
        all_fields.extend(box_decl.fields.clone());
        for init_field in &box_decl.init_fields {
            if !all_fields.contains(init_field) { all_fields.push(init_field.clone()); }
        }
        for (method_name, method_ast) in &box_decl.methods {
            all_methods.insert(method_name.clone(), method_ast.clone());
        }
        for interface_name in &box_decl.implements {
            let interface_decl = {
                let box_decls = self.shared.box_declarations.read().unwrap();
                box_decls.get(interface_name)
                    .ok_or(RuntimeError::UndefinedClass { name: interface_name.clone() })?
                    .clone()
            };
            if !interface_decl.is_interface {
                return Err(RuntimeError::InvalidOperation { message: format!("'{}' is not an interface", interface_name) });
            }
            for (required_method, _) in &interface_decl.methods {
                if !all_methods.contains_key(required_method) {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("Class '{}' must implement method '{}' from interface '{}'",
                            box_decl.name, required_method, interface_name),
                    });
                }
            }
        }
        Ok((all_fields, all_methods))
    }

    /// 🚀 ジェネリクス型を特殊化してBoxDeclarationを生成
    pub(super) fn specialize_generic_class(
        &self,
        generic_decl: &BoxDeclaration,
        type_arguments: &[String]
    ) -> Result<BoxDeclaration, RuntimeError> {
        use std::collections::HashMap;
        let specialized_name = format!("{}_{}", generic_decl.name, type_arguments.join("_"));
        let mut type_mapping = HashMap::new();
        for (i, param) in generic_decl.type_parameters.iter().enumerate() {
            type_mapping.insert(param.clone(), type_arguments[i].clone());
        }
        let mut specialized = generic_decl.clone();
        specialized.name = specialized_name.clone();
        specialized.type_parameters.clear();
        specialized.init_fields = self.substitute_types_in_fields(&specialized.init_fields, &type_mapping);
        let mut updated_constructors = HashMap::new();
        for (old_key, constructor_node) in &generic_decl.constructors {
            if let Some(args_count) = old_key.split('/').nth(1) {
                let new_key = format!("{}/{}", specialized_name, args_count);
                updated_constructors.insert(new_key, constructor_node.clone());
            }
        }
        specialized.constructors = updated_constructors;
        Ok(specialized)
    }

    /// フィールドの型置換（現状はそのまま）
    pub(super) fn substitute_types_in_fields(
        &self,
        fields: &[String],
        _type_mapping: &HashMap<String, String>
    ) -> Vec<String> {
        fields.to_vec()
    }
}
