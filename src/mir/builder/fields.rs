// Field access and assignment lowering
use super::{ConstValue, EffectMask, MirInstruction, ValueId};
use crate::ast::ASTNode;
use crate::mir::slot_registry;

impl super::MirBuilder {
    /// Build field access: object.field
    pub(super) fn build_field_access(
        &mut self,
        object: ASTNode,
        field: String,
    ) -> Result<ValueId, String> {
        let object_clone = object.clone();
        let object_value = self.build_expression(object)?;

        // Emit: field name const
        let field_name_id = self.value_gen.next();
        self.emit_instruction(MirInstruction::Const {
            dst: field_name_id,
            value: ConstValue::String(field.clone()),
        })?;
        // BoxCall: getField(name)
        let field_val = self.value_gen.next();
        self.emit_instruction(MirInstruction::BoxCall {
            dst: Some(field_val),
            box_val: object_value,
            method: "getField".to_string(),
            method_id: slot_registry::resolve_slot_by_type_name("InstanceBox", "getField"),
            args: vec![field_name_id],
            effects: EffectMask::READ,
        })?;

        // Propagate recorded origin class for this field if any
        if let Some(class_name) = self
            .field_origin_class
            .get(&(object_value, field.clone()))
            .cloned()
        {
            self.value_origin_newbox.insert(field_val, class_name);
        }

        // If base is a known newbox and field is weak, emit WeakLoad (+ optional barrier)
        let mut inferred_class: Option<String> =
            self.value_origin_newbox.get(&object_value).cloned();
        if inferred_class.is_none() {
            if let ASTNode::FieldAccess {
                object: inner_obj,
                field: inner_field,
                ..
            } = object_clone
            {
                if let Ok(base_id) = self.build_expression(*inner_obj.clone()) {
                    if let Some(cls) = self
                        .field_origin_class
                        .get(&(base_id, inner_field))
                        .cloned()
                    {
                        inferred_class = Some(cls);
                    }
                }
            }
        }
        if let Some(class_name) = inferred_class {
            if let Some(weak_set) = self.weak_fields_by_box.get(&class_name) {
                if weak_set.contains(&field) {
                    let loaded = self.emit_weak_load(field_val)?;
                    let _ = self.emit_barrier_read(loaded);
                    return Ok(loaded);
                }
            }
        }

        Ok(field_val)
    }

    /// Build field assignment: object.field = value
    pub(super) fn build_field_assignment(
        &mut self,
        object: ASTNode,
        field: String,
        value: ASTNode,
    ) -> Result<ValueId, String> {
        let object_value = self.build_expression(object)?;
        let mut value_result = self.build_expression(value)?;

        // If base is known and field is weak, create WeakRef before store
        if let Some(class_name) = self.value_origin_newbox.get(&object_value).cloned() {
            if let Some(weak_set) = self.weak_fields_by_box.get(&class_name) {
                if weak_set.contains(&field) {
                    value_result = self.emit_weak_new(value_result)?;
                }
            }
        }

        // Emit: field name const
        let field_name_id = self.value_gen.next();
        self.emit_instruction(MirInstruction::Const {
            dst: field_name_id,
            value: ConstValue::String(field.clone()),
        })?;
        // Set the field via BoxCall: setField(name, value)
        self.emit_instruction(MirInstruction::BoxCall {
            dst: None,
            box_val: object_value,
            method: "setField".to_string(),
            method_id: slot_registry::resolve_slot_by_type_name("InstanceBox", "setField"),
            args: vec![field_name_id, value_result],
            effects: EffectMask::WRITE,
        })?;

        // Write barrier if weak field
        if let Some(class_name) = self.value_origin_newbox.get(&object_value).cloned() {
            if let Some(weak_set) = self.weak_fields_by_box.get(&class_name) {
                if weak_set.contains(&field) {
                    let _ = self.emit_barrier_write(value_result);
                }
            }
        }

        // Record origin class for this field value if known
        if let Some(class_name) = self.value_origin_newbox.get(&value_result).cloned() {
            self.field_origin_class
                .insert((object_value, field.clone()), class_name);
        }

        Ok(value_result)
    }
}
