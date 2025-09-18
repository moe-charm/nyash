# Appendix - Technical Details

## A. Complete EBNF Grammar Definition

### A.1 Phase 15.6: Method-Level Postfix Modifiers

```ebnf
(* Core method declaration with postfix exception handling *)
methodDecl := 'method' IDENT '(' paramList? ')' typeHint? methodBody postfixModifiers?

methodBody := block

postfixModifiers := catchClause? finallyClause?

catchClause := 'catch' '(' catchParam? ')' block

catchParam := IDENT IDENT    (* type name *)
            | IDENT          (* untyped *)
            | ε              (* no binding *)

finallyClause := 'finally' block

(* Parameter list *)
paramList := param (',' param)*
param := IDENT typeHint?

(* Type hints *)
typeHint := ':' typeExpr
typeExpr := IDENT | IDENT '<' typeList '>'
typeList := typeExpr (',' typeExpr)*

(* Block definition *)
block := '{' stmt* '}'
stmt := assignment | expression | controlFlow | return

(* Lexical elements *)
IDENT := [a-zA-Z_][a-zA-Z0-9_]*
```

### A.2 Phase 16.1: Postfix Method Definition

```ebnf
(* Block-first method definition *)
postfixMethodDecl := block 'method' IDENT '(' paramList? ')' typeHint? postfixModifiers?

(* Forward reference resolution *)
blockWithForwardRefs := '{' stmtWithRefs* '}'
stmtWithRefs := stmt | forwardRef

forwardRef := IDENT  (* resolved against later-declared parameters *)
```

### A.3 Phase 16.2: Unified Block + Modifier Syntax

```ebnf
(* Complete unified syntax *)
unifiedDecl := block modifierChain

modifierChain := primaryModifier auxiliaryModifier*

primaryModifier := asField | asProperty | asMethod
auxiliaryModifier := catchClause | finallyClause | accessModifier | asyncModifier

asField := 'as' 'field' IDENT typeHint?
asProperty := 'as' 'property' IDENT typeHint?
asMethod := 'as' 'method' IDENT '(' paramList? ')' typeHint?

accessModifier := 'private' | 'public' | 'protected'
asyncModifier := 'async'
```

## B. AST Transformation Details

### B.1 Method Postfix to TryCatch Normalization

```rust
// Source AST node
#[derive(Debug, Clone)]
pub struct MethodWithPostfix {
    pub name: String,
    pub params: Vec<Parameter>,
    pub body: Block,
    pub catch_clause: Option<CatchClause>,
    pub finally_clause: Option<Block>,
    pub return_type: Option<TypeExpr>,
}

// Normalized AST transformation
impl MethodWithPostfix {
    pub fn normalize(self) -> Method {
        let try_catch_body = if self.catch_clause.is_some() || self.finally_clause.is_some() {
            vec![ASTNode::TryCatch {
                try_body: self.body.statements,
                catch_clauses: self.catch_clause.into_iter().collect(),
                finally_clause: self.finally_clause.map(|b| b.statements),
            }]
        } else {
            self.body.statements
        };

        Method {
            name: self.name,
            params: self.params,
            body: Block { statements: try_catch_body },
            return_type: self.return_type,
            visibility: Visibility::Public, // Default
        }
    }
}

// Catch clause representation
#[derive(Debug, Clone)]
pub struct CatchClause {
    pub param: Option<String>,
    pub type_hint: Option<TypeExpr>,
    pub body: Block,
}
```

### B.2 Forward Reference Resolution Algorithm

```rust
// Two-phase parsing for block-first methods
pub struct ForwardRefResolver {
    pending_refs: Vec<PendingReference>,
    param_scope: HashMap<String, Parameter>,
}

#[derive(Debug)]
struct PendingReference {
    name: String,
    location: SourceLocation,
    context: ReferenceContext,
}

impl ForwardRefResolver {
    // Phase 1: Parse block with forward references
    pub fn parse_block_with_refs(&mut self, tokens: &mut TokenStream) -> Result<Block, ParseError> {
        let mut statements = Vec::new();
        
        while !tokens.check("}") {
            match self.parse_statement_with_refs(tokens) {
                Ok(stmt) => statements.push(stmt),
                Err(ParseError::UnresolvedReference(name, loc)) => {
                    self.pending_refs.push(PendingReference {
                        name,
                        location: loc,
                        context: ReferenceContext::Variable,
                    });
                    // Insert placeholder
                    statements.push(ASTNode::UnresolvedRef(name));
                }
                Err(e) => return Err(e),
            }
        }
        
        Ok(Block { statements })
    }
    
    // Phase 2: Resolve references after parsing signature
    pub fn resolve_references(&mut self, method_sig: &MethodSignature) -> Result<(), SemanticError> {
        // Build parameter scope
        for param in &method_sig.params {
            self.param_scope.insert(param.name.clone(), param.clone());
        }
        
        // Resolve pending references
        for pending in &self.pending_refs {
            if let Some(param) = self.param_scope.get(&pending.name) {
                // Replace UnresolvedRef with proper Parameter reference
                // Implementation details omitted for brevity
            } else {
                return Err(SemanticError::UndefinedVariable(pending.name.clone()));
            }
        }
        
        Ok(())
    }
}
```

