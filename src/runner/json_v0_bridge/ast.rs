use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct ProgramV0 {
    pub(super) version: i32,
    pub(super) kind: String,
    pub(super) body: Vec<StmtV0>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub(super) enum StmtV0 {
    Return {
        expr: ExprV0,
    },
    Extern {
        iface: String,
        method: String,
        args: Vec<ExprV0>,
    },
    Expr {
        expr: ExprV0,
    },
    Local {
        name: String,
        expr: ExprV0,
    },
    If {
        cond: ExprV0,
        then: Vec<StmtV0>,
        #[serde(rename = "else", default)]
        r#else: Option<Vec<StmtV0>>,
    },
    Loop {
        cond: ExprV0,
        body: Vec<StmtV0>,
    },
    Break,
    Continue,
    Try {
        #[serde(rename = "try")]
        try_body: Vec<StmtV0>,
        #[serde(default)]
        catches: Vec<CatchV0>,
        #[serde(default)]
        finally: Vec<StmtV0>,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub(super) struct CatchV0 {
    #[serde(rename = "param", default)]
    pub(super) param: Option<String>,
    #[serde(rename = "typeHint", default)]
    pub(super) type_hint: Option<String>,
    #[serde(default)]
    pub(super) body: Vec<StmtV0>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub(super) enum ExprV0 {
    Int {
        value: serde_json::Value,
    },
    Str {
        value: String,
    },
    Bool {
        value: bool,
    },
    Binary {
        op: String,
        lhs: Box<ExprV0>,
        rhs: Box<ExprV0>,
    },
    Extern {
        iface: String,
        method: String,
        args: Vec<ExprV0>,
    },
    Compare {
        op: String,
        lhs: Box<ExprV0>,
        rhs: Box<ExprV0>,
    },
    Logical {
        op: String,
        lhs: Box<ExprV0>,
        rhs: Box<ExprV0>,
    },
    Call {
        name: String,
        args: Vec<ExprV0>,
    },
    Method {
        recv: Box<ExprV0>,
        method: String,
        args: Vec<ExprV0>,
    },
    New {
        class: String,
        args: Vec<ExprV0>,
    },
    Var {
        name: String,
    },
    Throw {
        expr: Box<ExprV0>,
    },
}
