/*!
 * Dependency Analysis Helpers
 *
 * Static box依存関係の解析と循環依存検出
 */

use crate::ast::ASTNode;
use crate::parser::{NyashParser, ParseError};
use std::collections::{HashMap, HashSet};

impl NyashParser {
    /// Static初期化ブロック内の文から依存関係を抽出
    pub(super) fn extract_dependencies_from_statements(
        &self,
        statements: &[ASTNode],
    ) -> HashSet<String> {
        let mut dependencies = HashSet::new();

        for stmt in statements {
            self.extract_dependencies_from_ast(stmt, &mut dependencies);
        }

        dependencies
    }

    /// AST内から静的Box参照を再帰的に検出
    pub(super) fn extract_dependencies_from_ast(
        &self,
        node: &ASTNode,
        dependencies: &mut HashSet<String>,
    ) {
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
            ASTNode::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
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
            ASTNode::Loop {
                condition, body, ..
            } => {
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
            _ => {
                // その他のASTノードは無視
            }
        }
    }

    /// 循環依存検出
    pub fn check_circular_dependencies(&self) -> Result<(), ParseError> {
        // すべてのstatic boxに対して循環検出を実行
        let all_boxes: Vec<_> = self.static_box_dependencies.keys().cloned().collect();

        for box_name in &all_boxes {
            let mut visited = HashSet::new();
            let mut stack = Vec::new();

            if self.has_cycle_dfs(box_name, &mut visited, &mut stack)? {
                // 循環を文字列化
                let cycle_str = stack.join(" -> ");
                return Err(ParseError::CircularDependency { cycle: cycle_str });
            }
        }

        Ok(())
    }

    /// DFSで循環依存を検出
    fn has_cycle_dfs(
        &self,
        current: &str,
        visited: &mut HashSet<String>,
        stack: &mut Vec<String>,
    ) -> Result<bool, ParseError> {
        // 既にスタックにあれば循環
        if stack.contains(&current.to_string()) {
            stack.push(current.to_string()); // 循環を完成させる
            return Ok(true);
        }

        // 既に訪問済みで循環がなければスキップ
        if visited.contains(current) {
            return Ok(false);
        }

        visited.insert(current.to_string());
        stack.push(current.to_string());

        // 依存先をチェック
        if let Some(dependencies) = self.static_box_dependencies.get(current) {
            for dep in dependencies {
                if self.has_cycle_dfs(dep, visited, stack)? {
                    return Ok(true);
                }
            }
        }

        stack.pop();
        Ok(false)
    }

    /// Override メソッドの検証
    pub(super) fn validate_override_methods(
        &self,
        child_name: &str,
        parent_name: &str,
        methods: &HashMap<String, ASTNode>,
    ) -> Result<(), ParseError> {
        // 現時点では簡単な検証のみ
        // TODO: 親クラスのメソッドシグネチャとの比較
        for (method_name, method_ast) in methods {
            if let ASTNode::FunctionDeclaration { is_override, .. } = method_ast {
                if *is_override {
                    // 将来的にここで親クラスのメソッドが存在するかチェック
                    eprintln!(
                        "🔍 Validating override method '{}' in '{}' from '{}'",
                        method_name, child_name, parent_name
                    );
                }
            }
        }
        Ok(())
    }
}
