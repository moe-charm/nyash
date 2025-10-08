use nyash_rust::ast::Span;
use nyash_rust::{ASTNode, ast::LiteralValue, ast::BinaryOperator};
use std::time::Instant;

/// HIR Patch description (MVP placeholder)
#[derive(Clone, Debug, Default)]
pub struct HirPatch {
    // In MVP, we keep it opaque; later will host add/replace nodes
}

pub struct MacroEngine {
    max_passes: usize,
    cycle_window: usize,
    trace: bool,
}

impl MacroEngine {
    pub fn new() -> Self {
        let max_passes = std::env::var("NYASH_MACRO_MAX_PASSES").ok().and_then(|v| v.parse().ok()).unwrap_or(32);
        let cycle_window = std::env::var("NYASH_MACRO_CYCLE_WINDOW").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
        let trace = std::env::var("NYASH_MACRO_TRACE").ok().as_deref() == Some("1");
        Self { max_passes, cycle_window, trace }
    }

    /// Expand all macros with depth/cycle guards and return patched AST.
    pub fn expand(&mut self, ast: &ASTNode) -> (ASTNode, Vec<HirPatch>) {
        let patches = Vec::new();
        let mut cur = ast.clone();
        let mut history: std::collections::VecDeque<ASTNode> = std::collections::VecDeque::new();
        for pass in 0..self.max_passes {
            let t0 = Instant::now();
            let before_len = crate::r#macro::ast_json::ast_to_json(&cur).to_string().len();
            let next0 = self.expand_node(&cur);
            // Apply user MacroBoxes once per pass (if enabled)
            let next = crate::r#macro::macro_box::expand_all_once(&next0);
            let after_len = crate::r#macro::ast_json::ast_to_json(&next).to_string().len();
            let dt = t0.elapsed();
            if self.trace { eprintln!("[macro][engine] pass={} changed={} bytes:{}=>{} dt={:?}", pass, (next != cur), before_len, after_len, dt); }
            jsonl_trace(pass, before_len, after_len, next != cur, dt);
            if next == cur { return (cur, patches); }
            // cycle detection in small window
            if history.iter().any(|h| *h == next) {
                eprintln!("[macro][engine] cycle detected at pass {} — stopping expansion", pass);
                return (cur, patches);
            }
            history.push_back(cur);
            if history.len() > self.cycle_window { let _ = history.pop_front(); }
            cur = next;
        }
        eprintln!("[macro][engine] max passes ({}) exceeded — stopping expansion", self.max_passes);
        (cur, patches)
    }