## C. MIR Lowering Implementation

### C.1 Result-Mode Exception Handling

```rust
// Enhanced ThrowCtx for method-level handling
pub struct ThrowContext {
    pub catch_target: BasicBlockId,
    pub incoming_exceptions: Vec<(BasicBlockId, ValueId)>,
    pub method_level: bool, // New flag for method-level catch
}

thread_local! {
    static THROW_CTX_STACK: RefCell<Vec<ThrowContext>> = RefCell::new(Vec::new());
}

pub mod throw_ctx {
    use super::*;
    
    pub fn push_method_level(catch_bb: BasicBlockId) {
        THROW_CTX_STACK.with(|stack| {
            stack.borrow_mut().push(ThrowContext {
                catch_target: catch_bb,
                incoming_exceptions: Vec::new(),
                method_level: true,
            });
        });
    }
    
    pub fn record_throw(throw_bb: BasicBlockId, exception_value: ValueId) {
        THROW_CTX_STACK.with(|stack| {
            if let Some(ctx) = stack.borrow_mut().last_mut() {
                ctx.incoming_exceptions.push((throw_bb, exception_value));
            }
        });
    }
    
    pub fn pop_method_level() -> Option<ThrowContext> {
        THROW_CTX_STACK.with(|stack| {
            let mut stack_ref = stack.borrow_mut();
            if stack_ref.last().map(|ctx| ctx.method_level).unwrap_or(false) {
                stack_ref.pop()
            } else {
                None
            }
        })
    }
}
```

### C.2 PHI-off Edge-Copy Alternative

