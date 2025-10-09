//! JSON test argument parsing for test harness
//!
//! Parses NYASH_TEST_ARGS_JSON environment variable to provide
//! runtime arguments for test functions.

use nyash_rust::ast::{ASTNode as A, LiteralValue, Span};
use std::collections::HashMap;

/// Test execution plan (function/method call with optional setup)
#[derive(Clone)]
pub struct TestPlan {
    pub label: String,
    pub setup: Option<nyash_rust::ASTNode>,
    pub call: nyash_rust::ASTNode,
}

/// Instance construction specification
#[derive(Clone, Default)]
pub struct InstanceSpec {
    pub ctor: String,
    pub args: Vec<nyash_rust::ASTNode>,
    pub type_args: Vec<String>,
}

/// Test argument specification (args + optional instance)
#[derive(Clone, Default)]
pub struct TestArgSpec {
    pub args: Vec<nyash_rust::ASTNode>,
    pub instance: Option<InstanceSpec>,
}

fn json_err(msg: &str) {
    eprintln!("[macro][test][args] {}", msg);
}

fn json_to_ast(v: &serde_json::Value) -> Result<nyash_rust::ASTNode, String> {
    match v {
        serde_json::Value::String(st) => Ok(A::Literal {
            value: LiteralValue::String(st.clone()),
            span: Span::unknown(),
        }),
        serde_json::Value::Bool(b) => Ok(A::Literal {
            value: LiteralValue::Bool(*b),
            span: Span::unknown(),
        }),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(A::Literal {
                    value: LiteralValue::Integer(i),
                    span: Span::unknown(),
                })
            } else if let Some(f) = n.as_f64() {
                Ok(A::Literal {
                    value: LiteralValue::Float(f),
                    span: Span::unknown(),
                })
            } else {
                Err("unsupported number literal".into())
            }
        }
        serde_json::Value::Null => Ok(A::Literal {
            value: LiteralValue::Null,
            span: Span::unknown(),
        }),
        serde_json::Value::Array(elems) => {
            // Treat nested arrays as ArrayLiteral by default
            let mut out = Vec::with_capacity(elems.len());
            for x in elems {
                out.push(json_to_ast(x)?);
            }
            Ok(A::ArrayLiteral {
                elements: out,
                span: Span::unknown(),
            })
        }
        serde_json::Value::Object(obj) => {
            // Typed shorthands accepted: {i:1}|{int:1}, {f:1.2}|{float:1.2}, {s:"x"}|{string:"x"}, {b:true}|{bool:true}
            if let Some(v) = obj.get("i").or_else(|| obj.get("int")) {
                return json_to_ast(v);
            }
            if let Some(v) = obj.get("f").or_else(|| obj.get("float")) {
                return json_to_ast(v);
            }
            if let Some(v) = obj.get("s").or_else(|| obj.get("string")) {
                return json_to_ast(v);
            }
            if let Some(v) = obj.get("b").or_else(|| obj.get("bool")) {
                return json_to_ast(v);
            }
            if let Some(map) = obj.get("map") {
                if let Some(mo) = map.as_object() {
                    let mut ents: Vec<(String, nyash_rust::ASTNode)> =
                        Vec::with_capacity(mo.len());
                    for (k, vv) in mo {
                        ents.push((k.clone(), json_to_ast(vv)?));
                    }
                    return Ok(A::MapLiteral {
                        entries: ents,
                        span: Span::unknown(),
                    });
                } else {
                    return Err("map must be an object".into());
                }
            }
            if let Some(arr) = obj.get("array") {
                if let Some(va) = arr.as_array() {
                    let mut out = Vec::with_capacity(va.len());
                    for x in va {
                        out.push(json_to_ast(x)?);
                    }
                    return Ok(A::ArrayLiteral {
                        elements: out,
                        span: Span::unknown(),
                    });
                } else {
                    return Err("array must be an array".into());
                }
            }
            if let Some(name) = obj.get("var").and_then(|v| v.as_str()) {
                return Ok(A::Variable {
                    name: name.to_string(),
                    span: Span::unknown(),
                });
            }
            if let Some(name) = obj.get("call").and_then(|v| v.as_str()) {
                let mut args: Vec<A> = Vec::new();
                if let Some(va) = obj.get("args").and_then(|v| v.as_array()) {
                    for x in va {
                        args.push(json_to_ast(x)?);
                    }
                }
                return Ok(A::FunctionCall {
                    name: name.to_string(),
                    arguments: args,
                    span: Span::unknown(),
                });
            }
            if let Some(method) = obj.get("method").and_then(|v| v.as_str()) {
                let objv = obj
                    .get("object")
                    .ok_or_else(|| "method requires 'object'".to_string())?;
                let object = json_to_ast(objv)?;
                let mut args: Vec<A> = Vec::new();
                if let Some(va) = obj.get("args").and_then(|v| v.as_array()) {
                    for x in va {
                        args.push(json_to_ast(x)?);
                    }
                }
                return Ok(A::MethodCall {
                    object: Box::new(object),
                    method: method.to_string(),
                    arguments: args,
                    span: Span::unknown(),
                });
            }
            if let Some(bx) = obj.get("box").and_then(|v| v.as_str()) {
                let mut args: Vec<A> = Vec::new();
                if let Some(va) = obj.get("args").and_then(|v| v.as_array()) {
                    for x in va {
                        args.push(json_to_ast(x)?);
                    }
                }
                let type_args: Vec<String> = obj
                    .get("type_args")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let ctor = obj.get("ctor").and_then(|v| v.as_str()).unwrap_or("new");
                if ctor == "new" {
                    return Ok(A::New {
                        class: bx.to_string(),
                        arguments: args,
                        type_arguments: type_args,
                        span: Span::unknown(),
                    });
                } else if ctor == "birth" {
                    return Ok(A::MethodCall {
                        object: Box::new(A::Variable {
                            name: bx.to_string(),
                            span: Span::unknown(),
                        }),
                        method: "birth".into(),
                        arguments: args,
                        span: Span::unknown(),
                    });
                } else {
                    return Err(format!(
                        "unknown ctor '{}', expected 'new' or 'birth'",
                        ctor
                    ));
                }
            }
            Err("unknown object mapping for AST".into())
        }
    }
}