    fn expand_node(&mut self, node: &ASTNode) -> ASTNode {
        match node.clone() {
            ASTNode::Program { statements, span } => {
                if std::env::var("NYASH_MACRO_TRACE").ok().as_deref() == Some("1") {
                    eprintln!("[macro][visit] Program: statements={}", statements.len());
                }
                // Use flat_map to support multi-node expansion (e.g., @enum → BoxDecl + StaticBox)
                let new_stmts = statements.into_iter().flat_map(|n| {
                    if std::env::var("NYASH_MACRO_TRACE").ok().as_deref() == Some("1") {
                        eprintln!("[macro][visit]  child kind...",);
                    }
                    let expanded = self.expand_node(&n);
                    // If expansion returns a Program node, unwrap its statements
                    match expanded {
                        ASTNode::Program { statements: inner, .. } => inner,
                        other => vec![other],
                    }
                }).collect();
                ASTNode::Program { statements: new_stmts, span }
            }
            ASTNode::BoxDeclaration { name, fields, public_fields, private_fields, mut methods, constructors, init_fields, weak_fields, is_interface, extends, implements, type_parameters, is_static, static_init, span } => {
                if std::env::var("NYASH_MACRO_TRACE").ok().as_deref() == Some("1") {
                    eprintln!("[macro][visit] BoxDeclaration: {} (fields={})", name, fields.len());
                }
                // Derive set: default Equals+ToString when macro is enabled
                let derive_all = std::env::var("NYASH_MACRO_DERIVE_ALL").ok().as_deref() == Some("1");
                let derive_set = std::env::var("NYASH_MACRO_DERIVE").ok().unwrap_or_else(|| "Equals,ToString".to_string());
                if std::env::var("NYASH_MACRO_TRACE").ok().as_deref() == Some("1") {
                    eprintln!("[macro][derive] box={} derive_all={} set={}", name, derive_all, derive_set);
                }
                let want_equals = derive_all || derive_set.contains("Equals");
                let want_tostring = derive_all || derive_set.contains("ToString");
                // Philosophy-2: respect box independence — operate on public interface only
                let field_view: &Vec<String> = &public_fields;
                if want_equals && !methods.contains_key("equals") {
                    if std::env::var("NYASH_MACRO_TRACE").ok().as_deref() == Some("1") {
                        eprintln!("[macro][derive] equals for {} (public fields: {})", name, field_view.len());
                    }
                    let m = build_equals_method(&name, field_view);
                    methods.insert("equals".to_string(), m);
                }
                if want_tostring && !methods.contains_key("toString") {
                    if std::env::var("NYASH_MACRO_TRACE").ok().as_deref() == Some("1") {
                        eprintln!("[macro][derive] toString for {} (public fields: {})", name, field_view.len());
                    }
                    let m = build_tostring_method(&name, field_view);
                    methods.insert("toString".to_string(), m);
                }
                ASTNode::BoxDeclaration { name, fields, public_fields, private_fields, methods, constructors, init_fields, weak_fields, is_interface, extends, implements, type_parameters, is_static, static_init, span }
            }
            ASTNode::EnumDeclaration { name, variants, span } => {
                if std::env::var("NYASH_MACRO_TRACE").ok().as_deref() == Some("1") {
                    eprintln!("[macro][visit] EnumDeclaration: {} ({} variants)", name, variants.len());
                }
                // Expand @enum to BoxDeclaration + StaticBoxDeclaration
                let box_nodes = expand_enum_to_boxes(&name, &variants, span);
                // Return wrapped in Program for flat_map unwrapping
                ASTNode::Program {
                    statements: box_nodes,
                    span,
                }
            }
            other => other,
        }
    }
}

fn jsonl_trace(pass: usize, before: usize, after: usize, changed: bool, dt: std::time::Duration) {
    if let Ok(path) = std::env::var("NYASH_MACRO_TRACE_JSONL") {
        if path.is_empty() { return; }
        let rec = serde_json::json!({
            "event": "macro_pass",
            "pass": pass,
            "changed": changed,
            "before_bytes": before,
            "after_bytes": after,
            "dt_us": dt.as_micros() as u64,
        }).to_string();
        let _ = std::fs::OpenOptions::new().create(true).append(true).open(path).and_then(|mut f| {
            use std::io::Write;
            writeln!(f, "{}", rec)
        });
    }
}

fn me_field(name: &str) -> ASTNode {
    ASTNode::FieldAccess {
        object: Box::new(ASTNode::Me { span: Span::unknown() }),
        field: name.to_string(),
        span: Span::unknown(),
    }
}

fn var_field(var: &str, field: &str) -> ASTNode {
    ASTNode::FieldAccess {
        object: Box::new(ASTNode::Variable { name: var.to_string(), span: Span::unknown() }),
        field: field.to_string(),
        span: Span::unknown(),
    }
}

fn bin_add(lhs: ASTNode, rhs: ASTNode) -> ASTNode {
    ASTNode::BinaryOp { operator: BinaryOperator::Add, left: Box::new(lhs), right: Box::new(rhs), span: Span::unknown() }
}

fn bin_and(lhs: ASTNode, rhs: ASTNode) -> ASTNode {
    ASTNode::BinaryOp { operator: BinaryOperator::And, left: Box::new(lhs), right: Box::new(rhs), span: Span::unknown() }
}

fn bin_eq(lhs: ASTNode, rhs: ASTNode) -> ASTNode {
    ASTNode::BinaryOp { operator: BinaryOperator::Equal, left: Box::new(lhs), right: Box::new(rhs), span: Span::unknown() }
}

