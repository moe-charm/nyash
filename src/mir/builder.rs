/*!
 * MIR Builder - Converts AST to MIR/SSA form
 * 
 * Implements AST → MIR conversion with SSA construction
 */

use super::{
    MirInstruction, BasicBlock, BasicBlockId, MirFunction, MirModule, 
    FunctionSignature, ValueId, ConstValue, BinaryOp, UnaryOp, CompareOp,
    MirType, EffectMask, Effect, BasicBlockIdGenerator, ValueIdGenerator
};
use crate::ast::{ASTNode, LiteralValue, BinaryOperator};
use std::collections::HashMap;

/// MIR builder for converting AST to SSA form
pub struct MirBuilder {
    /// Current module being built
    current_module: Option<MirModule>,
    
    /// Current function being built
    current_function: Option<MirFunction>,
    
    /// Current basic block being built
    current_block: Option<BasicBlockId>,
    
    /// Value ID generator
    value_gen: ValueIdGenerator,
    
    /// Basic block ID generator
    block_gen: BasicBlockIdGenerator,
    
    /// Variable name to ValueId mapping (for SSA conversion)
    variable_map: HashMap<String, ValueId>,
    
    /// Pending phi functions to be inserted
    pending_phis: Vec<(BasicBlockId, ValueId, String)>,
}

impl MirBuilder {
    /// Create a new MIR builder
    pub fn new() -> Self {
        Self {
            current_module: None,
            current_function: None,
            current_block: None,
            value_gen: ValueIdGenerator::new(),
            block_gen: BasicBlockIdGenerator::new(),
            variable_map: HashMap::new(),
            pending_phis: Vec::new(),
        }
    }
    
    /// Build a complete MIR module from AST
    pub fn build_module(&mut self, ast: ASTNode) -> Result<MirModule, String> {
        // Create a new module
        let mut module = MirModule::new("main".to_string());
        
        // Create a main function to contain the AST
        let main_signature = FunctionSignature {
            name: "main".to_string(),
            params: vec![],
            return_type: MirType::Void,
            effects: EffectMask::PURE,
        };
        
        let entry_block = self.block_gen.next();
        let mut main_function = MirFunction::new(main_signature, entry_block);
        main_function.metadata.is_entry_point = true;
        
        // Set up building context
        self.current_module = Some(module);
        self.current_function = Some(main_function);
        self.current_block = Some(entry_block);
        
        // Convert AST to MIR
        let result_value = self.build_expression(ast)?;
        
        // Add return instruction if needed
        if let Some(block_id) = self.current_block {
            if let Some(ref mut function) = self.current_function {
                if let Some(block) = function.get_block_mut(block_id) {
                    if !block.is_terminated() {
                        block.add_instruction(MirInstruction::Return {
                            value: Some(result_value),
                        });
                    }
                }
            }
        }
        
        // Finalize and return module
        let mut module = self.current_module.take().unwrap();
        let function = self.current_function.take().unwrap();
        module.add_function(function);
        
        Ok(module)
    }
    
    /// Build an expression and return its value ID
    fn build_expression(&mut self, ast: ASTNode) -> Result<ValueId, String> {
        match ast {
            ASTNode::Literal { value, .. } => {
                self.build_literal(value)
            },
            
            ASTNode::BinaryOp { left, operator, right, .. } => {
                self.build_binary_op(*left, operator, *right)
            },
            
            ASTNode::UnaryOp { operator, operand, .. } => {
                let op_string = match operator {
                    crate::ast::UnaryOperator::Minus => "-".to_string(),
                    crate::ast::UnaryOperator::Not => "not".to_string(),
                };
                self.build_unary_op(op_string, *operand)
            },
            
            ASTNode::Variable { name, .. } => {
                self.build_variable_access(name.clone())
            },
            
            ASTNode::Assignment { target, value, .. } => {
                // For now, assume target is a variable identifier
                if let ASTNode::Variable { name, .. } = target.as_ref() {
                    self.build_assignment(name.clone(), *value.clone())
                } else {
                    Err("Complex assignment targets not yet supported in MIR".to_string())
                }
            },
            
            ASTNode::FunctionCall { name, arguments, .. } => {
                self.build_function_call(name.clone(), arguments.clone())
            },
            
            ASTNode::Print { expression, .. } => {
                self.build_print_statement(*expression.clone())
            },
            
            ASTNode::Program { statements, .. } => {
                self.build_block(statements.clone())
            },
            
            ASTNode::If { condition, then_body, else_body, .. } => {
                let else_ast = if let Some(else_statements) = else_body {
                    Some(ASTNode::Program {
                        statements: else_statements.clone(),
                        span: crate::ast::Span::unknown(),
                    })
                } else {
                    None
                };
                
                self.build_if_statement(
                    *condition.clone(), 
                    ASTNode::Program {
                        statements: then_body.clone(),
                        span: crate::ast::Span::unknown(),
                    },
                    else_ast
                )
            },
            
            _ => {
                Err(format!("Unsupported AST node type: {:?}", ast))
            }
        }
    }
    