fn parse_test_arg_spec(v: &serde_json::Value) -> Option<TestArgSpec> {
    match v {
        serde_json::Value::Array(arr) => {
            let mut out: Vec<nyash_rust::ASTNode> = Vec::new();
            for a in arr {
                match json_to_ast(a) {
                    Ok(n) => out.push(n),
                    Err(e) => {
                        json_err(&format!("args element error: {}", e));
                        return None;
                    }
                }
            }
            Some(TestArgSpec {
                args: out,
                instance: None,
            })
        }
        serde_json::Value::Object(obj) => {
            let mut spec = TestArgSpec::default();
            if let Some(a) = obj.get("args").and_then(|v| v.as_array()) {
                let mut out: Vec<nyash_rust::ASTNode> = Vec::new();
                for x in a {
                    match json_to_ast(x) {
                        Ok(n) => out.push(n),
                        Err(e) => {
                            json_err(&format!("args element error: {}", e));
                            return None;
                        }
                    }
                }
                spec.args = out;
            }
            if let Some(inst) = obj.get("instance").and_then(|v| v.as_object()) {
                let ctor = inst
                    .get("ctor")
                    .and_then(|v| v.as_str())
                    .unwrap_or("new")
                    .to_string();
                let type_args: Vec<String> = inst
                    .get("type_args")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let mut args: Vec<nyash_rust::ASTNode> = Vec::new();
                if let Some(va) = inst.get("args").and_then(|v| v.as_array()) {
                    for x in va {
                        match json_to_ast(x) {
                            Ok(n) => args.push(n),
                            Err(e) => {
                                json_err(&format!("instance.args element error: {}", e));
                                return None;
                            }
                        }
                    }
                }
                spec.instance = Some(InstanceSpec {
                    ctor,
                    args,
                    type_args,
                });
            }
            Some(spec)
        }
        _ => {
            json_err("test value must be array or object");
            None
        }
    }
}

/// Parse test args map from NYASH_TEST_ARGS_JSON environment variable
pub fn parse_args_map_from_env() -> Option<HashMap<String, TestArgSpec>> {
    if let Ok(s) = std::env::var("NYASH_TEST_ARGS_JSON") {
        if s.trim().is_empty() {
            return None;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
            let mut map = HashMap::new();
            if let Some(obj) = v.as_object() {
                for (k, vv) in obj {
                    if let Some(spec) = parse_test_arg_spec(vv) {
                        map.insert(k.clone(), spec);
                    }
                }
                return Some(map);
            }
        }
    }
    None
}
