//! Properties parsing (once/birth_once, header-first)
use crate::ast::{ASTNode, Span};
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;
use std::collections::HashMap;


/// Try to parse a unified member property: `once name: Type ...` or `birth_once name: Type ...`
/// Returns Ok(true) if consumed and handled; otherwise Ok(false).
pub(crate) fn try_parse_unified_property(
    p: &mut NyashParser,
    kind_kw: &str,
    methods: &mut HashMap<String, ASTNode>,
    birth_once_props: &mut Vec<String>,
) -> Result<bool, ParseError> {
    if !(kind_kw == "once" || kind_kw == "birth_once") {
        return Ok(false);
    }
    // Name
    let name = if let TokenType::IDENTIFIER(n) = &p.current_token().token_type {
        let n2 = n.clone();
        p.advance();
        n2
    } else {
        return Err(ParseError::UnexpectedToken {
            found: p.current_token().token_type.clone(),
            expected: "identifier after once/birth_once".to_string(),
            line: p.current_token().line,
        });
    };
    // ':' TYPE (type is accepted and ignored for now)
    if p.match_token(&TokenType::COLON) {
        p.advance();
        if let TokenType::IDENTIFIER(_ty) = &p.current_token().token_type {
            p.advance();
        } else {
            return Err(ParseError::UnexpectedToken {
                found: p.current_token().token_type.clone(),
                expected: "type name".to_string(),
                line: p.current_token().line,
            });
        }
    } else {
        return Err(ParseError::UnexpectedToken {
            found: p.current_token().token_type.clone(),
            expected: ": type".to_string(),
            line: p.current_token().line,
        });
    }
    // Body: either fat arrow expr or block
    let orig_body: Vec<ASTNode> = if p.match_token(&TokenType::FatArrow) {
        p.advance(); // consume '=>'
        let expr = p.parse_expression()?;
        vec![ASTNode::Return { value: Some(Box::new(expr)), span: Span::unknown() }]
    } else {
        p.parse_block_statements()?
    };
    // Optional postfix handlers (Stage-3) directly after body
    let final_body = crate::parser::declarations::box_def::members::postfix::wrap_with_optional_postfix(p, orig_body)?;
    if kind_kw == "once" {
        // once: synthesize compute + getter with poison/cache
        let compute_name = format!("__compute_once_{}", name);
        let compute = ASTNode::FunctionDeclaration {
            name: compute_name.clone(),
            params: vec![],
            body: final_body,
            is_static: false,
            is_override: false,
            span: Span::unknown(),
        };
        methods.insert(compute_name.clone(), compute);
        // Build complex getter wrapper identical to legacy impl
        let key = format!("__once_{}", name);
        let poison_key = format!("__once_poison_{}", name);
        let cached_local = format!("__ny_cached_{}", name);
        let poison_local = format!("__ny_poison_{}", name);
        let val_local = format!("__ny_val_{}", name);
        let me_node = ASTNode::Me { span: Span::unknown() };
        let get_cached = ASTNode::MethodCall {
            object: Box::new(me_node.clone()),
            method: "getField".to_string(),
            arguments: vec![ASTNode::Literal { value: crate::ast::LiteralValue::String(key.clone()), span: Span::unknown() }],
            span: Span::unknown(),
        };
        let local_cached = ASTNode::Local { variables: vec![cached_local.clone()], initial_values: vec![Some(Box::new(get_cached))], span: Span::unknown() };
        let cond_cached = ASTNode::BinaryOp { operator: crate::ast::BinaryOperator::NotEqual, left: Box::new(ASTNode::Variable { name: cached_local.clone(), span: Span::unknown() }), right: Box::new(ASTNode::Literal { value: crate::ast::LiteralValue::Null, span: Span::unknown() }), span: Span::unknown() };
        let then_ret_cached = vec![ASTNode::Return { value: Some(Box::new(ASTNode::Variable { name: cached_local.clone(), span: Span::unknown() })), span: Span::unknown() }];
        let if_cached = ASTNode::If { condition: Box::new(cond_cached), then_body: then_ret_cached, else_body: None, span: Span::unknown() };
        let get_poison = ASTNode::MethodCall { object: Box::new(me_node.clone()), method: "getField".to_string(), arguments: vec![ASTNode::Literal { value: crate::ast::LiteralValue::String(poison_key.clone()), span: Span::unknown() }], span: Span::unknown() };
        let local_poison = ASTNode::Local { variables: vec![poison_local.clone()], initial_values: vec![Some(Box::new(get_poison))], span: Span::unknown() };
        let cond_poison = ASTNode::BinaryOp { operator: crate::ast::BinaryOperator::NotEqual, left: Box::new(ASTNode::Variable { name: poison_local.clone(), span: Span::unknown() }), right: Box::new(ASTNode::Literal { value: crate::ast::LiteralValue::Null, span: Span::unknown() }), span: Span::unknown() };
        let then_throw = vec![ASTNode::Throw { expression: Box::new(ASTNode::Literal { value: crate::ast::LiteralValue::String(format!("once '{}' previously failed", name)), span: Span::unknown() }), span: Span::unknown() }];
        let if_poison = ASTNode::If { condition: Box::new(cond_poison), then_body: then_throw, else_body: None, span: Span::unknown() };
        let call_compute = ASTNode::MethodCall { object: Box::new(me_node.clone()), method: compute_name.clone(), arguments: vec![], span: Span::unknown() };
        let local_val = ASTNode::Local { variables: vec![val_local.clone()], initial_values: vec![Some(Box::new(call_compute))], span: Span::unknown() };
        let set_call = ASTNode::MethodCall { object: Box::new(me_node.clone()), method: "setField".to_string(), arguments: vec![ASTNode::Literal { value: crate::ast::LiteralValue::String(key.clone()), span: Span::unknown() }, ASTNode::Variable { name: val_local.clone(), span: Span::unknown() }], span: Span::unknown() };
        let ret_stmt = ASTNode::Return { value: Some(Box::new(ASTNode::Variable { name: val_local.clone(), span: Span::unknown() })), span: Span::unknown() };
        let getter_body = vec![local_cached, if_cached, local_poison, if_poison, local_val, set_call, ret_stmt];
        let getter_name = format!("__get_once_{}", name);
        let getter = ASTNode::FunctionDeclaration { name: getter_name.clone(), params: vec![], body: getter_body, is_static: false, is_override: false, span: Span::unknown() };
        methods.insert(getter_name, getter);
        return Ok(true);
    }
    // birth_once
    birth_once_props.push(name.clone());
    let compute_name = format!("__compute_birth_{}", name);
    let compute = ASTNode::FunctionDeclaration { name: compute_name.clone(), params: vec![], body: final_body, is_static: false, is_override: false, span: Span::unknown() };
    methods.insert(compute_name.clone(), compute);
    let me_node = ASTNode::Me { span: Span::unknown() };
    // getter: me.getField("__birth_name")
    let get_call = ASTNode::MethodCall { object: Box::new(me_node.clone()), method: "getField".to_string(), arguments: vec![ASTNode::Literal { value: crate::ast::LiteralValue::String(format!("__birth_{}", name)), span: Span::unknown() }], span: Span::unknown() };
    let getter_body = vec![ASTNode::Return { value: Some(Box::new(get_call)), span: Span::unknown() }];
    let getter_name = format!("__get_birth_{}", name);
    let getter = ASTNode::FunctionDeclaration { name: getter_name.clone(), params: vec![], body: getter_body, is_static: false, is_override: false, span: Span::unknown() };
    methods.insert(getter_name, getter);
    Ok(true)
}