    /// Build a literal value
    fn build_literal(&mut self, literal: LiteralValue) -> Result<ValueId, String> {
        let const_value = match literal {
            LiteralValue::Integer(n) => ConstValue::Integer(n),
            LiteralValue::Float(f) => ConstValue::Float(f),
            LiteralValue::String(s) => ConstValue::String(s),
            LiteralValue::Bool(b) => ConstValue::Bool(b),
            LiteralValue::Void => ConstValue::Void,
        };
        
        let dst = self.value_gen.next();
        self.emit_instruction(MirInstruction::Const {
            dst,
            value: const_value,
        })?;
        
        Ok(dst)
    }
    
    /// Build a binary operation
    fn build_binary_op(&mut self, left: ASTNode, operator: BinaryOperator, right: ASTNode) -> Result<ValueId, String> {
        let lhs = self.build_expression(left)?;
        let rhs = self.build_expression(right)?;
        let dst = self.value_gen.next();
        
        let mir_op = self.convert_binary_operator(operator)?;
        
        match mir_op {
            // Arithmetic operations
            BinaryOpType::Arithmetic(op) => {
                self.emit_instruction(MirInstruction::BinOp {
                    dst, op, lhs, rhs
                })?;
            },
            
            // Comparison operations
            BinaryOpType::Comparison(op) => {
                self.emit_instruction(MirInstruction::Compare {
                    dst, op, lhs, rhs
                })?;
            },
        }
        
        Ok(dst)
    }
    
    /// Build a unary operation
    fn build_unary_op(&mut self, operator: String, operand: ASTNode) -> Result<ValueId, String> {
        let operand_val = self.build_expression(operand)?;
        let dst = self.value_gen.next();
        
        let mir_op = self.convert_unary_operator(operator)?;
        
        self.emit_instruction(MirInstruction::UnaryOp {
            dst,
            op: mir_op,
            operand: operand_val,
        })?;
        
        Ok(dst)
    }
    
    /// Build variable access
    fn build_variable_access(&mut self, name: String) -> Result<ValueId, String> {
        if let Some(&value_id) = self.variable_map.get(&name) {
            Ok(value_id)
        } else {
            Err(format!("Undefined variable: {}", name))
        }
    }
    
    /// Build assignment
    fn build_assignment(&mut self, var_name: String, value: ASTNode) -> Result<ValueId, String> {
        let value_id = self.build_expression(value)?;
        
        // In SSA form, each assignment creates a new value
        self.variable_map.insert(var_name, value_id);
        
        Ok(value_id)
    }
    
