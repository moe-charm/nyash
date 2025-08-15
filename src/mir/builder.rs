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
        
        // Add safepoint at function entry
        self.emit_instruction(MirInstruction::Safepoint)?;
        
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
            
            ASTNode::Me { .. } => {
                self.build_me_expression()
            },
            
            ASTNode::MethodCall { object, method, arguments, .. } => {
                self.build_method_call(*object.clone(), method.clone(), arguments.clone())
            },
            
            ASTNode::FromCall { parent, method, arguments, .. } => {
                self.build_from_expression(parent.clone(), method.clone(), arguments.clone())
            },
            
            ASTNode::Assignment { target, value, .. } => {
                // Check if target is a field access for RefSet
                if let ASTNode::FieldAccess { object, field, .. } = target.as_ref() {
                    self.build_field_assignment(*object.clone(), field.clone(), *value.clone())
                } else if let ASTNode::Variable { name, .. } = target.as_ref() {
                    // Plain variable assignment - existing behavior
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
            
            ASTNode::Loop { condition, body, .. } => {
                self.build_loop_statement(*condition.clone(), body.clone())
            },
            
            ASTNode::TryCatch { try_body, catch_clauses, finally_body, .. } => {
                self.build_try_catch_statement(try_body.clone(), catch_clauses.clone(), finally_body.clone())
            },
            
            ASTNode::Throw { expression, .. } => {
                self.build_throw_statement(*expression.clone())
            },
            
            ASTNode::Return { value, .. } => {
                self.build_return_statement(value.clone())
            },
            
            ASTNode::Local { variables, initial_values, .. } => {
                self.build_local_statement(variables.clone(), initial_values.clone())
            },
            
            ASTNode::BoxDeclaration { name, methods, is_static, fields, .. } => {
                if is_static && name == "Main" {
                    self.build_static_main_box(methods.clone())
                } else {
                    // Support user-defined boxes - handle as statement, return void
                    self.build_box_declaration(name.clone(), methods.clone(), fields.clone())?;
                    
                    // Return a void value since this is a statement
                    let void_val = self.value_gen.next();
                    self.emit_instruction(MirInstruction::Const {
                        dst: void_val,
                        value: ConstValue::Void,
                    })?;
                    Ok(void_val)
                }
            },
            
            ASTNode::FieldAccess { object, field, .. } => {
                self.build_field_access(*object.clone(), field.clone())
            },
            
            ASTNode::New { class, arguments, .. } => {
                self.build_new_expression(class.clone(), arguments.clone())
            },
            
            // Phase 7: Async operations
            ASTNode::Nowait { variable, expression, .. } => {
                self.build_nowait_statement(variable.clone(), *expression.clone())
            },
            
            ASTNode::AwaitExpression { expression, .. } => {
                self.build_await_expression(*expression.clone())
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
            effects: EffectMask::PURE.add(Effect::Io),
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
        if !self.is_current_block_terminated() {
            self.emit_instruction(MirInstruction::Jump { target: merge_block })?;
        }
        
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
        if !self.is_current_block_terminated() {
            self.emit_instruction(MirInstruction::Jump { target: merge_block })?;
        }
        
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
    
    /// Build a loop statement: loop(condition) { body }
    fn build_loop_statement(&mut self, condition: ASTNode, body: Vec<ASTNode>) -> Result<ValueId, String> {
        // Add safepoint at loop entry
        self.emit_instruction(MirInstruction::Safepoint)?;
        
        let loop_header = self.block_gen.next();
        let loop_body = self.block_gen.next();
        let loop_exit = self.block_gen.next();
        
        // Jump to loop header
        self.emit_instruction(MirInstruction::Jump { target: loop_header })?;
        
        // Create loop header block
        self.start_new_block(loop_header)?;
        
        // Evaluate condition
        let condition_value = self.build_expression(condition)?;
        
        // Branch based on condition
        self.emit_instruction(MirInstruction::Branch {
            condition: condition_value,
            then_bb: loop_body,
            else_bb: loop_exit,
        })?;
        
        // Create loop body block
        self.start_new_block(loop_body)?;
        
        // Add safepoint at loop body start
        self.emit_instruction(MirInstruction::Safepoint)?;
        
        // Build loop body
        let body_ast = ASTNode::Program {
            statements: body,
            span: crate::ast::Span::unknown(),
        };
        self.build_expression(body_ast)?;
        
        // Jump back to loop header
        self.emit_instruction(MirInstruction::Jump { target: loop_header })?;
        
        // Create exit block
        self.start_new_block(loop_exit)?;
        
        // Return void value
        let void_dst = self.value_gen.next();
        self.emit_instruction(MirInstruction::Const {
            dst: void_dst,
            value: ConstValue::Void,
        })?;
        
        Ok(void_dst)
    }
    
    /// Build a try/catch statement
    fn build_try_catch_statement(&mut self, try_body: Vec<ASTNode>, catch_clauses: Vec<crate::ast::CatchClause>, finally_body: Option<Vec<ASTNode>>) -> Result<ValueId, String> {
        let try_block = self.block_gen.next();
        let catch_block = self.block_gen.next();
        let finally_block = if finally_body.is_some() { Some(self.block_gen.next()) } else { None };
        let exit_block = self.block_gen.next();
        
        // Set up exception handler for the try block (before we enter it)
        if let Some(catch_clause) = catch_clauses.first() {
            let exception_value = self.value_gen.next();
            
            // Register catch handler for exceptions that may occur in try block
            self.emit_instruction(MirInstruction::Catch {
                exception_type: catch_clause.exception_type.clone(),
                exception_value,
                handler_bb: catch_block,
            })?;
        }
        
        // Jump to try block
        self.emit_instruction(MirInstruction::Jump { target: try_block })?;
        
        // Build try block
        self.start_new_block(try_block)?;
        
        let try_ast = ASTNode::Program {
            statements: try_body,
            span: crate::ast::Span::unknown(),
        };
        let _try_result = self.build_expression(try_ast)?;
        
        // Normal completion of try block - jump to finally or exit (if not already terminated)
        if !self.is_current_block_terminated() {
            let next_target = finally_block.unwrap_or(exit_block);
            self.emit_instruction(MirInstruction::Jump { target: next_target })?;
        }
        
        // Build catch block (reachable via exception handling)
        self.start_new_block(catch_block)?;
        
        // Handle catch clause
        if let Some(catch_clause) = catch_clauses.first() {
            // Build catch body
            let catch_ast = ASTNode::Program {
                statements: catch_clause.body.clone(),
                span: crate::ast::Span::unknown(),
            };
            self.build_expression(catch_ast)?;
        }
        
        // Catch completion - jump to finally or exit (if not already terminated)
        if !self.is_current_block_terminated() {
            let next_target = finally_block.unwrap_or(exit_block);
            self.emit_instruction(MirInstruction::Jump { target: next_target })?;
        }
        
        // Build finally block if present
        if let (Some(finally_block_id), Some(finally_statements)) = (finally_block, finally_body) {
            self.start_new_block(finally_block_id)?;
            
            let finally_ast = ASTNode::Program {
                statements: finally_statements,
                span: crate::ast::Span::unknown(),
            };
            self.build_expression(finally_ast)?;
            
            self.emit_instruction(MirInstruction::Jump { target: exit_block })?;
        }
        
        // Create exit block
        self.start_new_block(exit_block)?;
        
        // Return void for now (in a complete implementation, would use phi for try/catch values)
        let result = self.value_gen.next();
        self.emit_instruction(MirInstruction::Const {
            dst: result,
            value: ConstValue::Void,
        })?;
        
        Ok(result)
    }
    
    /// Build a throw statement
    fn build_throw_statement(&mut self, expression: ASTNode) -> Result<ValueId, String> {
        let exception_value = self.build_expression(expression)?;
        
        // Emit throw instruction with PANIC effect (this is a terminator)
        self.emit_instruction(MirInstruction::Throw {
            exception: exception_value,
            effects: EffectMask::PANIC,
        })?;
        
        // Throw doesn't return normally, but we need to return a value for the type system
        // We can't add more instructions after throw, so just return the exception value
        Ok(exception_value)
    }
    
    /// Build local variable declarations with optional initial values
    fn build_local_statement(&mut self, variables: Vec<String>, initial_values: Vec<Option<Box<ASTNode>>>) -> Result<ValueId, String> {
        let mut last_value = None;
        
        // Process each variable declaration
        for (i, var_name) in variables.iter().enumerate() {
            let value_id = if i < initial_values.len() && initial_values[i].is_some() {
                // Variable has initial value - evaluate it
                let init_expr = initial_values[i].as_ref().unwrap();
                self.build_expression(*init_expr.clone())?
            } else {
                // No initial value - assign void (uninitialized)
                let void_dst = self.value_gen.next();
                self.emit_instruction(MirInstruction::Const {
                    dst: void_dst,
                    value: ConstValue::Void,
                })?;
                void_dst
            };
            
            // Register variable in SSA form
            self.variable_map.insert(var_name.clone(), value_id);
            last_value = Some(value_id);
        }
        
        // Return the last assigned value, or void if no variables
        Ok(last_value.unwrap_or_else(|| {
            let void_val = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const {
                dst: void_val,
                value: ConstValue::Void,
            }).unwrap();
            void_val
        }))
    }
    
    /// Build return statement
    fn build_return_statement(&mut self, value: Option<Box<ASTNode>>) -> Result<ValueId, String> {
        let return_value = if let Some(expr) = value {
            self.build_expression(*expr)?
        } else {
            // Return void if no value specified
            let void_dst = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const {
                dst: void_dst,
                value: ConstValue::Void,
            })?;
            void_dst
        };
        
        // Emit return instruction
        self.emit_instruction(MirInstruction::Return {
            value: Some(return_value),
        })?;
        
        Ok(return_value)
    }
    
    /// Build static box Main - extracts main() method body and converts to Program
    fn build_static_main_box(&mut self, methods: std::collections::HashMap<String, ASTNode>) -> Result<ValueId, String> {
        // Look for the main() method
        if let Some(main_method) = methods.get("main") {
            if let ASTNode::FunctionDeclaration { body, .. } = main_method {
                // Convert the method body to a Program AST node and lower it
                let program_ast = ASTNode::Program {
                    statements: body.clone(),
                    span: crate::ast::Span::unknown(),
                };
                
                // Use existing Program lowering logic
                self.build_expression(program_ast)
            } else {
                Err("main method in static box Main is not a FunctionDeclaration".to_string())
            }
        } else {
            Err("static box Main must contain a main() method".to_string())
        }
    }
    
    /// Build field access: object.field
    fn build_field_access(&mut self, object: ASTNode, field: String) -> Result<ValueId, String> {
        // First, build the object expression to get its ValueId
        let object_value = self.build_expression(object)?;
        
        // Get the field from the object using RefGet
        let result_id = self.value_gen.next();
        self.emit_instruction(MirInstruction::RefGet {
            dst: result_id,
            reference: object_value,
            field,
        })?;
        
        Ok(result_id)
    }
    
    /// Build new expression: new ClassName(arguments)
    fn build_new_expression(&mut self, class: String, arguments: Vec<ASTNode>) -> Result<ValueId, String> {
        // For Phase 6.1, we'll create a simple RefNew without processing arguments
        // In a full implementation, arguments would be used for constructor calls
        let dst = self.value_gen.next();
        
        // For now, create a "box type" value representing the class
        let type_value = self.value_gen.next();
        self.emit_instruction(MirInstruction::Const {
            dst: type_value,
            value: ConstValue::String(class),
        })?;
        
        // Create the reference using RefNew
        self.emit_instruction(MirInstruction::RefNew {
            dst,
            box_val: type_value,
        })?;
        
        Ok(dst)
    }
    
    /// Build field assignment: object.field = value
    fn build_field_assignment(&mut self, object: ASTNode, field: String, value: ASTNode) -> Result<ValueId, String> {
        // Build the object and value expressions
        let object_value = self.build_expression(object)?;
        let value_result = self.build_expression(value)?;
        
        // Set the field using RefSet
        self.emit_instruction(MirInstruction::RefSet {
            reference: object_value,
            field,
            value: value_result,
        })?;
        
        // Return the assigned value
        Ok(value_result)
    }
    
    /// Start a new basic block
    fn start_new_block(&mut self, block_id: BasicBlockId) -> Result<(), String> {
        if let Some(ref mut function) = self.current_function {
            function.add_block(BasicBlock::new(block_id));
            self.current_block = Some(block_id);
            Ok(())
        } else {
            Err("No current function".to_string())
        }
    }
    
    /// Check if the current basic block is terminated
    fn is_current_block_terminated(&self) -> bool {
        if let (Some(block_id), Some(ref function)) = (self.current_block, &self.current_function) {
            if let Some(block) = function.get_block(block_id) {
                return block.is_terminated();
            }
        }
        false
    }
    
    /// Convert AST binary operator to MIR operator
    fn convert_binary_operator(&self, op: BinaryOperator) -> Result<BinaryOpType, String> {
        match op {
            BinaryOperator::Add => Ok(BinaryOpType::Arithmetic(BinaryOp::Add)),
            BinaryOperator::Subtract => Ok(BinaryOpType::Arithmetic(BinaryOp::Sub)),
            BinaryOperator::Multiply => Ok(BinaryOpType::Arithmetic(BinaryOp::Mul)),
            BinaryOperator::Divide => Ok(BinaryOpType::Arithmetic(BinaryOp::Div)),
            BinaryOperator::Modulo => Ok(BinaryOpType::Arithmetic(BinaryOp::Mod)),
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
    
    /// Build nowait statement: nowait variable = expression
    fn build_nowait_statement(&mut self, variable: String, expression: ASTNode) -> Result<ValueId, String> {
        // Evaluate the expression
        let expression_value = self.build_expression(expression)?;
        
        // Create a new Future with the evaluated expression as the initial value
        let future_id = self.value_gen.next();
        self.emit_instruction(MirInstruction::FutureNew {
            dst: future_id,
            value: expression_value,
        })?;
        
        // Store the future in the variable
        self.variable_map.insert(variable.clone(), future_id);
        
        Ok(future_id)
    }
    
    /// Build await expression: await expression
    fn build_await_expression(&mut self, expression: ASTNode) -> Result<ValueId, String> {
        // Evaluate the expression (should be a Future)
        let future_value = self.build_expression(expression)?;
        
        // Create destination for await result
        let result_id = self.value_gen.next();
        
        // Emit await instruction
        self.emit_instruction(MirInstruction::Await {
            dst: result_id,
            future: future_value,
        })?;
        
        Ok(result_id)
    }
    
    /// Build me expression: me
    fn build_me_expression(&mut self) -> Result<ValueId, String> {
        // For now, return a reference to the current instance
        // In a full implementation, this would resolve to the actual instance reference
        let me_value = self.value_gen.next();
        
        // For simplicity, emit a constant representing "me"
        // In practice, this should resolve to the current instance context
        self.emit_instruction(MirInstruction::Const {
            dst: me_value,
            value: ConstValue::String("__me__".to_string()),
        })?;
        
        Ok(me_value)
    }
    
    /// Build method call: object.method(arguments)
    fn build_method_call(&mut self, object: ASTNode, method: String, arguments: Vec<ASTNode>) -> Result<ValueId, String> {
        // Build the object expression
        let object_value = self.build_expression(object.clone())?;
        
        // Build argument expressions
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.build_expression(arg)?);
        }
        
        // Create result value
        let result_id = self.value_gen.next();
        
        // Check if this is an external call (console.log, canvas.fillRect, etc.)
        if let ASTNode::Variable { name: object_name, .. } = object {
            match (object_name.as_str(), method.as_str()) {
                ("console", "log") => {
                    // Generate ExternCall for console.log
                    self.emit_instruction(MirInstruction::ExternCall {
                        dst: None, // console.log is void
                        iface_name: "env.console".to_string(),
                        method_name: "log".to_string(),
                        args: arg_values,
                        effects: EffectMask::IO, // Console output is I/O
                    })?;
                    
                    // Return void value
                    let void_id = self.value_gen.next();
                    self.emit_instruction(MirInstruction::Const {
                        dst: void_id,
                        value: ConstValue::Void,
                    })?;
                    return Ok(void_id);
                },
                ("canvas", "fillRect") => {
                    // Generate ExternCall for canvas.fillRect
                    self.emit_instruction(MirInstruction::ExternCall {
                        dst: None, // canvas.fillRect is void
                        iface_name: "env.canvas".to_string(),
                        method_name: "fillRect".to_string(),
                        args: arg_values,
                        effects: EffectMask::IO, // Canvas operations are I/O
                    })?;
                    
                    // Return void value
                    let void_id = self.value_gen.next();
                    self.emit_instruction(MirInstruction::Const {
                        dst: void_id,
                        value: ConstValue::Void,
                    })?;
                    return Ok(void_id);
                },
                ("canvas", "fillText") => {
                    // Generate ExternCall for canvas.fillText
                    self.emit_instruction(MirInstruction::ExternCall {
                        dst: None, // canvas.fillText is void
                        iface_name: "env.canvas".to_string(),
                        method_name: "fillText".to_string(),
                        args: arg_values,
                        effects: EffectMask::IO, // Canvas operations are I/O
                    })?;
                    
                    // Return void value
                    let void_id = self.value_gen.next();
                    self.emit_instruction(MirInstruction::Const {
                        dst: void_id,
                        value: ConstValue::Void,
                    })?;
                    return Ok(void_id);
                },
                _ => {
                    // Regular method call - continue with BoxCall
                }
            }
        }
        
        // Emit a BoxCall instruction for regular method calls
        self.emit_instruction(MirInstruction::BoxCall {
            dst: Some(result_id),
            box_val: object_value,
            method,
            args: arg_values,
            effects: EffectMask::READ.add(Effect::ReadHeap), // Method calls may have side effects
        })?;
        
        Ok(result_id)
    }
    
    /// Build from expression: from Parent.method(arguments)
    fn build_from_expression(&mut self, parent: String, method: String, arguments: Vec<ASTNode>) -> Result<ValueId, String> {
        // Build argument expressions
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.build_expression(arg)?);
        }
        
        // Create a synthetic "parent reference" value
        let parent_value = self.value_gen.next();
        self.emit_instruction(MirInstruction::Const {
            dst: parent_value,
            value: ConstValue::String(parent),
        })?;
        
        // Create result value
        let result_id = self.value_gen.next();
        
        // Emit a BoxCall instruction for delegation
        self.emit_instruction(MirInstruction::BoxCall {
            dst: Some(result_id),
            box_val: parent_value,
            method,
            args: arg_values,
            effects: EffectMask::READ.add(Effect::ReadHeap),
        })?;
        
        Ok(result_id)
    }
    
    /// Build box declaration: box Name { fields... methods... }
    fn build_box_declaration(&mut self, name: String, methods: std::collections::HashMap<String, ASTNode>, fields: Vec<String>) -> Result<(), String> {
        // For Phase 8.4, we'll emit metadata instructions to register the box type
        // In a full implementation, this would register type information for later use
        
        // Create a type registration constant
        let type_id = self.value_gen.next();
        self.emit_instruction(MirInstruction::Const {
            dst: type_id,
            value: ConstValue::String(format!("__box_type_{}", name)),
        })?;
        
        // For each field, emit metadata about the field
        for field in fields {
            let field_id = self.value_gen.next();
            self.emit_instruction(MirInstruction::Const {
                dst: field_id,
                value: ConstValue::String(format!("__field_{}_{}", name, field)),
            })?;
        }
        
        // Process methods - now methods is a HashMap
        for (method_name, method_ast) in methods {
            if let ASTNode::FunctionDeclaration { .. } = method_ast {
                let method_id = self.value_gen.next();
                self.emit_instruction(MirInstruction::Const {
                    dst: method_id,
                    value: ConstValue::String(format!("__method_{}_{}", name, method_name)),
                })?;
            }
        }
        
        Ok(())
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
