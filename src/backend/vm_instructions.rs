/*!
 * VM Instruction Handlers
 *
 * Purpose: Implementation of each MIR instruction handler (invoked by vm.rs)
 * Responsibilities: load/store/branch/phi/call/array/ref/await/extern_call
 * Key APIs: execute_const/execute_binop/execute_unaryop/execute_compare/...
 * Typical Callers: VM::execute_instruction (dispatch point)
 */

use crate::mir::{ConstValue, BinaryOp, CompareOp, UnaryOp, ValueId, BasicBlockId, TypeOpKind, MirType};
use crate::box_trait::{NyashBox, BoolBox, VoidBox};
use crate::boxes::ArrayBox;
use std::sync::Arc;
use super::{VM, VMValue, VMError};
use super::vm::ControlFlow;

impl VM {
    // moved helpers to backend::gc_helpers
    /// Build a PIC key from receiver and method identity
    fn build_pic_key(&self, recv: &VMValue, method: &str, method_id: Option<u16>) -> String {
        let label = self.cache_label_for_recv(recv);
        let ver = self.cache_version_for_label(&label);
        if let Some(mid) = method_id {
            format!("v{}:{}#{}", ver, label, mid)
        } else {
            format!("v{}:{}#{}", ver, label, method)
        }
    }

    /// Record a PIC hit for the given key (skeleton: increments a counter)
    fn pic_record_hit(&mut self, key: &str) {
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

    /// Read current PIC hit count for a key
    fn pic_hits(&self, key: &str) -> u32 {
        *self.boxcall_pic_hits.get(key).unwrap_or(&0)
    }

    /// Build vtable cache key for InstanceBox: TypeName#slot/arity
    fn build_vtable_key(&self, class_name: &str, method_id: u16, arity: usize) -> String {
        // Use same versioning as PIC for BoxRef<Class>
        let label = format!("BoxRef:{}", class_name);
        let ver = self.cache_version_for_label(&label);
        format!("VT@v{}:{}#{}{}", ver, class_name, method_id, format!("/{}", arity))
    }

    /// Poly-PIC probe: try up to 4 cached entries for this call-site
    fn try_poly_pic(&mut self, pic_site_key: &str, recv: &VMValue) -> Option<String> {
        let label = self.cache_label_for_recv(recv);
        let ver = self.cache_version_for_label(&label);
        if let Some(entries) = self.boxcall_poly_pic.get_mut(pic_site_key) {
            // find match and move to end for naive LRU behavior
            if let Some(idx) = entries.iter().position(|(l, v, _)| *l == label && *v == ver) {
                let entry = entries.remove(idx);
                entries.push(entry.clone());
                return Some(entry.2);
            }
        }
        None
    }

    /// Poly-PIC record: insert or refresh an entry for this call-site
    fn record_poly_pic(&mut self, pic_site_key: &str, recv: &VMValue, func_name: &str) {
        let label = self.cache_label_for_recv(recv);
        let ver = self.cache_version_for_label(&label);
        use std::collections::hash_map::Entry;
        match self.boxcall_poly_pic.entry(pic_site_key.to_string()) {
            Entry::Occupied(mut e) => {
                let v = e.get_mut();
                if let Some(idx) = v.iter().position(|(l, vv, _)| *l == label && *vv == ver) {
                    v.remove(idx);
                }
                if v.len() >= 4 { v.remove(0); }
                v.push((label.clone(), ver, func_name.to_string()));
            }
            Entry::Vacant(v) => {
                v.insert(vec![(label.clone(), ver, func_name.to_string())]);
            }
        }
        if std::env::var("NYASH_VM_PIC_STATS").ok().as_deref() == Some("1") {
            // minimal per-site size log
            if let Some(v) = self.boxcall_poly_pic.get(pic_site_key) {
                eprintln!("[PIC] site={} size={} last=({}, v{}) -> {}",
                    pic_site_key, v.len(), label, ver, func_name);
            }
        }
    }

    /// Compute cache label for a receiver
    fn cache_label_for_recv(&self, recv: &VMValue) -> String {
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

    /// Get current version for a cache label (default 0)
    fn cache_version_for_label(&self, label: &str) -> u32 {
        // Prefer global cache versions so that loaders can invalidate across VMs
        crate::runtime::cache_versions::get_version(label)
    }

    /// Bump version for a label (used to invalidate caches)
    #[allow(dead_code)]
    pub fn bump_cache_version(&mut self, label: &str) {
        crate::runtime::cache_versions::bump_version(label)
    }
    /// Execute a constant instruction
    pub(super) fn execute_const(&mut self, dst: ValueId, value: &ConstValue) -> Result<ControlFlow, VMError> {
        let vm_value = VMValue::from(value);
        self.set_value(dst, vm_value);
        Ok(ControlFlow::Continue)
    }

    /// Execute a binary operation instruction
    pub(super) fn execute_binop(&mut self, dst: ValueId, op: &BinaryOp, lhs: ValueId, rhs: ValueId) -> Result<ControlFlow, VMError> {
        // Short-circuit semantics for logical ops using boolean coercion
        match *op {
            BinaryOp::And | BinaryOp::Or => {
                if std::env::var("NYASH_VM_DEBUG_ANDOR").ok().as_deref() == Some("1") {
                    eprintln!("[VM] And/Or short-circuit path");
                }
                let left = self.get_value(lhs)?;
                let right = self.get_value(rhs)?;
                let lb = left.as_bool()?;
                let rb = right.as_bool()?;
                let out = match *op { BinaryOp::And => lb && rb, BinaryOp::Or => lb || rb, _ => unreachable!() };
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

    /// Execute a unary operation instruction
    pub(super) fn execute_unaryop(&mut self, dst: ValueId, op: &UnaryOp, operand: ValueId) -> Result<ControlFlow, VMError> {
        let operand_val = self.get_value(operand)?;
        let result = self.execute_unary_op(op, &operand_val)?;
        self.set_value(dst, result);
        Ok(ControlFlow::Continue)
    }

    /// Execute a comparison instruction
    pub(super) fn execute_compare(&mut self, dst: ValueId, op: &CompareOp, lhs: ValueId, rhs: ValueId) -> Result<ControlFlow, VMError> {
        let debug_cmp = std::env::var("NYASH_VM_DEBUG").ok().as_deref() == Some("1") ||
            std::env::var("NYASH_VM_DEBUG_CMP").ok().as_deref() == Some("1");
        if debug_cmp { eprintln!("[VM] execute_compare enter op={:?} lhs={:?} rhs={:?}", op, lhs, rhs); }
        let mut left = self.get_value(lhs)?;
        let mut right = self.get_value(rhs)?;
        if debug_cmp { eprintln!("[VM] execute_compare values: left={:?} right={:?}", left, right); }

        // Canonicalize BoxRef(any) → try Integer via downcast/parse (no type_name reliance)
        left = match left {
            VMValue::BoxRef(b) => {
                if let Some(ib) = b.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                    VMValue::Integer(ib.value)
                } else {
                    match b.to_string_box().value.trim().parse::<i64>() { Ok(n) => VMValue::Integer(n), Err(_) => VMValue::BoxRef(b) }
                }
            }
            other => other,
        };
        right = match right {
            VMValue::BoxRef(b) => {
                if let Some(ib) = b.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                    VMValue::Integer(ib.value)
                } else {
                    match b.to_string_box().value.trim().parse::<i64>() { Ok(n) => VMValue::Integer(n), Err(_) => VMValue::BoxRef(b) }
                }
            }
            other => other,
        };

        let result = self.execute_compare_op(op, &left, &right)?;
        self.set_value(dst, VMValue::Bool(result));
        Ok(ControlFlow::Continue)
    }

    /// Execute a print instruction
    pub(super) fn execute_print(&self, value: ValueId) -> Result<ControlFlow, VMError> {
        let val = self.get_value(value)?;
        println!("{}", val.to_string());
        Ok(ControlFlow::Continue)
    }

    /// Execute control flow instructions (Jump, Branch, Return)
    pub(super) fn execute_jump(&self, target: BasicBlockId) -> Result<ControlFlow, VMError> {
        Ok(ControlFlow::Jump(target))
    }

    pub(super) fn execute_branch(&self, condition: ValueId, then_bb: BasicBlockId, else_bb: BasicBlockId) -> Result<ControlFlow, VMError> {
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
                    return Err(VMError::TypeError(
                        format!("Branch condition must be bool, void, or integer, got BoxRef({})", b.type_name())
                    ));
                }
            }
            _ => return Err(VMError::TypeError(
                format!("Branch condition must be bool, void, or integer, got {:?}", cond_val)
            )),
        };
        
        Ok(ControlFlow::Jump(if should_branch { then_bb } else { else_bb }))
    }

    pub(super) fn execute_return(&self, value: Option<ValueId>) -> Result<ControlFlow, VMError> {
        if let Some(val_id) = value {
            let return_val = self.get_value(val_id)?;
            Ok(ControlFlow::Return(return_val))
        } else {
            Ok(ControlFlow::Return(VMValue::Void))
        }
    }

