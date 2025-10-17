//! LoopFormBox — Loop structure normalization for PHI bug prevention
//!
//! **Purpose**: Enforce loop structure normalization to structurally prevent PHI bugs
//!
//! **Design Principles**:
//! 1. **Scope Boundary Enforcement**: Prohibit variable binding in Header
//! 2. **Side-Effect Isolation**: Build condition expression in separate block
//! 3. **Structure Verification**: Detect structure violations immediately with Verifier
//!
//! **Core Idea**:
//! ```
//! Header = PHI group + Branch only (structure enforcement)
//! Condition expression = Built in separate block (side-effect isolation)
//! Temporary values = pin slots (__pin$...) only
//! ```
//!
//! **Integration with ValueIdAllocatorBox**:
//! - PHI value allocation uses `builder.safe_next_value()` → collision impossible
//! - Dual guarantee: Path normalization (ValueIdAllocatorBox) + Structure normalization (LoopFormBox)

use crate::mir::{BasicBlockId, MirBuilder, MirFunction, MirInstruction, ValueId};
use std::collections::{HashMap, HashSet};

/// LoopForm structure guarantee Box
pub struct LoopFormBox {
    /// Header block ID
    pub header_bb: BasicBlockId,

    /// Condition block ID (side-effect isolation)
    pub condition_bb: Option<BasicBlockId>,

    /// PHI nodes (carrier variables only)
    pub phi_nodes: Vec<PhiNode>,

    /// Carrier variables
    pub carrier_vars: HashSet<String>,

    /// Pin slots (temporary values)
    pub pin_slots: Vec<ValueId>,

    /// Preheader block ID
    pub preheader_bb: BasicBlockId,

    /// Latch block ID
    pub latch_bb: Option<BasicBlockId>,

    /// Exit block ID
    pub exit_bb: Option<BasicBlockId>,

    /// Latch variable values (captured after body, before latch)
    latch_vars: HashMap<String, ValueId>,
}

/// PHI node descriptor
#[derive(Debug, Clone)]
pub struct PhiNode {
    /// Variable name
    pub var_name: String,

    /// PHI value (result)
    pub phi_value: ValueId,

    /// Preheader input
    pub preheader_input: ValueId,

    /// Latch input (None until Latch is determined)
    pub latch_input: Option<ValueId>,
}

/// LoopForm structure (output)
#[derive(Debug, Clone)]
pub struct LoopStructure {
    /// Header block ID
    pub header_bb: BasicBlockId,

    /// Condition block ID
    pub condition_bb: Option<BasicBlockId>,

    /// Body block ID
    pub body_bb: BasicBlockId,

    /// Latch block ID
    pub latch_bb: BasicBlockId,

    /// Exit block ID
    pub exit_bb: BasicBlockId,

    /// PHI nodes
    pub phi_nodes: Vec<PhiNode>,

    /// Carrier variables
    pub carrier_vars: HashSet<String>,
}

impl LoopFormBox {
    /// Create LoopFormBox
    ///
    /// **preheader_bb**: Preheader block ID (current block before loop construction)
    pub fn new(preheader_bb: BasicBlockId) -> Self {
        Self {
            header_bb: BasicBlockId::new(0), // Placeholder, will be set in build_loop()
            condition_bb: None,
            phi_nodes: Vec::new(),
            carrier_vars: HashSet::new(),
            pin_slots: Vec::new(),
            preheader_bb,
            latch_bb: None,
            exit_bb: None,
            latch_vars: HashMap::new(),
        }
    }

