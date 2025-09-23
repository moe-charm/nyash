fn map_expr_to_stmt(e: nyash_rust::ASTNode) -> nyash_rust::ASTNode { e }

fn transform_peek_to_if_expr(peek: &nyash_rust::ASTNode) -> Option<nyash_rust::ASTNode> {
    use nyash_rust::ast::{ASTNode as A, BinaryOperator, Span};
    if let A::MatchExpr { scrutinee, arms, else_expr, .. } = peek {
        let mut conds_bodies: Vec<(nyash_rust::ast::LiteralValue, A)> = Vec::new();
        for (lit, body) in arms { conds_bodies.push((lit.clone(), (*body).clone())); }
        let mut current: A = *(*else_expr).clone();
        for (lit, body) in conds_bodies.into_iter().rev() {
            let rhs = A::Literal { value: lit, span: Span::unknown() };
            let cond = A::BinaryOp { operator: BinaryOperator::Equal, left: scrutinee.clone(), right: Box::new(rhs), span: Span::unknown() };
            let then_body = vec![map_expr_to_stmt(body)];
            let else_body = Some(vec![map_expr_to_stmt(current)]);
            current = A::If { condition: Box::new(cond), then_body, else_body, span: Span::unknown() };
        }
        Some(current)
    } else { None }
}

fn transform_peek_to_if_stmt_assign(peek: &nyash_rust::ASTNode, target: &nyash_rust::ASTNode) -> Option<nyash_rust::ASTNode> {
    use nyash_rust::ast::{ASTNode as A, BinaryOperator, Span};
    if let A::MatchExpr { scrutinee, arms, else_expr, .. } = peek {
        let mut pairs: Vec<(nyash_rust::ast::LiteralValue, A)> = Vec::new();
        for (lit, body) in arms { pairs.push((lit.clone(), (*body).clone())); }
        let mut current: A = *(*else_expr).clone();
        for (lit, body) in pairs.into_iter().rev() {
            let rhs = A::Literal { value: lit, span: Span::unknown() };
            let cond = A::BinaryOp { operator: BinaryOperator::Equal, left: scrutinee.clone(), right: Box::new(rhs), span: Span::unknown() };
            let then_body = vec![A::Assignment { target: Box::new(target.clone()), value: Box::new(body), span: Span::unknown() }];
            let else_body = Some(vec![map_expr_to_stmt(current)]);
            current = A::If { condition: Box::new(cond), then_body, else_body, span: Span::unknown() };
        }
        Some(current)
    } else { None }
}

fn transform_peek_to_if_stmt_return(peek: &nyash_rust::ASTNode) -> Option<nyash_rust::ASTNode> {
    use nyash_rust::ast::{ASTNode as A, BinaryOperator, Span};
    if let A::MatchExpr { scrutinee, arms, else_expr, .. } = peek {
        let mut pairs: Vec<(nyash_rust::ast::LiteralValue, A)> = Vec::new();
        for (lit, body) in arms { pairs.push((lit.clone(), (*body).clone())); }
        let mut current: A = *(*else_expr).clone();
        for (lit, body) in pairs.into_iter().rev() {
            let rhs = A::Literal { value: lit, span: Span::unknown() };
            let cond = A::BinaryOp { operator: BinaryOperator::Equal, left: scrutinee.clone(), right: Box::new(rhs), span: Span::unknown() };
            let then_body = vec![A::Return { value: Some(Box::new(body)), span: Span::unknown() }];
            let else_body = Some(vec![map_expr_to_stmt(current)]);
            current = A::If { condition: Box::new(cond), then_body, else_body, span: Span::unknown() };
        }
        Some(current)
    } else { None }
}