    /// Execute TypeOp instruction
    pub(super) fn execute_typeop(&mut self, dst: ValueId, op: &TypeOpKind, value: ValueId, ty: &MirType) -> Result<ControlFlow, VMError> {
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
                    // Integer to Float
                    (VMValue::Integer(i), MirType::Float) => VMValue::Float(*i as f64),
                    // Float to Integer
                    (VMValue::Float(f), MirType::Integer) => VMValue::Integer(*f as i64),
                    // Identity casts
                    (VMValue::Integer(_), MirType::Integer) => val.clone(),
                    (VMValue::Float(_), MirType::Float) => val.clone(),
                    (VMValue::Bool(_), MirType::Bool) => val.clone(),
                    (VMValue::String(_), MirType::String) => val.clone(),
                    // BoxRef identity cast
                    (VMValue::BoxRef(arc_box), MirType::Box(box_name)) if arc_box.type_name() == box_name => {
                        val.clone()
                    }
                    // Invalid cast
                    _ => {
                        return Err(VMError::TypeError(
                            format!("Cannot cast {:?} to {:?}", val, ty)
                        ));
                    }
                };
                self.set_value(dst, result);
                Ok(ControlFlow::Continue)
            }
        }
    }

    /// Execute Phi instruction
    pub(super) fn execute_phi(&mut self, dst: ValueId, inputs: &[(BasicBlockId, ValueId)]) -> Result<ControlFlow, VMError> {
        // Minimal correct phi: select input based on previous_block via LoopExecutor
        let selected = self.loop_execute_phi(dst, inputs)?;
        self.set_value(dst, selected);
        Ok(ControlFlow::Continue)
    }

    /// Execute Load/Store instructions
    pub(super) fn execute_load(&mut self, dst: ValueId, ptr: ValueId) -> Result<ControlFlow, VMError> {
        let loaded_value = self.get_value(ptr)?;
        self.set_value(dst, loaded_value);
        Ok(ControlFlow::Continue)
    }

    pub(super) fn execute_store(&mut self, value: ValueId, ptr: ValueId) -> Result<ControlFlow, VMError> {
        let val = self.get_value(value)?;
        self.set_value(ptr, val);
        Ok(ControlFlow::Continue)
    }

    /// Execute Copy instruction
    pub(super) fn execute_copy(&mut self, dst: ValueId, src: ValueId) -> Result<ControlFlow, VMError> {
        let value = self.get_value(src)?;
        let cloned = match &value {
            VMValue::BoxRef(arc_box) => {
                // Use clone_or_share to handle cloning properly
                let cloned_box = arc_box.clone_or_share();
                VMValue::BoxRef(Arc::from(cloned_box))
            }
            other => other.clone(),
        };
        self.set_value(dst, cloned);
        Ok(ControlFlow::Continue)
    }

    /// Execute Call instruction
    pub(super) fn execute_call(&mut self, dst: Option<ValueId>, func: ValueId, args: &[ValueId]) -> Result<ControlFlow, VMError> {
        // Get the function name from the ValueId
        let func_name = match self.get_value(func)? {
            VMValue::String(s) => s,
            _ => return Err(VMError::TypeError("Function name must be a string".to_string())),
        };
        
        let arg_values: Vec<VMValue> = args.iter()
            .map(|arg| self.get_value(*arg))
            .collect::<Result<Vec<_>, _>>()?;
        
        let result = self.call_function_by_name(&func_name, arg_values)?;
        
        if let Some(dst_id) = dst {
            self.set_value(dst_id, result);
        }
        
        Ok(ControlFlow::Continue)
    }

    /// Execute NewBox instruction
    pub(super) fn execute_newbox(&mut self, dst: ValueId, box_type: &str, args: &[ValueId]) -> Result<ControlFlow, VMError> {
        // Convert args to NyashBox values
        let arg_values: Vec<Box<dyn NyashBox>> = args.iter()
            .map(|arg| {
                let val = self.get_value(*arg)?;
                Ok(val.to_nyash_box())
            })
            .collect::<Result<Vec<_>, VMError>>()?;

        // Create new box using runtime's registry
        let new_box = {
            let registry = self.runtime.box_registry.lock()
                .map_err(|_| VMError::InvalidInstruction("Failed to lock box registry".to_string()))?;
            registry.create_box(box_type, &arg_values)
                .map_err(|e| VMError::InvalidInstruction(format!("Failed to create {}: {}", box_type, e)))?
        };
        
        // 80/20: Basic boxes are stored as primitives in VMValue for simpler ops
        if box_type == "IntegerBox" {
            if let Some(ib) = new_box.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                self.set_value(dst, VMValue::Integer(ib.value));
                return Ok(ControlFlow::Continue);
            }
        } else if box_type == "BoolBox" {
            if let Some(bb) = new_box.as_any().downcast_ref::<crate::box_trait::BoolBox>() {
                self.set_value(dst, VMValue::Bool(bb.value));
                return Ok(ControlFlow::Continue);
            }
        } else if box_type == "StringBox" {
            if let Some(sb) = new_box.as_any().downcast_ref::<crate::box_trait::StringBox>() {
                self.set_value(dst, VMValue::String(sb.value.clone()));
                return Ok(ControlFlow::Continue);
            }
        }

        self.set_value(dst, VMValue::BoxRef(Arc::from(new_box)));
        Ok(ControlFlow::Continue)
    }

    /// Execute ArrayGet instruction
    pub(super) fn execute_array_get(&mut self, dst: ValueId, array: ValueId, index: ValueId) -> Result<ControlFlow, VMError> {
        let array_val = self.get_value(array)?;
        let index_val = self.get_value(index)?;
        
        if let VMValue::BoxRef(array_box) = &array_val {
            if let Some(array) = array_box.as_any().downcast_ref::<ArrayBox>() {
                // ArrayBox expects Box<dyn NyashBox> for index
                let index_box = index_val.to_nyash_box();
                let result = array.get(index_box);
                self.set_value(dst, VMValue::BoxRef(Arc::from(result)));
                Ok(ControlFlow::Continue)
            } else {
                Err(VMError::TypeError("ArrayGet requires an ArrayBox".to_string()))
            }
        } else {
            Err(VMError::TypeError("ArrayGet requires array and integer index".to_string()))
        }
    }

    /// Execute ArraySet instruction
    pub(super) fn execute_array_set(&mut self, array: ValueId, index: ValueId, value: ValueId) -> Result<ControlFlow, VMError> {
        let array_val = self.get_value(array)?;
        let index_val = self.get_value(index)?;
        let value_val = self.get_value(value)?;
        
        if let VMValue::BoxRef(array_box) = &array_val {
            if let Some(array) = array_box.as_any().downcast_ref::<ArrayBox>() {
                // GC write barrier (array contents mutation)
                crate::backend::gc_helpers::gc_write_barrier_site(&self.runtime, "ArraySet");
                // ArrayBox expects Box<dyn NyashBox> for index
                let index_box = index_val.to_nyash_box();
                let box_value = value_val.to_nyash_box();
                array.set(index_box, box_value);
                Ok(ControlFlow::Continue)
            } else {
                Err(VMError::TypeError("ArraySet requires an ArrayBox".to_string()))
            }
        } else {
            Err(VMError::TypeError("ArraySet requires array and integer index".to_string()))
        }
    }

    /// Execute RefNew instruction
    pub(super) fn execute_ref_new(&mut self, dst: ValueId, box_val: ValueId) -> Result<ControlFlow, VMError> {
        // For now, a reference is just the same as the box value
        // In a real implementation, this would create a proper reference
        let box_value = self.get_value(box_val)?;
        self.set_value(dst, box_value);
        Ok(ControlFlow::Continue)
    }

    /// Execute RefGet instruction
    pub(super) fn execute_ref_get(&mut self, dst: ValueId, reference: ValueId, field: &str) -> Result<ControlFlow, VMError> {
        let debug_ref = std::env::var("NYASH_VM_DEBUG_REF").ok().as_deref() == Some("1");
        if debug_ref { eprintln!("[VM] RefGet ref={:?} field={}", reference, field); }
        // Visibility check (if class known and visibility declared). Skip for internal refs.
        let is_internal = self.object_internal.contains(&reference);
        if !is_internal {
            if let Some(class_name) = self.object_class.get(&reference) {
                if let Ok(decls) = self.runtime.box_declarations.read() {
                    if let Some(decl) = decls.get(class_name) {
                        let has_vis = !decl.public_fields.is_empty() || !decl.private_fields.is_empty();
                        if has_vis && !decl.public_fields.iter().any(|f| f == field) {
                            return Err(VMError::TypeError(format!("Field '{}' is private in {}", field, class_name)));
                        }
                    }
                }
            }
        }
        // Get field value from object
        let mut field_value = if let Some(fields) = self.object_fields.get(&reference) {
            if let Some(value) = fields.get(field) {
                if debug_ref { eprintln!("[VM] RefGet hit: {} -> {:?}", field, value); }
                value.clone()
            } else {
                // Field not set yet, return default
                if debug_ref { eprintln!("[VM] RefGet miss: {} -> default 0", field); }
                VMValue::Integer(0)
            }
        } else {
            // Object has no fields yet, return default
            if debug_ref { eprintln!("[VM] RefGet no fields: -> default 0"); }
            VMValue::Integer(0)
        };

        // Special binding for environment-like fields: map 'console' to plugin ConsoleBox
        if matches!(field_value, VMValue::Integer(0)) && field == "console" {
            if debug_ref { eprintln!("[VM] RefGet special binding: console -> Plugin ConsoleBox"); }
            let host = crate::runtime::get_global_plugin_host();
            let host = host.read().unwrap();
            if let Ok(pbox) = host.create_box("ConsoleBox", &[]) {
                field_value = VMValue::from_nyash_box(pbox);
                // Cache on the object so subsequent ref_get uses the same instance
                if !self.object_fields.contains_key(&reference) {
                    self.object_fields.insert(reference, std::collections::HashMap::new());
                }
                if let Some(fields) = self.object_fields.get_mut(&reference) {
                    fields.insert(field.to_string(), field_value.clone());
                }
            }
        }
        
        self.set_value(dst, field_value);
        Ok(ControlFlow::Continue)
    }

    /// Execute RefSet instruction
    pub(super) fn execute_ref_set(&mut self, reference: ValueId, field: &str, value: ValueId) -> Result<ControlFlow, VMError> {
        let debug_ref = std::env::var("NYASH_VM_DEBUG_REF").ok().as_deref() == Some("1");
        // Get the value to set
        let new_value = self.get_value(value)?;
        if debug_ref { eprintln!("[VM] RefSet ref={:?} field={} value={:?}", reference, field, new_value); }
        // Visibility check (Skip for internal refs; otherwise enforce public)
        let is_internal = self.object_internal.contains(&reference);
        if !is_internal {
            if let Some(class_name) = self.object_class.get(&reference) {
                if let Ok(decls) = self.runtime.box_declarations.read() {
                    if let Some(decl) = decls.get(class_name) {
                        let has_vis = !decl.public_fields.is_empty() || !decl.private_fields.is_empty();
                        if has_vis && !decl.public_fields.iter().any(|f| f == field) {
                            return Err(VMError::TypeError(format!("Field '{}' is private in {}", field, class_name)));
                        }
                    }
                }
            }
        }
        
        // Ensure object has field storage
        if !self.object_fields.contains_key(&reference) {
            self.object_fields.insert(reference, std::collections::HashMap::new());
        }
        
        // Set the field (with GC write barrier)
        crate::backend::gc_helpers::gc_write_barrier_site(&self.runtime, "RefSet");
        if let Some(fields) = self.object_fields.get_mut(&reference) {
            fields.insert(field.to_string(), new_value);
            if debug_ref { eprintln!("[VM] RefSet stored: {}", field); }
        }
        
        Ok(ControlFlow::Continue)
    }

    /// Execute WeakNew instruction
    pub(super) fn execute_weak_new(&mut self, dst: ValueId, box_val: ValueId) -> Result<ControlFlow, VMError> {
        // For now, a weak reference is just a copy of the value
        // In a real implementation, this would create a proper weak reference
        let box_value = self.get_value(box_val)?;
        self.set_value(dst, box_value);
        Ok(ControlFlow::Continue)
    }

    /// Execute WeakLoad instruction
    pub(super) fn execute_weak_load(&mut self, dst: ValueId, weak_ref: ValueId) -> Result<ControlFlow, VMError> {
        // For now, loading from weak ref is the same as getting the value
        // In a real implementation, this would check if the weak ref is still valid
        let weak_value = self.get_value(weak_ref)?;
        self.set_value(dst, weak_value);
        Ok(ControlFlow::Continue)
    }

    /// Execute BarrierRead instruction
    pub(super) fn execute_barrier_read(&mut self, dst: ValueId, value: ValueId) -> Result<ControlFlow, VMError> {
        // Memory barrier read is currently a simple value copy
        // In a real implementation, this would ensure memory ordering
        let val = self.get_value(value)?;
        self.set_value(dst, val);
        Ok(ControlFlow::Continue)
    }

    /// Execute BarrierWrite instruction
    pub(super) fn execute_barrier_write(&mut self, _value: ValueId) -> Result<ControlFlow, VMError> {
        // Memory barrier write is a no-op for now
        // In a real implementation, this would ensure memory ordering
        Ok(ControlFlow::Continue)
    }

    /// Execute Throw instruction
    pub(super) fn execute_throw(&mut self, exception: ValueId) -> Result<ControlFlow, VMError> {
        let exc_value = self.get_value(exception)?;
        Err(VMError::InvalidInstruction(format!("Exception thrown: {:?}", exc_value)))
    }

    /// Execute Catch instruction
    pub(super) fn execute_catch(&mut self, exception_value: ValueId) -> Result<ControlFlow, VMError> {
        // For now, catch is a no-op
        // In a real implementation, this would handle exception catching
        self.set_value(exception_value, VMValue::Void);
        Ok(ControlFlow::Continue)
    }

    /// Execute Await instruction
    pub(super) fn execute_await(&mut self, dst: ValueId, future: ValueId) -> Result<ControlFlow, VMError> {
        let future_val = self.get_value(future)?;
        
        if let VMValue::Future(ref future_box) = future_val {
            // Cooperative wait with scheduler polling and timeout to avoid deadlocks
            let max_ms: u64 = crate::config::env::await_max_ms();
            let start = std::time::Instant::now();
            let mut spins = 0usize;
            while !future_box.ready() {
                // Poll GC/scheduler similar to Safepoint
                self.runtime.gc.safepoint();
                if let Some(s) = &self.runtime.scheduler { s.poll(); }
                std::thread::yield_now();
                spins += 1;
                if spins % 1024 == 0 { std::thread::sleep(std::time::Duration::from_millis(1)); }
                if start.elapsed() >= std::time::Duration::from_millis(max_ms) {
                    // Timeout -> Result.Err("Timeout")
                    let err = Box::new(crate::box_trait::StringBox::new("Timeout"));
                    let rb = crate::boxes::result::NyashResultBox::new_err(err);
                    let vm_value = VMValue::from_nyash_box(Box::new(rb));
                    self.set_value(dst, vm_value);
                    return Ok(ControlFlow::Continue);
                }
            }
            // Ready: get value and wrap into Result.Ok
            let result = future_box.get();
            let ok = crate::boxes::result::NyashResultBox::new_ok(result);
            let vm_value = VMValue::from_nyash_box(Box::new(ok));
            self.set_value(dst, vm_value);
            Ok(ControlFlow::Continue)
        } else {
            Err(VMError::TypeError(format!("Expected Future, got {:?}", future_val)))
        }
    }

    /// Execute ExternCall instruction
    pub(super) fn execute_extern_call(&mut self, dst: Option<ValueId>, iface_name: &str, method_name: &str, args: &[ValueId]) -> Result<ControlFlow, VMError> {
        
        // Evaluate arguments as NyashBox for loader
        let mut nyash_args: Vec<Box<dyn NyashBox>> = Vec::new();
        for arg_id in args {
            let arg_value = self.get_value(*arg_id)?;
            nyash_args.push(arg_value.to_nyash_box());
        }
        
        // Optional trace
        if crate::config::env::extern_trace() {
            if let Some(slot) = crate::runtime::extern_registry::resolve_slot(iface_name, method_name) {
                eprintln!("[EXT] call {}.{} slot={} argc={}", iface_name, method_name, slot, nyash_args.len());
            } else {
                eprintln!("[EXT] call {}.{} argc={}", iface_name, method_name, nyash_args.len());
            }
        }
        // Route through unified plugin host (delegates to v2, handles env.* stubs)
        let host = crate::runtime::get_global_plugin_host();
        let host = host.read().map_err(|_| VMError::InvalidInstruction("Plugin host lock poisoned".into()))?;
        match host.extern_call(iface_name, method_name, &nyash_args) {
            Ok(Some(result_box)) => {
                if let Some(dst_id) = dst {
                    self.set_value(dst_id, VMValue::from_nyash_box(result_box));
                }
            }
            Ok(None) => {
                if let Some(dst_id) = dst {
                    self.set_value(dst_id, VMValue::Void);
                }
            }
            Err(_) => {
                let strict = crate::config::env::extern_strict() || crate::config::env::abi_strict();
                // Build suggestion list
                let mut msg = String::new();
                if strict { msg.push_str("ExternCall STRICT: unregistered or unsupported call "); } else { msg.push_str("ExternCall failed: "); }
                msg.push_str(&format!("{}.{}", iface_name, method_name));
                // Arity check when known
                if let Err(detail) = crate::runtime::extern_registry::check_arity(iface_name, method_name, nyash_args.len()) {
                    msg.push_str(&format!(" ({})", detail));
                }
                if let Some(spec) = crate::runtime::extern_registry::resolve(iface_name, method_name) {
                    msg.push_str(&format!(" (expected arity {}..{})", spec.min_arity, spec.max_arity));
                } else {
                    let known = crate::runtime::extern_registry::known_for_iface(iface_name);
                    if !known.is_empty() {
                        msg.push_str(&format!("; known methods: {}", known.join(", ")));
                    } else {
                        let ifaces = crate::runtime::extern_registry::all_ifaces();
                        msg.push_str(&format!("; known interfaces: {}", ifaces.join(", ")));
                    }
                }
                return Err(VMError::InvalidInstruction(msg));
            }
        }
        Ok(ControlFlow::Continue)
    }

    /// Execute BoxCall instruction
    pub(super) fn execute_boxcall(&mut self, dst: Option<ValueId>, box_val: ValueId, method: &str, method_id: Option<u16>, args: &[ValueId]) -> Result<ControlFlow, VMError> {
        let recv = self.get_value(box_val)?;
        
        // Debug logging if enabled
        let debug_boxcall = std::env::var("NYASH_VM_DEBUG_BOXCALL").is_ok();
        
        // Phase 12 Tier-0: vtable優先経路（雛形）
        if crate::config::env::abi_vtable() {
            if let Some(res) = self.try_boxcall_vtable_stub(dst, &recv, method, method_id, args) { return res; }
        }

        // Record PIC hit (per-receiver-type × method)
        let pic_key = self.build_pic_key(&recv, method, method_id);
        self.pic_record_hit(&pic_key);

        // Explicit fast-path: ArrayBox get/set by (type, slot) or method name
        if let VMValue::BoxRef(arc_box) = &recv {
            if arc_box.as_any().downcast_ref::<crate::boxes::array::ArrayBox>().is_some() {
                let get_slot = crate::mir::slot_registry::resolve_slot_by_type_name("ArrayBox", "get");
                let set_slot = crate::mir::slot_registry::resolve_slot_by_type_name("ArrayBox", "set");
                let is_get = (method_id.is_some() && method_id == get_slot) || method == "get";
                let is_set = (method_id.is_some() && method_id == set_slot) || method == "set";
                if is_get && args.len() >= 1 {
                    // Convert index
                    let idx_val = self.get_value(args[0])?;
                    let idx_box = idx_val.to_nyash_box();
                    // Call builtin directly
                    let arr = arc_box.as_any().downcast_ref::<crate::boxes::array::ArrayBox>().unwrap();
                    let out = arr.get(idx_box);
                    if let Some(dst_id) = dst { self.set_value(dst_id, VMValue::from_nyash_box(out)); }
                    return Ok(ControlFlow::Continue);
                } else if is_set && args.len() >= 2 {
                    // Barrier for mutation
                    crate::backend::gc_helpers::gc_write_barrier_site(&self.runtime, "BoxCall.fastpath.Array.set");
                    let idx_val = self.get_value(args[0])?;
                    let val_val = self.get_value(args[1])?;
                    let idx_box = idx_val.to_nyash_box();
                    let val_box = val_val.to_nyash_box();
                    let arr = arc_box.as_any().downcast_ref::<crate::boxes::array::ArrayBox>().unwrap();
                    let out = arr.set(idx_box, val_box);
                    if let Some(dst_id) = dst { self.set_value(dst_id, VMValue::from_nyash_box(out)); }
                    return Ok(ControlFlow::Continue);
                }
            }
            // Explicit fast-path: InstanceBox getField/setField by name
            if let Some(inst) = arc_box.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                let is_getf = method == "getField";
                let is_setf = method == "setField";
                if (is_getf && args.len() >= 1) || (is_setf && args.len() >= 2) {
                    // Extract field name
                    let name_val = self.get_value(args[0])?;
                    let field_name = match &name_val {
                        VMValue::String(s) => s.clone(),
                        _ => name_val.to_string(),
                    };
                    if is_getf {
                        let out_opt = inst.get_field_unified(&field_name);
                        let out_vm = match out_opt {
                            Some(nv) => match nv {
                                crate::value::NyashValue::Integer(i) => VMValue::Integer(i),
                                crate::value::NyashValue::Float(f) => VMValue::Float(f),
                                crate::value::NyashValue::Bool(b) => VMValue::Bool(b),
                                crate::value::NyashValue::String(s) => VMValue::String(s),
                                crate::value::NyashValue::Void | crate::value::NyashValue::Null => VMValue::Void,
                                crate::value::NyashValue::Box(b) => {
                                    if let Ok(g) = b.lock() { VMValue::from_nyash_box(g.share_box()) } else { VMValue::Void }
                                }
                                _ => VMValue::Void,
                            },
                            None => VMValue::Void,
                        };
                        if let Some(dst_id) = dst { self.set_value(dst_id, out_vm); }
                        return Ok(ControlFlow::Continue);
                    } else {
                        // setField
                        crate::backend::gc_helpers::gc_write_barrier_site(&self.runtime, "BoxCall.fastpath.Instance.setField");
                        let val_vm = self.get_value(args[1])?;
                        let nv_opt = match val_vm.clone() {
                            VMValue::Integer(i) => Some(crate::value::NyashValue::Integer(i)),
                            VMValue::Float(f) => Some(crate::value::NyashValue::Float(f)),
                            VMValue::Bool(b) => Some(crate::value::NyashValue::Bool(b)),
                            VMValue::String(s) => Some(crate::value::NyashValue::String(s)),
                            VMValue::Void => Some(crate::value::NyashValue::Void),
                            _ => None,
                        };
                        if let Some(nv) = nv_opt {
                            let _ = inst.set_field_unified(field_name, nv);
                            if let Some(dst_id) = dst { self.set_value(dst_id, VMValue::Void); }
                            return Ok(ControlFlow::Continue);
                        }
                    }
                }
            }
        }

        // VTable-like direct call using method_id via TypeMeta thunk table (InstanceBox/PluginBoxV2/Builtin)
        if let (Some(mid), VMValue::BoxRef(arc_box)) = (method_id, &recv) {
            // Determine class label for TypeMeta
            let mut class_label: Option<String> = None;
            let mut is_instance = false;
            let mut is_plugin = false;
            let mut is_builtin = false;
            if let Some(inst) = arc_box.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                class_label = Some(inst.class_name.clone());
                is_instance = true;
            } else if let Some(p) = arc_box.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                class_label = Some(p.box_type.clone());
                is_plugin = true;
            } else {
                class_label = Some(arc_box.type_name().to_string());
                is_builtin = true;
            }
            if let Some(label) = class_label {
                let tm = crate::runtime::type_meta::get_or_create_type_meta(&label);
                if let Some(th) = tm.get_thunk(mid as usize) {
                    if let Some(target) = th.get_target() {
                        match target {
                            crate::runtime::type_meta::ThunkTarget::MirFunction(func_name) => {
                                if std::env::var("NYASH_VM_VT_STATS").ok().as_deref() == Some("1") {
                                    eprintln!("[VT] hit class={} slot={} -> {}", label, mid, func_name);
                                }
                                // 実行: 受け取り→VM引数並べ→関数呼出
                                let mut vm_args = Vec::with_capacity(1 + args.len());
                                vm_args.push(recv.clone());
                                for a in args { vm_args.push(self.get_value(*a)?); }
                                let res = self.call_function_by_name(&func_name, vm_args)?;

                                // 10_e: Thunk経路でもPIC/VTableを直結更新するにゃ
                                // - Poly-PIC: 直ちに記録（最大4件ローカルLRU）
                                self.record_poly_pic(&pic_key, &recv, &func_name);
                                // - HotならMono-PICにも格納（しきい値=8）
                                const PIC_THRESHOLD: u32 = 8;
                                if self.pic_hits(&pic_key) >= PIC_THRESHOLD {
                                    self.boxcall_pic_funcname.insert(pic_key.clone(), func_name.clone());
                                }
                                // - InstanceBoxならVTableキーにも登録（method_id/arity直結）
                                if is_instance {
                                    let vkey = self.build_vtable_key(&label, mid, args.len());
                                    self.boxcall_vtable_funcname.entry(vkey).or_insert(func_name.clone());
                                }

                                if let Some(dst_id) = dst { self.set_value(dst_id, res); }
                                return Ok(ControlFlow::Continue);
                            }
                            crate::runtime::type_meta::ThunkTarget::PluginInvoke { method_id: mid2 } => {
                                if is_plugin {
                                    if let Some(p) = arc_box.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                                        // Root region for plugin call (pin recv + args)
                                        self.enter_root_region();
                                        // Convert args prepared earlier (we need NyashBox args)
                                        let nyash_args: Vec<Box<dyn NyashBox>> = args.iter()
                                            .map(|arg| {
                                                let val = self.get_value(*arg)?;
                                                Ok(val.to_nyash_box())
                                            })
                                            .collect::<Result<Vec<_>, VMError>>()?;
                                        // Pin roots: receiver and VMValue args
                                        self.pin_roots(std::iter::once(&recv));
                                        let pinned_args: Vec<VMValue> = args.iter().filter_map(|a| self.get_value(*a).ok()).collect();
                                        self.pin_roots(pinned_args.iter());
                                        // Encode minimal TLV (int/string/handle) same as fast-path
                                        let mut tlv = crate::runtime::plugin_ffi_common::encode_tlv_header(nyash_args.len() as u16);
                                        let mut enc_failed = false;
                                        for a in &nyash_args {
                                            if let Some(s) = a.as_any().downcast_ref::<crate::box_trait::StringBox>() {
                                                crate::runtime::plugin_ffi_common::encode::string(&mut tlv, &s.value);
                                            } else if let Some(i) = a.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                                                crate::runtime::plugin_ffi_common::encode::i32(&mut tlv, i.value as i32);
                                            } else if let Some(h) = a.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                                                crate::runtime::plugin_ffi_common::encode::plugin_handle(&mut tlv, h.inner.type_id, h.inner.instance_id);
                                            } else {
                                                enc_failed = true; break;
                                            }
                                        }
                                        if !enc_failed {
                                            let mut out = vec![0u8; 4096];
                                            let mut out_len: usize = out.len();
                // Bind current VM for potential reverse host calls
                crate::runtime::host_api::set_current_vm(self as *mut _);
                let code = unsafe {
                    (p.inner.invoke_fn)(
                        p.inner.type_id,
                        mid2 as u32,
                        p.inner.instance_id,
                        tlv.as_ptr(),
                        tlv.len(),
                        out.as_mut_ptr(),
                        &mut out_len,
                    )
                };
                crate::runtime::host_api::clear_current_vm();
                                            if code == 0 {
                                                let vm_out = if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len]) {
                                                    match tag {
                                                        6 | 7 => VMValue::String(crate::runtime::plugin_ffi_common::decode::string(payload)),
                                                        2 => crate::runtime::plugin_ffi_common::decode::i32(payload).map(|v| VMValue::Integer(v as i64)).unwrap_or(VMValue::Void),
                                                        9 => {
                                                            if let Some(h) = crate::runtime::plugin_ffi_common::decode::u64(payload) {
                                                                if let Some(arc) = crate::runtime::host_handles::get(h) { VMValue::BoxRef(arc) } else { VMValue::Void }
                                                            } else { VMValue::Void }
                                                        }
                                                        _ => VMValue::Void,
                                                    }
                                                } else { VMValue::Void };
                                                if let Some(dst_id) = dst { self.set_value(dst_id, vm_out); }
                                                // Leave root region
                                                self.leave_root_region();
                                                return Ok(ControlFlow::Continue);
                                            }
                                        // Leave root region also on error path
                                        self.leave_root_region();
                                        }
                                    }
                                }
                            }
                            crate::runtime::type_meta::ThunkTarget::BuiltinCall { method: ref m } => {
                                if is_builtin {
                                    // Prepare NyashBox args and call by name
                                    let nyash_args: Vec<Box<dyn NyashBox>> = args.iter()
                                        .map(|arg| {
                                            let val = self.get_value(*arg)?;
                                            Ok(val.to_nyash_box())
                                        })
                                        .collect::<Result<Vec<_>, VMError>>()?;
                                    // Write barrier for known mutating builtins or setField
                                    if crate::backend::gc_helpers::is_mutating_builtin_call(&recv, m) {
                                        crate::backend::gc_helpers::gc_write_barrier_site(&self.runtime, "BoxCall.builtin");
                                    } else if m == "setField" {
                                        crate::backend::gc_helpers::gc_write_barrier_site(&self.runtime, "BoxCall.setField");
                                    }
                                    let cloned_box = arc_box.share_box();
                                    self.boxcall_hits_generic = self.boxcall_hits_generic.saturating_add(1);
                                    let out = self.call_box_method(cloned_box, m, nyash_args)?;
                                    let vm_out = VMValue::from_nyash_box(out);
                                    if let Some(dst_id) = dst { self.set_value(dst_id, vm_out); }
                                    return Ok(ControlFlow::Continue);
                                }
                            }
                        }
                    }
                }
                // Backward-compat: consult legacy vtable cache for InstanceBox if TypeMeta empty
                if is_instance {
                    let inst = arc_box.as_any().downcast_ref::<crate::instance_v2::InstanceBox>().unwrap();
                    let vkey = self.build_vtable_key(&inst.class_name, mid, args.len());
                    if let Some(func_name) = self.boxcall_vtable_funcname.get(&vkey).cloned() {
                        let mut vm_args = Vec::with_capacity(1 + args.len());
                        vm_args.push(recv.clone());
                        for a in args { vm_args.push(self.get_value(*a)?); }
                        let res = self.call_function_by_name(&func_name, vm_args)?;
                        if let Some(dst_id) = dst { self.set_value(dst_id, res); }
                        return Ok(ControlFlow::Continue);
                    }
                }
            }
        }

        // Poly-PIC direct call: if we cached a target for this site and receiver is InstanceBox, use it
        if let VMValue::BoxRef(arc_box) = &recv {
            if arc_box.as_any().downcast_ref::<crate::instance_v2::InstanceBox>().is_some() {
                if let Some(func_name) = self.try_poly_pic(&pic_key, &recv) {
                    if crate::config::env::vm_pic_trace() { eprintln!("[PIC] poly hit {}", pic_key); }
                    self.boxcall_hits_poly_pic = self.boxcall_hits_poly_pic.saturating_add(1);
                    let mut vm_args = Vec::with_capacity(1 + args.len());
                    vm_args.push(recv.clone());
                    for a in args { vm_args.push(self.get_value(*a)?); }
                    let res = self.call_function_by_name(&func_name, vm_args)?;
                    if let Some(dst_id) = dst { self.set_value(dst_id, res); }
                    return Ok(ControlFlow::Continue);
                }
                // Fallback to Mono-PIC (legacy) if present
                if let Some(func_name) = self.boxcall_pic_funcname.get(&pic_key).cloned() {
                    if crate::config::env::vm_pic_trace() { eprintln!("[PIC] mono hit {}", pic_key); }
                    self.boxcall_hits_mono_pic = self.boxcall_hits_mono_pic.saturating_add(1);
                    // Build VM args: receiver first, then original args
                    let mut vm_args = Vec::with_capacity(1 + args.len());
                    vm_args.push(recv.clone());
                    for a in args { vm_args.push(self.get_value(*a)?); }
                    let res = self.call_function_by_name(&func_name, vm_args)?;
                    if let Some(dst_id) = dst { self.set_value(dst_id, res); }
                    return Ok(ControlFlow::Continue);
                }
            }
        }

        // Fast path: universal method slots via method_id (0..3)
        if let Some(mid) = method_id {
            if let Some(fast_res) = self.try_fast_universal(mid, &recv, args)? {
                if let Some(dst_id) = dst { self.set_value(dst_id, fast_res); }
                return Ok(ControlFlow::Continue);
            }
        }

        // Convert args to NyashBox
        let nyash_args: Vec<Box<dyn NyashBox>> = args.iter()
            .map(|arg| {
                let val = self.get_value(*arg)?;
                Ok(val.to_nyash_box())
            })
            .collect::<Result<Vec<_>, VMError>>()?;

                // PluginBoxV2 fast-path via method_id -> direct invoke_fn (skip name->id resolution)
                if let (Some(mid), VMValue::BoxRef(arc_box)) = (method_id, &recv) {
                    if let Some(p) = arc_box.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                        // Root region for plugin call
                        self.enter_root_region();
                        // Encode TLV args (support: int, string, plugin handle)
                        let mut tlv = crate::runtime::plugin_ffi_common::encode_tlv_header(nyash_args.len() as u16);
                        let mut enc_failed = false;
                        for a in &nyash_args {
                    // Prefer BufferBox → bytes
                    if let Some(buf) = a.as_any().downcast_ref::<crate::boxes::buffer::BufferBox>() {
                        let snapshot = buf.to_vec();
                        crate::runtime::plugin_ffi_common::encode::bytes(&mut tlv, &snapshot);
                        continue;
                    }
                    if let Some(s) = a.as_any().downcast_ref::<crate::box_trait::StringBox>() {
                        crate::runtime::plugin_ffi_common::encode::string(&mut tlv, &s.value);
                    } else if let Some(i) = a.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                        // Prefer 32-bit if fits, else i64
                        if i.value >= i32::MIN as i64 && i.value <= i32::MAX as i64 {
                            crate::runtime::plugin_ffi_common::encode::i32(&mut tlv, i.value as i32);
                        } else {
                            crate::runtime::plugin_ffi_common::encode::i64(&mut tlv, i.value as i64);
                        }
                    } else if let Some(b) = a.as_any().downcast_ref::<crate::box_trait::BoolBox>() {
                        crate::runtime::plugin_ffi_common::encode::bool(&mut tlv, b.value);
                    } else if let Some(f) = a.as_any().downcast_ref::<crate::boxes::math_box::FloatBox>() {
                        crate::runtime::plugin_ffi_common::encode::f64(&mut tlv, f.value);
                    } else if let Some(arr) = a.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                        // Try encode as bytes if all elements are 0..255 integers
                        let items = arr.items.read().unwrap();
                        let mut tmp = Vec::with_capacity(items.len());
                        let mut ok = true;
                        for item in items.iter() {
                            if let Some(intb) = item.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                                if intb.value >= 0 && intb.value <= 255 { tmp.push(intb.value as u8); } else { ok = false; break; }
                            } else {
                                ok = false; break;
                            }
                        }
                        if ok { crate::runtime::plugin_ffi_common::encode::bytes(&mut tlv, &tmp); }
                        else { enc_failed = true; break; }
                    } else if let Some(h) = a.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                        crate::runtime::plugin_ffi_common::encode::plugin_handle(&mut tlv, h.inner.type_id, h.inner.instance_id);
                    } else {
                        enc_failed = true; break;
                    }
                }
                if !enc_failed {
                    let mut out = vec![0u8; 4096];
                    let mut out_len: usize = out.len();
                    let code = unsafe {
                        (p.inner.invoke_fn)(
                            p.inner.type_id,
                            mid as u32,
                            p.inner.instance_id,
                            tlv.as_ptr(),
                            tlv.len(),
                            out.as_mut_ptr(),
                            &mut out_len,
                        )
                    };
                    if code == 0 {
                        // Record TypeMeta thunk for plugin invoke so next time VT path can hit
                        let tm = crate::runtime::type_meta::get_or_create_type_meta(&p.box_type);
                        tm.set_thunk_plugin_invoke(mid as usize, mid as u16);
                        // Try decode TLV first entry (string/i32); else return void
                        let vm_out = if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len]) {
                            match tag {
                                1 => crate::runtime::plugin_ffi_common::decode::bool(payload).map(VMValue::Bool).unwrap_or(VMValue::Void),
                                2 => crate::runtime::plugin_ffi_common::decode::i32(payload).map(|v| VMValue::Integer(v as i64)).unwrap_or(VMValue::Void),
                                5 => crate::runtime::plugin_ffi_common::decode::f64(payload).map(VMValue::Float).unwrap_or(VMValue::Void),
                                6 => VMValue::String(crate::runtime::plugin_ffi_common::decode::string(payload)),
                                _ => VMValue::Void,
                            }
                        } else {
                            VMValue::Void
                        };
                        if let Some(dst_id) = dst { self.set_value(dst_id, vm_out); }
                        self.leave_root_region();
                        return Ok(ControlFlow::Continue);
                    }
                    // Leave root region also on non-zero code path
                    self.leave_root_region();
                }
            }
        }
        
        if debug_boxcall {
            self.debug_log_boxcall(&recv, method, &nyash_args, "START", None);
        }
        
        // Call the method based on receiver type
        let result = match &recv {
            VMValue::BoxRef(arc_box) => {
                // If this is a user InstanceBox, redirect to lowered function: Class.method/arity
                if let Some(inst) = arc_box.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                    let func_name = format!("{}.{}{}", inst.class_name, method, format!("/{}", args.len()));
                    // Populate TypeMeta thunk table and legacy vtable cache if method_id is known
                    if let Some(mid) = method_id { 
                        // TypeMeta preferred store (MIR target)
                        let tm = crate::runtime::type_meta::get_or_create_type_meta(&inst.class_name);
                        tm.set_thunk_mir_target(mid as usize, func_name.clone());
                        // Legacy cache retained for compatibility
                        let vkey = self.build_vtable_key(&inst.class_name, mid, args.len());
                        self.boxcall_vtable_funcname.entry(vkey).or_insert(func_name.clone());
                    }
                    // Record in Poly-PIC immediately (size <= 4)
                    self.record_poly_pic(&pic_key, &recv, &func_name);
                    // If this call-site is hot, also cache in legacy Mono-PIC for backward behavior
                    const PIC_THRESHOLD: u32 = 8;
                    if self.pic_hits(&pic_key) >= PIC_THRESHOLD {
                        self.boxcall_pic_funcname.insert(pic_key.clone(), func_name.clone());
                    }
                    if debug_boxcall { eprintln!("[BoxCall] InstanceBox -> call {}", func_name); }
                    // Build VMValue args: receiver first, then original VMValue args
                    let mut vm_args = Vec::with_capacity(1 + args.len());
                    vm_args.push(recv.clone());
                    for a in args { vm_args.push(self.get_value(*a)?); }
                    let res = self.call_function_by_name(&func_name, vm_args)?;
                    return {
                        if let Some(dst_id) = dst { self.set_value(dst_id, res); }
                        Ok(ControlFlow::Continue)
                    };
                }
                // Otherwise, direct box method call
                if debug_boxcall { eprintln!("[BoxCall] Taking BoxRef path for method '{}'", method); }
                // Populate TypeMeta for builtin path if method_id present (BuiltinCall thunk)
                if let Some(mid) = method_id {
                    let label = arc_box.type_name().to_string();
                    let tm = crate::runtime::type_meta::get_or_create_type_meta(&label);
                    tm.set_thunk_builtin(mid as usize, method.to_string());
                }
                // Write barrier for known mutating builtins or setField
                if crate::backend::gc_helpers::is_mutating_builtin_call(&recv, method) {
                    crate::backend::gc_helpers::gc_write_barrier_site(&self.runtime, "BoxCall");
                } else if method == "setField" {
                    crate::backend::gc_helpers::gc_write_barrier_site(&self.runtime, "BoxCall.setField");
                }
                let cloned_box = arc_box.share_box();
                self.call_box_method(cloned_box, method, nyash_args)?
            }
            _ => {
                // Convert primitive to box and call
                if debug_boxcall { eprintln!("[BoxCall] Converting primitive to box for method '{}'", method); }
                
                let box_value = recv.to_nyash_box();
                self.call_box_method(box_value, method, nyash_args)?
            }
        };
        
        // Convert result back to VMValue
        let result_val = VMValue::from_nyash_box(result);
        
        if debug_boxcall {
            self.debug_log_boxcall(&recv, method, &[], "END", Some(&result_val));
        }
        
        if let Some(dst_id) = dst {
            self.set_value(dst_id, result_val);
        }
        
        Ok(ControlFlow::Continue)
    }

    /// Phase 12 Tier-0: vtable優先経路の雛形（常に未処理）。
    /// 目的: 将来のTypeBox ABI配線ポイントを先置きしても既存挙動を変えないこと。
    fn try_boxcall_vtable_stub(&mut self, _dst: Option<ValueId>, _recv: &VMValue, _method: &str, _method_id: Option<u16>, _args: &[ValueId]) -> Option<Result<ControlFlow, VMError>> {
        // Tier-1 PoC: Array/Map/String の get/set/len/size/has を vtable 経路で処理（read-onlyまたは明示barrier不要）
        if let VMValue::BoxRef(b) = _recv {
            // 型解決（雛形レジストリ使用）
            let ty_name = b.type_name();
            if let Some(_tb) = crate::runtime::type_registry::resolve_typebox_by_name(ty_name) {
                // name+arity→slot 解決
                let slot = crate::runtime::type_registry::resolve_slot_by_name(ty_name, _method, _args.len());
                // MapBox: size/len/has/get/set
                if let Some(map) = b.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                    if matches!(slot, Some(200|201)) {
                        self.boxcall_hits_vtable = self.boxcall_hits_vtable.saturating_add(1);
                        if crate::config::env::vm_vt_trace() { eprintln!("[VT] MapBox.size/len slot={:?}", slot); }
                        let out = map.size();
                        if let Some(dst_id) = _dst { self.set_value(dst_id, VMValue::from_nyash_box(out)); }
                        return Some(Ok(ControlFlow::Continue));
                    }
                    if matches!(slot, Some(202)) {
                        if let Ok(arg_v) = self.get_value(_args[0]) {
                            let key_box = match arg_v {
                                VMValue::String(ref s) => Box::new(crate::box_trait::StringBox::new(s)) as Box<dyn NyashBox>,
                                VMValue::Integer(i) => Box::new(crate::box_trait::IntegerBox::new(i)),
                                VMValue::Bool(b) => Box::new(crate::box_trait::BoolBox::new(b)),
                                VMValue::BoxRef(ref bx) => bx.share_box(),
                                VMValue::Float(f) => Box::new(crate::boxes::FloatBox::new(f)),
                                VMValue::Future(ref fut) => Box::new(fut.clone()),
                                VMValue::Void => Box::new(crate::box_trait::VoidBox::new()),
                            };
                            self.boxcall_hits_vtable = self.boxcall_hits_vtable.saturating_add(1);
                            self.boxcall_hits_vtable = self.boxcall_hits_vtable.saturating_add(1);
                            if crate::config::env::vm_vt_trace() { eprintln!("[VT] MapBox.has"); }
                            let out = map.has(key_box);
                            if let Some(dst_id) = _dst { self.set_value(dst_id, VMValue::from_nyash_box(out)); }
                            return Some(Ok(ControlFlow::Continue));
                        }
                    }
                    if matches!(slot, Some(203)) {
                        if let Ok(arg_v) = self.get_value(_args[0]) {
                            let key_box = match arg_v {
                                VMValue::String(ref s) => Box::new(crate::box_trait::StringBox::new(s)) as Box<dyn NyashBox>,
                                VMValue::Integer(i) => Box::new(crate::box_trait::IntegerBox::new(i)),
                                VMValue::Bool(b) => Box::new(crate::box_trait::BoolBox::new(b)),
                                VMValue::BoxRef(ref bx) => bx.share_box(),
                                VMValue::Float(f) => Box::new(crate::boxes::FloatBox::new(f)),
                                VMValue::Future(ref fut) => Box::new(fut.clone()),
                                VMValue::Void => Box::new(crate::box_trait::VoidBox::new()),
                            };
                            self.boxcall_hits_vtable = self.boxcall_hits_vtable.saturating_add(1);
                            self.boxcall_hits_vtable = self.boxcall_hits_vtable.saturating_add(1);
                            if crate::config::env::vm_vt_trace() { eprintln!("[VT] MapBox.get"); }
                            let out = map.get(key_box);
                            if let Some(dst_id) = _dst { self.set_value(dst_id, VMValue::from_nyash_box(out)); }
                            return Some(Ok(ControlFlow::Continue));
                        }
                    }
                    if matches!(slot, Some(204)) {
                        if let (Ok(a0), Ok(a1)) = (self.get_value(_args[0]), self.get_value(_args[1])) {
                            let key_box: Box<dyn NyashBox> = match a0 {
                                VMValue::Integer(i) => Box::new(crate::box_trait::IntegerBox::new(i)),
                                VMValue::String(ref s) => Box::new(crate::box_trait::StringBox::new(s)),
                                VMValue::Bool(b) => Box::new(crate::box_trait::BoolBox::new(b)),
                                VMValue::Float(f) => Box::new(crate::boxes::math_box::FloatBox::new(f)),
                                VMValue::BoxRef(ref bx) => bx.share_box(),
                                VMValue::Future(ref fut) => Box::new(fut.clone()),
                                VMValue::Void => Box::new(crate::box_trait::VoidBox::new()),
                            };
                            let val_box: Box<dyn NyashBox> = match a1 {
                                VMValue::Integer(i) => Box::new(crate::box_trait::IntegerBox::new(i)),
                                VMValue::String(ref s) => Box::new(crate::box_trait::StringBox::new(s)),
                                VMValue::Bool(b) => Box::new(crate::box_trait::BoolBox::new(b)),
                                VMValue::Float(f) => Box::new(crate::boxes::math_box::FloatBox::new(f)),
                                VMValue::BoxRef(ref bx) => bx.share_box(),
                                VMValue::Future(ref fut) => Box::new(fut.clone()),
                                VMValue::Void => Box::new(crate::box_trait::VoidBox::new()),
                            };
                            // Barrier: mutation
                            crate::backend::gc_helpers::gc_write_barrier_site(&self.runtime, "VTable.Map.set");
                            self.boxcall_hits_vtable = self.boxcall_hits_vtable.saturating_add(1);
                            if crate::config::env::vm_vt_trace() { eprintln!("[VT] MapBox.set"); }
                            let out = map.set(key_box, val_box);
                            if let Some(dst_id) = _dst { self.set_value(dst_id, VMValue::from_nyash_box(out)); }
                            return Some(Ok(ControlFlow::Continue));
                        }
                    }
                }
                // ArrayBox: get/set/len
                if let Some(arr) = b.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                    if matches!(slot, Some(102)) {
                        self.boxcall_hits_vtable = self.boxcall_hits_vtable.saturating_add(1);
                        if crate::config::env::vm_vt_trace() { eprintln!("[VT] ArrayBox.len"); }
                        let out = arr.length();
                        if let Some(dst_id) = _dst { self.set_value(dst_id, VMValue::from_nyash_box(out)); }
                        return Some(Ok(ControlFlow::Continue));
                    }
                    if matches!(slot, Some(100)) {
                        if let Ok(arg_v) = self.get_value(_args[0]) {
                            let idx_box: Box<dyn NyashBox> = match arg_v {
                                VMValue::Integer(i) => Box::new(crate::box_trait::IntegerBox::new(i)),
                                VMValue::String(ref s) => Box::new(crate::box_trait::IntegerBox::new(s.parse::<i64>().unwrap_or(0))),
                                VMValue::Bool(b) => Box::new(crate::box_trait::IntegerBox::new(if b {1} else {0})),
                                VMValue::Float(f) => Box::new(crate::box_trait::IntegerBox::new(f as i64)),
                                VMValue::BoxRef(_) | VMValue::Future(_) | VMValue::Void => Box::new(crate::box_trait::IntegerBox::new(0)),
                            };
                            self.boxcall_hits_vtable = self.boxcall_hits_vtable.saturating_add(1);
                            self.boxcall_hits_vtable = self.boxcall_hits_vtable.saturating_add(1);
                            if crate::config::env::vm_vt_trace() { eprintln!("[VT] ArrayBox.get"); }
                            let out = arr.get(idx_box);
                            if let Some(dst_id) = _dst { self.set_value(dst_id, VMValue::from_nyash_box(out)); }
                            return Some(Ok(ControlFlow::Continue));
                        }
                    }
                    if matches!(slot, Some(101)) {
                        if let (Ok(a0), Ok(a1)) = (self.get_value(_args[0]), self.get_value(_args[1])) {
                            let idx_box: Box<dyn NyashBox> = match a0 {
                                VMValue::Integer(i) => Box::new(crate::box_trait::IntegerBox::new(i)),
                                VMValue::String(ref s) => Box::new(crate::box_trait::IntegerBox::new(s.parse::<i64>().unwrap_or(0))),
                                VMValue::Bool(b) => Box::new(crate::box_trait::IntegerBox::new(if b {1} else {0})),
                                VMValue::Float(f) => Box::new(crate::box_trait::IntegerBox::new(f as i64)),
                                VMValue::BoxRef(_) | VMValue::Future(_) | VMValue::Void => Box::new(crate::box_trait::IntegerBox::new(0)),
                            };
                            let val_box: Box<dyn NyashBox> = match a1 {
                                VMValue::Integer(i) => Box::new(crate::box_trait::IntegerBox::new(i)),
                                VMValue::String(ref s) => Box::new(crate::box_trait::StringBox::new(s)),
                                VMValue::Bool(b) => Box::new(crate::box_trait::BoolBox::new(b)),
                                VMValue::Float(f) => Box::new(crate::boxes::math_box::FloatBox::new(f)),
                                VMValue::BoxRef(ref bx) => bx.share_box(),
                                VMValue::Future(ref fut) => Box::new(fut.clone()),
                                VMValue::Void => Box::new(crate::box_trait::VoidBox::new()),
                            };
                            self.boxcall_hits_vtable = self.boxcall_hits_vtable.saturating_add(1);
                            // Barrier: mutation
                            crate::backend::gc_helpers::gc_write_barrier_site(&self.runtime, "VTable.Array.set");
                            self.boxcall_hits_vtable = self.boxcall_hits_vtable.saturating_add(1);
                            if crate::config::env::vm_vt_trace() { eprintln!("[VT] ArrayBox.set"); }
                            let out = arr.set(idx_box, val_box);
                            if let Some(dst_id) = _dst { self.set_value(dst_id, VMValue::from_nyash_box(out)); }
                            return Some(Ok(ControlFlow::Continue));
                        }
                    }
                }
                // StringBox: len
                if let Some(sb) = b.as_any().downcast_ref::<crate::box_trait::StringBox>() {
                    if matches!(slot, Some(300)) {
                        self.boxcall_hits_vtable = self.boxcall_hits_vtable.saturating_add(1);
                        if crate::config::env::vm_vt_trace() { eprintln!("[VT] StringBox.len"); }
                        let out = crate::box_trait::IntegerBox::new(sb.value.len() as i64);
                        if let Some(dst_id) = _dst { self.set_value(dst_id, VMValue::from_nyash_box(Box::new(out))); }
                        return Some(Ok(ControlFlow::Continue));
                    }
                }
                // STRICT: 型は登録されているがメソッドが未対応 → エラー
                if crate::config::env::abi_strict() {
                    return Some(Err(VMError::TypeError(format!("ABI_STRICT: vtable method not found: {}.{}", ty_name, _method))));
                }
            }
        }
        None
    }

    /// Execute a forced plugin invocation (no builtin fallback)
    pub(super) fn execute_plugin_invoke(&mut self, dst: Option<ValueId>, box_val: ValueId, method: &str, args: &[ValueId]) -> Result<ControlFlow, VMError> {
        // Helper: extract UTF-8 string from internal StringBox, Result.Ok(String-like), or plugin StringBox via toUtf8
        fn extract_string_from_box(bx: &dyn crate::box_trait::NyashBox) -> Option<String> {
            // Internal StringBox
            if let Some(sb) = bx.as_any().downcast_ref::<crate::box_trait::StringBox>() {
                return Some(sb.value.clone());
            }
            // Result.Ok(inner) → recurse
            if let Some(res) = bx.as_any().downcast_ref::<crate::boxes::result::NyashResultBox>() {
                if let crate::boxes::result::NyashResultBox::Ok(inner) = res { return extract_string_from_box(inner.as_ref()); }
            }
            // Plugin StringBox → call toUtf8
            if let Some(p) = bx.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                if p.box_type == "StringBox" {
                    let host = crate::runtime::get_global_plugin_host();
                    let tmp: Option<String> = if let Ok(ro) = host.read() {
                        if let Ok(val_opt) = ro.invoke_instance_method("StringBox", "toUtf8", p.inner.instance_id, &[]) {
                            if let Some(vb) = val_opt {
                                if let Some(sb2) = vb.as_any().downcast_ref::<crate::box_trait::StringBox>() {
                                    Some(sb2.value.clone())
                                } else { None }
                            } else { None }
                        } else { None }
                    } else { None };
                    if tmp.is_some() { return tmp; }
                }
            }
            None
        }
        let recv = self.get_value(box_val)?;
        // Allow static birth on primitives/builtin boxes to create a plugin instance.
        if method == "birth" && !matches!(recv, VMValue::BoxRef(ref b) if b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>().is_some()) {
            eprintln!("[VM PluginInvoke] static birth fallback recv={:?}", recv);
            // Map primitive/builtin receiver to plugin box type name and constructor args
            let mut created: Option<VMValue> = None;
            match &recv {
                VMValue::String(s) => {
                    // Create plugin StringBox with initial content
                    let host = crate::runtime::get_global_plugin_host();
                    let host = host.read().unwrap();
                    let sb: Box<dyn crate::box_trait::NyashBox> = Box::new(crate::box_trait::StringBox::new(s.clone()));
                    if let Ok(b) = host.create_box("StringBox", &[sb]) {
                        created = Some(VMValue::from_nyash_box(b));
                    }
                }
                VMValue::Integer(_n) => {
                    // Create plugin IntegerBox (value defaults to 0); args ignored by current plugin
                    let host = crate::runtime::get_global_plugin_host();
                    let host = host.read().unwrap();
                    if let Ok(b) = host.create_box("IntegerBox", &[]) {
                        created = Some(VMValue::from_nyash_box(b));
                    }
                }
                _ => {
                    // no-op
                }
            }
            if let Some(val) = created {
                if let Some(dst_id) = dst { self.set_value(dst_id, val); }
                return Ok(ControlFlow::Continue);
            }
        }

        // Only allowed on plugin boxes
        if let VMValue::BoxRef(pbox) = &recv {
            if let Some(p) = pbox.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                // Resolve method_id via unified host
                let host = crate::runtime::get_global_plugin_host();
                let host = host.read().unwrap();
                let mh = host.resolve_method(&p.box_type, method)
                    .map_err(|_| VMError::InvalidInstruction(format!("Plugin method not found: {}.{}", p.box_type, method)))?;

                // Encode args to TLV
                let mut tlv = crate::runtime::plugin_ffi_common::encode_tlv_header(args.len() as u16);
                for (idx, a) in args.iter().enumerate() {
                    let v = self.get_value(*a)?;
                    match v {
                        VMValue::Integer(n) => {
                            if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                                eprintln!("[VM→Plugin][vm] arg[{}] encode I64 {}", idx, n);
                            }
                            crate::runtime::plugin_ffi_common::encode::i64(&mut tlv, n)
                        }
                        VMValue::Float(x) => {
                            if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                                eprintln!("[VM→Plugin][vm] arg[{}] encode F64 {}", idx, x);
                            }
                            crate::runtime::plugin_ffi_common::encode::f64(&mut tlv, x)
                        }
                        VMValue::Bool(b) => crate::runtime::plugin_ffi_common::encode::bool(&mut tlv, b),
                        VMValue::String(ref s) => crate::runtime::plugin_ffi_common::encode::string(&mut tlv, s),
                        VMValue::BoxRef(ref b) => {
                            if let Some(h) = b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                                // Coerce common plugin primitives to TLV primitives instead of handle when sensible
                                if h.box_type == "StringBox" {
                                    // toUtf8 -> TLV string
                                    let host = crate::runtime::get_global_plugin_host();
                                    let host = host.read().unwrap();
                                    if let Ok(val_opt) = host.invoke_instance_method("StringBox", "toUtf8", h.inner.instance_id, &[]) {
                                        if let Some(sb) = val_opt.and_then(|bx| bx.as_any().downcast_ref::<crate::box_trait::StringBox>().map(|s| s.value.clone())) {
                                            crate::runtime::plugin_ffi_common::encode::string(&mut tlv, &sb);
                                            continue;
                                        }
                                    }
                                } else if h.box_type == "IntegerBox" {
                                    // get() -> TLV i64
                                    let host = crate::runtime::get_global_plugin_host();
                                    let host = host.read().unwrap();
                                    if let Ok(val_opt) = host.invoke_instance_method("IntegerBox", "get", h.inner.instance_id, &[]) {
                                        if let Some(ib) = val_opt.and_then(|bx| bx.as_any().downcast_ref::<crate::box_trait::IntegerBox>().map(|i| i.value)) {
                                            crate::runtime::plugin_ffi_common::encode::i64(&mut tlv, ib);
                                            continue;
                                        }
                                    }
                                }
                                // Fallback: pass plugin handle
                                crate::runtime::plugin_ffi_common::encode::plugin_handle(&mut tlv, h.inner.type_id, h.inner.instance_id);
                            } else {
                                // HostHandle: expose user/builtin box via host-managed handle (tag=9)
                                let h = crate::runtime::host_handles::to_handle_arc(b.clone());
                                crate::runtime::plugin_ffi_common::encode::host_handle(&mut tlv, h);
                            }
                        }
                        VMValue::Void => crate::runtime::plugin_ffi_common::encode::string(&mut tlv, "void"),
                        _ => {
                            return Err(VMError::TypeError(format!("Unsupported VMValue in PluginInvoke args: {:?}", v)));
                        }
                    }
                }

                if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                    eprintln!("[VM→Plugin] invoke {}.{} inst_id={} argc={} (direct)", p.box_type, method, p.inner.instance_id, args.len());
                }
                let mut out = vec![0u8; 4096];
                let mut out_len: usize = out.len();
                let code = unsafe {
                    (p.inner.invoke_fn)(
                        p.inner.type_id,
                        mh.method_id,
                        p.inner.instance_id,
                        tlv.as_ptr(),
                        tlv.len(),
                        out.as_mut_ptr(),
                        &mut out_len,
                    )
                };
                if code != 0 {
                    let host = crate::runtime::get_global_plugin_host();
                    let host = host.read().unwrap();
                    let rr = host.method_returns_result(&p.box_type, method);
                    if rr {
                        let be = crate::bid::BidError::from_raw(code);
                        let err = crate::exception_box::ErrorBox::new(&format!("{} (code: {})", be.message(), code));
                        let res = crate::boxes::result::NyashResultBox::new_err(Box::new(err));
                        let vmv = VMValue::BoxRef(std::sync::Arc::from(Box::new(res) as Box<dyn crate::box_trait::NyashBox>));
                        if let Some(dst_id) = dst { self.set_value(dst_id, vmv); }
                        return Ok(ControlFlow::Continue);
                    } else {
                        return Err(VMError::InvalidInstruction(format!("PluginInvoke failed: {}.{} rc={}", p.box_type, method, code)));
                    }
                }
                let vm_out_raw = if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len]) {
                    if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                        let preview_len = std::cmp::min(payload.len(), 16);
                        let preview: Vec<String> = payload[..preview_len].iter().map(|b| format!("{:02X}", b)).collect();
                        eprintln!("[Plugin→VM][vm] {}.{} tag={} sz={} preview=\n{}", p.box_type, method, tag, _sz, preview.join(" "));
                    }
                    match tag {
                        1 => crate::runtime::plugin_ffi_common::decode::bool(payload).map(VMValue::Bool).unwrap_or(VMValue::Void),
                        2 => crate::runtime::plugin_ffi_common::decode::i32(payload).map(|v| VMValue::Integer(v as i64)).unwrap_or(VMValue::Void),
                        3 => { // I64
                            if payload.len() == 8 { let mut b=[0u8;8]; b.copy_from_slice(&payload[0..8]); VMValue::Integer(i64::from_le_bytes(b)) } else { VMValue::Void }
                        }
                        5 => crate::runtime::plugin_ffi_common::decode::f64(payload).map(VMValue::Float).unwrap_or(VMValue::Void),
                        6 => VMValue::String(crate::runtime::plugin_ffi_common::decode::string(payload)),
                        8 => {
                            // Handle return: map (type_id, instance_id) to PluginBoxV2 using central config
                            if payload.len() == 8 {
                                let mut t = [0u8;4]; t.copy_from_slice(&payload[0..4]);
                                let mut i = [0u8;4]; i.copy_from_slice(&payload[4..8]);
                                let r_type = u32::from_le_bytes(t);
                                let r_inst = u32::from_le_bytes(i);
                                // Resolve box name via [box_types] reverse lookup
                                let host_arc = crate::runtime::get_global_plugin_host();
                                let host_ro = host_arc.read().unwrap();
                                if let Some(cfg) = host_ro.config_ref() {
                                    let mut box_name_opt: Option<String> = None;
                                    for (name, id) in cfg.box_types.iter() { if *id == r_type { box_name_opt = Some(name.clone()); break; } }
                                    if let Some(box_name) = box_name_opt {
                                        // Find library providing this box
                                        if let Some((lib_name, _)) = cfg.find_library_for_box(&box_name) {
                                            // Read nyash.toml to resolve fini method
                                            let cfg_path = "nyash.toml";
                                            if let Ok(toml_content) = std::fs::read_to_string(cfg_path) {
                                                if let Ok(toml_value) = toml::from_str::<toml::Value>(&toml_content) {
                                                    let fini_id = cfg.get_box_config(lib_name, &box_name, &toml_value)
                                                        .and_then(|bc| bc.methods.get("fini").map(|m| m.method_id));
                                                    let pbox = crate::runtime::plugin_loader_v2::construct_plugin_box(
                                                        box_name,
                                                        r_type,
                                                        p.inner.invoke_fn,
                                                        r_inst,
                                                        fini_id,
                                                    );
                                                    VMValue::BoxRef(Arc::from(Box::new(pbox) as Box<dyn crate::box_trait::NyashBox>))
                                                } else { VMValue::Void }
                                            } else { VMValue::Void }
                                        } else { VMValue::Void }
                                    } else { VMValue::Void }
                                } else { VMValue::Void }
                            } else { VMValue::Void }
                        }
                        _ => VMValue::Void,
                    }
                } else { VMValue::Void };
                // Wrap into Result.Ok when method is declared returns_result
                let vm_out = {
                    let host = crate::runtime::get_global_plugin_host();
                    let host = host.read().unwrap();
                    let rr = host.method_returns_result(&p.box_type, method);
                    if rr {
                        // Boxify vm_out into NyashBox first
                        let boxed: Box<dyn crate::box_trait::NyashBox> = match vm_out_raw {
                            VMValue::Integer(i) => Box::new(crate::box_trait::IntegerBox::new(i)),
                            VMValue::Float(f) => Box::new(crate::boxes::math_box::FloatBox::new(f)),
                            VMValue::Bool(b) => Box::new(crate::box_trait::BoolBox::new(b)),
                            VMValue::String(s) => Box::new(crate::box_trait::StringBox::new(s)),
                            VMValue::BoxRef(b) => b.share_box(),
                            VMValue::Void => Box::new(crate::box_trait::VoidBox::new()),
                            _ => Box::new(crate::box_trait::StringBox::new(vm_out_raw.to_string()))
                        };
                        let res = crate::boxes::result::NyashResultBox::new_ok(boxed);
                        VMValue::BoxRef(std::sync::Arc::from(Box::new(res) as Box<dyn crate::box_trait::NyashBox>))
                    } else {
                        vm_out_raw
                    }
                };
                if let Some(dst_id) = dst { self.set_value(dst_id, vm_out); }
                return Ok(ControlFlow::Continue);
            }
        }
        // Fallback: support common string-like methods without requiring PluginBox receiver
        if let VMValue::BoxRef(ref bx) = recv {
            // Try to view receiver as string (internal, plugin, or Result.Ok)
            if let Some(s) = extract_string_from_box(bx.as_ref()) {
                match method {
                    "length" => {
                        if let Some(dst_id) = dst { self.set_value(dst_id, VMValue::Integer(s.len() as i64)); }
                        return Ok(ControlFlow::Continue);
                    }
                    "is_empty" | "isEmpty" => {
                        if let Some(dst_id) = dst { self.set_value(dst_id, VMValue::Bool(s.is_empty())); }
                        return Ok(ControlFlow::Continue);
                    }
                    "charCodeAt" => {
                        let idx_v = if let Some(a0) = args.get(0) { self.get_value(*a0)? } else { VMValue::Integer(0) };
                        let idx = match idx_v { VMValue::Integer(i) => i.max(0) as usize, _ => 0 };
                        let code = s.chars().nth(idx).map(|c| c as u32 as i64).unwrap_or(0);
                        if let Some(dst_id) = dst { self.set_value(dst_id, VMValue::Integer(code)); }
                        return Ok(ControlFlow::Continue);
                    }
                    "concat" => {
                        let rhs_v = if let Some(a0) = args.get(0) { self.get_value(*a0)? } else { VMValue::String(String::new()) };
                        let rhs_s = match rhs_v {
                            VMValue::String(ss) => ss,
                            VMValue::BoxRef(br) => extract_string_from_box(br.as_ref()).unwrap_or_else(|| br.to_string_box().value),
                            _ => rhs_v.to_string(),
                        };
                        let mut new_s = s.clone();
                        new_s.push_str(&rhs_s);
                        let out = Box::new(crate::box_trait::StringBox::new(new_s));
                        if let Some(dst_id) = dst { self.set_value(dst_id, VMValue::BoxRef(std::sync::Arc::from(out as Box<dyn crate::box_trait::NyashBox>))); }
                        return Ok(ControlFlow::Continue);
                    }
                    _ => {}
                }
            }
        }
        Err(VMError::InvalidInstruction(format!("PluginInvoke requires PluginBox receiver; method={} got {:?}", method, recv)))
    }
}

