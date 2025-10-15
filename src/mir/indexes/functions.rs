use std::collections::HashMap;

use crate::mir::MirFunction;

/// Lightweight function index over a MirModule's function table.
/// Provides common queries used across builder/diagnostics.
pub struct FunctionIndex<'a> {
    names: &'a HashMap<String, MirFunction>,
}

pub enum TailQueryResult {
    None,
    Unique(String),
    Ambiguous(Vec<String>),
}

impl<'a> FunctionIndex<'a> {
    pub fn new(module: &'a crate::mir::MirModule) -> Self { Self { names: &module.functions } }
    pub fn from_map(map: &'a HashMap<String, MirFunction>) -> Self { Self { names: map } }

    pub fn contains(&self, name: &str) -> bool {
        self.names.contains_key(name)
    }

    /// Return the canonical name if present.
    pub fn exact(&self, name: &str) -> Option<String> {
        if self.contains(name) { Some(name.to_string()) } else { None }
    }

    /// Tail-unique lookup.
    /// - class_opt: Some(class or alias prefix) to restrict candidates; None = any class
    /// - method: method name
    /// - arity: method arity
    pub fn tail_unique(&self, class_opt: Option<&str>, method: &str, arity: usize) -> TailQueryResult {
        let tail = format!(".{}{}", method, format!("/{}", arity));
        let mut cands: Vec<String> = Vec::new();
        match class_opt {
            Some(cls) => {
                for k in self.names.keys() {
                    if k.ends_with(&tail) && (k.starts_with(&format!("{}.", cls)) || k.starts_with(&format!("{}_", cls))) {
                        cands.push(k.clone());
                    }
                }
            }
            None => {
                for k in self.names.keys() {
                    if k.ends_with(&tail) { cands.push(k.clone()); }
                }
            }
        }
        match cands.len() {
            0 => TailQueryResult::None,
            1 => TailQueryResult::Unique(cands.remove(0)),
            _ => { cands.sort(); TailQueryResult::Ambiguous(cands) }
        }
    }
}

/// Prefer candidates that belong to the same box as current function name.
/// If exactly one candidate matches the `cur_box.` prefix, return that single-item list.
/// Otherwise, return the original candidates (cloned).
pub fn prefer_current_box(cur_fn: &str, cands: &[String]) -> Vec<String> {
    let cur_box = cur_fn.split('.').next().unwrap_or("");
    let scoped: Vec<String> = cands
        .iter()
        .filter(|k| k.starts_with(&format!("{}.", cur_box)))
        .cloned()
        .collect();
    if scoped.len() == 1 { scoped } else { cands.to_vec() }
}