    /// Build function call
    fn build_function_call(&mut self, name: String, args: Vec<ASTNode>) -> Result<ValueId, String> {
        // Build argument values
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.build_expression(arg)?);
        }
        
        let dst = self.value_gen.next();
        
        // For now, treat all function calls as Box method calls
        if arg_values.is_empty() {
            return Err("Function calls require at least one argument (the object)".to_string());
        }
        
        let box_val = arg_values.remove(0);
        
        self.emit_instruction(MirInstruction::BoxCall {
            dst: Some(dst),
            box_val,
            method: name,
            args: arg_values,
            effects: EffectMask::PURE.add(Effect::ReadHeap), // Conservative default
        })?;
        
        Ok(dst)
    }
    
    /// Build print statement - converts to console output
    fn build_print_statement(&mut self, expression: ASTNode) -> Result<ValueId, String> {
        let value = self.build_expression(expression)?;
        
        // For now, use a special Print instruction (minimal scope)
        self.emit_instruction(MirInstruction::Print {
            value,
            effects: EffectMask::PURE.add(Effect::IO),
        })?;
        
        // Return the value that was printed
        Ok(value)
    }
    
    /// Build a block of statements
    fn build_block(&mut self, statements: Vec<ASTNode>) -> Result<ValueId, String> {
        let mut last_value = None;
        
        for statement in statements {
            last_value = Some(self.build_expression(statement)?);
        }
        
        // Return last value or void
        Ok(last_value.unwrap_or_else(|| {
            let void_val = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const {
                dst: void_val,
                value: ConstValue::Void,
            }).unwrap();
            void_val
        }))
    }
    
    /// Build if statement with conditional branches
    fn build_if_statement(&mut self, condition: ASTNode, then_branch: ASTNode, else_branch: Option<ASTNode>) -> Result<ValueId, String> {
        let condition_val = self.build_expression(condition)?;
        
        // Create basic blocks for then/else/merge
        let then_block = self.block_gen.next();
        let else_block = self.block_gen.next();
        let merge_block = self.block_gen.next();
        
        // Emit branch instruction in current block
        self.emit_instruction(MirInstruction::Branch {
            condition: condition_val,
            then_bb: then_block,
            else_bb: else_block,
        })?;
        
        // Build then branch
        self.current_block = Some(then_block);
        self.ensure_block_exists(then_block)?;
        let then_value = self.build_expression(then_branch)?;
        self.emit_instruction(MirInstruction::Jump { target: merge_block })?;
        
        // Build else branch
        self.current_block = Some(else_block);
        self.ensure_block_exists(else_block)?;
        let else_value = if let Some(else_ast) = else_branch {
            self.build_expression(else_ast)?
        } else {
            // No else branch, use void
            let void_val = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const {
                dst: void_val,
                value: ConstValue::Void,
            })?;
            void_val
        };
        self.emit_instruction(MirInstruction::Jump { target: merge_block })?;
        
        // Create merge block with phi function
        self.current_block = Some(merge_block);
        self.ensure_block_exists(merge_block)?;
        let result_val = self.value_gen.next();
        
        self.emit_instruction(MirInstruction::Phi {
            dst: result_val,
            inputs: vec![
                (then_block, then_value),
                (else_block, else_value),
            ],
        })?;
        
        Ok(result_val)
    }
    
    /// Emit an instruction to the current basic block
    fn emit_instruction(&mut self, instruction: MirInstruction) -> Result<(), String> {
        let block_id = self.current_block.ok_or("No current basic block")?;
        
        if let Some(ref mut function) = self.current_function {
            if let Some(block) = function.get_block_mut(block_id) {
                block.add_instruction(instruction);
                Ok(())
            } else {
                Err(format!("Basic block {} does not exist", block_id))
            }
        } else {
            Err("No current function".to_string())
        }
    }
    
    /// Ensure a basic block exists in the current function
    fn ensure_block_exists(&mut self, block_id: BasicBlockId) -> Result<(), String> {
        if let Some(ref mut function) = self.current_function {
            if !function.blocks.contains_key(&block_id) {
                let block = BasicBlock::new(block_id);
                function.add_block(block);
            }
            Ok(())
        } else {
            Err("No current function".to_string())
        }
    }
    
    /// Convert AST binary operator to MIR operator
    fn convert_binary_operator(&self, op: BinaryOperator) -> Result<BinaryOpType, String> {
        match op {
            BinaryOperator::Add => Ok(BinaryOpType::Arithmetic(BinaryOp::Add)),
            BinaryOperator::Subtract => Ok(BinaryOpType::Arithmetic(BinaryOp::Sub)),
            BinaryOperator::Multiply => Ok(BinaryOpType::Arithmetic(BinaryOp::Mul)),
            BinaryOperator::Divide => Ok(BinaryOpType::Arithmetic(BinaryOp::Div)),
            BinaryOperator::Equal => Ok(BinaryOpType::Comparison(CompareOp::Eq)),
            BinaryOperator::NotEqual => Ok(BinaryOpType::Comparison(CompareOp::Ne)),
            BinaryOperator::Less => Ok(BinaryOpType::Comparison(CompareOp::Lt)),
            BinaryOperator::LessEqual => Ok(BinaryOpType::Comparison(CompareOp::Le)),
            BinaryOperator::Greater => Ok(BinaryOpType::Comparison(CompareOp::Gt)),
            BinaryOperator::GreaterEqual => Ok(BinaryOpType::Comparison(CompareOp::Ge)),
            BinaryOperator::And => Ok(BinaryOpType::Arithmetic(BinaryOp::And)),
            BinaryOperator::Or => Ok(BinaryOpType::Arithmetic(BinaryOp::Or)),
        }
    }
    
    /// Convert AST unary operator to MIR operator
    fn convert_unary_operator(&self, op: String) -> Result<UnaryOp, String> {
        match op.as_str() {
            "-" => Ok(UnaryOp::Neg),
            "!" | "not" => Ok(UnaryOp::Not),
            "~" => Ok(UnaryOp::BitNot),
            _ => Err(format!("Unsupported unary operator: {}", op)),
        }
    }
}

