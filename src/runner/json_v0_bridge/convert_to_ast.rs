use super::ast::{ProgramV0, StmtV0, ExprV0, CatchV0, MatchArmV0};
use crate::ast::{ASTNode, LiteralValue, Span, BinaryOperator, CatchClause};

pub(super) fn convert_program_to_ast(p: ProgramV0) -> Result<ASTNode, String> {
    let mut out: Vec<ASTNode> = Vec::with_capacity(p.body.len());
    for s in p.body.iter() {
        out.push(convert_stmt(s)?);
    }
    Ok(ASTNode::Program { statements: out, span: Span::unknown() })
}

fn convert_stmt(s: &StmtV0) -> Result<ASTNode, String> {
    Ok(match s {
        StmtV0::Return { expr } => ASTNode::Return { value: Some(Box::new(convert_expr(expr)?)), span: Span::unknown() },
        StmtV0::Expr { expr } => convert_expr(expr)?,
        StmtV0::Local { name, expr } => ASTNode::Local {
            variables: vec![name.clone()],
            initial_values: vec![Some(Box::new(convert_expr(expr)?))],
            span: Span::unknown(),
        },
        StmtV0::If { cond, then, r#else } => {
            let mut then_nodes: Vec<ASTNode> = Vec::with_capacity(then.len());
            for t in then.iter() { then_nodes.push(convert_stmt(t)?); }
            let else_nodes = if let Some(es) = r#else.as_ref() {
                let mut v = Vec::with_capacity(es.len());
                for e in es.iter() { v.push(convert_stmt(e)?); }
                Some(v)
            } else { None };
            ASTNode::If {
                condition: Box::new(convert_expr(cond)?),
                then_body: then_nodes,
                else_body: else_nodes,
                span: Span::unknown(),
            }
        }
        StmtV0::Loop { cond, body } => {
            let mut body_nodes: Vec<ASTNode> = Vec::with_capacity(body.len());
            for t in body.iter() { body_nodes.push(convert_stmt(t)?); }
            ASTNode::Loop { condition: Box::new(convert_expr(cond)?), body: body_nodes, span: Span::unknown() }
        }
        StmtV0::Break => ASTNode::Break { span: Span::unknown() },
        StmtV0::Continue => ASTNode::Continue { span: Span::unknown() },
        StmtV0::Try { try_body, catches, finally } => {
            let mut t: Vec<ASTNode> = Vec::with_capacity(try_body.len());
            for s in try_body.iter() { t.push(convert_stmt(s)?); }
            let mut cs: Vec<CatchClause> = Vec::with_capacity(catches.len());
            for CatchV0 { param, type_hint, body } in catches.iter() {
                let mut b: Vec<ASTNode> = Vec::with_capacity(body.len());
                for s in body.iter() { b.push(convert_stmt(s)?); }
                cs.push(CatchClause { exception_type: type_hint.clone(), variable_name: param.clone(), body: b, span: Span::unknown() });
            }
            let fb = if finally.is_empty() { None } else {
                let mut f: Vec<ASTNode> = Vec::with_capacity(finally.len());
                for s in finally.iter() { f.push(convert_stmt(s)?); }
                Some(f)
            };
            ASTNode::TryCatch { try_body: t, catch_clauses: cs, finally_body: fb, span: Span::unknown() }
        }
        StmtV0::Extern { iface, method, args } => {
            let (ns, tail) = parse_iface(iface)?;
            let obj = ASTNode::FieldAccess { object: Box::new(ASTNode::Variable { name: ns, span: Span::unknown() }), field: tail, span: Span::unknown() };
            let mut a: Vec<ASTNode> = Vec::with_capacity(args.len());
            for e in args.iter() { a.push(convert_expr(e)?); }
            ASTNode::MethodCall { object: Box::new(obj), method: method.clone(), arguments: a, span: Span::unknown() }
        },
    })
}