impl VM {
    /// Try fast universal-thunk dispatch using reserved method slots 0..3.
    /// Returns Some(result) if handled, or None to fall back to legacy path.
    fn try_fast_universal(&mut self, method_id: u16, recv: &VMValue, args: &[ValueId]) -> Result<Option<VMValue>, VMError> {
        match method_id {
            0 => { // toString
                let s = recv.to_string();
                return Ok(Some(VMValue::String(s)));
            }
            1 => { // type
                let t = match recv {
                    VMValue::Integer(_) => "Integer".to_string(),
                    VMValue::Float(_) => "Float".to_string(),
                    VMValue::Bool(_) => "Bool".to_string(),
                    VMValue::String(_) => "String".to_string(),
                    VMValue::Future(_) => "Future".to_string(),
                    VMValue::Void => "Void".to_string(),
                    VMValue::BoxRef(b) => b.type_name().to_string(),
                };
                return Ok(Some(VMValue::String(t)));
            }
            2 => { // equals
                // equals(other): bool — naive implementation via VMValue equivalence/coercion
                let other = if let Some(arg0) = args.get(0) { self.get_value(*arg0)? } else { VMValue::Void };
                let res = match (recv, &other) {
                    (VMValue::Integer(a), VMValue::Integer(b)) => a == b,
                    (VMValue::Bool(a), VMValue::Bool(b)) => a == b,
                    (VMValue::String(a), VMValue::String(b)) => a == b,
                    (VMValue::Void, VMValue::Void) => true,
                    _ => recv.to_string() == other.to_string(),
                };
                return Ok(Some(VMValue::Bool(res)));
            }
            3 => { // clone
                // For primitives: just copy. For BoxRef: share_box then wrap back.
                let v = match recv {
                    VMValue::Integer(i) => VMValue::Integer(*i),
                    VMValue::Float(f) => VMValue::Float(*f),
                    VMValue::Bool(b) => VMValue::Bool(*b),
                    VMValue::String(s) => VMValue::String(s.clone()),
                    VMValue::Future(f) => VMValue::Future(f.clone()),
                    VMValue::Void => VMValue::Void,
                    VMValue::BoxRef(b) => VMValue::from_nyash_box(b.share_box()),
                };
                return Ok(Some(v));
            }
            _ => {}
        }
        Ok(None)
    }
}
