/*!
 * MIR Instruction to WASM Conversion
 */

use crate::backend::wasm::{wasm_resolve_signature, WasmError, WasmExternSignature};
use crate::mir::{BinaryOp, CompareOp, ConstValue, MirInstruction, ValueId};

impl super::WasmCodegen {
    /// Generate WASM instructions for a single MIR instruction
    pub(super) fn generate_instruction(
        &mut self,
        instruction: &MirInstruction,
    ) -> Result<Vec<String>, WasmError> {
        match instruction {
            // Phase 8.2 PoC1: Basic operations
            MirInstruction::Const { dst, value } => self.generate_const(*dst, value),

            MirInstruction::BinOp { dst, op, lhs, rhs } => {
                self.generate_binop(*dst, *op, *lhs, *rhs)
            }

            MirInstruction::Compare { dst, op, lhs, rhs } => {
                self.generate_compare(*dst, *op, *lhs, *rhs)
            }

            MirInstruction::Return { value } => self.generate_return(value.as_ref()),

            MirInstruction::Print { value, .. } => self.generate_print(*value),

            // Phase 8.3 PoC2: Reference operations
            MirInstruction::RefNew { dst, box_val } => {
                // Create a new reference to a Box by copying the Box value
                // This assumes box_val contains a Box pointer already
                Ok(vec![
                    format!("local.get ${}", self.get_local_index(*box_val)?),
                    format!("local.set ${}", self.get_local_index(*dst)?),
                ])
            }

            MirInstruction::RefGet {
                dst,
                reference,
                field: _,
            } => {
                // Load field value from Box through reference
                // reference contains Box pointer, field is the field name
                // For now, assume all fields are at offset 12 (first field after header)
                // TODO: Add proper field offset calculation
                Ok(vec![
                    format!("local.get ${}", self.get_local_index(*reference)?),
                    "i32.const 12".to_string(), // Offset: header (12 bytes) + first field
                    "i32.add".to_string(),
                    "i32.load".to_string(),
                    format!("local.set ${}", self.get_local_index(*dst)?),
                ])
            }

            MirInstruction::RefSet {
                reference,
                field: _,
                value,
            } => {
                // Store field value to Box through reference
                // reference contains Box pointer, field is the field name, value is new value
                // For now, assume all fields are at offset 12 (first field after header)
                // TODO: Add proper field offset calculation
                Ok(vec![
                    format!("local.get ${}", self.get_local_index(*reference)?),
                    "i32.const 12".to_string(), // Offset: header (12 bytes) + first field
                    "i32.add".to_string(),
                    format!("local.get ${}", self.get_local_index(*value)?),
                    "i32.store".to_string(),
                ])
            }

            MirInstruction::NewBox {
                dst,
                box_type,
                args,
            } => {
                // Create a new Box using the generic allocator
                match box_type.as_str() {
                    "DataBox" => {
                        // Use specific allocator for known types
                        let mut instructions = vec![
                            "call $alloc_databox".to_string(),
                            format!("local.set ${}", self.get_local_index(*dst)?),
                        ];

                        // Initialize fields with arguments if provided
                        for (i, arg) in args.iter().enumerate() {
                            instructions.extend(vec![
                                format!("local.get ${}", self.get_local_index(*dst)?),
                                format!("i32.const {}", 12 + i * 4), // Field offset
                                "i32.add".to_string(),
                                format!("local.get ${}", self.get_local_index(*arg)?),
                                "i32.store".to_string(),
                            ]);
                        }

                        Ok(instructions)
                    }
                    _ => {
                        // Use generic allocator for unknown types
                        // This is a fallback - in a real implementation, all Box types should be known
                        Ok(vec![
                            "i32.const 8192".to_string(), // Default unknown type ID
                            format!("i32.const {}", args.len()),
                            "call $box_alloc".to_string(),
                            format!("local.set ${}", self.get_local_index(*dst)?),
                        ])
                    }
                }
            }

            // Phase 8.4 PoC3: Extension stubs
            MirInstruction::WeakNew { dst, box_val }
            | MirInstruction::FutureNew {
                dst,
                value: box_val,
            } => {
                // Treat as regular reference for now
                Ok(vec![
                    format!("local.get ${}", self.get_local_index(*box_val)?),
                    format!("local.set ${}", self.get_local_index(*dst)?),
                ])
            }

            MirInstruction::WeakLoad { dst, weak_ref }
            | MirInstruction::Await {
                dst,
                future: weak_ref,
            } => {
                // Always succeed for now
                Ok(vec![
                    format!("local.get ${}", self.get_local_index(*weak_ref)?),
                    format!("local.set ${}", self.get_local_index(*dst)?),
                ])
            }

            MirInstruction::BarrierRead { .. }
            | MirInstruction::BarrierWrite { .. }
            | MirInstruction::FutureSet { .. }
            | MirInstruction::Safepoint => {
                // No-op for now
                Ok(vec!["nop".to_string()])
            }

            // Control Flow Instructions (Critical for loops and conditions)
            MirInstruction::Jump { target } => {
                // Unconditional jump to target basic block
                // Use WASM br instruction to break to the target block
                Ok(vec![format!("br $block_{}", target.as_u32())])
            }

            MirInstruction::Branch {
                condition,
                then_bb,
                else_bb,
            } => {
                // Conditional branch based on condition value
                // Load condition value and branch accordingly
                Ok(vec![
                    // Load condition value onto stack
                    format!("local.get ${}", self.get_local_index(*condition)?),
                    // If condition is true (non-zero), branch to then_bb
                    format!("br_if $block_{}", then_bb.as_u32()),
                    // Otherwise, fall through to else_bb
                    format!("br $block_{}", else_bb.as_u32()),
                ])
            }

            // Phase 9.7: External Function Calls
            MirInstruction::ExternCall {
                dst,
                iface_name,
                method_name,
                args,
                effects: _,
            } => {
                // Generate call to external function import
                let signature = match (iface_name.as_str(), method_name.as_str()) {
                    ("env.console", "log") => Some(WasmExternSignature {
                        module: "env".to_string(),
                        name: "console_log".to_string(),
                        params: vec!["i32".to_string(), "i32".to_string()],
                        result: None,
                    }),
                    ("env.canvas", "fillRect") => Some(WasmExternSignature {
                        module: "env".to_string(),
                        name: "canvas_fillRect".to_string(),
                        params: vec![
                            "i32".to_string(),
                            "i32".to_string(),
                            "i32".to_string(),
                            "i32".to_string(),
                            "i32".to_string(),
                            "i32".to_string(),
                            "i32".to_string(),
                            "i32".to_string(),
                        ],
                        result: None,
                    }),
                    ("env.canvas", "fillText") => Some(WasmExternSignature {
                        module: "env".to_string(),
                        name: "canvas_fillText".to_string(),
                        params: vec![
                            "i32".to_string(),
                            "i32".to_string(),
                            "i32".to_string(),
                            "i32".to_string(),
                            "i32".to_string(),
                            "i32".to_string(),
                            "i32".to_string(),
                            "i32".to_string(),
                            "i32".to_string(),
                            "i32".to_string(),
                        ],
                        result: None,
                    }),
                    (iface, method) => wasm_resolve_signature(iface, method),
                };

                let signature = signature.ok_or_else(|| {
                    WasmError::UnsupportedInstruction(format!(
                        "Unsupported extern call: {}.{}",
                        iface_name, method_name
                    ))
                })?;

                let returns_value = signature.result.is_some();
                let call_target = signature.name;

                let mut instructions = Vec::new();

                // Load all arguments onto stack in order
                for arg in args {
                    instructions.push(format!("local.get ${}", self.get_local_index(*arg)?));
                }

                // Call the external function
                instructions.push(format!("call ${}", call_target));

                if returns_value {
                    if let Some(dst) = dst {
                        instructions.push(format!(
                            "local.set ${}",
                            self.get_local_index(*dst)?
                        ));
                    } else {
                        instructions.push("drop".to_string());
                    }
                } else if let Some(dst) = dst {
                    instructions.push("i32.const 0".to_string());
                    instructions.push(format!("local.set ${}", self.get_local_index(*dst)?));
                }

                Ok(instructions)
            }

            // BoxCall codegen - critical Box method calls
            MirInstruction::BoxCall {
                dst,
                box_val,
                method,
                args,
                effects: _,
                ..
            } => self.generate_box_call(*dst, *box_val, method, args),

            // Unsupported instructions
            _ => Err(WasmError::UnsupportedInstruction(format!(
                "Instruction not yet supported: {:?}",
                instruction
            ))),
        }
    }

