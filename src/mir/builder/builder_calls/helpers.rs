// Helper functions for call building
use super::super::{Effect, EffectMask, MirBuilder, MirType, ValueId};
use crate::ast::ASTNode;
use crate::mir::builder::calls::special_handlers;

impl MirBuilder {
    /// Annotate a call result `dst` with the return type and origin if the callee
    /// is a known user/static function in the current module.
    pub(in super::super) fn annotate_call_result_from_func_name<S: AsRef<str>>(&mut self, dst: ValueId, func_name: S) {
        let name = func_name.as_ref();
        // 1) Prefer module signature when available
        if let Some(ref module) = self.current_module {
            if let Some(func) = module.functions.get(name) {
                let mut ret = func.signature.return_type.clone();
                // Targeted stabilization: JsonParser.parse/1 should produce JsonNode
                // If signature is Unknown/Void, normalize to Box("JsonNode")
                if name == "JsonParser.parse/1" {
                    if matches!(ret, MirType::Unknown | MirType::Void) {
                        ret = MirType::Box("JsonNode".into());
                    }
                }
                // Token path: JsonParser.current_token/0 should produce JsonToken
                if name == "JsonParser.current_token/0" {
                    if matches!(ret, MirType::Unknown | MirType::Void) {
                        ret = MirType::Box("JsonToken".into());
                    }
                }
                // Parser factory: JsonParserModule.create_parser/0 returns JsonParser
                if name == "JsonParserModule.create_parser/0" {
                    // Normalize to Known Box(JsonParser)
                    ret = MirType::Box("JsonParser".into());
                }
                self.value_types.insert(dst, ret.clone());
                if let MirType::Box(bx) = ret {
                    self.origin_register(dst, bx);
                    if super::super::utils::builder_debug_enabled() || std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
                        let bx = self.origin_get(dst).unwrap_or("-");
                        super::super::utils::builder_debug_log(&format!("annotate call dst={} from {} -> Box({})", dst.0, name, bx));
                    }
                }
                return;
            }
        }
        // 2) No module signature—apply minimal heuristic for known functions
        if name == "JsonParser.parse/1" {
            let ret = MirType::Box("JsonNode".into());
            self.value_types.insert(dst, ret.clone());
            if let MirType::Box(bx) = ret { self.origin_register(dst, bx); }
            if super::super::utils::builder_debug_enabled() || std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
                super::super::utils::builder_debug_log(&format!("annotate call (fallback) dst={} from {} -> Box(JsonNode)", dst.0, name));
            }
        } else if name == "JsonParser.current_token/0" {
            let ret = MirType::Box("JsonToken".into());
            self.value_types.insert(dst, ret.clone());
            if let MirType::Box(bx) = ret { self.origin_register(dst, bx); }
            if super::super::utils::builder_debug_enabled() || std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
                super::super::utils::builder_debug_log(&format!("annotate call (fallback) dst={} from {} -> Box(JsonToken)", dst.0, name));
            }
        } else if name == "JsonTokenizer.tokenize/0" {
            // Tokenize returns an ArrayBox of tokens
            let ret = MirType::Box("ArrayBox".into());
            self.value_types.insert(dst, ret.clone());
            if let MirType::Box(bx) = ret { self.origin_register(dst, bx); }
            if super::super::utils::builder_debug_enabled() || std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
                super::super::utils::builder_debug_log(&format!("annotate call (fallback) dst={} from {} -> Box(ArrayBox)", dst.0, name));
            }
        } else if name == "JsonParserModule.create_parser/0" {
            // Fallback path for parser factory
            let ret = MirType::Box("JsonParser".into());
            self.value_types.insert(dst, ret.clone());
            if let MirType::Box(bx) = ret { self.origin_register(dst, bx); }
            if super::super::utils::builder_debug_enabled() || std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
                super::super::utils::builder_debug_log(&format!("annotate call (fallback) dst={} from {} -> Box(JsonParser)", dst.0, name));
            }
        } else {
            // Generic tiny whitelist for known primitive-like utilities (spec unchanged)
            crate::mir::builder::types::annotation::annotate_from_function(self, dst, name);
        }
    }

    // Map a user-facing type name to MIR type
    pub(in super::super) fn parse_type_name_to_mir(name: &str) -> MirType {
        special_handlers::parse_type_name_to_mir(name)
    }

    // Extract string literal from AST node if possible
    pub(in super::super) fn extract_string_literal(node: &ASTNode) -> Option<String> {
        special_handlers::extract_string_literal(node)
    }

    // Build from expression: from Parent.method(arguments)
    pub(in super::super) fn build_from_expression(
        &mut self,
        parent: String,
        method: String,
        arguments: Vec<ASTNode>,
    ) -> Result<ValueId, String> {
        // Lower arguments first
        let mut arg_values = Vec::new();
        for arg in arguments { arg_values.push(self.build_expression(arg)?); }

        // Prefer static ModuleFunction when the current module defines it (BoxName.method/Arity)
        if let Some(ref module) = self.current_module {
            let func_name = format!("{}.{}{}", parent, method, format!("/{}", arg_values.len()));
            if module.functions.contains_key(&func_name) {
                let dst = self.safe_next_value();
                let fun_val = crate::mir::builder::name_const::make_name_const_result(self, &func_name)?;
                self.emit_call_with_guard(
                    Some(dst),
                    fun_val,
                    crate::mir::Callee::ModuleFunction(func_name.clone()),
                    arg_values,
                    EffectMask::READ.add(Effect::ReadHeap),
                )?;
                // Minimal type/origin annotation when available
                self.annotate_call_result_from_func_name(dst, &func_name);
                return Ok(dst);
            }
        }

        // Fallback: treat as Box/Plugin call using parent as box type name string
        let parent_value = crate::mir::builder::emission::constant::emit_string(self, parent);
        let result_id = self.safe_next_value();
        self.emit_box_or_plugin_call(
            Some(result_id),
            parent_value,
            method,
            None,
            arg_values,
            EffectMask::READ.add(Effect::ReadHeap),
            false,
        )?;
        Ok(result_id)
    }
}
