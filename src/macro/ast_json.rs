use serde_json::{json, Value};
use nyash_rust::ast::{ASTNode, LiteralValue, BinaryOperator, UnaryOperator, Span};

pub fn ast_to_json(ast: &ASTNode) -> Value {
    match ast.clone() {
        ASTNode::Program { statements, .. } => json!({
            "kind": "Program",
            "statements": statements.into_iter().map(|s| ast_to_json(&s)).collect::<Vec<_>>()
        }),
        ASTNode::Loop { condition, body, .. } => json!({
            "kind": "Loop",
            "condition": ast_to_json(&condition),
            "body": body.into_iter().map(|s| ast_to_json(&s)).collect::<Vec<_>>()
        }),
        ASTNode::Print { expression, .. } => json!({
            "kind": "Print",
            "expression": ast_to_json(&expression),
        }),
        ASTNode::Return { value, .. } => json!({
            "kind": "Return",
            "value": value.as_ref().map(|v| ast_to_json(v)),
        }),
        ASTNode::Break { .. } => json!({"kind":"Break"}),
        ASTNode::Continue { .. } => json!({"kind":"Continue"}),
        ASTNode::Assignment { target, value, .. } => json!({
            "kind": "Assignment",
            "target": ast_to_json(&target),
            "value": ast_to_json(&value),
        }),
        ASTNode::Local { variables, initial_values, .. } => json!({
            "kind": "Local",
            "variables": variables,
            "inits": initial_values.into_iter().map(|opt| opt.map(|v| ast_to_json(&v))).collect::<Vec<_>>()
        }),
        ASTNode::If { condition, then_body, else_body, .. } => json!({
            "kind": "If",
            "condition": ast_to_json(&condition),
            "then": then_body.into_iter().map(|s| ast_to_json(&s)).collect::<Vec<_>>(),
            "else": else_body.map(|v| v.into_iter().map(|s| ast_to_json(&s)).collect::<Vec<_>>()),
        }),
        ASTNode::TryCatch { try_body, catch_clauses, finally_body, .. } => json!({
            "kind": "TryCatch",
            "try": try_body.into_iter().map(|s| ast_to_json(&s)).collect::<Vec<_>>(),
            "catch": catch_clauses.into_iter().map(|cc| json!({
                "type": cc.exception_type,
                "var": cc.variable_name,
                "body": cc.body.into_iter().map(|s| ast_to_json(&s)).collect::<Vec<_>>()
            })).collect::<Vec<_>>(),
            "cleanup": finally_body.map(|v| v.into_iter().map(|s| ast_to_json(&s)).collect::<Vec<_>>())
        }),
        ASTNode::FunctionDeclaration { name, params, body, is_static, is_override, .. } => json!({
            "kind": "FunctionDeclaration",
            "name": name,
            "params": params,
            "body": body.into_iter().map(|s| ast_to_json(&s)).collect::<Vec<_>>(),
            "static": is_static,
            "override": is_override,
        }),
        ASTNode::Variable { name, .. } => json!({"kind":"Variable","name":name}),
        ASTNode::Literal { value, .. } => json!({"kind":"Literal","value": lit_to_json(&value)}),
        ASTNode::BinaryOp { operator, left, right, .. } => json!({
            "kind":"BinaryOp",
            "op": bin_to_str(&operator),
            "left": ast_to_json(&left),
            "right": ast_to_json(&right),
        }),
        ASTNode::UnaryOp { operator, operand, .. } => json!({
            "kind":"UnaryOp",
            "op": un_to_str(&operator),
            "operand": ast_to_json(&operand),
        }),
        ASTNode::MethodCall { object, method, arguments, .. } => json!({
            "kind":"MethodCall",
            "object": ast_to_json(&object),
            "method": method,
            "arguments": arguments.into_iter().map(|a| ast_to_json(&a)).collect::<Vec<_>>()
        }),
        ASTNode::FunctionCall { name, arguments, .. } => json!({
            "kind":"FunctionCall",
            "name": name,
            "arguments": arguments.into_iter().map(|a| ast_to_json(&a)).collect::<Vec<_>>()
        }),
        ASTNode::ArrayLiteral { elements, .. } => json!({
            "kind":"Array",
            "elements": elements.into_iter().map(|e| ast_to_json(&e)).collect::<Vec<_>>()
        }),
        ASTNode::MapLiteral { entries, .. } => json!({
            "kind":"Map",
            "entries": entries.into_iter().map(|(k,v)| json!({"k":k,"v":ast_to_json(&v)})).collect::<Vec<_>>()
        }),
        ASTNode::MatchExpr { scrutinee, arms, else_expr, .. } => json!({
            "kind":"MatchExpr",
            "scrutinee": ast_to_json(&scrutinee),
            "arms": arms.into_iter().map(|(lit, body)| json!({
                "literal": {
                    "kind": "Literal",
                    "value": lit_to_json(&lit)
                },
                "body": ast_to_json(&body)
            })).collect::<Vec<_>>(),
            "else": ast_to_json(&else_expr),
        }),
        other => json!({"kind":"Unsupported","debug": format!("{:?}", other)}),
    }
}

