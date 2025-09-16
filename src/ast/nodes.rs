//! Lightweight wrapper structs around ASTNode variants (non-breaking).
//!
//! Purpose: provide a gentle path to work with structured nodes via
//! TryFrom/From without changing the canonical AST enum. This enables
//! gradual refactors in builders by converting once at the boundary and
//! then matching on small, typed wrappers.

use super::{ASTNode, Span};

// ----------------
// Statements
// ----------------

#[derive(Debug, Clone)]
pub struct AssignStmt {
    pub target: Box<ASTNode>,
    pub value: Box<ASTNode>,
    pub span: Span,
}

impl TryFrom<ASTNode> for AssignStmt {
    type Error = ASTNode;
    fn try_from(node: ASTNode) -> Result<Self, Self::Error> {
        match node {
            ASTNode::Assignment { target, value, span } => Ok(AssignStmt { target, value, span }),
            other => Err(other),
        }
    }
}

impl From<AssignStmt> for ASTNode {
    fn from(s: AssignStmt) -> Self {
        ASTNode::Assignment { target: s.target, value: s.value, span: s.span }
    }
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub value: Option<Box<ASTNode>>,
    pub span: Span,
}

impl TryFrom<ASTNode> for ReturnStmt {
    type Error = ASTNode;
    fn try_from(node: ASTNode) -> Result<Self, Self::Error> {
        match node {
            ASTNode::Return { value, span } => Ok(ReturnStmt { value, span }),
            other => Err(other),
        }
    }
}

impl From<ReturnStmt> for ASTNode {
    fn from(s: ReturnStmt) -> Self {
        ASTNode::Return { value: s.value, span: s.span }
    }
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Box<ASTNode>,
    pub then_body: Vec<ASTNode>,
    pub else_body: Option<Vec<ASTNode>>,
    pub span: Span,
}

impl TryFrom<ASTNode> for IfStmt {
    type Error = ASTNode;
    fn try_from(node: ASTNode) -> Result<Self, Self::Error> {
        match node {
            ASTNode::If { condition, then_body, else_body, span } => Ok(IfStmt { condition, then_body, else_body, span }),
            other => Err(other),
        }
    }
}

impl From<IfStmt> for ASTNode {
    fn from(s: IfStmt) -> Self {
        ASTNode::If { condition: s.condition, then_body: s.then_body, else_body: s.else_body, span: s.span }
    }
}

// ----------------
// Expressions
// ----------------

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub operator: super::BinaryOperator,
    pub left: Box<ASTNode>,
    pub right: Box<ASTNode>,
    pub span: Span,
}

impl TryFrom<ASTNode> for BinaryExpr {
    type Error = ASTNode;
    fn try_from(node: ASTNode) -> Result<Self, Self::Error> {
        match node {
            ASTNode::BinaryOp { operator, left, right, span } =>
                Ok(BinaryExpr { operator, left, right, span }),
            other => Err(other),
        }
    }
}

impl From<BinaryExpr> for ASTNode {
    fn from(e: BinaryExpr) -> Self {
        ASTNode::BinaryOp { operator: e.operator, left: e.left, right: e.right, span: e.span }
    }
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub name: String,
    pub arguments: Vec<ASTNode>,
    pub span: Span,
}

impl TryFrom<ASTNode> for CallExpr {
    type Error = ASTNode;
    fn try_from(node: ASTNode) -> Result<Self, Self::Error> {
        match node {
            ASTNode::FunctionCall { name, arguments, span } => Ok(CallExpr { name, arguments, span }),
            other => Err(other),
        }
    }
}

impl From<CallExpr> for ASTNode {
    fn from(c: CallExpr) -> Self {
        ASTNode::FunctionCall { name: c.name, arguments: c.arguments, span: c.span }
    }
}

#[derive(Debug, Clone)]
pub struct MethodCallExpr {
    pub object: Box<ASTNode>,
    pub method: String,
    pub arguments: Vec<ASTNode>,
    pub span: Span,
}

impl TryFrom<ASTNode> for MethodCallExpr {
    type Error = ASTNode;
    fn try_from(node: ASTNode) -> Result<Self, Self::Error> {
        match node {
            ASTNode::MethodCall { object, method, arguments, span } =>
                Ok(MethodCallExpr { object, method, arguments, span }),
            other => Err(other),
        }
    }
}

impl From<MethodCallExpr> for ASTNode {
    fn from(m: MethodCallExpr) -> Self {
        ASTNode::MethodCall { object: m.object, method: m.method, arguments: m.arguments, span: m.span }
    }
}

#[derive(Debug, Clone)]
pub struct FieldAccessExpr {
    pub object: Box<ASTNode>,
    pub field: String,
    pub span: Span,
}

impl TryFrom<ASTNode> for FieldAccessExpr {
    type Error = ASTNode;
    fn try_from(node: ASTNode) -> Result<Self, Self::Error> {
        match node {
            ASTNode::FieldAccess { object, field, span } =>
                Ok(FieldAccessExpr { object, field, span }),
            other => Err(other),
        }
    }
}

impl From<FieldAccessExpr> for ASTNode {
    fn from(f: FieldAccessExpr) -> Self {
        ASTNode::FieldAccess { object: f.object, field: f.field, span: f.span }
    }
}