    /// Identify carrier variables (preheader definitions ∩ body assignments)
    ///
    /// **preheader_vars**: Variables defined in preheader (from builder.variable_map)
    /// **body**: Loop body statements (ASTNode slice)
    ///
    /// **Returns**: Set of carrier variable names
    pub fn identify_carriers(
        &mut self,
        preheader_vars: &HashMap<String, ValueId>,
        body: &[crate::ast::ASTNode],
    ) -> Result<HashSet<String>, String> {
        // Use existing LoopCarrierAnalyzerBox::analyze() (static method)
        let carriers = super::carrier_analyzer::LoopCarrierAnalyzerBox::analyze(preheader_vars, body);

        self.carrier_vars = carriers.clone();

        // Trace output (dev-only)
        if crate::runtime::env_gate_box::bool_any(&[
            "HAKO_TRACE_LOOPFORM",
            "NYASH_TRACE_LOOPFORM",
        ]) {
            eprintln!(
                "[loopform] 🔍 identify_carriers: found {} carriers: {:?}",
                carriers.len(),
                carriers
            );
        }

        Ok(carriers)
    }

    /// Build loop structure (main entry point)
    ///
    /// **loop_builder**: Loop builder (for continue/break support)
    /// **condition**: Loop condition expression
    /// **preheader_vars**: Variables defined in preheader
    /// **body**: Loop body statements
    ///
    /// **Returns**: LoopStructure (Header/Condition/Body/Latch/Exit block IDs + PHI nodes)
    pub fn build_loop(
        &mut self,
        loop_builder: &mut super::LoopBuilder,
        condition: &crate::ast::ASTNode,
        preheader_vars: &HashMap<String, ValueId>,
        body: &[crate::ast::ASTNode],
    ) -> Result<LoopStructure, String> {
        // Step 1: Identify carrier variables
        let carriers = self.identify_carriers(preheader_vars, body)?;

        // Trace output
        if crate::runtime::env_gate_box::bool_any(&[
            "HAKO_TRACE_LOOPFORM",
            "NYASH_TRACE_LOOPFORM",
        ]) {
            eprintln!(
                "[loopform] 🏗️ build_loop: {} carriers identified",
                carriers.len()
            );
        }

        // Step 2: Create Header block (PHI nodes only)
        let header_bb = self.create_header(loop_builder.parent_builder)?;

        // Step 3: Create Condition block (side-effect isolation)
        let cond_bb = self.create_condition_block(loop_builder.parent_builder, condition)?;

        // Step 4: Create Body/Latch/Exit blocks
        let (body_bb, latch_bb, exit_bb) = self.create_body_latch_exit(loop_builder, body)?;

        // Step 5: Update PHI inputs (Latch confirmed)
        self.update_phi_inputs(loop_builder.parent_builder, latch_bb)?;

        // Step 5.5: Wire control flow
        self.wire_control_flow(loop_builder.parent_builder, cond_bb, body_bb, exit_bb)?;

        // Step 6: Structure verification
        self.verify_structure(loop_builder.parent_builder)?;

        // Trace output
        if crate::runtime::env_gate_box::bool_any(&[
            "HAKO_TRACE_LOOPFORM",
            "NYASH_TRACE_LOOPFORM",
        ]) {
            // 🔥 CRITICAL DEBUG: Check condition block inst_count at the END of build_loop
            let final_cond_inst_count = if let Some(ref function) = loop_builder.parent_builder.current_function {
                if let Some(block) = function.get_block(cond_bb) {
                    block.instructions.len()
                } else {
                    0
                }
            } else {
                0
            };
            eprintln!(
                "[loopform] ✅ build_loop complete: header={:?} body={:?} latch={:?} exit={:?}",
                header_bb, body_bb, latch_bb, exit_bb
            );
            eprintln!(
                "[loopform] 🔥 FINAL CHECK: cond_bb={:?} final_inst_count={}",
                cond_bb, final_cond_inst_count
            );
        }

        Ok(LoopStructure {
            header_bb,
            condition_bb: Some(cond_bb),
            body_bb,
            latch_bb,
            exit_bb,
            phi_nodes: self.phi_nodes.clone(),
            carrier_vars: self.carrier_vars.clone(),
        })
    }

    // ========== Helper Methods (Day 3 implementation) ==========