```rust
// PHI-free variable merging using edge copies
pub fn lower_method_with_postfix(
    f: &mut MirFunction,
    method: &MethodWithPostfix,
    env: &BridgeEnv,
) -> Result<(), LoweringError> {
    let entry_bb = f.entry_block();
    let mut vars = HashMap::new();
    
    // Initialize parameter variables
    for (i, param) in method.params.iter().enumerate() {
        let param_val = f.get_param_value(i);
        vars.insert(param.name.clone(), param_val);
    }
    
    if method.catch_clause.is_some() || method.finally_clause.is_some() {
        // Method-level exception handling
        let try_bb = f.new_block();
        let catch_bb = method.catch_clause.as_ref().map(|_| f.new_block());
        let finally_bb = method.finally_clause.as_ref().map(|_| f.new_block());
        let exit_bb = f.new_block();
        
        // Set up method-level ThrowCtx
        if let Some(catch_target) = catch_bb {
            throw_ctx::push_method_level(catch_target);
        }
        
        // Lower try body
        f.get_block_mut(entry_bb).unwrap().set_terminator(
            MirInstruction::Jump { target: try_bb }
        );
        
        let mut try_vars = vars.clone();
        let try_end = lower_stmt_list(f, try_bb, &method.body.statements, &mut try_vars, env)?;
        
        // Collect exception information
        let throw_info = throw_ctx::pop_method_level();
        
        // Handle catch block if present
        let catch_end = if let (Some(catch_clause), Some(catch_bb)) = 
            (&method.catch_clause, catch_bb) {
            
            let mut catch_vars = vars.clone();
            
            // Bind exception parameter using edge copies instead of PHI
            if let Some(param_name) = &catch_clause.param {
                if let Some(throw_info) = &throw_info {
                    let exception_val = f.next_value_id();
                    
                    // Insert edge copies on all incoming exception edges
                    for &(throw_bb, exc_val) in &throw_info.incoming_exceptions {
                        f.get_block_mut(throw_bb).unwrap().add_instruction(
                            MirInstruction::Copy { dst: exception_val, src: exc_val }
                        );
                    }
                    
                    catch_vars.insert(param_name.clone(), exception_val);
                }
            }
            
            Some(lower_stmt_list(f, catch_bb, &catch_clause.body.statements, &mut catch_vars, env)?)
        } else {
            None
        };
        
        // Variable merging using edge copies
        if env.mir_no_phi {
            merge_variables_with_edge_copies(f, &[
                (try_end, try_vars),
                catch_end.map(|bb| (bb, catch_vars.unwrap_or_default())).into_iter().collect()
            ], finally_bb.unwrap_or(exit_bb))?;
        }
        
        // Handle finally block
        if let Some(finally_bb) = finally_bb {
            // Finally implementation
            // ... details omitted for brevity
        }
        
    } else {
        // Simple method without exception handling
        lower_stmt_list(f, entry_bb, &method.body.statements, &mut vars, env)?;
    }
    
    Ok(())
}

fn merge_variables_with_edge_copies(
    f: &mut MirFunction,
    branches: &[(BasicBlockId, HashMap<String, ValueId>)],
    target_bb: BasicBlockId,
) -> Result<(), LoweringError> {
    let all_vars: HashSet<String> = branches.iter()
        .flat_map(|(_, vars)| vars.keys())
        .cloned()
        .collect();
    
    for var_name in all_vars {
        let mut sources = Vec::new();
        for &(bb, ref vars) in branches {
            if let Some(&val) = vars.get(&var_name) {
                sources.push((bb, val));
            }
        }
        
        if sources.len() > 1 {
            // Multiple sources, need merge value
            let merged_val = f.next_value_id();
            
            // Insert copy instructions on predecessor edges
            for &(pred_bb, src_val) in &sources {
                f.get_block_mut(pred_bb).unwrap().add_instruction(
                    MirInstruction::Copy { dst: merged_val, src: src_val }
                );
            }
        }
    }
    
    Ok(())
}
```

## D. Parser Implementation Details

### D.1 Recursive Descent Parser Extension

```rust
pub struct Parser {
    tokens: TokenStream,
    current: usize,
    errors: Vec<ParseError>,
    forward_refs: ForwardRefResolver,
}

impl Parser {
    pub fn parse_method_declaration(&mut self) -> Result<ASTNode, ParseError> {
        if self.check_sequence(&["{", "..."]) && self.peek_after_block_is("method") {
            // Block-first method definition (Phase 16.1)
            self.parse_postfix_method()
        } else if self.check("method") {
            // Traditional method with possible postfix modifiers (Phase 15.6)
            self.parse_traditional_method_with_postfix()
        } else {
            Err(ParseError::ExpectedMethod)
        }
    }
    
    fn parse_traditional_method_with_postfix(&mut self) -> Result<ASTNode, ParseError> {
        self.consume("method")?;
        let name = self.consume_identifier()?;
        
        self.consume("(")?;
        let params = if !self.check(")") {
            self.parse_parameter_list()?
        } else {
            Vec::new()
        };
        self.consume(")")?;
        
        let return_type = if self.check(":") {
            self.advance(); // consume ':'
            Some(self.parse_type_expression()?)
        } else {
            None
        };
        
        let body = self.parse_block()?;
        
        // Check for postfix modifiers
        let mut catch_clause = None;
        let mut finally_clause = None;
        
        if self.check("catch") {
            catch_clause = Some(self.parse_catch_clause()?);
        }
        
        if self.check("finally") {
            finally_clause = Some(self.parse_finally_clause()?);
        }
        
        Ok(ASTNode::MethodWithPostfix(MethodWithPostfix {
            name,
            params,
            body,
            catch_clause,
            finally_clause,
            return_type,
        }))
    }
    
    fn parse_postfix_method(&mut self) -> Result<ASTNode, ParseError> {
        // Parse block first with forward reference support
        let body = self.forward_refs.parse_block_with_refs(&mut self.tokens)?;
        
        self.consume("method")?;
        let name = self.consume_identifier()?;
        
        self.consume("(")?;
        let params = if !self.check(")") {
            self.parse_parameter_list()?
        } else {
            Vec::new()
        };
        self.consume(")")?;
        
        let return_type = if self.check(":") {
            self.advance();
            Some(self.parse_type_expression()?)
        } else {
            None
        };
        
        // Parse postfix modifiers
        let mut catch_clause = None;
        let mut finally_clause = None;
        
        if self.check("catch") {
            catch_clause = Some(self.parse_catch_clause()?);
        }
        
        if self.check("finally") {
            finally_clause = Some(self.parse_finally_clause()?);
        }
        
        // Resolve forward references
        let method_sig = MethodSignature { name: name.clone(), params, return_type };
        self.forward_refs.resolve_references(&method_sig)?;
        
        Ok(ASTNode::PostfixMethod(PostfixMethod {
            name,
            params: method_sig.params,
            body,
            catch_clause,
            finally_clause,
            return_type: method_sig.return_type,
        }))
    }
    
    fn parse_catch_clause(&mut self) -> Result<CatchClause, ParseError> {
        self.consume("catch")?;
        self.consume("(")?;
        
        let param = if self.check(")") {
            None
        } else {
            let param_name = self.consume_identifier()?;
            let type_hint = if !self.check(")") && self.current_token().is_identifier() {
                // Type hint present: catch (e Exception)
                Some(self.parse_type_expression()?)
            } else {
                None
            };
            Some((param_name, type_hint))
        };
        
        self.consume(")")?;
        let body = self.parse_block()?;
        
        Ok(CatchClause {
            param: param.map(|(name, _)| name),
            type_hint: param.and_then(|(_, typ)| typ),
            body,
        })
    }
    
    fn parse_finally_clause(&mut self) -> Result<Block, ParseError> {
        self.consume("finally")?;
        self.parse_block()
    }
    
    // Lookahead helper for block-first detection
    fn peek_after_block_is(&self, expected: &str) -> bool {
        let mut depth = 0;
        let mut pos = self.current;
        
        if pos >= self.tokens.len() || !self.tokens[pos].is("{") {
            return false;
        }
        
        pos += 1; // skip opening '{'
        depth += 1;
        
        while pos < self.tokens.len() && depth > 0 {
            match self.tokens[pos].kind() {
                TokenKind::LeftBrace => depth += 1,
                TokenKind::RightBrace => depth -= 1,
                _ => {}
            }
            pos += 1;
        }
        
        // Check if next token is the expected one
        pos < self.tokens.len() && self.tokens[pos].is(expected)
    }
}
```