fn lit_str(s: &str) -> ASTNode { ASTNode::Literal { value: LiteralValue::String(s.to_string()), span: Span::unknown() } }

fn build_equals_method(_box_name: &str, fields: &Vec<String>) -> ASTNode {
    // equals(other) { return me.f1 == other.f1 && ...; }
    let cond = if fields.is_empty() {
        ASTNode::Literal { value: LiteralValue::Bool(true), span: Span::unknown() }
    } else {
        let mut it = fields.iter();
        let first = it.next().unwrap();
        let mut expr = bin_eq(me_field(first), var_field("__ny_other", first));
        for f in it {
            expr = bin_and(expr, bin_eq(me_field(f), var_field("__ny_other", f)));
        }
        expr
    };
    // Hygiene: use gensym-like param to avoid collisions
    let param_name = "__ny_other".to_string();
    ASTNode::FunctionDeclaration {
        name: "equals".to_string(),
        params: vec![param_name.clone()],
        body: vec![ASTNode::Return { value: Some(Box::new(cond)), span: Span::unknown() }],
        is_static: false,
        is_override: false,
        span: Span::unknown(),
    }
}

fn build_tostring_method(box_name: &str, fields: &Vec<String>) -> ASTNode {
    // toString() { return "Name(" + me.f1 + "," + me.f2 + ")" }
    let mut expr = lit_str(&format!("{}(", box_name));
    let mut first = true;
    for f in fields {
        if !first { expr = bin_add(expr, lit_str(",")); }
        first = false;
        expr = bin_add(expr, me_field(f));
    }
    expr = bin_add(expr, lit_str(")"));
    ASTNode::FunctionDeclaration {
        name: "toString".to_string(),
        params: vec![],
        body: vec![ASTNode::Return { value: Some(Box::new(expr)), span: Span::unknown() }],
        is_static: false,
        is_override: false,
        span: Span::unknown(),
    }
}

/// Expand @enum to BoxDeclaration + StaticBoxDeclaration
/// Example: @enum Result { Ok(value) Err(error) }
/// → ResultBox { _tag, value, error, birth(), is_Ok(), as_Ok(), ... }
/// → static box Result { Ok(v), Err(e) }
fn expand_enum_to_boxes(name: &str, variants: &[nyash_rust::ast::EnumVariant], span: Span) -> Vec<ASTNode> {
    use std::collections::HashMap;

    let box_name = format!("{}Box", name);

    // Collect all field names across all variants
    let mut all_fields: Vec<String> = vec!["_tag".to_string()];
    for variant in variants {
        for field in &variant.fields {
            if !all_fields.contains(field) {
                all_fields.push(field.clone());
            }
        }
    }

    // Build BoxDeclaration
    let mut methods: HashMap<String, ASTNode> = HashMap::new();

    // birth() method: initialize _tag and fields
    methods.insert("birth".to_string(), build_enum_birth_method(&all_fields));

    // is_* methods for each variant
    for variant in variants {
        let method_name = format!("is_{}", variant.name);
        methods.insert(method_name, build_enum_is_method(&variant.name));
    }

    // as_* methods for each variant
    for variant in variants {
        if !variant.fields.is_empty() {
            let method_name = format!("as_{}", variant.name);
            methods.insert(method_name, build_enum_as_method(&variant.name, &variant.fields));
        }
    }

    let box_decl = ASTNode::BoxDeclaration {
        name: box_name,
        fields: all_fields.clone(),
        public_fields: all_fields.clone(),
        private_fields: vec![],
        methods,
        constructors: HashMap::new(),
        init_fields: vec![],
        weak_fields: vec![],
        is_interface: false,
        extends: vec![],
        implements: vec![],
        type_parameters: vec![],
        is_static: false,
        static_init: None,
        span,
    };

    // Build StaticBoxDeclaration
    let mut static_methods: HashMap<String, ASTNode> = HashMap::new();

    // Constructor methods: Ok(v), Err(e), etc.
    for variant in variants {
        static_methods.insert(variant.name.clone(), build_enum_constructor(&name, &variant));
    }

    let static_box_decl = ASTNode::BoxDeclaration {
        name: name.to_string(),
        fields: vec![],
        public_fields: vec![],
        private_fields: vec![],
        methods: static_methods,
        constructors: HashMap::new(),
        init_fields: vec![],
        weak_fields: vec![],
        is_interface: false,
        extends: vec![],
        implements: vec![],
        type_parameters: vec![],
        is_static: true,
        static_init: None,
        span,
    };

    vec![box_decl, static_box_decl]
}