    /// Create Header block (PHI + Branch only)
    ///
    /// **Enforces**: Header = PHI group + Branch (structure guarantee)
    fn create_header(&mut self, builder: &mut MirBuilder) -> Result<BasicBlockId, String> {
        // Create header block
        let header_bb = builder.block_gen.next();

        // Trace output
        if crate::runtime::env_gate_box::bool_any(&[
            "HAKO_TRACE_LOOPFORM",
            "NYASH_TRACE_LOOPFORM",
        ]) {
            eprintln!(
                "[loopform] 📍 create_header: header_bb={:?} carriers={:?}",
                header_bb, self.carrier_vars
            );
        }

        // Start the new header block
        builder.start_new_block(header_bb)?;

        // Convert carrier_vars to sorted Vec for stable iteration
        let mut carriers: Vec<String> = self.carrier_vars.iter().cloned().collect();
        carriers.sort();

        // For each carrier variable, create PHI node
        for var_name in carriers {
            // Get preheader value from variable_map
            let preheader_value = builder.variable_map.get(&var_name)
                .copied()
                .ok_or_else(|| format!("Carrier variable '{}' not found in variable_map", var_name))?;

            // Allocate PHI value using safe_next_value()
            let phi_value = builder.safe_next_value();

            // Emit PHI instruction with preheader input only (Latch will be added later)
            builder.emit_instruction(MirInstruction::Phi {
                dst: phi_value,
                inputs: vec![(self.preheader_bb, preheader_value)],
            })?;

            // Record PhiNode
            self.phi_nodes.push(PhiNode {
                var_name: var_name.clone(),
                phi_value,
                preheader_input: preheader_value,
                latch_input: None,
            });

            // Update variable_map to use PHI value
            builder.variable_map.insert(var_name.clone(), phi_value);

            // Trace output
            if crate::runtime::env_gate_box::bool_any(&[
                "HAKO_TRACE_LOOPFORM",
                "NYASH_TRACE_LOOPFORM",
            ]) {
                eprintln!(
                    "[loopform]   PHI: {} = PHI(bb{}:v%{}, bb?:TBD) -> v%{}",
                    var_name,
                    self.preheader_bb.as_u32(),
                    preheader_value.as_u32(),
                    phi_value.as_u32()
                );
            }
        }

        // Store header block ID
        self.header_bb = header_bb;

        // Trace output
        if crate::runtime::env_gate_box::bool_any(&[
            "HAKO_TRACE_LOOPFORM",
            "NYASH_TRACE_LOOPFORM",
        ]) {
            eprintln!(
                "[loopform] ✅ create_header complete: {} PHI nodes created",
                self.phi_nodes.len()
            );
        }

        Ok(header_bb)
    }