## E. Performance Optimization Details

### E.1 Zero-Cost Abstraction Implementation

```rust
// Compile-time optimization for method-level exception handling
pub struct MethodOptimizer;

impl MethodOptimizer {
    pub fn optimize_method_postfix(method: &mut MirFunction) -> OptimizationResult {
        let mut changes = 0;
        
        // 1. Dead catch elimination
        changes += self.eliminate_unused_catch_blocks(method)?;
        
        // 2. Exception flow analysis
        changes += self.optimize_exception_paths(method)?;
        
        // 3. Finally block consolidation
        changes += self.consolidate_finally_blocks(method)?;
        
        OptimizationResult {
            optimizations_applied: changes,
            performance_gain: self.estimate_performance_gain(changes),
        }
    }
    
    fn eliminate_unused_catch_blocks(&self, method: &mut MirFunction) -> Result<usize, OptError> {
        let mut eliminated = 0;
        
        // Analyze throw sites
        let throw_sites = self.find_all_throw_sites(method);
        
        // Find catch blocks with no incoming throws
        for block_id in method.block_ids() {
            let block = method.get_block(block_id).unwrap();
            
            if self.is_catch_block(block) && !self.has_incoming_throws(block_id, &throw_sites) {
                // This catch block is unreachable
                method.remove_block(block_id);
                eliminated += 1;
            }
        }
        
        Ok(eliminated)
    }
    
    fn optimize_exception_paths(&self, method: &mut MirFunction) -> Result<usize, OptError> {
        let mut optimizations = 0;
        
        // Look for patterns like: throw -> immediate catch -> simple return
        for throw_site in self.find_all_throw_sites(method) {
            if let Some(direct_catch) = self.find_direct_catch_target(method, throw_site) {
                if self.is_simple_return_catch(method, direct_catch) {
                    // Replace throw+catch with direct return
                    self.replace_with_direct_return(method, throw_site, direct_catch);
                    optimizations += 1;
                }
            }
        }
        
        Ok(optimizations)
    }
    
    fn consolidate_finally_blocks(&self, method: &mut MirFunction) -> Result<usize, OptError> {
        let mut consolidations = 0;
        
        // Find duplicate finally blocks
        let finally_blocks = self.find_finally_blocks(method);
        let mut consolidated_groups = HashMap::new();
        
        for finally_block in finally_blocks {
            let signature = self.compute_block_signature(method, finally_block);
            
            consolidated_groups.entry(signature)
                .or_insert_with(Vec::new)
                .push(finally_block);
        }
        
        // Merge identical finally blocks
        for (_, blocks) in consolidated_groups {
            if blocks.len() > 1 {
                let canonical = blocks[0];
                for &duplicate in &blocks[1..] {
                    self.redirect_finally_references(method, duplicate, canonical);
                    method.remove_block(duplicate);
                    consolidations += 1;
                }
            }
        }
        
        Ok(consolidations)
    }
}

// Optimization result tracking
#[derive(Debug)]
pub struct OptimizationResult {
    pub optimizations_applied: usize,
    pub performance_gain: f64, // Estimated percentage improvement
}
```

