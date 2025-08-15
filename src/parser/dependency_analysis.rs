/*!
 * Dependency Analysis Module
 * 
 * 静的Box間の循環依存検出とAST内の依存関係抽出機能
 * static box初期化順序の検証に使用
 */

use std::collections::{HashMap, HashSet};
use crate::ast::ASTNode;
use super::ParseError;

impl super::NyashParser {
    /// 循環依存検出（深さ優先探索）
    pub(super) fn check_circular_dependencies(&self) -> Result<(), ParseError> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();
        
        for box_name in self.static_box_dependencies.keys() {
            if !visited.contains(box_name) {
                if self.has_cycle_dfs(box_name, &mut visited, &mut rec_stack, &mut path)? {
                    return Ok(()); // エラーは既にhas_cycle_dfs内で返される
                }
            }
        }
        
        Ok(())
    }
    
    /// DFS による循環依存検出
    pub(super) fn has_cycle_dfs(
        &self,
        current: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Result<bool, ParseError> {
        visited.insert(current.to_string());
        rec_stack.insert(current.to_string());
        path.push(current.to_string());
        
        if let Some(dependencies) = self.static_box_dependencies.get(current) {
            for dependency in dependencies {
                if !visited.contains(dependency) {
                    if self.has_cycle_dfs(dependency, visited, rec_stack, path)? {
                        return Ok(true);
                    }
                } else if rec_stack.contains(dependency) {
                    // 循環依存を発見！
                    let cycle_start_pos = path.iter().position(|x| x == dependency).unwrap_or(0);
                    let cycle_path: Vec<String> = path[cycle_start_pos..].iter().cloned().collect();
                    let cycle_display = format!("{} -> {}", cycle_path.join(" -> "), dependency);
                    
                    return Err(ParseError::CircularDependency { 
                        cycle: cycle_display 
                    });
                }
            }
        }
        
        rec_stack.remove(current);
        path.pop();
        Ok(false)
    }
    
    /// Static初期化ブロック内の文から依存関係を抽出
    pub(super) fn extract_dependencies_from_statements(&self, statements: &[ASTNode]) -> HashSet<String> {
        let mut dependencies = HashSet::new();
        
        for stmt in statements {
            self.extract_dependencies_from_ast(stmt, &mut dependencies);
        }
        
        dependencies
    }
    
    /// AST内から静的Box参照を再帰的に検出
    pub(super) fn extract_dependencies_from_ast(&self, node: &ASTNode, dependencies: &mut HashSet<String>) {
        match node {
            ASTNode::FieldAccess { object, .. } => {
                // Math.PI のような参照を検出
                if let ASTNode::Variable { name, .. } = object.as_ref() {
                    dependencies.insert(name.clone());
                }
            }
            ASTNode::MethodCall { object, .. } => {
                // Config.getDebug() のような呼び出しを検出
                if let ASTNode::Variable { name, .. } = object.as_ref() {
                    dependencies.insert(name.clone());
                }
            }
            ASTNode::Assignment { target, value, .. } => {
                self.extract_dependencies_from_ast(target, dependencies);
                self.extract_dependencies_from_ast(value, dependencies);
            }
            ASTNode::BinaryOp { left, right, .. } => {
                self.extract_dependencies_from_ast(left, dependencies);
                self.extract_dependencies_from_ast(right, dependencies);
            }
            ASTNode::UnaryOp { operand, .. } => {
                self.extract_dependencies_from_ast(operand, dependencies);
            }
            ASTNode::If { condition, then_body, else_body, .. } => {
                self.extract_dependencies_from_ast(condition, dependencies);
                for stmt in then_body {
                    self.extract_dependencies_from_ast(stmt, dependencies);
                }
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        self.extract_dependencies_from_ast(stmt, dependencies);
                    }
                }
            }
            ASTNode::Loop { condition, body, .. } => {
                self.extract_dependencies_from_ast(condition, dependencies);
                for stmt in body {
                    self.extract_dependencies_from_ast(stmt, dependencies);
                }
            }
            ASTNode::Return { value, .. } => {
                if let Some(val) = value {
                    self.extract_dependencies_from_ast(val, dependencies);
                }
            }
            ASTNode::Print { expression, .. } => {
                self.extract_dependencies_from_ast(expression, dependencies);
            }
            // 他のAST nodeタイプも必要に応じて追加
            _ => {}
        }
    }
}