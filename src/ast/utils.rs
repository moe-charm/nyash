//! Utility helpers for Nyash AST nodes extracted from `ast.rs`.

use super::{ASTNode, ASTNodeType, Span};
use std::fmt;

impl ASTNode {
    /// AST nodeの種類を文字列で取得 (デバッグ用)
    pub fn node_type(&self) -> &'static str {
        match self {
            ASTNode::Program { .. } => "Program",
            ASTNode::Assignment { .. } => "Assignment",
            ASTNode::Print { .. } => "Print",
            ASTNode::If { .. } => "If",
            ASTNode::Loop { .. } => "Loop",
            ASTNode::Return { .. } => "Return",
            ASTNode::Break { .. } => "Break",
            ASTNode::Continue { .. } => "Continue",
            ASTNode::UsingStatement { .. } => "UsingStatement",
            ASTNode::ImportStatement { .. } => "ImportStatement",
            ASTNode::BoxDeclaration { .. } => "BoxDeclaration",
            ASTNode::FunctionDeclaration { .. } => "FunctionDeclaration",
            ASTNode::GlobalVar { .. } => "GlobalVar",
            ASTNode::Literal { .. } => "Literal",
            ASTNode::Variable { .. } => "Variable",
            ASTNode::UnaryOp { .. } => "UnaryOp",
            ASTNode::BinaryOp { .. } => "BinaryOp",
            ASTNode::MethodCall { .. } => "MethodCall",
            ASTNode::FieldAccess { .. } => "FieldAccess",
            ASTNode::New { .. } => "New",
            ASTNode::This { .. } => "This",
            ASTNode::Me { .. } => "Me",
            ASTNode::FromCall { .. } => "FromCall",
            ASTNode::ThisField { .. } => "ThisField",
            ASTNode::MeField { .. } => "MeField",
            ASTNode::Include { .. } => "Include",
            ASTNode::Local { .. } => "Local",
            ASTNode::Outbox { .. } => "Outbox",
            ASTNode::FunctionCall { .. } => "FunctionCall",
            ASTNode::Call { .. } => "Call",
            ASTNode::Nowait { .. } => "Nowait",
            ASTNode::Arrow { .. } => "Arrow",
            ASTNode::TryCatch { .. } => "TryCatch",
            ASTNode::Throw { .. } => "Throw",
            ASTNode::AwaitExpression { .. } => "AwaitExpression",
            ASTNode::QMarkPropagate { .. } => "QMarkPropagate",
            ASTNode::MatchExpr { .. } => "MatchExpr",
            ASTNode::Lambda { .. } => "Lambda",
            ASTNode::ArrayLiteral { .. } => "ArrayLiteral",
            ASTNode::MapLiteral { .. } => "MapLiteral",
            // Optional diagnostic-only wrapper
            ASTNode::ScopeBox { .. } => "ScopeBox",
        }
    }

    /// 🌟 AST分類 - ChatGPTアドバイス統合による革新的分類システム
    /// Structure/Expression/Statement の明確な分離
    pub fn classify(&self) -> ASTNodeType {
        match self {
            // Structure nodes - 言語の基本構造
            ASTNode::BoxDeclaration { .. } => ASTNodeType::Structure,
            ASTNode::FunctionDeclaration { .. } => ASTNodeType::Structure,
            ASTNode::If { .. } => ASTNodeType::Structure,
            ASTNode::Loop { .. } => ASTNodeType::Structure,
            ASTNode::TryCatch { .. } => ASTNodeType::Structure,

            // Expression nodes - 値を生成する表現
            ASTNode::Literal { .. } => ASTNodeType::Expression,
            ASTNode::Variable { .. } => ASTNodeType::Expression,
            ASTNode::BinaryOp { .. } => ASTNodeType::Expression,
            ASTNode::UnaryOp { .. } => ASTNodeType::Expression,
            ASTNode::FunctionCall { .. } => ASTNodeType::Expression,
            ASTNode::Call { .. } => ASTNodeType::Expression,
            ASTNode::MethodCall { .. } => ASTNodeType::Expression,
            ASTNode::FieldAccess { .. } => ASTNodeType::Expression,
            ASTNode::New { .. } => ASTNodeType::Expression,
            ASTNode::This { .. } => ASTNodeType::Expression,
            ASTNode::Me { .. } => ASTNodeType::Expression,
            ASTNode::FromCall { .. } => ASTNodeType::Expression,
            ASTNode::ThisField { .. } => ASTNodeType::Expression,
            ASTNode::MeField { .. } => ASTNodeType::Expression,
            ASTNode::MatchExpr { .. } => ASTNodeType::Expression,
            ASTNode::QMarkPropagate { .. } => ASTNodeType::Expression,
            ASTNode::Lambda { .. } => ASTNodeType::Expression,
            ASTNode::ArrayLiteral { .. } => ASTNodeType::Expression,
            ASTNode::MapLiteral { .. } => ASTNodeType::Expression,

            // Diagnostic-only wrapper treated as structure
            ASTNode::ScopeBox { .. } => ASTNodeType::Structure,

            // Statement nodes - 実行可能なアクション
            ASTNode::Program { .. } => ASTNodeType::Statement, // プログラム全体
            ASTNode::Assignment { .. } => ASTNodeType::Statement,
            ASTNode::Print { .. } => ASTNodeType::Statement,
            ASTNode::Return { .. } => ASTNodeType::Statement,
            ASTNode::Break { .. } => ASTNodeType::Statement,
            ASTNode::Continue { .. } => ASTNodeType::Statement,
            ASTNode::UsingStatement { .. } => ASTNodeType::Statement,
            ASTNode::ImportStatement { .. } => ASTNodeType::Statement,
            ASTNode::GlobalVar { .. } => ASTNodeType::Statement,
            ASTNode::Include { .. } => ASTNodeType::Statement,
            ASTNode::Local { .. } => ASTNodeType::Statement,
            ASTNode::Outbox { .. } => ASTNodeType::Statement,
            ASTNode::Nowait { .. } => ASTNodeType::Statement,
            ASTNode::Arrow { .. } => ASTNodeType::Statement,
            ASTNode::Throw { .. } => ASTNodeType::Statement,
            ASTNode::AwaitExpression { .. } => ASTNodeType::Expression,
        }
    }

    /// 🎯 構造パターンチェック - 2段階パーサー用
    pub fn is_structure(&self) -> bool {
        matches!(self.classify(), ASTNodeType::Structure)
    }

    /// ⚡ 式パターンチェック - 評価エンジン用
    pub fn is_expression(&self) -> bool {
        matches!(self.classify(), ASTNodeType::Expression)
    }

    /// 📝 文パターンチェック - 実行エンジン用
    pub fn is_statement(&self) -> bool {
        matches!(self.classify(), ASTNodeType::Statement)
    }

    /// AST nodeの詳細情報を取得 (デバッグ用)
    pub fn info(&self) -> String {
        match self {
            ASTNode::Program { statements, .. } => {
                format!("Program({} statements)", statements.len())
            }
            ASTNode::Assignment { target, .. } => {
                format!("Assignment(target: {})", target.info())
            }
            ASTNode::Print { .. } => "Print".to_string(),
            ASTNode::If { .. } => "If".to_string(),
            ASTNode::Loop {
                condition: _, body, ..
            } => {
                format!("Loop({} statements)", body.len())
            }
            ASTNode::Return { value, .. } => {
                if value.is_some() {
                    "Return(with value)".to_string()
                } else {
                    "Return(void)".to_string()
                }
            }
            ASTNode::Break { .. } => "Break".to_string(),
            ASTNode::Continue { .. } => "Continue".to_string(),
            ASTNode::UsingStatement { namespace_name, .. } => {
                format!("UsingStatement({})", namespace_name)
            }
            ASTNode::ImportStatement { path, alias, .. } => {
                if let Some(a) = alias {
                    format!("ImportStatement({}, as {})", path, a)
                } else {
                    format!("ImportStatement({})", path)
                }
            }
            ASTNode::BoxDeclaration {
                name,
                fields,
                methods,
                constructors,
                is_interface,
                extends,
                implements,
                ..
            } => {
                let mut desc = if *is_interface {
                    format!("InterfaceBox({}, {} methods", name, methods.len())
                } else {
                    format!(
                        "BoxDeclaration({}, {} fields, {} methods, {} constructors",
                        name,
                        fields.len(),
                        methods.len(),
                        constructors.len()
                    )
                };

                if !extends.is_empty() {
                    desc.push_str(&format!(", extends [{}]", extends.join(", ")));
                }

                if !implements.is_empty() {
                    desc.push_str(&format!(", implements [{}]", implements.join(", ")));
                }

                desc.push(')');
                desc
            }
            ASTNode::FunctionDeclaration {
                name,
                params,
                body,
                is_static,
                is_override,
                ..
            } => {
                let static_str = if *is_static { "static " } else { "" };
                let override_str = if *is_override { "override " } else { "" };
                format!(
                    "FunctionDeclaration({}{}{}({}), {} statements)",
                    override_str,
                    static_str,
                    name,
                    params.join(", "),
                    body.len()
                )
            }
            ASTNode::GlobalVar { name, .. } => {
                format!("GlobalVar({})", name)
            }
            ASTNode::Literal { .. } => "Literal".to_string(),
            ASTNode::Variable { name, .. } => {
                format!("Variable({})", name)
            }
            ASTNode::UnaryOp { operator, .. } => {
                format!("UnaryOp({})", operator)
            }
            ASTNode::BinaryOp { operator, .. } => {
                format!("BinaryOp({})", operator)
            }
            ASTNode::MethodCall {
                method, arguments, ..
            } => {
                format!("MethodCall({}, {} args)", method, arguments.len())
            }
            ASTNode::FieldAccess { field, .. } => {
                format!("FieldAccess({})", field)
            }
            ASTNode::New {
                class,
                arguments,
                type_arguments,
                ..
            } => {
                if type_arguments.is_empty() {
                    format!("New({}, {} args)", class, arguments.len())
                } else {
                    format!(
                        "New({}<{}>, {} args)",
                        class,
                        type_arguments.join(", "),
                        arguments.len()
                    )
                }
            }
            ASTNode::This { .. } => "This".to_string(),
            ASTNode::Me { .. } => "Me".to_string(),
            ASTNode::FromCall {
                parent,
                method,
                arguments,
                ..
            } => {
                format!("FromCall({}.{}, {} args)", parent, method, arguments.len())
            }
            ASTNode::ThisField { field, .. } => {
                format!("ThisField({})", field)
            }
            ASTNode::MeField { field, .. } => {
                format!("MeField({})", field)
            }
            ASTNode::Include { filename, .. } => {
                format!("Include({})", filename)
            }
            ASTNode::Local { variables, .. } => {
                format!("Local({})", variables.join(", "))
            }
            ASTNode::Outbox { variables, .. } => {
                format!("Outbox({})", variables.join(", "))
            }
            ASTNode::FunctionCall {
                name, arguments, ..
            } => {
                format!("FunctionCall({}, {} args)", name, arguments.len())
            }
            ASTNode::Call { .. } => "Call".to_string(),
            ASTNode::Nowait { variable, .. } => {
                format!("Nowait({})", variable)
            }
            ASTNode::Arrow { .. } => "Arrow(>>)".to_string(),
            ASTNode::TryCatch {
                try_body,
                catch_clauses,
                finally_body,
                ..
            } => {
                let mut desc = format!(
                    "TryCatch({} try statements, {} catch clauses",
                    try_body.len(),
                    catch_clauses.len()
                );
                if finally_body.is_some() {
                    desc.push_str(", has finally");
                }
                desc.push(')');
                desc
            }
            ASTNode::Throw { .. } => "Throw".to_string(),
            ASTNode::AwaitExpression { expression, .. } => {
                format!("Await({:?})", expression)
            }
            ASTNode::MatchExpr { .. } => "MatchExpr".to_string(),
            ASTNode::QMarkPropagate { .. } => "QMarkPropagate".to_string(),
            ASTNode::Lambda { params, body, .. } => {
                format!("Lambda({} params, {} statements)", params.len(), body.len())
            }
            ASTNode::ArrayLiteral { elements, .. } => {
                format!("ArrayLiteral({} elements)", elements.len())
            }
            ASTNode::MapLiteral { entries, .. } => {
                format!("MapLiteral({} entries)", entries.len())
            }
            ASTNode::ScopeBox { .. } => "ScopeBox".to_string(),
        }
    }

    /// ASTノードからSpan情報を取得
    pub fn span(&self) -> Span {
        match self {
            ASTNode::Program { span, .. } => *span,
            ASTNode::Assignment { span, .. } => *span,
            ASTNode::Print { span, .. } => *span,
            ASTNode::If { span, .. } => *span,
            ASTNode::Loop { span, .. } => *span,
            ASTNode::Return { span, .. } => *span,
            ASTNode::Break { span, .. } => *span,
            ASTNode::Continue { span, .. } => *span,
            ASTNode::UsingStatement { span, .. } => *span,
            ASTNode::ImportStatement { span, .. } => *span,
            ASTNode::Nowait { span, .. } => *span,
            ASTNode::Arrow { span, .. } => *span,
            ASTNode::TryCatch { span, .. } => *span,
            ASTNode::Throw { span, .. } => *span,
            ASTNode::BoxDeclaration { span, .. } => *span,
            ASTNode::FunctionDeclaration { span, .. } => *span,
            ASTNode::GlobalVar { span, .. } => *span,
            ASTNode::Literal { span, .. } => *span,
            ASTNode::Variable { span, .. } => *span,
            ASTNode::UnaryOp { span, .. } => *span,
            ASTNode::BinaryOp { span, .. } => *span,
            ASTNode::MethodCall { span, .. } => *span,
            ASTNode::FieldAccess { span, .. } => *span,
            ASTNode::New { span, .. } => *span,
            ASTNode::This { span, .. } => *span,
            ASTNode::Me { span, .. } => *span,
            ASTNode::FromCall { span, .. } => *span,
            ASTNode::ThisField { span, .. } => *span,
            ASTNode::MeField { span, .. } => *span,
            ASTNode::Include { span, .. } => *span,
            ASTNode::Local { span, .. } => *span,
            ASTNode::Outbox { span, .. } => *span,
            ASTNode::FunctionCall { span, .. } => *span,
            ASTNode::Call { span, .. } => *span,
            ASTNode::AwaitExpression { span, .. } => *span,
            ASTNode::MatchExpr { span, .. } => *span,
            ASTNode::QMarkPropagate { span, .. } => *span,
            ASTNode::Lambda { span, .. } => *span,
            ASTNode::ArrayLiteral { span, .. } => *span,
            ASTNode::MapLiteral { span, .. } => *span,
            ASTNode::ScopeBox { span, .. } => *span,
        }
    }
}

impl fmt::Display for ASTNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.info())
    }
}

impl ASTNode {
    /// FunctionDeclarationのパラメータ数を取得
    pub fn get_param_count(&self) -> usize {
        match self {
            ASTNode::FunctionDeclaration { params, .. } => params.len(),
            _ => 0,
        }
    }
}
