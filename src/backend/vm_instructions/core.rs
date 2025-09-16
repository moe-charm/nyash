use crate::backend::vm::ControlFlow;
use crate::backend::{VMError, VMValue, VM};
use crate::box_trait::{BoolBox, VoidBox};
use crate::boxes::ArrayBox;
use crate::mir::{
    BasicBlockId, BinaryOp, CompareOp, ConstValue, MirType, TypeOpKind, UnaryOp, ValueId,
};
use std::sync::Arc;

impl VM {
    // ---- Helpers (PIC/VTable bookkeeping) ----
    pub(super) fn build_pic_key(
        &self,
        recv: &VMValue,
        method: &str,
        method_id: Option<u16>,
    ) -> String {
        let label = self.cache_label_for_recv(recv);
        let ver = self.cache_version_for_label(&label);
        if let Some(mid) = method_id {
            format!("v{}:{}#{}", ver, label, mid)
        } else {
            format!("v{}:{}#{}", ver, label, method)
        }
    }
    pub(super) fn pic_record_hit(&mut self, key: &str) {
        use std::collections::hash_map::Entry;
        match self.boxcall_pic_hits.entry(key.to_string()) {
            Entry::Occupied(mut e) => {
                let v = e.get_mut();
                *v = v.saturating_add(1);
                if std::env::var("NYASH_VM_PIC_DEBUG").ok().as_deref() == Some("1") {
                    if *v == 8 || *v == 32 {
                        eprintln!("[PIC] Hot BoxCall site '{}' hits={} (skeleton)", key, v);
                    }
                }
            }
            Entry::Vacant(v) => {
                v.insert(1);
            }
        }
    }
    pub(super) fn pic_hits(&self, key: &str) -> u32 {
        *self.boxcall_pic_hits.get(key).unwrap_or(&0)
    }
    pub(super) fn build_vtable_key(
        &self,
        class_name: &str,
        method_id: u16,
        arity: usize,
    ) -> String {
        let label = format!("BoxRef:{}", class_name);
        let ver = self.cache_version_for_label(&label);
        format!(
            "VT@v{}:{}#{}{}",
            ver,
            class_name,
            method_id,
            format!("/{}", arity)
        )
    }
    pub(super) fn try_poly_pic(&mut self, pic_site_key: &str, recv: &VMValue) -> Option<String> {
        let label = self.cache_label_for_recv(recv);
        let ver = self.cache_version_for_label(&label);
        if let Some(entries) = self.boxcall_poly_pic.get_mut(pic_site_key) {
            if let Some(idx) = entries
                .iter()
                .position(|(l, v, _)| *l == label && *v == ver)
            {
                let entry = entries.remove(idx);
                entries.push(entry.clone());
                return Some(entry.2);
            }
        }
        None
    }
    pub(super) fn record_poly_pic(&mut self, pic_site_key: &str, recv: &VMValue, func_name: &str) {
        let label = self.cache_label_for_recv(recv);
        let ver = self.cache_version_for_label(&label);
        use std::collections::hash_map::Entry;
        match self.boxcall_poly_pic.entry(pic_site_key.to_string()) {
            Entry::Occupied(mut e) => {
                let v = e.get_mut();
                if let Some(idx) = v.iter().position(|(l, vv, _)| *l == label && *vv == ver) {
                    v.remove(idx);
                }
                if v.len() >= 4 {
                    v.remove(0);
                }
                v.push((label.clone(), ver, func_name.to_string()));
            }
            Entry::Vacant(v) => {
                v.insert(vec![(label.clone(), ver, func_name.to_string())]);
            }
        }
        if crate::config::env::vm_pic_stats() {
            if let Some(v) = self.boxcall_poly_pic.get(pic_site_key) {
                eprintln!(
                    "[PIC] site={} size={} last=({}, v{}) -> {}",
                    pic_site_key,
                    v.len(),
                    label,
                    ver,
                    func_name
                );
            }
        }
    }
    pub(super) fn cache_label_for_recv(&self, recv: &VMValue) -> String {
        match recv {
            VMValue::Integer(_) => "Int".to_string(),
            VMValue::Float(_) => "Float".to_string(),
            VMValue::Bool(_) => "Bool".to_string(),
            VMValue::String(_) => "String".to_string(),
            VMValue::Future(_) => "Future".to_string(),
            VMValue::Void => "Void".to_string(),
            VMValue::BoxRef(b) => format!("BoxRef:{}", b.type_name()),
        }
    }
    pub(super) fn cache_version_for_label(&self, label: &str) -> u32 {
        crate::runtime::cache_versions::get_version(label)
    }
    #[allow(dead_code)]
    pub fn bump_cache_version(&mut self, label: &str) {
        crate::runtime::cache_versions::bump_version(label)
    }