fn transform_peek_to_if_stmt_print(peek: &nyash_rust::ASTNode) -> Option<nyash_rust::ASTNode> {
    use nyash_rust::ast::{ASTNode as A, BinaryOperator, Span};
    if let A::MatchExpr { scrutinee, arms, else_expr, .. } = peek {
        let mut pairs: Vec<(nyash_rust::ast::LiteralValue, A)> = Vec::new();
        for (lit, body) in arms { pairs.push((lit.clone(), (*body).clone())); }
        let mut current: A = *(*else_expr).clone();
        for (lit, body) in pairs.into_iter().rev() {
            let rhs = A::Literal { value: lit, span: Span::unknown() };
            let cond = A::BinaryOp { operator: BinaryOperator::Equal, left: scrutinee.clone(), right: Box::new(rhs), span: Span::unknown() };
            let then_body = vec![A::Print { expression: Box::new(body), span: Span::unknown() }];
            let else_body = Some(vec![map_expr_to_stmt(current)]);
            current = A::If { condition: Box::new(cond), then_body, else_body, span: Span::unknown() };
        }
        Some(current)
    } else { None }
}

pub(super) fn transform_peek_match_literal(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    use nyash_rust::ast::ASTNode as A;
    match ast.clone() {
        A::Program { statements, span } => A::Program { statements: statements.into_iter().map(|n| transform_peek_match_literal(&n)).collect(), span },
        A::If { condition, then_body, else_body, span } => A::If {
            condition: Box::new(transform_peek_match_literal(&condition)),
            then_body: then_body.into_iter().map(|n| transform_peek_match_literal(&n)).collect(),
            else_body: else_body.map(|v| v.into_iter().map(|n| transform_peek_match_literal(&n)).collect()),
            span,
        },
        A::Loop { condition, body, span } => A::Loop {
            condition: Box::new(transform_peek_match_literal(&condition)),
            body: body.into_iter().map(|n| transform_peek_match_literal(&n)).collect(),
            span,
        },
        A::BinaryOp { operator, left, right, span } => A::BinaryOp { operator, left: Box::new(transform_peek_match_literal(&left)), right: Box::new(transform_peek_match_literal(&right)), span },
        A::UnaryOp { operator, operand, span } => A::UnaryOp { operator, operand: Box::new(transform_peek_match_literal(&operand)), span },
        A::MethodCall { object, method, arguments, span } => A::MethodCall { object: Box::new(transform_peek_match_literal(&object)), method, arguments: arguments.into_iter().map(|a| transform_peek_match_literal(&a)).collect(), span },
        A::FunctionCall { name, arguments, span } => {
            if let Some(if_expr) = transform_peek_to_if_expr(&A::FunctionCall { name: name.clone(), arguments: arguments.clone(), span }) {
                if_expr
            } else { A::FunctionCall { name, arguments: arguments.into_iter().map(|a| transform_peek_match_literal(&a)).collect(), span } }
        }
        A::ArrayLiteral { elements, span } => A::ArrayLiteral { elements: elements.into_iter().map(|e| transform_peek_match_literal(&e)).collect(), span },
        A::MapLiteral { entries, span } => A::MapLiteral { entries: entries.into_iter().map(|(k, v)| (k, transform_peek_match_literal(&v))).collect(), span },
        A::Assignment { target, value, span } => {
            if let Some(ifstmt) = transform_peek_to_if_stmt_assign(&value, &target) { ifstmt }
            else { A::Assignment { target, value: Box::new(transform_peek_match_literal(&value)), span } }
        }
        A::Return { value, span } => {
            if let Some(v) = &value {
                if let Some(ifstmt) = transform_peek_to_if_stmt_return(v) { ifstmt }
                else { A::Return { value: Some(Box::new(transform_peek_match_literal(v))), span } }
            } else { A::Return { value: None, span } }
        }
        A::Print { expression, span } => {
            if let Some(ifstmt) = transform_peek_to_if_stmt_print(&expression) { ifstmt }
            else { A::Print { expression: Box::new(transform_peek_match_literal(&expression)), span } }
        }
        other => other,
    }
}