/// Try to parse a block-first unified member: `{ body } as [once|birth_once]? name : Type [postfix]`
/// Returns Ok(true) if a member was parsed and emitted into `methods`.
pub(crate) fn try_parse_block_first_property(
    p: &mut NyashParser,
    methods: &mut HashMap<String, ASTNode>,
    birth_once_props: &mut Vec<String>,
) -> Result<bool, ParseError> {
    if !(crate::config::env::unified_members() && p.match_token(&TokenType::LBRACE)) {
        return Ok(false);
    }
    // 1) Parse block body first
    let mut final_body = p.parse_block_statements()?;

    // 2) Expect 'as'
    if let TokenType::IDENTIFIER(kw) = &p.current_token().token_type {
        if kw != "as" {
            let line = p.current_token().line;
            return Err(ParseError::UnexpectedToken {
                found: p.current_token().token_type.clone(),
                expected: "'as' after block for block-first member".to_string(),
                line,
            });
        }
    } else {
        let line = p.current_token().line;
        return Err(ParseError::UnexpectedToken {
            found: p.current_token().token_type.clone(),
            expected: "'as' after block for block-first member".to_string(),
            line,
        });
    }
    p.advance(); // consume 'as'

    // 3) Optional kind keyword: once | birth_once
    let mut kind = "computed".to_string();
    if let TokenType::IDENTIFIER(k) = &p.current_token().token_type {
        if k == "once" || k == "birth_once" {
            kind = k.clone();
            p.advance();
        }
    }

    // 4) Name : Type
    let name = if let TokenType::IDENTIFIER(n) = &p.current_token().token_type {
        let s = n.clone();
        p.advance();
        s
    } else {
        let line = p.current_token().line;
        return Err(ParseError::UnexpectedToken {
            found: p.current_token().token_type.clone(),
            expected: "identifier for member name".to_string(),
            line,
        });
    };
    if p.match_token(&TokenType::COLON) {
        p.advance();
        if let TokenType::IDENTIFIER(_ty) = &p.current_token().token_type {
            p.advance();
        } else {
            let line = p.current_token().line;
            return Err(ParseError::UnexpectedToken {
                found: p.current_token().token_type.clone(),
                expected: "type name after ':'".to_string(),
                line,
            });
        }
    } else {
        let line = p.current_token().line;
        return Err(ParseError::UnexpectedToken {
            found: p.current_token().token_type.clone(),
            expected: ": type".to_string(),
            line,
        });
    }

    // 5) Optional postfix handlers (Stage‑3) directly after block (shared helper)
    final_body = crate::parser::declarations::box_def::members::postfix::wrap_with_optional_postfix(p, final_body)?;

    // 6) Generate methods per kind (fully equivalent to legacy inline branch)
    if kind == "once" {
        // __compute_once_<name>
        let compute_name = format!("__compute_once_{}", name);
        let compute = ASTNode::FunctionDeclaration {
            name: compute_name.clone(),
            params: vec![],
            body: final_body,
            is_static: false,
            is_override: false,
            span: Span::unknown(),
        };
        methods.insert(compute_name.clone(), compute);

        // Getter with cache + poison handling
        let key = format!("__once_{}", name);
        let poison_key = format!("__once_poison_{}", name);
        let cached_local = format!("__ny_cached_{}", name);
        let poison_local = format!("__ny_poison_{}", name);
        let val_local = format!("__ny_val_{}", name);
        let me_node = ASTNode::Me { span: Span::unknown() };
        let get_cached = ASTNode::MethodCall {
            object: Box::new(me_node.clone()),
            method: "getField".to_string(),
            arguments: vec![ASTNode::Literal { value: crate::ast::LiteralValue::String(key.clone()), span: Span::unknown() }],
            span: Span::unknown(),
        };
        let local_cached = ASTNode::Local { variables: vec![cached_local.clone()], initial_values: vec![Some(Box::new(get_cached))], span: Span::unknown() };
        let cond_cached = ASTNode::BinaryOp { operator: crate::ast::BinaryOperator::NotEqual, left: Box::new(ASTNode::Variable { name: cached_local.clone(), span: Span::unknown() }), right: Box::new(ASTNode::Literal { value: crate::ast::LiteralValue::Null, span: Span::unknown() }), span: Span::unknown() };
        let then_ret_cached = vec![ASTNode::Return { value: Some(Box::new(ASTNode::Variable { name: cached_local.clone(), span: Span::unknown() })), span: Span::unknown() }];
        let if_cached = ASTNode::If { condition: Box::new(cond_cached), then_body: then_ret_cached, else_body: None, span: Span::unknown() };

        let get_poison = ASTNode::MethodCall { object: Box::new(me_node.clone()), method: "getField".to_string(), arguments: vec![ASTNode::Literal { value: crate::ast::LiteralValue::String(poison_key.clone()), span: Span::unknown() }], span: Span::unknown() };
        let local_poison = ASTNode::Local { variables: vec![poison_local.clone()], initial_values: vec![Some(Box::new(get_poison))], span: Span::unknown() };
        let cond_poison = ASTNode::BinaryOp { operator: crate::ast::BinaryOperator::NotEqual, left: Box::new(ASTNode::Variable { name: poison_local.clone(), span: Span::unknown() }), right: Box::new(ASTNode::Literal { value: crate::ast::LiteralValue::Null, span: Span::unknown() }), span: Span::unknown() };
        let then_throw = vec![ASTNode::Throw { expression: Box::new(ASTNode::Literal { value: crate::ast::LiteralValue::String(format!("once '{}' previously failed", name)), span: Span::unknown() }), span: Span::unknown() }];
        let if_poison = ASTNode::If { condition: Box::new(cond_poison), then_body: then_throw, else_body: None, span: Span::unknown() };

        let call_compute = ASTNode::MethodCall { object: Box::new(me_node.clone()), method: compute_name.clone(), arguments: vec![], span: Span::unknown() };
        let local_val = ASTNode::Local { variables: vec![val_local.clone()], initial_values: vec![Some(Box::new(call_compute))], span: Span::unknown() };
        let set_call = ASTNode::MethodCall { object: Box::new(me_node.clone()), method: "setField".to_string(), arguments: vec![ASTNode::Literal { value: crate::ast::LiteralValue::String(key.clone()), span: Span::unknown() }, ASTNode::Variable { name: val_local.clone(), span: Span::unknown() }], span: Span::unknown() };
        let ret_stmt = ASTNode::Return { value: Some(Box::new(ASTNode::Variable { name: val_local.clone(), span: Span::unknown() })), span: Span::unknown() };
        let getter_body = vec![local_cached, if_cached, local_poison, if_poison, local_val, set_call, ret_stmt];
        let getter_name = format!("__get_once_{}", name);
        let getter = ASTNode::FunctionDeclaration { name: getter_name.clone(), params: vec![], body: getter_body, is_static: false, is_override: false, span: Span::unknown() };
        methods.insert(getter_name, getter);
        return Ok(true);
    }

    // birth_once
    birth_once_props.push(name.clone());
    let compute_name = format!("__compute_birth_{}", name);
    let compute = ASTNode::FunctionDeclaration { name: compute_name.clone(), params: vec![], body: final_body, is_static: false, is_override: false, span: Span::unknown() };
    methods.insert(compute_name.clone(), compute);
    let me_node = ASTNode::Me { span: Span::unknown() };
    let get_call = ASTNode::MethodCall { object: Box::new(me_node.clone()), method: "getField".to_string(), arguments: vec![ASTNode::Literal { value: crate::ast::LiteralValue::String(format!("__birth_{}", name)), span: Span::unknown() }], span: Span::unknown() };
    let getter_body = vec![ASTNode::Return { value: Some(Box::new(get_call)), span: Span::unknown() }];
    let getter_name = format!("__get_birth_{}", name);
    let getter = ASTNode::FunctionDeclaration { name: getter_name.clone(), params: vec![], body: getter_body, is_static: false, is_override: false, span: Span::unknown() };
    methods.insert(getter_name, getter);
    Ok(true)
}