/// Build birth() method for enum box: initialize all fields to null
/// birth() { me._tag = null; me.value = null; me.error = null }
fn build_enum_birth_method(fields: &[String]) -> ASTNode {
    let mut body: Vec<ASTNode> = Vec::new();
    for field in fields {
        body.push(ASTNode::Assignment {
            target: Box::new(me_field(field)),
            value: Box::new(ASTNode::Literal {
                value: LiteralValue::Null,
                span: Span::unknown(),
            }),
            span: Span::unknown(),
        });
    }
    ASTNode::FunctionDeclaration {
        name: "birth".to_string(),
        params: vec![],
        body,
        is_static: false,
        is_override: false,
        span: Span::unknown(),
    }
}

/// Build is_* method: is_Ok() { return me._tag == "Ok" }
fn build_enum_is_method(variant_name: &str) -> ASTNode {
    let condition = bin_eq(
        me_field("_tag"),
        lit_str(variant_name),
    );
    ASTNode::FunctionDeclaration {
        name: format!("is_{}", variant_name),
        params: vec![],
        body: vec![ASTNode::Return {
            value: Some(Box::new(condition)),
            span: Span::unknown(),
        }],
        is_static: false,
        is_override: false,
        span: Span::unknown(),
    }
}

/// Build as_* method: as_Ok() { return me.value }
/// For multi-field variants, returns first field (simplified for MVP)
fn build_enum_as_method(variant_name: &str, fields: &[String]) -> ASTNode {
    let first_field = fields.first().unwrap_or(&"value".to_string()).clone();
    ASTNode::FunctionDeclaration {
        name: format!("as_{}", variant_name),
        params: vec![],
        body: vec![ASTNode::Return {
            value: Some(Box::new(me_field(&first_field))),
            span: Span::unknown(),
        }],
        is_static: false,
        is_override: false,
        span: Span::unknown(),
    }
}

/// Build constructor method: Ok(v) { local r = new ResultBox(); r._tag = "Ok"; r.value = v; return r }
fn build_enum_constructor(enum_name: &str, variant: &nyash_rust::ast::EnumVariant) -> ASTNode {
    let box_name = format!("{}Box", enum_name);
    let inst_var = "r".to_string();

    let mut body: Vec<ASTNode> = Vec::new();

    // local r = new ResultBox()
    body.push(ASTNode::Local {
        variables: vec![inst_var.clone()],
        initial_values: vec![Some(Box::new(ASTNode::New {
            class: box_name,
            arguments: vec![],
            type_arguments: vec![],
            span: Span::unknown(),
        }))],
        span: Span::unknown(),
    });

    // r._tag = "Ok"
    body.push(ASTNode::Assignment {
        target: Box::new(var_field(&inst_var, "_tag")),
        value: Box::new(lit_str(&variant.name)),
        span: Span::unknown(),
    });

    // r.value = v (for each field)
    for field in &variant.fields {
        body.push(ASTNode::Assignment {
            target: Box::new(var_field(&inst_var, field)),
            value: Box::new(ASTNode::Variable {
                name: field.clone(),
                span: Span::unknown(),
            }),
            span: Span::unknown(),
        });
    }

    // return r
    body.push(ASTNode::Return {
        value: Some(Box::new(ASTNode::Variable {
            name: inst_var,
            span: Span::unknown(),
        })),
        span: Span::unknown(),
    });

    ASTNode::FunctionDeclaration {
        name: variant.name.clone(),
        params: variant.fields.clone(),
        body,
        is_static: true,
        is_override: false,
        span: Span::unknown(),
    }
}