pub fn json_to_ast(v: &Value) -> Option<ASTNode> {
    let k = v.get("kind")?.as_str()?;
    Some(match k {
        "Program" => {
            let stmts = v.get("statements")?.as_array()?.iter().filter_map(json_to_ast).collect::<Vec<_>>();
            ASTNode::Program { statements: stmts, span: Span::unknown() }
        }
        "Loop" => ASTNode::Loop {
            condition: Box::new(json_to_ast(v.get("condition")?)?),
            body: v.get("body")?.as_array()?.iter().filter_map(json_to_ast).collect::<Vec<_>>(),
            span: Span::unknown(),
        },
        "Print" => ASTNode::Print { expression: Box::new(json_to_ast(v.get("expression")?)?), span: Span::unknown() },
        "Return" => ASTNode::Return { value: v.get("value").and_then(json_to_ast).map(Box::new), span: Span::unknown() },
        "Break" => ASTNode::Break { span: Span::unknown() },
        "Continue" => ASTNode::Continue { span: Span::unknown() },
        "Assignment" => ASTNode::Assignment { target: Box::new(json_to_ast(v.get("target")?)?), value: Box::new(json_to_ast(v.get("value")?)?), span: Span::unknown() },
        "Local" => {
            let vars = v.get("variables")?.as_array()?.iter().filter_map(|s| s.as_str().map(|x| x.to_string())).collect();
            let inits = v.get("inits")?.as_array()?.iter().map(|initv| {
                if initv.is_null() { None } else { json_to_ast(initv).map(Box::new) }
            }).collect();
            ASTNode::Local { variables: vars, initial_values: inits, span: Span::unknown() }
        },
        "If" => ASTNode::If { condition: Box::new(json_to_ast(v.get("condition")?)?), then_body: v.get("then")?.as_array()?.iter().filter_map(json_to_ast).collect::<Vec<_>>(), else_body: v.get("else").and_then(|a| a.as_array().map(|arr| arr.iter().filter_map(json_to_ast).collect::<Vec<_>>())), span: Span::unknown() },
        "FunctionDeclaration" => ASTNode::FunctionDeclaration {
            name: v.get("name")?.as_str()?.to_string(),
            params: v.get("params")?.as_array()?.iter().filter_map(|s| s.as_str().map(|x| x.to_string())).collect(),
            body: v.get("body")?.as_array()?.iter().filter_map(json_to_ast).collect(),
            is_static: v.get("static").and_then(|b| b.as_bool()).unwrap_or(false),
            is_override: v.get("override").and_then(|b| b.as_bool()).unwrap_or(false),
            span: Span::unknown(),
        },
        "Variable" => ASTNode::Variable { name: v.get("name")?.as_str()?.to_string(), span: Span::unknown() },
        "Literal" => ASTNode::Literal { value: json_to_lit(v.get("value")?)?, span: Span::unknown() },
        "BinaryOp" => ASTNode::BinaryOp { operator: str_to_bin(v.get("op")?.as_str()?)?, left: Box::new(json_to_ast(v.get("left")?)?), right: Box::new(json_to_ast(v.get("right")?)?), span: Span::unknown() },
        "UnaryOp" => ASTNode::UnaryOp { operator: str_to_un(v.get("op")?.as_str()?)?, operand: Box::new(json_to_ast(v.get("operand")?)?), span: Span::unknown() },
        "MethodCall" => ASTNode::MethodCall { object: Box::new(json_to_ast(v.get("object")?)?), method: v.get("method")?.as_str()?.to_string(), arguments: v.get("arguments")?.as_array()?.iter().filter_map(json_to_ast).collect(), span: Span::unknown() },
        "FunctionCall" => ASTNode::FunctionCall { name: v.get("name")?.as_str()?.to_string(), arguments: v.get("arguments")?.as_array()?.iter().filter_map(json_to_ast).collect(), span: Span::unknown() },
        "Array" => ASTNode::ArrayLiteral { elements: v.get("elements")?.as_array()?.iter().filter_map(json_to_ast).collect(), span: Span::unknown() },
        "Map" => ASTNode::MapLiteral { entries: v.get("entries")?.as_array()?.iter().filter_map(|e| {
            Some((e.get("k")?.as_str()?.to_string(), json_to_ast(e.get("v")?)?))
        }).collect(), span: Span::unknown() },
        "MatchExpr" => {
            let scr = json_to_ast(v.get("scrutinee")?)?;
            let arms_json = v.get("arms")?.as_array()?.iter();
            let mut arms = Vec::new();
            for arm_v in arms_json {
                let lit_val = arm_v.get("literal")?.get("value")?;
                let lit = json_to_lit(lit_val)?;
                let body = json_to_ast(arm_v.get("body")?)?;
                arms.push((lit, body));
            }
            let else_expr = json_to_ast(v.get("else")?)?;
            ASTNode::MatchExpr {
                scrutinee: Box::new(scr),
                arms,
                else_expr: Box::new(else_expr),
                span: Span::unknown(),
            }
        }
        "TryCatch" => {
            let try_b = v.get("try")?.as_array()?.iter().filter_map(json_to_ast).collect::<Vec<_>>();
            let mut catches = Vec::new();
            if let Some(arr) = v.get("catch").and_then(|x| x.as_array()) {
                for c in arr.iter() {
                    let exc_t = match c.get("type") { Some(t) if !t.is_null() => t.as_str().map(|s| s.to_string()), _ => None };
                    let var = match c.get("var") { Some(vv) if !vv.is_null() => vv.as_str().map(|s| s.to_string()), _ => None };
                    let body = c.get("body")?.as_array()?.iter().filter_map(json_to_ast).collect::<Vec<_>>();
                    catches.push(nyash_rust::ast::CatchClause { exception_type: exc_t, variable_name: var, body, span: Span::unknown() });
                }
            }
            let cleanup = v.get("cleanup").and_then(|cl| cl.as_array().map(|arr| arr.iter().filter_map(json_to_ast).collect::<Vec<_>>()));
            ASTNode::TryCatch { try_body: try_b, catch_clauses: catches, finally_body: cleanup, span: Span::unknown() }
        }
        _ => return None,
    })
}

