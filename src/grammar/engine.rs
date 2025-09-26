use once_cell::sync::Lazy;

use super::generated;

pub struct UnifiedGrammarEngine;

impl UnifiedGrammarEngine {
    pub fn load() -> Self {
        Self
    }
    pub fn is_keyword_str(&self, word: &str) -> Option<&'static str> {
        generated::lookup_keyword(word)
    }
    pub fn add_coercion_strategy(&self) -> &'static str {
        generated::OPERATORS_ADD_COERCION
    }
    pub fn add_rules(&self) -> &'static [(&'static str, &'static str, &'static str, &'static str)] {
        generated::OPERATORS_ADD_RULES
    }
    pub fn decide_add_result(
        &self,
        left_ty: &str,
        right_ty: &str,
    ) -> Option<(&'static str, &'static str)> {
        for (l, r, res, act) in self.add_rules() {
            if *l == left_ty && *r == right_ty {
                return Some((*res, *act));
            }
        }
        None
    }

    pub fn sub_coercion_strategy(&self) -> &'static str {
        generated::OPERATORS_SUB_COERCION
    }
    pub fn sub_rules(&self) -> &'static [(&'static str, &'static str, &'static str, &'static str)] {
        generated::OPERATORS_SUB_RULES
    }
    pub fn decide_sub_result(
        &self,
        left_ty: &str,
        right_ty: &str,
    ) -> Option<(&'static str, &'static str)> {
        for (l, r, res, act) in self.sub_rules() {
            if *l == left_ty && *r == right_ty {
                return Some((*res, *act));
            }
        }
        None
    }

    pub fn mul_coercion_strategy(&self) -> &'static str {
        generated::OPERATORS_MUL_COERCION
    }
    pub fn mul_rules(&self) -> &'static [(&'static str, &'static str, &'static str, &'static str)] {
        generated::OPERATORS_MUL_RULES
    }
    pub fn decide_mul_result(
        &self,
        left_ty: &str,
        right_ty: &str,
    ) -> Option<(&'static str, &'static str)> {
        for (l, r, res, act) in self.mul_rules() {
            if *l == left_ty && *r == right_ty {
                return Some((*res, *act));
            }
        }
        None
    }

    pub fn div_coercion_strategy(&self) -> &'static str {
        generated::OPERATORS_DIV_COERCION
    }
    pub fn div_rules(&self) -> &'static [(&'static str, &'static str, &'static str, &'static str)] {
        generated::OPERATORS_DIV_RULES
    }
    pub fn decide_div_result(
        &self,
        left_ty: &str,
        right_ty: &str,
    ) -> Option<(&'static str, &'static str)> {
        for (l, r, res, act) in self.div_rules() {
            if *l == left_ty && *r == right_ty {
                return Some((*res, *act));
            }
        }
        None
    }
}

pub static ENGINE: Lazy<UnifiedGrammarEngine> = Lazy::new(UnifiedGrammarEngine::load);

pub fn get() -> &'static UnifiedGrammarEngine {
    &ENGINE
}

// --- Syntax rule helpers (generated-backed) ---
impl UnifiedGrammarEngine {
    pub fn syntax_is_allowed_statement(&self, keyword: &str) -> bool {
        super::generated::SYNTAX_ALLOWED_STATEMENTS
            .iter()
            .any(|k| *k == keyword)
    }
    pub fn syntax_is_allowed_binop(&self, op: &str) -> bool {
        super::generated::SYNTAX_ALLOWED_BINOPS
            .iter()
            .any(|k| *k == op)
    }
}