### E.2 Memory Layout Optimization

```rust
// Optimized representation for method-level exception handling
#[repr(C)]
pub struct OptimizedMethodFrame {
    // Standard frame data
    pub local_vars: *mut ValueSlot,
    pub param_count: u16,
    pub local_count: u16,
    
    // Exception handling data (compact representation)
    pub exception_info: ExceptionInfo,
}

#[repr(packed)]
pub struct ExceptionInfo {
    pub has_catch: bool,
    pub has_finally: bool,
    pub catch_target: u16,    // Relative offset
    pub finally_target: u16,  // Relative offset
    pub exception_slot: u8,   // Local variable slot for exception value
}

impl OptimizedMethodFrame {
    pub fn new(params: u16, locals: u16, exception_info: ExceptionInfo) -> Self {
        let total_slots = (params + locals) as usize;
        let local_vars = unsafe {
            std::alloc::alloc(
                std::alloc::Layout::array::<ValueSlot>(total_slots).unwrap()
            ) as *mut ValueSlot
        };
        
        Self {
            local_vars,
            param_count: params,
            local_count: locals,
            exception_info,
        }
    }
    
    // Optimized exception dispatch
    #[inline]
    pub fn handle_exception(&mut self, exception: RuntimeException) -> ControlFlow {
        if self.exception_info.has_catch {
            // Store exception in designated slot
            unsafe {
                *self.local_vars.add(self.exception_info.exception_slot as usize) = 
                    ValueSlot::Exception(exception);
            }
            
            // Jump to catch handler
            ControlFlow::Jump(self.exception_info.catch_target)
        } else {
            // Propagate to caller
            ControlFlow::Propagate(exception)
        }
    }
    
    // Optimized finally execution
    #[inline]
    pub fn execute_finally(&self) -> ControlFlow {
        if self.exception_info.has_finally {
            ControlFlow::Jump(self.exception_info.finally_target)
        } else {
            ControlFlow::Return
        }
    }
}

// Compact value representation
#[repr(C)]
pub union ValueSlot {
    integer: i64,
    float: f64,
    pointer: *mut RuntimeBox,
    exception: RuntimeException,
}
```

## F. Testing Infrastructure

### F.1 Comprehensive Test Suite