fn lit_to_json(v: &LiteralValue) -> Value {
    match v {
        LiteralValue::String(s) => json!({"type":"string","value":s}),
        LiteralValue::Integer(i) => json!({"type":"int","value":i}),
        LiteralValue::Float(f) => json!({"type":"float","value":f}),
        LiteralValue::Bool(b) => json!({"type":"bool","value":b}),
        LiteralValue::Null => json!({"type":"null"}),
        LiteralValue::Void => json!({"type":"void"}),
    }
}

fn json_to_lit(v: &Value) -> Option<LiteralValue> {
    let t = v.get("type")?.as_str()?;
    Some(match t {
        "string" => LiteralValue::String(v.get("value")?.as_str()?.to_string()),
        "int" => LiteralValue::Integer(v.get("value")?.as_i64()?),
        "float" => LiteralValue::Float(v.get("value")?.as_f64()?),
        "bool" => LiteralValue::Bool(v.get("value")?.as_bool()?),
        "null" => LiteralValue::Null,
        "void" => LiteralValue::Void,
        _ => return None,
    })
}

fn bin_to_str(op: &BinaryOperator) -> &'static str {
    match op {
        BinaryOperator::Add => "+",
        BinaryOperator::Subtract => "-",
        BinaryOperator::Multiply => "*",
        BinaryOperator::Divide => "/",
        BinaryOperator::Modulo => "%",
        BinaryOperator::BitAnd => "&",
        BinaryOperator::BitOr => "|",
        BinaryOperator::BitXor => "^",
        BinaryOperator::Shl => "<<",
        BinaryOperator::Shr => ">>",
        BinaryOperator::Equal => "==",
        BinaryOperator::NotEqual => "!=",
        BinaryOperator::Less => "<",
        BinaryOperator::Greater => ">",
        BinaryOperator::LessEqual => "<=",
        BinaryOperator::GreaterEqual => ">=",
        BinaryOperator::And => "&&",
        BinaryOperator::Or => "||",
    }
}

fn str_to_bin(s: &str) -> Option<BinaryOperator> {
    Some(match s {
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
        "==" => BinaryOperator::Equal,
        "!=" => BinaryOperator::NotEqual,
        "<" => BinaryOperator::Less,
        ">" => BinaryOperator::Greater,
        "<=" => BinaryOperator::LessEqual,
        ">=" => BinaryOperator::GreaterEqual,
        "&&" => BinaryOperator::And,
        "||" => BinaryOperator::Or,
        _ => return None,
    })
}

fn un_to_str(op: &UnaryOperator) -> &'static str {
    match op {
        UnaryOperator::Minus => "-",
        UnaryOperator::Not => "not",
        UnaryOperator::BitNot => "~",
    }
}

fn str_to_un(s: &str) -> Option<UnaryOperator> {
    Some(match s {
        "-" => UnaryOperator::Minus,
        "not" => UnaryOperator::Not,
        "~" => UnaryOperator::BitNot,
        _ => return None,
    })
}