    /// Create Condition block (side-effect isolation)
    ///
    /// **Enforces**: Condition expression built in separate block, variable_map snapshot/restore
    fn create_condition_block(
        &mut self,
        builder: &mut MirBuilder,
        condition: &crate::ast::ASTNode,
    ) -> Result<BasicBlockId, String> {
        // Create condition block
        let cond_bb = builder.block_gen.next();

        // Trace output
        if crate::runtime::env_gate_box::bool_any(&[
            "HAKO_TRACE_LOOPFORM",
            "NYASH_TRACE_LOOPFORM",
        ]) {
            eprintln!("[loopform] 📍 create_condition_block: cond_bb={:?}", cond_bb);
        }

        // Snapshot variable_map (to prevent side effects from affecting header scope)
        let snapshot = builder.variable_map.clone();

        // Set condition block as current
        builder.start_new_block(cond_bb)?;

        // Get instruction count before build_expression
        let inst_count_before = if let Some(ref function) = builder.current_function {
            if let Some(block) = function.get_block(cond_bb) {
                block.instructions.len()
            } else {
                0
            }
        } else {
            0
        };

        // Trace: current_block before build_expression
        if crate::runtime::env_gate_box::bool_any(&[
            "HAKO_TRACE_LOOPFORM",
            "NYASH_TRACE_LOOPFORM",
        ]) {
            let func_name = builder.current_function.as_ref()
                .map(|f| f.signature.name.as_str())
                .unwrap_or("<no-func>");
            eprintln!(
                "[loopform] 🔍 before build_expression: func={} current_block={:?} cond_bb={:?} inst_count={}",
                func_name,
                builder.current_block,
                cond_bb,
                inst_count_before
            );
        }

        // Build condition expression (using PHI values from Header)
        let cond_value = builder.build_expression(condition.clone())?;

        // Get instruction count after build_expression
        let inst_count_after = if let Some(ref function) = builder.current_function {
            if let Some(block) = function.get_block(cond_bb) {
                block.instructions.len()
            } else {
                0
            }
        } else {
            0
        };

        // Trace: current_block after build_expression
        if crate::runtime::env_gate_box::bool_any(&[
            "HAKO_TRACE_LOOPFORM",
            "NYASH_TRACE_LOOPFORM",
        ]) {
            let func_name = builder.current_function.as_ref()
                .map(|f| f.signature.name.as_str())
                .unwrap_or("<no-func>");
            eprintln!(
                "[loopform] 🔍 after build_expression: func={} current_block={:?} cond_bb={:?} cond_value=v%{} inst_count={}",
                func_name,
                builder.current_block,
                cond_bb,
                cond_value.as_u32(),
                inst_count_after
            );
        }

        // 🔥 CRITICAL VERIFICATION: Ensure build_expression() emitted instructions
        if inst_count_after == inst_count_before {
            return Err(format!(
                "⚠️ LOOPFORM BUG: build_expression() emitted 0 instructions in condition block {:?}. \
                 Condition value v%{} is undefined! This will cause VM execution to fail with 'use of undefined value'. \
                 Likely cause: Method vs Function compilation path difference.",
                cond_bb, cond_value.as_u32()
            ));
        }

        // Record pin slot (condition value for later branching)
        self.pin_slots.push(cond_value);

        // Restore variable_map (revert any side effects from condition evaluation)
        builder.variable_map = snapshot;

        // Store condition block ID
        self.condition_bb = Some(cond_bb);

        // Trace output
        if crate::runtime::env_gate_box::bool_any(&[
            "HAKO_TRACE_LOOPFORM",
            "NYASH_TRACE_LOOPFORM",
        ]) {
            eprintln!(
                "[loopform] ✅ create_condition_block complete: cond_value=v%{}",
                cond_value.as_u32()
            );
        }

        Ok(cond_bb)
    }

    /// Create Body/Latch/Exit blocks
    ///
    /// **Returns**: (body_bb, latch_bb, exit_bb)
    fn create_body_latch_exit(
        &mut self,
        loop_builder: &mut super::LoopBuilder,
        body: &[crate::ast::ASTNode],
    ) -> Result<(BasicBlockId, BasicBlockId, BasicBlockId), String> {
        // Create body/latch/exit blocks
        let body_bb = loop_builder.parent_builder.block_gen.next();
        let latch_bb = loop_builder.parent_builder.block_gen.next();
        let exit_bb = loop_builder.parent_builder.block_gen.next();

        // Trace output
        if crate::runtime::env_gate_box::bool_any(&[
            "HAKO_TRACE_LOOPFORM",
            "NYASH_TRACE_LOOPFORM",
        ]) {
            eprintln!(
                "[loopform] 📍 create_body_latch_exit: body={:?} latch={:?} exit={:?}",
                body_bb, latch_bb, exit_bb
            );
        }

        // Build Body block
        loop_builder.parent_builder.start_new_block(body_bb)?;

        // 🔥 FIX: Use loop_builder.build_statement() for continue/break support
        // Build body statements
        for stmt in body {
            loop_builder.build_statement(stmt.clone())?;

            // Check if block is terminated (break/continue/return)
            if let Some(ref func) = loop_builder.parent_builder.current_function {
                if let Some(block) = func.get_block(body_bb) {
                    if block.is_terminated() {
                        break;
                    }
                }
            }
        }

        // Capture variable_map for PHI latch inputs (after body, before latch)
        self.latch_vars = loop_builder.parent_builder.variable_map.clone();

        // Emit Jump from Body to Latch (if not already terminated)
        let body_terminated = if let Some(ref func) = loop_builder.parent_builder.current_function {
            if let Some(block) = func.get_block(body_bb) {
                block.is_terminated()
            } else {
                false
            }
        } else {
            false
        };

        if !body_terminated {
            loop_builder.parent_builder.emit_instruction(MirInstruction::Jump { target: latch_bb })?;
        }

        // Build Latch block
        loop_builder.parent_builder.start_new_block(latch_bb)?;

        // Emit Jump from Latch to Header
        loop_builder.parent_builder.emit_instruction(MirInstruction::Jump { target: self.header_bb })?;

        // Build Exit block (empty for now, caller will set as current for continuation)
        loop_builder.parent_builder.start_new_block(exit_bb)?;

        // Store block IDs
        self.latch_bb = Some(latch_bb);
        self.exit_bb = Some(exit_bb);

        // Trace output
        if crate::runtime::env_gate_box::bool_any(&[
            "HAKO_TRACE_LOOPFORM",
            "NYASH_TRACE_LOOPFORM",
        ]) {
            eprintln!("[loopform] ✅ create_body_latch_exit complete");
        }

        Ok((body_bb, latch_bb, exit_bb))
    }

