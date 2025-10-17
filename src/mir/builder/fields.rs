// Field access and assignment lowering
use super::{EffectMask, MirInstruction, ValueId};
use crate::ast::ASTNode;
use crate::mir::slot_registry;

impl super::MirBuilder {
    /// Build field access: object.field
    pub(super) fn build_field_access(
        &mut self,
        object: ASTNode,
        field: String,
    ) -> Result<ValueId, String> {
        // Guard: static box does not support `me.field` access
        if let Some(cls) = self.current_static_box.clone() {
            let is_box_self = match &object {
                ASTNode::Variable { name, .. } => {
                    let alias_alias = format!("{}_{}", cls, cls);
                    name == "me" || name == &cls || name == &alias_alias
                }
                ASTNode::Me { .. } => true,
                _ => false,
            };
            if is_box_self {
                return Err(format!(
                    "Static box field access is not supported: use methods or instance boxes (found `me.{}`)",
                    field
                ));
            }
        }
        let object_clone = object.clone();
        let object_value = self.build_expression(object.clone())?;
        let object_value = self.local_field_base(object_value);

        // Unified members: if object class is known and has a synthetic getter for `field`,
        // rewrite to method call `__get_<field>()`.
        if let Some(class_name) = self.origin_get(object_value) {
            if let Some(map) = self.property_getters_by_box.get(class_name) {
                if let Some(kind) = map.get(&field) {
                    let mname = match kind {
                        super::PropertyKind::Computed => format!("__get_{}", field),
                        super::PropertyKind::Once => format!("__get_once_{}", field),
                        super::PropertyKind::BirthOnce => format!("__get_birth_{}", field),
                    };
                    return self.build_method_call(object_clone, mname, vec![]);
                }
            }
        }

        // Emit: field name const (boxed)
        let field_name_id = crate::mir::builder::emission::constant::emit_string(self, field.clone());
        // Finalize operands (base + args) in current block
        let mut base = object_value;
        let mut args_vec = vec![field_name_id];
        crate::mir::builder::ssa::local::finalize_field_base_and_args(self, &mut base, &mut args_vec);
        // BoxCall: getField(name)
        let field_val = self.safe_next_value();
        self.emit_instruction(MirInstruction::BoxCall {
            dst: Some(field_val),
            box_val: base,
            method: "getField".to_string(),
            method_id: slot_registry::resolve_slot_by_type_name("InstanceBox", "getField"),
            args: args_vec,
            effects: EffectMask::READ,
        })?;

        // Propagate recorded origin class for this field if any (FieldOriginRegistryBox)
        let base_cls_hint = self.origin_get(object_value).map(|s| s.to_string());
        if let Some(class_name) = self
            .field_origin_registry
            .infer_field_origin(object_value, &field, base_cls_hint.as_deref())
        {
            if super::utils::builder_debug_enabled() || std::env::var("NYASH_BUILDER_DEBUG").ok().as_deref() == Some("1") {
                if let Some(ref base_cls) = base_cls_hint {
                    super::utils::builder_debug_log(&format!("field-origin hit: base={} .{} -> {}", base_cls, field, class_name));
                }
            }
            self.origin_register(field_val, class_name);
        }

        // If base is a known newbox and field is weak, emit WeakLoad (+ optional barrier)
        let mut inferred_class: Option<String> =
            self.origin_get(object_value).map(|s| s.to_string());
        if inferred_class.is_none() {
            if let ASTNode::FieldAccess {
                object: inner_obj,
                field: inner_field,
                ..
            } = object_clone
            {
                if let Ok(base_id) = self.build_expression(*inner_obj.clone()) {
                    // Use FieldOriginRegistryBox for nested field lookup
                    if let Some(cls) = self.field_origin_registry.infer_field_origin(base_id, &inner_field, None) {
                        inferred_class = Some(cls);
                    }
                }
            }
        }
        if let Some(class_name) = inferred_class {
            if self.weak_field_registry.is_weak_field(&class_name, &field) {
                let loaded = self.emit_weak_load(field_val)?;
                let _ = self.emit_barrier_read(loaded);
                return Ok(loaded);
            }
        }

        // Correctness-first: slotify field values so they have block-local defs
        // and participate in PHI merges when reused across branches.
        let pinned = self.pin_to_slot(field_val, "@field")?;
        Ok(pinned)
    }

    /// Build field assignment: object.field = value
    pub(super) fn build_field_assignment(
        &mut self,
        object: ASTNode,
        field: String,
        value: ASTNode,
    ) -> Result<ValueId, String> {
        // Guard: static box does not support `me.field = ...` assignment
        if let Some(cls) = self.current_static_box.clone() {
            let is_box_self = match &object {
                ASTNode::Variable { name, .. } => {
                    let alias_alias = format!("{}_{}", cls, cls);
                    name == "me" || name == &cls || name == &alias_alias
                }
                ASTNode::Me { .. } => true,
                _ => false,
            };
            if is_box_self {
                return Err(format!(
                    "Static box field assignment is not supported: use methods or instance boxes (found `me.{}`)",
                    field
                ));
            }
        }
        let object_value = self.build_expression(object)?;
        let object_value = self.local_field_base(object_value);
        let mut value_result = self.build_expression(value)?;
        // LocalSSA: argument in-block (optional safety)
        value_result = self.local_arg(value_result);

        // If base is known and field is weak, create WeakRef before store
        if let Some(class_name) = self.origin_get(object_value).map(|s| s.to_string()) {
            if self.weak_field_registry.is_weak_field(&class_name, &field) {
                value_result = self.emit_weak_new(value_result)?;
            }
        }

        // Emit: field name const
        let field_name_id = crate::mir::builder::emission::constant::emit_string(self, field.clone());
        // Finalize operands (base + args) in current block
        let mut base = object_value;
        let mut args_vec = vec![field_name_id, value_result];
        crate::mir::builder::ssa::local::finalize_field_base_and_args(self, &mut base, &mut args_vec);
        // Set the field via BoxCall: setField(name, value)
        self.emit_instruction(MirInstruction::BoxCall {
            dst: None,
            box_val: base,
            method: "setField".to_string(),
            method_id: slot_registry::resolve_slot_by_type_name("InstanceBox", "setField"),
            args: args_vec,
            effects: EffectMask::WRITE,
        })?;

        // Write barrier if weak field
        if let Some(class_name) = self.origin_get(object_value).map(|s| s.to_string()) {
            if self.weak_field_registry.is_weak_field(&class_name, &field) {
                let _ = self.emit_barrier_write(value_result);
            }
        }

        // Record origin class for this field value if known (FieldOriginRegistryBox)
        if let Some(val_cls) = self.origin_get(value_result).map(|s| s.to_string()) {
            // Register value-level field origin
            self.field_origin_registry
                .register_value_field(object_value, field.clone(), val_cls.clone());
            // Also register box-level mapping if base object class is known
            if let Some(base_cls) = self.origin_get(object_value).map(|s| s.to_string()) {
                self.field_origin_registry
                    .register_box_field(base_cls, field.clone(), val_cls);
            }
        }

        Ok(value_result)
    }
}