    /// Generate constant loading
    pub(super) fn generate_const(
        &mut self,
        dst: ValueId,
        value: &ConstValue,
    ) -> Result<Vec<String>, WasmError> {
        let const_instruction = match value {
            ConstValue::Integer(n) => format!("i32.const {}", n),
            ConstValue::Bool(b) => format!("i32.const {}", if *b { 1 } else { 0 }),
            ConstValue::Void => "i32.const 0".to_string(),
            ConstValue::String(s) => {
                // Register the string literal and get its offset
                let data_offset = self.register_string_literal(s);
                let string_len = s.len() as u32;

                // Generate code to allocate a StringBox and return its pointer
                // This is more complex and will need StringBox allocation
                return self.generate_string_box_const(dst, data_offset, string_len);
            }
            _ => {
                return Err(WasmError::UnsupportedInstruction(format!(
                    "Unsupported constant type: {:?}",
                    value
                )))
            }
        };

        Ok(vec![
            const_instruction,
            format!("local.set ${}", self.get_local_index(dst)?),
        ])
    }

    /// Generate binary operation
    pub(super) fn generate_binop(
        &self,
        dst: ValueId,
        op: BinaryOp,
        lhs: ValueId,
        rhs: ValueId,
    ) -> Result<Vec<String>, WasmError> {
        let wasm_op = match op {
            BinaryOp::Add => "i32.add",
            BinaryOp::Sub => "i32.sub",
            BinaryOp::Mul => "i32.mul",
            BinaryOp::Div => "i32.div_s",
            BinaryOp::And => "i32.and",
            BinaryOp::Or => "i32.or",
            _ => {
                return Err(WasmError::UnsupportedInstruction(format!(
                    "Unsupported binary operation: {:?}",
                    op
                )))
            }
        };

        Ok(vec![
            format!("local.get ${}", self.get_local_index(lhs)?),
            format!("local.get ${}", self.get_local_index(rhs)?),
            wasm_op.to_string(),
            format!("local.set ${}", self.get_local_index(dst)?),
        ])
    }

    /// Generate comparison operation
    pub(super) fn generate_compare(
        &self,
        dst: ValueId,
        op: CompareOp,
        lhs: ValueId,
        rhs: ValueId,
    ) -> Result<Vec<String>, WasmError> {
        let wasm_op = match op {
            CompareOp::Eq => "i32.eq",
            CompareOp::Ne => "i32.ne",
            CompareOp::Lt => "i32.lt_s",
            CompareOp::Le => "i32.le_s",
            CompareOp::Gt => "i32.gt_s",
            CompareOp::Ge => "i32.ge_s",
        };

        Ok(vec![
            format!("local.get ${}", self.get_local_index(lhs)?),
            format!("local.get ${}", self.get_local_index(rhs)?),
            wasm_op.to_string(),
            format!("local.set ${}", self.get_local_index(dst)?),
        ])
    }

    /// Generate return instruction
    pub(super) fn generate_return(
        &self,
        value: Option<&ValueId>,
    ) -> Result<Vec<String>, WasmError> {
        if let Some(value_id) = value {
            Ok(vec![
                format!("local.get ${}", self.get_local_index(*value_id)?),
                "return".to_string(),
            ])
        } else {
            Ok(vec!["return".to_string()])
        }
    }
}