```rust
// Test framework for method-level postfix exception handling
pub mod tests {
    use super::*;
    
    #[test_suite]
    pub struct MethodPostfixTests;
    
    impl MethodPostfixTests {
        #[test]
        fn test_basic_catch() {
            let code = r#"
                method process() {
                    return riskyOperation()
                } catch (e) {
                    return "fallback"
                }
            "#;
            
            let result = self.compile_and_run(code, &[]);
            assert_eq!(result, RuntimeValue::String("fallback".to_string()));
        }
        
        #[test]
        fn test_catch_with_finally() {
            let code = r#"
                method processWithCleanup() {
                    return complexOperation()
                } catch (e) {
                    return "error"
                } finally {
                    cleanup()
                }
            "#;
            
            let mut context = TestContext::new();
            let result = self.compile_and_run_with_context(code, &[], &mut context);
            
            assert_eq!(result, RuntimeValue::String("error".to_string()));
            assert!(context.cleanup_called);
        }
        
        #[test]
        fn test_nested_exceptions() {
            let code = r#"
                method outer() {
                    return me.inner()
                } catch (e) {
                    return "outer_catch"
                }
                
                method inner() {
                    return me.deepest()
                } catch (e) {
                    return "inner_catch"
                }
                
                method deepest() {
                    throw new Exception("deep_error")
                }
            "#;
            
            let result = self.compile_and_run(code, &[]);
            assert_eq!(result, RuntimeValue::String("inner_catch".to_string()));
        }
        
        #[test]
        fn test_performance_baseline() {
            let traditional_code = r#"
                method traditionalProcess() {
                    try {
                        return heavyComputation()
                    } catch (e) {
                        return fallbackValue()
                    } finally {
                        cleanup()
                    }
                }
            "#;
            
            let postfix_code = r#"
                method postfixProcess() {
                    return heavyComputation()
                } catch (e) {
                    return fallbackValue()
                } finally {
                    cleanup()
                }
            "#;
            
            let traditional_time = self.benchmark_code(traditional_code, 10000);
            let postfix_time = self.benchmark_code(postfix_code, 10000);
            
            // Performance should be identical (zero-cost abstraction)
            assert!((traditional_time - postfix_time).abs() / traditional_time < 0.05);
        }
        
        #[test]
        fn test_ast_normalization() {
            let code = r#"
                method example() {
                    return computation()
                } catch (e) {
                    return defaultValue()
                }
            "#;
            
            let ast = self.parse(code).unwrap();
            let normalized = self.normalize_ast(ast);
            
            // Should normalize to traditional try-catch structure
            match normalized {
                ASTNode::Method { body, .. } => {
                    assert!(matches!(body.statements[0], ASTNode::TryCatch { .. }));
                }
                _ => panic!("Expected method node"),
            }
        }
    }
    
    struct TestContext {
        cleanup_called: bool,
        exception_log: Vec<String>,
    }
    
    impl TestContext {
        fn new() -> Self {
            Self {
                cleanup_called: false,
                exception_log: Vec::new(),
            }
        }
    }
}
```

### F.2 Property-Based Testing

```rust
// Property-based tests for method postfix exception handling
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_exception_safety_property(
        method_body in generate_method_body(),
        exception_type in generate_exception_type(),
        catch_behavior in generate_catch_behavior(),
    ) {
        let code = format!(r#"
            method testMethod() {{
                {}
            }} catch (e {}) {{
                {}
            }}
        "#, method_body, exception_type, catch_behavior);
        
        let result = compile_and_analyze_safety(&code);
        
        // Property: All exceptions should be handled
        prop_assert!(result.all_exceptions_handled);
        
        // Property: No resource leaks
        prop_assert!(result.no_resource_leaks);
        
        // Property: Control flow is well-defined
        prop_assert!(result.control_flow_valid);
    }
    
    #[test]
    fn test_performance_consistency(
        traditional_method in generate_traditional_method(),
        equivalent_postfix in generate_equivalent_postfix_method(),
    ) {
        let traditional_time = benchmark_method(&traditional_method);
        let postfix_time = benchmark_method(&equivalent_postfix);
        
        // Property: Performance should be equivalent
        let diff_ratio = (traditional_time - postfix_time).abs() / traditional_time;
        prop_assert!(diff_ratio < 0.1); // Within 10%
    }
}

fn generate_method_body() -> impl Strategy<Value = String> {
    prop_oneof![
        "return simpleValue()",
        "return complexComputation(arg)",
        "local temp = process(); return temp",
        "for i in range(10) { process(i) }; return result",
    ]
}

fn generate_exception_type() -> impl Strategy<Value = String> {
    prop_oneof![
        "",
        "Exception",
        "RuntimeError", 
        "IOException",
    ]
}

fn generate_catch_behavior() -> impl Strategy<Value = String> {
    prop_oneof![
        "return defaultValue()",
        "log(e); return fallback()",
        "me.handleError(e); return recovery()",
    ]
}
```

---

## Summary

This appendix provides comprehensive technical details supporting the main paper:

- **Section A**: Complete EBNF grammar definitions for all three phases
- **Section B**: Detailed AST transformation algorithms
- **Section C**: MIR lowering implementation with Result-mode support
- **Section D**: Parser implementation with forward reference resolution
- **Section E**: Performance optimization strategies and memory layout
- **Section F**: Comprehensive testing infrastructure including property-based tests

These technical details demonstrate the practical implementability and theoretical soundness of method-level postfix exception handling while maintaining the zero-cost abstraction principle fundamental to the Nyash language design.