    /// Update PHI inputs (Latch confirmed)
    ///
    /// **Purpose**: Update PHI nodes with Latch input values
    fn update_phi_inputs(
        &mut self,
        builder: &mut MirBuilder,
        latch_bb: BasicBlockId,
    ) -> Result<(), String> {
        // Trace output
        if crate::runtime::env_gate_box::bool_any(&[
            "HAKO_TRACE_LOOPFORM",
            "NYASH_TRACE_LOOPFORM",
        ]) {
            eprintln!(
                "[loopform] 📍 update_phi_inputs: latch_bb={:?} phi_nodes={}",
                latch_bb,
                self.phi_nodes.len()
            );
        }

        // For each PHI node, update with Latch input
        for phi_node in &mut self.phi_nodes {
            // Get Latch value from captured variable_map
            let latch_value = self.latch_vars
                .get(&phi_node.var_name)
                .copied()
                .ok_or_else(|| {
                    format!(
                        "Latch value for '{}' not found in latch_vars",
                        phi_node.var_name
                    )
                })?;

            // Update PhiNode record
            phi_node.latch_input = Some(latch_value);

            // Update PHI instruction in Header block
            if let Some(ref mut function) = builder.current_function {
                if let Some(block) = function.get_block_mut(self.header_bb) {
                    // Find the PHI instruction for this variable
                    for inst in block.instructions.iter_mut() {
                        if let MirInstruction::Phi { dst, inputs } = inst {
                            if *dst == phi_node.phi_value {
                                // Update inputs: add latch edge
                                inputs.push((latch_bb, latch_value));
                                break;
                            }
                        }
                    }
                } else {
                    return Err(format!("Header block {:?} not found", self.header_bb));
                }
            } else {
                return Err("No current function".to_string());
            }

            // Trace output
            if crate::runtime::env_gate_box::bool_any(&[
                "HAKO_TRACE_LOOPFORM",
                "NYASH_TRACE_LOOPFORM",
            ]) {
                eprintln!(
                    "[loopform]   PHI update: {} = PHI(bb{}:v%{}, bb{}:v%{}) -> v%{}",
                    phi_node.var_name,
                    self.preheader_bb.as_u32(),
                    phi_node.preheader_input.as_u32(),
                    latch_bb.as_u32(),
                    latch_value.as_u32(),
                    phi_node.phi_value.as_u32()
                );
            }
        }

        // Trace output
        if crate::runtime::env_gate_box::bool_any(&[
            "HAKO_TRACE_LOOPFORM",
            "NYASH_TRACE_LOOPFORM",
        ]) {
            eprintln!(
                "[loopform] ✅ update_phi_inputs complete: {} PHI nodes updated",
                self.phi_nodes.len()
            );
        }

        Ok(())
    }

