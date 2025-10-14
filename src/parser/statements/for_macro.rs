/*!
 * @for macro (parser-level syntactic sugar)
 *
 * Syntax (MVP):
 *   @for (x in expr) { body }
 *
 * Lowering (arrays only):
 *   {
 *     local __ny_seq = <expr>;
 *     local __ny_i = 0;
 *     loop(__ny_i < __ny_seq.length()) {
 *       local x = __ny_seq.get(__ny_i);
 *       <body>
 *       __ny_i = __ny_i + 1;
 *     }
 *   }
 */

use crate::ast::{ASTNode, LiteralValue, Span};
use crate::ast::BinaryOperator;
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;

impl NyashParser {
    pub(super) fn parse_for_macro(&mut self) -> Result<ASTNode, ParseError> {
        // Current token is '@'
        self.advance(); // consume '@'
        match &self.current_token().token_type {
            TokenType::IDENTIFIER(s) if s == "for" => { self.advance(); }
            other => {
                return Err(ParseError::UnexpectedToken { found: other.clone(), expected: "'for'".to_string(), line: self.current_token().line });
            }
        }
        self.consume(TokenType::LPAREN)?;
        // Parse loop variable(s)
        let var1 = match &self.current_token().token_type {
            TokenType::IDENTIFIER(s) => { let v = s.clone(); self.advance(); v }
            other => { return Err(ParseError::UnexpectedToken { found: other.clone(), expected: "identifier".to_string(), line: self.current_token().line }); }
        };
        let mut var2: Option<String> = None;
        if self.match_token(&TokenType::COMMA) {
            self.advance(); // consume ','
            var2 = match &self.current_token().token_type {
                TokenType::IDENTIFIER(s) => { let v = s.clone(); self.advance(); Some(v) }
                other => { return Err(ParseError::UnexpectedToken { found: other.clone(), expected: "identifier".to_string(), line: self.current_token().line }); }
            };
        }
        // Expect 'in'
        match &self.current_token().token_type {
            TokenType::IDENTIFIER(s) if s == "in" => { self.advance(); }
            other => { return Err(ParseError::UnexpectedToken { found: other.clone(), expected: "'in'".to_string(), line: self.current_token().line }); }
        }
        // Parse sequence or range expression
        let left_expr = self.parse_expression()?;
        // Two ways to represent range:
        // 1) Token-level '..' still pending (RANGE token follows)
        // 2) Expression already lowered to FunctionCall("Range", [start, end]) by expr_parse_range
        let mut is_range = self.match_token(&TokenType::RANGE);
        let mut left_expr_norm = left_expr.clone();
        let mut right_expr_opt: Option<ASTNode> = None;
        if is_range {
            self.advance();
            right_expr_opt = Some(self.parse_expression()?);
        } else {
            if let ASTNode::FunctionCall { name, arguments, .. } = &left_expr_norm {
                if name == "Range" && arguments.len() == 2 {
                    // Normalize: treat as range(start..end)
                    is_range = true;
                    // Extract and move arguments out by cloning (AST is cheap here)
                    if let [a, b] = &arguments[..] {
                        let a2 = (*a).clone();
                        let b2 = (*b).clone();
                        left_expr_norm = a2;
                        right_expr_opt = Some(b2);
                    }
                }
            }
        }
        self.consume(TokenType::RPAREN)?;
        // Parse loop body block
        let loop_body_user = self.parse_block_statements()?;

        if is_range {
            if var2.is_some() {
                return Err(ParseError::UnexpectedToken { found: TokenType::COMMA, expected: "single loop variable for range".into(), line: self.current_token().line });
            }
            // Range lowering: start..end (exclusive)
            let start_expr = left_expr_norm;
            let end_expr = right_expr_opt.expect("range end expr");
            let local_start = ASTNode::Local { variables: vec!["__ny_start".into()], initial_values: vec![Some(Box::new(start_expr))], span: Span::unknown() };
            let local_end = ASTNode::Local { variables: vec!["__ny_end".into()], initial_values: vec![Some(Box::new(end_expr))], span: Span::unknown() };
            let local_i = ASTNode::Local { variables: vec!["__ny_i".into()], initial_values: vec![Some(Box::new(ASTNode::Variable { name: "__ny_start".into(), span: Span::unknown() }))], span: Span::unknown() };
            let cond = ASTNode::BinaryOp { operator: BinaryOperator::Less, left: Box::new(ASTNode::Variable { name: "__ny_i".into(), span: Span::unknown() }), right: Box::new(ASTNode::Variable { name: "__ny_end".into(), span: Span::unknown() }), span: Span::unknown() };
            let local_it = ASTNode::Local { variables: vec![var1.clone()], initial_values: vec![Some(Box::new(ASTNode::Variable { name: "__ny_i".into(), span: Span::unknown() }))], span: Span::unknown() };
            let inc = ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "__ny_i".into(), span: Span::unknown() }), value: Box::new(ASTNode::BinaryOp { operator: BinaryOperator::Add, left: Box::new(ASTNode::Variable { name: "__ny_i".into(), span: Span::unknown() }), right: Box::new(ASTNode::Literal { value: LiteralValue::Integer(1), span: Span::unknown() }), span: Span::unknown() }), span: Span::unknown() };
            let mut loop_body = Vec::<ASTNode>::new();
            loop_body.push(local_it);
            loop_body.extend(loop_body_user);
            loop_body.push(inc);
            let loop_stmt = ASTNode::Loop { condition: Box::new(cond), body: loop_body, span: Span::unknown() };
            Ok(ASTNode::ScopeBox { body: vec![local_start, local_end, local_i, loop_stmt], span: Span::unknown() })
        } else if let Some(v2) = var2 {
            // Map pair lowering: for (k, v in map)
            let local_map = ASTNode::Local { variables: vec!["__ny_map".into()], initial_values: vec![Some(Box::new(left_expr))], span: Span::unknown() };
            let local_keys = ASTNode::Local { variables: vec!["__ny_keys".into()], initial_values: vec![Some(Box::new(ASTNode::MethodCall { object: Box::new(ASTNode::Variable { name: "__ny_map".into(), span: Span::unknown() }), method: "keys".into(), arguments: vec![], span: Span::unknown() }))], span: Span::unknown() };
            let local_i = ASTNode::Local { variables: vec!["__ny_i".into()], initial_values: vec![Some(Box::new(ASTNode::Literal { value: LiteralValue::Integer(0), span: Span::unknown() }))], span: Span::unknown() };
            let cond = ASTNode::BinaryOp { operator: BinaryOperator::Less, left: Box::new(ASTNode::Variable { name: "__ny_i".into(), span: Span::unknown() }), right: Box::new(ASTNode::MethodCall { object: Box::new(ASTNode::Variable { name: "__ny_keys".into(), span: Span::unknown() }), method: "length".into(), arguments: vec![], span: Span::unknown() }), span: Span::unknown() };
            let local_k = ASTNode::Local { variables: vec![var1.clone()], initial_values: vec![Some(Box::new(ASTNode::MethodCall { object: Box::new(ASTNode::Variable { name: "__ny_keys".into(), span: Span::unknown() }), method: "get".into(), arguments: vec![ASTNode::Variable { name: "__ny_i".into(), span: Span::unknown() }], span: Span::unknown() }))], span: Span::unknown() };
            let local_v = ASTNode::Local { variables: vec![v2.clone()], initial_values: vec![Some(Box::new(ASTNode::MethodCall { object: Box::new(ASTNode::Variable { name: "__ny_map".into(), span: Span::unknown() }), method: "get".into(), arguments: vec![ASTNode::Variable { name: var1.clone(), span: Span::unknown() }], span: Span::unknown() }))], span: Span::unknown() };
            let inc = ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "__ny_i".into(), span: Span::unknown() }), value: Box::new(ASTNode::BinaryOp { operator: BinaryOperator::Add, left: Box::new(ASTNode::Variable { name: "__ny_i".into(), span: Span::unknown() }), right: Box::new(ASTNode::Literal { value: LiteralValue::Integer(1), span: Span::unknown() }), span: Span::unknown() }), span: Span::unknown() };
            let mut loop_body = Vec::<ASTNode>::new();
            loop_body.push(local_k);
            loop_body.push(local_v);
            loop_body.extend(loop_body_user);
            loop_body.push(inc);
            let loop_stmt = ASTNode::Loop { condition: Box::new(cond), body: loop_body, span: Span::unknown() };
            Ok(ASTNode::ScopeBox { body: vec![local_map, local_keys, local_i, loop_stmt], span: Span::unknown() })
        } else {
            // Array single-var lowering (existing)
            let local_seq = ASTNode::Local { variables: vec!["__ny_seq".to_string()], initial_values: vec![Some(Box::new(left_expr))], span: Span::unknown() };
            let local_i = ASTNode::Local { variables: vec!["__ny_i".to_string()], initial_values: vec![Some(Box::new(ASTNode::Literal { value: LiteralValue::Integer(0), span: Span::unknown() }))], span: Span::unknown() };
            let cond = ASTNode::BinaryOp { operator: BinaryOperator::Less, left: Box::new(ASTNode::Variable { name: "__ny_i".into(), span: Span::unknown() }), right: Box::new(ASTNode::MethodCall { object: Box::new(ASTNode::Variable { name: "__ny_seq".into(), span: Span::unknown() }), method: "length".to_string(), arguments: vec![], span: Span::unknown() }), span: Span::unknown() };
            let local_x = ASTNode::Local { variables: vec![var1.clone()], initial_values: vec![Some(Box::new(ASTNode::MethodCall { object: Box::new(ASTNode::Variable { name: "__ny_seq".into(), span: Span::unknown() }), method: "get".to_string(), arguments: vec![ASTNode::Variable { name: "__ny_i".into(), span: Span::unknown() }], span: Span::unknown() }))], span: Span::unknown() };
            let inc = ASTNode::Assignment { target: Box::new(ASTNode::Variable { name: "__ny_i".into(), span: Span::unknown() }), value: Box::new(ASTNode::BinaryOp { operator: BinaryOperator::Add, left: Box::new(ASTNode::Variable { name: "__ny_i".into(), span: Span::unknown() }), right: Box::new(ASTNode::Literal { value: LiteralValue::Integer(1), span: Span::unknown() }), span: Span::unknown() }), span: Span::unknown() };
            let mut loop_body = Vec::<ASTNode>::new();
            loop_body.push(local_x);
            loop_body.extend(loop_body_user);
            loop_body.push(inc);
            let loop_stmt = ASTNode::Loop { condition: Box::new(cond), body: loop_body, span: Span::unknown() };
            Ok(ASTNode::ScopeBox { body: vec![local_seq, local_i, loop_stmt], span: Span::unknown() })
        }
    }
}
