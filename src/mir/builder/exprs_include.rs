use super::ValueId;

impl super::MirBuilder {
    // Include lowering: include "path"
    pub(super) fn build_include_expression(&mut self, filename: String) -> Result<ValueId, String> {
        let mut path = super::utils::resolve_include_path_builder(&filename);
        if std::path::Path::new(&path).is_dir() { path = format!("{}/index.nyash", path.trim_end_matches('/')); }
        else if std::path::Path::new(&path).extension().is_none() { path.push_str(".nyash"); }

        if self.include_loading.contains(&path) { return Err(format!("Circular include detected: {}", path)); }
        if let Some(name) = self.include_box_map.get(&path).cloned() { return self.build_new_expression(name, vec![]); }

        self.include_loading.insert(path.clone());
        let content = std::fs::read_to_string(&path).map_err(|e| format!("Include read error '{}': {}", filename, e))?;
        let included_ast = crate::parser::NyashParser::parse_from_string(&content).map_err(|e| format!("Include parse error '{}': {:?}", filename, e))?;
        let mut box_name: Option<String> = None;
        if let crate::ast::ASTNode::Program { statements, .. } = &included_ast {
            for st in statements { if let crate::ast::ASTNode::BoxDeclaration { name, is_static, .. } = st { if *is_static { box_name = Some(name.clone()); break; } } }
        }
        let bname = box_name.ok_or_else(|| format!("Include target '{}' has no static box", filename))?;
        let _ = self.build_expression_impl(included_ast)?;
        self.include_loading.remove(&path);
        self.include_box_map.insert(path.clone(), bname.clone());
        self.build_new_expression(bname, vec![])
    }
}