fn convert_expr(e: &ExprV0) -> Result<ASTNode, String> {
    Ok(match e {
        ExprV0::Int { value } => {
            let n = value.as_i64().ok_or_else(|| format!("invalid Int value: {}", value))?;
            ASTNode::Literal { value: LiteralValue::Integer(n), span: Span::unknown() }
        }
        ExprV0::Str { value } => ASTNode::Literal { value: LiteralValue::String(value.clone()), span: Span::unknown() },
        ExprV0::Bool { value } => ASTNode::Literal { value: LiteralValue::Bool(*value), span: Span::unknown() },
        ExprV0::Binary { op, lhs, rhs } => ASTNode::BinaryOp {
            operator: parse_bin_op(op)?,
            left: Box::new(convert_expr(lhs)?),
            right: Box::new(convert_expr(rhs)?),
            span: Span::unknown(),
        },
        ExprV0::Compare { op, lhs, rhs } => ASTNode::BinaryOp {
            operator: parse_cmp_op(op)?,
            left: Box::new(convert_expr(lhs)?),
            right: Box::new(convert_expr(rhs)?),
            span: Span::unknown(),
        },
        ExprV0::Logical { op, lhs, rhs } => ASTNode::BinaryOp {
            operator: match op.as_str() { "&&" => BinaryOperator::And, "||" => BinaryOperator::Or, _ => return Err(format!("unsupported logical op: {}", op)) },
            left: Box::new(convert_expr(lhs)?),
            right: Box::new(convert_expr(rhs)?),
            span: Span::unknown(),
        },
        ExprV0::Call { name, args } => {
            let mut out: Vec<ASTNode> = Vec::with_capacity(args.len());
            for a in args.iter() { out.push(convert_expr(a)?); }
            ASTNode::FunctionCall { name: name.clone(), arguments: out, span: Span::unknown() }
        }
        ExprV0::Method { recv, method, args } => {
            let mut out: Vec<ASTNode> = Vec::with_capacity(args.len());
            for a in args.iter() { out.push(convert_expr(a)?); }
            ASTNode::MethodCall { object: Box::new(convert_expr(recv)?), method: method.clone(), arguments: out, span: Span::unknown() }
        }
        ExprV0::New { class, args } => {
            let mut out: Vec<ASTNode> = Vec::with_capacity(args.len());
            for a in args.iter() { out.push(convert_expr(a)?); }
            ASTNode::New { class: class.clone(), arguments: out, type_arguments: vec![], span: Span::unknown() }
        }
        ExprV0::Var { name } => ASTNode::Variable { name: name.clone(), span: Span::unknown() },
        ExprV0::Throw { expr } => ASTNode::Throw { expression: Box::new(convert_expr(expr)?), span: Span::unknown() },
        ExprV0::Ternary { cond, then, r#else } => {
            let then_prog = vec![convert_expr(then)?];
            let else_prog = vec![convert_expr(r#else)?];
            ASTNode::If {
                condition: Box::new(convert_expr(cond)?),
                then_body: then_prog,
                else_body: Some(else_prog),
                span: Span::unknown(),
            }
        },
        ExprV0::Match { scrutinee, arms, r#else } => {
            let s = Box::new(convert_expr(scrutinee)?);
            let mut out_arms: Vec<(LiteralValue, ASTNode)> = Vec::with_capacity(arms.len());
            for MatchArmV0 { label, expr } in arms.iter() {
                out_arms.push((LiteralValue::String(label.clone()), convert_expr(expr)?));
            }
            let else_e = Box::new(convert_expr(r#else)?);
            ASTNode::MatchExpr { scrutinee: s, arms: out_arms, else_expr: else_e, span: Span::unknown() }
        }
        ExprV0::Extern { iface, method, args } => {
            let (ns, tail) = parse_iface(iface)?;
            let obj = ASTNode::FieldAccess { object: Box::new(ASTNode::Variable { name: ns, span: Span::unknown() }), field: tail, span: Span::unknown() };
            let mut a: Vec<ASTNode> = Vec::with_capacity(args.len());
            for e in args.iter() { a.push(convert_expr(e)?); }
            ASTNode::MethodCall { object: Box::new(obj), method: method.clone(), arguments: a, span: Span::unknown() }
        },
    })
}

fn parse_bin_op(op: &str) -> Result<BinaryOperator, String> {
    Ok(match op {
        "+" => BinaryOperator::Add,
        "-" => BinaryOperator::Subtract,
        "*" => BinaryOperator::Multiply,
        "/" => BinaryOperator::Divide,
        "%" => BinaryOperator::Modulo,
        "&" => BinaryOperator::BitAnd,
        "|" => BinaryOperator::BitOr,
        "^" => BinaryOperator::BitXor,
        "<<" => BinaryOperator::Shl,
        ">>" => BinaryOperator::Shr,
        _ => return Err(format!("unsupported binary op: {}", op)),
    })
}

fn parse_cmp_op(op: &str) -> Result<BinaryOperator, String> {
    Ok(match op {
        "==" => BinaryOperator::Equal,
        "!=" => BinaryOperator::NotEqual,
        "<" => BinaryOperator::Less,
        ">" => BinaryOperator::Greater,
        "<=" => BinaryOperator::LessEqual,
        ">=" => BinaryOperator::GreaterEqual,
        _ => return Err(format!("unsupported compare op: {}", op)),
    })
}


fn parse_iface(iface: &str) -> Result<(String, String), String> {
    if let Some((ns, rest)) = iface.split_once('.') {
        if ns == "env" || ns == "nyrt" { return Ok((ns.to_string(), rest.to_string())); }
        return Err(format!("unsupported extern namespace: {}", ns));
    }
    Ok(("env".to_string(), iface.to_string()))
}
