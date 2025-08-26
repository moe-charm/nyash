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
        let field_value = if let Some(fields) = self.object_fields.get(&reference) {
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
        
        // Set the field
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
            // This blocks until the future is ready
            let result = future_box.get();
            // Convert NyashBox back to VMValue
            let vm_value = VMValue::from_nyash_box(result);
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
                return Err(VMError::InvalidInstruction(format!("ExternCall failed: {}.{}", iface_name, method_name)));
            }
        }
        Ok(ControlFlow::Continue)
    }

    /// Execute BoxCall instruction
    pub(super) fn execute_boxcall(&mut self, dst: Option<ValueId>, box_val: ValueId, method: &str, method_id: Option<u16>, args: &[ValueId]) -> Result<ControlFlow, VMError> {
        let recv = self.get_value(box_val)?;
        
        // Debug logging if enabled
        let debug_boxcall = std::env::var("NYASH_VM_DEBUG_BOXCALL").is_ok();
        
        // Record PIC hit (per-receiver-type × method)
        let pic_key = self.build_pic_key(&recv, method, method_id);
        self.pic_record_hit(&pic_key);

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
                                let mut vm_args = Vec::with_capacity(1 + args.len());
                                vm_args.push(recv.clone());
                                for a in args { vm_args.push(self.get_value(*a)?); }
                                let res = self.call_function_by_name(&func_name, vm_args)?;
                                if let Some(dst_id) = dst { self.set_value(dst_id, res); }
                                return Ok(ControlFlow::Continue);
                            }
                            crate::runtime::type_meta::ThunkTarget::PluginInvoke { method_id: mid2 } => {
                                if is_plugin {
                                    if let Some(p) = arc_box.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                                        // Convert args prepared earlier (we need NyashBox args)
                                        let nyash_args: Vec<Box<dyn NyashBox>> = args.iter()
                                            .map(|arg| {
                                                let val = self.get_value(*arg)?;
                                                Ok(val.to_nyash_box())
                                            })
                                            .collect::<Result<Vec<_>, VMError>>()?;
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
                                            if code == 0 {
                                                let vm_out = if let Some((_tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(&out[..out_len]) {
                                                    let s = crate::runtime::plugin_ffi_common::decode::string(payload);
                                                    if !s.is_empty() {
                                                        VMValue::String(s)
                                                    } else if let Some(v) = crate::runtime::plugin_ffi_common::decode::i32(payload) {
                                                        VMValue::Integer(v as i64)
                                                    } else {
                                                        VMValue::Void
                                                    }
                                                } else {
                                                    VMValue::Void
                                                };
                                                if let Some(dst_id) = dst { self.set_value(dst_id, vm_out); }
                                                return Ok(ControlFlow::Continue);
                                            }
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
                                    let cloned_box = arc_box.share_box();
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
                    let mut vm_args = Vec::with_capacity(1 + args.len());
                    vm_args.push(recv.clone());
                    for a in args { vm_args.push(self.get_value(*a)?); }
                    let res = self.call_function_by_name(&func_name, vm_args)?;
                    if let Some(dst_id) = dst { self.set_value(dst_id, res); }
                    return Ok(ControlFlow::Continue);
                }
                // Fallback to Mono-PIC (legacy) if present
                if let Some(func_name) = self.boxcall_pic_funcname.get(&pic_key).cloned() {
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
                        return Ok(ControlFlow::Continue);
                    }
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
