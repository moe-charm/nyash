// Auto-generated from grammar/unified-grammar.toml
pub static KEYWORDS: &[(&str, &str)] = &[
    ("me", "ME"),
    ("from", "FROM"),
    ("loop", "LOOP"),
];
pub static OPERATORS_ADD_COERCION: &str = "string_priority";
pub static OPERATORS_SUB_COERCION: &str = "numeric_only";
pub static OPERATORS_MUL_COERCION: &str = "numeric_only";
pub static OPERATORS_DIV_COERCION: &str = "numeric_only";
pub static OPERATORS_ADD_RULES: &[(&str, &str, &str, &str)] = &[
    ("String", "String", "String", "concat"),
    ("String", "Integer", "String", "concat"),
    ("Integer", "String", "String", "concat"),
    ("String", "Bool", "String", "concat"),
    ("Bool", "String", "String", "concat"),
    ("String", "Other", "String", "concat"),
    ("Other", "String", "String", "concat"),
    ("Integer", "Integer", "Integer", "add_i64"),
    ("Float", "Float", "Float", "add_f64"),
];
pub static OPERATORS_SUB_RULES: &[(&str, &str, &str, &str)] = &[
    ("Integer", "Integer", "Integer", "sub_i64"),
    ("Float", "Float", "Float", "sub_f64"),
];
pub static OPERATORS_MUL_RULES: &[(&str, &str, &str, &str)] = &[
    ("Integer", "Integer", "Integer", "mul_i64"),
    ("Float", "Float", "Float", "mul_f64"),
];
pub static OPERATORS_DIV_RULES: &[(&str, &str, &str, &str)] = &[
    ("Integer", "Integer", "Integer", "div_i64"),
    ("Float", "Float", "Float", "div_f64"),
];
pub fn lookup_keyword(word: &str) -> Option<&'static str> {
    for (k, t) in KEYWORDS {
        if *k == word { return Some(*t); }
    }
    None
}

pub static SYNTAX_ALLOWED_STATEMENTS: &[&str] = &[
    "box",
    "global",
    "function",
    "static",
    "if",
    "loop",
    "break",
    "return",
    "print",
    "nowait",
    "include",
    "import",
    "local",
    "outbox",
    "try",
    "throw",
    "using",
    "from",
];
pub static SYNTAX_ALLOWED_BINOPS: &[&str] = &[
    "add",
    "sub",
    "mul",
    "div",
    "and",
    "or",
    "eq",
    "ne",
];