/// Helper enum for binary operator classification
#[derive(Debug)]
enum BinaryOpType {
    Arithmetic(BinaryOp),
    Comparison(CompareOp),
}

impl Default for MirBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ASTNode, LiteralValue, Span};
    
    #[test]
    fn test_literal_building() {
        let mut builder = MirBuilder::new();
        
        let ast = ASTNode::Literal {
            value: LiteralValue::Integer(42),
            span: Span::unknown(),
        };
        
        let result = builder.build_module(ast);
        assert!(result.is_ok());
        
        let module = result.unwrap();
        assert_eq!(module.function_names().len(), 1);
        assert!(module.get_function("main").is_some());
    }
    
    #[test]
    fn test_binary_op_building() {
        let mut builder = MirBuilder::new();
        
        let ast = ASTNode::BinaryOp {
            left: Box::new(ASTNode::Literal {
                value: LiteralValue::Integer(10),
                span: Span::unknown(),
            }),
            operator: BinaryOperator::Add,
            right: Box::new(ASTNode::Literal {
                value: LiteralValue::Integer(32),
                span: Span::unknown(),
            }),
            span: Span::unknown(),
        };
        
        let result = builder.build_module(ast);
        assert!(result.is_ok());
        
        let module = result.unwrap();
        let function = module.get_function("main").unwrap();
        
        // Should have constants and binary operation
        let stats = function.stats();
        assert!(stats.instruction_count >= 3); // 2 constants + 1 binop + 1 return
    }
    
    #[test]
    fn test_if_statement_building() {
        let mut builder = MirBuilder::new();
        
        // Adapt test to current AST: If with statement bodies
        let ast = ASTNode::If {
            condition: Box::new(ASTNode::Literal {
                value: LiteralValue::Bool(true),
                span: Span::unknown(),
            }),
            then_body: vec![ASTNode::Literal {
                value: LiteralValue::Integer(1),
                span: Span::unknown(),
            }],
            else_body: Some(vec![ASTNode::Literal {
                value: LiteralValue::Integer(2),
                span: Span::unknown(),
            }]),
            span: Span::unknown(),
        };
        
        let result = builder.build_module(ast);
        assert!(result.is_ok());
        
        let module = result.unwrap();
        let function = module.get_function("main").unwrap();
        
        // Should have multiple blocks for if/then/else/merge
        assert!(function.blocks.len() >= 3);
        
        // Should have phi function in merge block
        let stats = function.stats();
        assert!(stats.phi_count >= 1);
    }
}