    /// Wire control flow (Header→Condition, Condition→Body/Exit)
    ///
    /// **Purpose**: Connect loop blocks with control flow edges
    fn wire_control_flow(
        &self,
        builder: &mut MirBuilder,
        cond_bb: BasicBlockId,
        body_bb: BasicBlockId,
        exit_bb: BasicBlockId,
    ) -> Result<(), String> {
        // Trace output
        if crate::runtime::env_gate_box::bool_any(&[
            "HAKO_TRACE_LOOPFORM",
            "NYASH_TRACE_LOOPFORM",
        ]) {
            eprintln!(
                "[loopform] 📍 wire_control_flow: header→cond={:?}, cond→body/exit={:?}/{:?}",
                cond_bb, body_bb, exit_bb
            );
        }

        // Wire Header → Condition (Jump)
        if let Some(ref mut function) = builder.current_function {
            if let Some(block) = function.get_block_mut(self.header_bb) {
                // 🔧 FIX: Only add terminator if not already present
                if block.terminator.is_none() {
                    block.add_instruction(MirInstruction::Jump { target: cond_bb });
                }
            } else {
                return Err(format!("Header block {:?} not found", self.header_bb));
            }
        } else {
            return Err("No current function".to_string());
        }

        // Wire Condition → Body/Exit (Branch)
        let cond_value = self.pin_slots.get(0)
            .copied()
            .ok_or_else(|| "Condition value not found in pin_slots".to_string())?;

        if let Some(ref mut function) = builder.current_function {
            if let Some(block) = function.get_block_mut(cond_bb) {
                // 🔧 FIX: Only add terminator if not already present
                if block.terminator.is_none() {
                    block.add_instruction(MirInstruction::Branch {
                        condition: cond_value,
                        then_bb: body_bb,
                        else_bb: exit_bb,
                    });
                }
            } else {
                return Err(format!("Condition block {:?} not found", cond_bb));
            }
        } else {
            return Err("No current function".to_string());
        }

        // Trace output
        if crate::runtime::env_gate_box::bool_any(&[
            "HAKO_TRACE_LOOPFORM",
            "NYASH_TRACE_LOOPFORM",
        ]) {
            // Get final instruction count after wiring
            let final_inst_count = if let Some(ref function) = builder.current_function {
                if let Some(block) = function.get_block(cond_bb) {
                    block.instructions.len()
                } else {
                    0
                }
            } else {
                0
            };
            eprintln!("[loopform] ✅ wire_control_flow complete: cond_bb={:?} final_inst_count={}", cond_bb, final_inst_count);
        }

        Ok(())
    }

    /// Structure verification (LoopFormVerifierBox integration)
    ///
    /// **Purpose**: Verify loop structure conformance
    fn verify_structure(&self, builder: &MirBuilder) -> Result<(), String> {
        // Call LoopFormVerifierBox to verify header structure
        let result = super::loopform_verifier_box::LoopFormVerifierBox::verify_loop_header(
            builder,
            self.header_bb,
        );

        // Convert VerificationResult to Result<(), String>
        result.to_result()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loopform_box_new() {
        let bb0 = BasicBlockId::new(0);
        let loopform = LoopFormBox::new(bb0);

        assert_eq!(loopform.preheader_bb, bb0);
        assert_eq!(loopform.carrier_vars.len(), 0);
        assert_eq!(loopform.phi_nodes.len(), 0);
    }

    #[test]
    fn test_identify_carriers_empty() {
        let bb0 = BasicBlockId::new(0);
        let mut loopform = LoopFormBox::new(bb0);

        let preheader_vars = HashMap::new();
        let body: Vec<crate::ast::ASTNode> = vec![];

        let carriers = loopform.identify_carriers(&preheader_vars, &body).unwrap();
        assert_eq!(carriers.len(), 0);
    }
}