    // ---- Basics ----
    pub(crate) fn execute_const(
        &mut self,
        dst: ValueId,
        value: &ConstValue,
    ) -> Result<ControlFlow, VMError> {
        let vm_value = VMValue::from(value);
        self.set_value(dst, vm_value);
        Ok(ControlFlow::Continue)
    }
    pub(crate) fn execute_binop(
        &mut self,
        dst: ValueId,
        op: &BinaryOp,
        lhs: ValueId,
        rhs: ValueId,
    ) -> Result<ControlFlow, VMError> {
        match *op {
            BinaryOp::And | BinaryOp::Or => {
                if std::env::var("NYASH_VM_DEBUG_ANDOR").ok().as_deref() == Some("1") {
                    eprintln!("[VM] And/Or short-circuit path");
                }
                let left = self.get_value(lhs)?;
                let right = self.get_value(rhs)?;
                let lb = left.as_bool()?;
                let rb = right.as_bool()?;
                let out = match *op {
                    BinaryOp::And => lb && rb,
                    BinaryOp::Or => lb || rb,
                    _ => unreachable!(),
                };
                self.set_value(dst, VMValue::Bool(out));
                Ok(ControlFlow::Continue)
            }
            _ => {
                let left = self.get_value(lhs)?;
                let right = self.get_value(rhs)?;
                let result = self.execute_binary_op(op, &left, &right)?;
                self.set_value(dst, result);
                Ok(ControlFlow::Continue)
            }
        }
    }
    pub(crate) fn execute_unaryop(
        &mut self,
        dst: ValueId,
        op: &UnaryOp,
        operand: ValueId,
    ) -> Result<ControlFlow, VMError> {
        let operand_val = self.get_value(operand)?;
        let result = self.execute_unary_op(op, &operand_val)?;
        self.set_value(dst, result);
        Ok(ControlFlow::Continue)
    }
    pub(crate) fn execute_compare(
        &mut self,
        dst: ValueId,
        op: &CompareOp,
        lhs: ValueId,
        rhs: ValueId,
    ) -> Result<ControlFlow, VMError> {
        let debug_cmp = std::env::var("NYASH_VM_DEBUG").ok().as_deref() == Some("1")
            || std::env::var("NYASH_VM_DEBUG_CMP").ok().as_deref() == Some("1");
        if debug_cmp {
            eprintln!(
                "[VM] execute_compare enter op={:?} lhs={:?} rhs={:?}",
                op, lhs, rhs
            );
        }
        let mut left = self.get_value(lhs)?;
        let mut right = self.get_value(rhs)?;
        if debug_cmp {
            eprintln!(
                "[VM] execute_compare values: left={:?} right={:?}",
                left, right
            );
        }
        left = match left {
            VMValue::BoxRef(b) => {
                if let Some(ib) = b.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                    VMValue::Integer(ib.value)
                } else {
                    match b.to_string_box().value.trim().parse::<i64>() {
                        Ok(n) => VMValue::Integer(n),
                        Err(_) => VMValue::BoxRef(b),
                    }
                }
            }
            other => other,
        };
        right = match right {
            VMValue::BoxRef(b) => {
                if let Some(ib) = b.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                    VMValue::Integer(ib.value)
                } else {
                    match b.to_string_box().value.trim().parse::<i64>() {
                        Ok(n) => VMValue::Integer(n),
                        Err(_) => VMValue::BoxRef(b),
                    }
                }
            }
            other => other,
        };
        let result = self.execute_compare_op(op, &left, &right)?;
        self.set_value(dst, VMValue::Bool(result));
        Ok(ControlFlow::Continue)
    }
    pub(crate) fn execute_print(&self, value: ValueId) -> Result<ControlFlow, VMError> {
        let val = self.get_value(value)?;
        println!("{}", val.to_string());
        Ok(ControlFlow::Continue)
    }
    pub(crate) fn execute_jump(&self, target: BasicBlockId) -> Result<ControlFlow, VMError> {
        Ok(ControlFlow::Jump(target))
    }
    pub(crate) fn execute_branch(
        &self,
        condition: ValueId,
        then_bb: BasicBlockId,
        else_bb: BasicBlockId,
    ) -> Result<ControlFlow, VMError> {
        let cond_val = self.get_value(condition)?;
        let should_branch = match &cond_val {
            VMValue::Bool(b) => *b,
            VMValue::Void => false,
            VMValue::Integer(i) => *i != 0,
            VMValue::BoxRef(b) => {
                if let Some(bool_box) = b.as_any().downcast_ref::<BoolBox>() {
                    bool_box.value
                } else if b.as_any().downcast_ref::<VoidBox>().is_some() {
                    false
                } else {
                    return Err(VMError::TypeError(format!(
                        "Branch condition must be bool, void, or integer, got BoxRef({})",
                        b.type_name()
                    )));
                }
            }
            _ => {
                return Err(VMError::TypeError(format!(
                    "Branch condition must be bool, void, or integer, got {:?}",
                    cond_val
                )))
            }
        };
        Ok(ControlFlow::Jump(if should_branch {
            then_bb
        } else {
            else_bb
        }))
    }
    pub(crate) fn execute_return(&self, value: Option<ValueId>) -> Result<ControlFlow, VMError> {
        if let Some(val_id) = value {
            let return_val = self.get_value(val_id)?;
            if crate::config::env::vm_vt_trace() {
                eprintln!(
                    "[VT] Return id={} val={}",
                    val_id.to_usize(),
                    return_val.to_string()
                );
            }
            Ok(ControlFlow::Return(return_val))
        } else {
            if crate::config::env::vm_vt_trace() {
                eprintln!("[VT] Return void");
            }
            Ok(ControlFlow::Return(VMValue::Void))
        }
    }
    pub(crate) fn execute_typeop(
        &mut self,
        dst: ValueId,
        op: &TypeOpKind,
        value: ValueId,
        ty: &MirType,
    ) -> Result<ControlFlow, VMError> {
        let val = self.get_value(value)?;
        match op {
            TypeOpKind::Check => {
                let is_type = match (&val, ty) {
                    (VMValue::Integer(_), MirType::Integer) => true,
                    (VMValue::Float(_), MirType::Float) => true,
                    (VMValue::Bool(_), MirType::Bool) => true,
                    (VMValue::String(_), MirType::String) => true,
                    (VMValue::Void, MirType::Void) => true,
                    (VMValue::BoxRef(arc_box), MirType::Box(box_name)) => {
                        arc_box.type_name() == box_name
                    }
                    _ => false,
                };
                self.set_value(dst, VMValue::Bool(is_type));
                Ok(ControlFlow::Continue)
            }
            TypeOpKind::Cast => {
                let result = match (&val, ty) {
                    (VMValue::Integer(i), MirType::Float) => VMValue::Float(*i as f64),
                    (VMValue::Float(f), MirType::Integer) => VMValue::Integer(*f as i64),
                    (VMValue::Integer(_), MirType::Integer)
                    | (VMValue::Float(_), MirType::Float)
                    | (VMValue::Bool(_), MirType::Bool)
                    | (VMValue::String(_), MirType::String) => val.clone(),
                    (VMValue::BoxRef(arc_box), MirType::Box(box_name))
                        if arc_box.type_name() == box_name =>
                    {
                        val.clone()
                    }
                    _ => {
                        return Err(VMError::TypeError(format!(
                            "Cannot cast {:?} to {:?}",
                            val, ty
                        )));
                    }
                };
                self.set_value(dst, result);
                Ok(ControlFlow::Continue)
            }
        }
    }
    pub(crate) fn execute_phi(
        &mut self,
        dst: ValueId,
        inputs: &[(BasicBlockId, ValueId)],
    ) -> Result<ControlFlow, VMError> {
        let selected = self.loop_execute_phi(dst, inputs)?;
        self.set_value(dst, selected);
        Ok(ControlFlow::Continue)
    }
    pub(crate) fn execute_load(
        &mut self,
        dst: ValueId,
        ptr: ValueId,
    ) -> Result<ControlFlow, VMError> {
        let loaded_value = self.get_value(ptr)?;
        self.set_value(dst, loaded_value);
        Ok(ControlFlow::Continue)
    }
    pub(crate) fn execute_store(
        &mut self,
        value: ValueId,
        ptr: ValueId,
    ) -> Result<ControlFlow, VMError> {
        let val = self.get_value(value)?;
        self.set_value(ptr, val);
        Ok(ControlFlow::Continue)
    }
    pub(crate) fn execute_copy(
        &mut self,
        dst: ValueId,
        src: ValueId,
    ) -> Result<ControlFlow, VMError> {
        let value = self.get_value(src)?;
        let cloned = match &value {
            VMValue::BoxRef(arc_box) => {
                let cloned_box = arc_box.clone_or_share();
                VMValue::BoxRef(Arc::from(cloned_box))
            }
            other => other.clone(),
        };
        self.set_value(dst, cloned);
        Ok(ControlFlow::Continue)
    }

    // ---- Arrays ----
    pub(crate) fn execute_array_get(
        &mut self,
        dst: ValueId,
        array: ValueId,
        index: ValueId,
    ) -> Result<ControlFlow, VMError> {
        let array_val = self.get_value(array)?;
        let index_val = self.get_value(index)?;
        if let VMValue::BoxRef(array_box) = &array_val {
            if let Some(array) = array_box.as_any().downcast_ref::<ArrayBox>() {
                let index_box = index_val.to_nyash_box();
                let result = array.get(index_box);
                self.set_value(dst, VMValue::BoxRef(Arc::from(result)));
                Ok(ControlFlow::Continue)
            } else {
                Err(VMError::TypeError(
                    "ArrayGet requires an ArrayBox".to_string(),
                ))
            }
        } else {
            Err(VMError::TypeError(
                "ArrayGet requires array and integer index".to_string(),
            ))
        }
    }
    pub(crate) fn execute_array_set(
        &mut self,
        array: ValueId,
        index: ValueId,
        value: ValueId,
    ) -> Result<ControlFlow, VMError> {
        let array_val = self.get_value(array)?;
        let index_val = self.get_value(index)?;
        let value_val = self.get_value(value)?;
        if let VMValue::BoxRef(array_box) = &array_val {
            if let Some(array) = array_box.as_any().downcast_ref::<ArrayBox>() {
                crate::backend::gc_helpers::gc_write_barrier_site(&self.runtime, "ArraySet");
                let index_box = index_val.to_nyash_box();
                let box_value = value_val.to_nyash_box();
                array.set(index_box, box_value);
                Ok(ControlFlow::Continue)
            } else {
                Err(VMError::TypeError(
                    "ArraySet requires an ArrayBox".to_string(),
                ))
            }
        } else {
            Err(VMError::TypeError(
                "ArraySet requires array and integer index".to_string(),
            ))
        }
    }

    // ---- Refs/Weak/Barriers ----
    pub(crate) fn execute_ref_new(
        &mut self,
        dst: ValueId,
        box_val: ValueId,
    ) -> Result<ControlFlow, VMError> {
        let box_value = self.get_value(box_val)?;
        self.set_value(dst, box_value);
        Ok(ControlFlow::Continue)
    }
    pub(crate) fn execute_ref_get(
        &mut self,
        dst: ValueId,
        reference: ValueId,
        field: &str,
    ) -> Result<ControlFlow, VMError> {
        let debug_ref = std::env::var("NYASH_VM_DEBUG_REF").ok().as_deref() == Some("1");
        if debug_ref {
            eprintln!("[VM] RefGet ref={:?} field={}", reference, field);
        }
        let is_internal = self.object_internal.contains(&reference);
        if !is_internal {
            if let Some(class_name) = self.object_class.get(&reference) {
                if let Ok(decls) = self.runtime.box_declarations.read() {
                    if let Some(decl) = decls.get(class_name) {
                        let has_vis =
                            !decl.public_fields.is_empty() || !decl.private_fields.is_empty();
                        if has_vis && !decl.public_fields.iter().any(|f| f == field) {
                            return Err(VMError::TypeError(format!(
                                "Field '{}' is private in {}",
                                field, class_name
                            )));
                        }
                    }
                }
            }
        }
        let mut field_value = if let Some(fields) = self.object_fields.get(&reference) {
            if let Some(value) = fields.get(field) {
                if debug_ref {
                    eprintln!("[VM] RefGet hit: {} -> {:?}", field, value);
                }
                value.clone()
            } else {
                if debug_ref {
                    eprintln!("[VM] RefGet miss: {} -> default 0", field);
                }
                VMValue::Integer(0)
            }
        } else {
            if debug_ref {
                eprintln!("[VM] RefGet no fields: -> default 0");
            }
            VMValue::Integer(0)
        };
        if matches!(field_value, VMValue::Integer(0)) && field == "console" {
            if debug_ref {
                eprintln!("[VM] RefGet special binding: console -> Plugin ConsoleBox");
            }
            let host = crate::runtime::get_global_plugin_host();
            let host = host.read().unwrap();
            if let Ok(pbox) = host.create_box("ConsoleBox", &[]) {
                field_value = VMValue::from_nyash_box(pbox);
                if !self.object_fields.contains_key(&reference) {
                    self.object_fields
                        .insert(reference, std::collections::HashMap::new());
                }
                if let Some(fields) = self.object_fields.get_mut(&reference) {
                    fields.insert(field.to_string(), field_value.clone());
                }
            }
        }
        self.set_value(dst, field_value);
        Ok(ControlFlow::Continue)
    }
    pub(crate) fn execute_ref_set(
        &mut self,
        reference: ValueId,
        field: &str,
        value: ValueId,
    ) -> Result<ControlFlow, VMError> {
        let debug_ref = std::env::var("NYASH_VM_DEBUG_REF").ok().as_deref() == Some("1");
        let new_value = self.get_value(value)?;
        if debug_ref {
            eprintln!(
                "[VM] RefSet ref={:?} field={} value={:?}",
                reference, field, new_value
            );
        }
        let is_internal = self.object_internal.contains(&reference);
        if !is_internal {
            if let Some(class_name) = self.object_class.get(&reference) {
                if let Ok(decls) = self.runtime.box_declarations.read() {
                    if let Some(decl) = decls.get(class_name) {
                        let has_vis =
                            !decl.public_fields.is_empty() || !decl.private_fields.is_empty();
                        if has_vis && !decl.public_fields.iter().any(|f| f == field) {
                            return Err(VMError::TypeError(format!(
                                "Field '{}' is private in {}",
                                field, class_name
                            )));
                        }
                    }
                }
            }
        }
        if !self.object_fields.contains_key(&reference) {
            self.object_fields
                .insert(reference, std::collections::HashMap::new());
        }
        crate::backend::gc_helpers::gc_write_barrier_site(&self.runtime, "RefSet");
        if let Some(fields) = self.object_fields.get_mut(&reference) {
            fields.insert(field.to_string(), new_value);
            if debug_ref {
                eprintln!("[VM] RefSet stored: {}", field);
            }
        }
        Ok(ControlFlow::Continue)
    }
    pub(crate) fn execute_weak_new(
        &mut self,
        dst: ValueId,
        box_val: ValueId,
    ) -> Result<ControlFlow, VMError> {
        let box_value = self.get_value(box_val)?;
        self.set_value(dst, box_value);
        Ok(ControlFlow::Continue)
    }
    pub(crate) fn execute_weak_load(
        &mut self,
        dst: ValueId,
        weak_ref: ValueId,
    ) -> Result<ControlFlow, VMError> {
        let weak_value = self.get_value(weak_ref)?;
        self.set_value(dst, weak_value);
        Ok(ControlFlow::Continue)
    }
    pub(crate) fn execute_barrier_read(
        &mut self,
        dst: ValueId,
        value: ValueId,
    ) -> Result<ControlFlow, VMError> {
        let val = self.get_value(value)?;
        self.set_value(dst, val);
        Ok(ControlFlow::Continue)
    }
    pub(crate) fn execute_barrier_write(
        &mut self,
        _value: ValueId,
    ) -> Result<ControlFlow, VMError> {
        Ok(ControlFlow::Continue)
    }
    pub(crate) fn execute_throw(&mut self, exception: ValueId) -> Result<ControlFlow, VMError> {
        let exc_value = self.get_value(exception)?;
        Err(VMError::InvalidInstruction(format!(
            "Exception thrown: {:?}",
            exc_value
        )))
    }
    pub(crate) fn execute_catch(
        &mut self,
        exception_value: ValueId,
    ) -> Result<ControlFlow, VMError> {
        self.set_value(exception_value, VMValue::Void);
        Ok(ControlFlow::Continue)
    }

    // ---- Futures ----
    pub(crate) fn execute_await(
        &mut self,
        dst: ValueId,
        future: ValueId,
    ) -> Result<ControlFlow, VMError> {
        let future_val = self.get_value(future)?;
        if let VMValue::Future(ref future_box) = future_val {
            let max_ms: u64 = crate::config::env::await_max_ms();
            let start = std::time::Instant::now();
            let mut spins = 0usize;
            while !future_box.ready() {
                self.runtime.gc.safepoint();
                if let Some(s) = &self.runtime.scheduler {
                    s.poll();
                }
                std::thread::yield_now();
                spins += 1;
                if spins % 1024 == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                if start.elapsed() >= std::time::Duration::from_millis(max_ms) {
                    let err = Box::new(crate::box_trait::StringBox::new("Timeout"));
                    let rb = crate::boxes::result::NyashResultBox::new_err(err);
                    let vm_value = VMValue::from_nyash_box(Box::new(rb));
                    self.set_value(dst, vm_value);
                    return Ok(ControlFlow::Continue);
                }
            }
            let result = future_box.get();
            let ok = crate::boxes::result::NyashResultBox::new_ok(result);
            let vm_value = VMValue::from_nyash_box(Box::new(ok));
            self.set_value(dst, vm_value);
            Ok(ControlFlow::Continue)
        } else {
            Err(VMError::TypeError(format!(
                "Expected Future, got {:?}",
                future_val
            )))
        }
    }
}
