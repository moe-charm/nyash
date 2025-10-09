/*!
 * BoxCall Elimination Pass - WASM Backend
 *
 * Transforms BoxCall instructions into unified Call forms better suited
 * for WASM codegen. This pass is optional and safe: unhandled cases are
 * left intact to preserve semantics.
 */

use crate::mir::{
    basic_block::BasicBlockId,
    definitions::call_unified::{Callee, TypeCertainty},
    BasicBlock, MirFunction, MirInstruction, MirModule, ValueId,
};

/// BoxCall Eliminator for WASM backend
pub struct BoxCallEliminator;

impl BoxCallEliminator {
    /// Lower a MIR function, transforming BoxCall instructions where profitable.
    pub fn lower_function(func: &MirFunction) -> MirFunction {
        let mut out = func.clone();
        let block_ids: Vec<BasicBlockId> = out.block_ids();
        for bid in block_ids {
            if let Some(block) = out.get_block_mut(bid) {
                // Map instructions
                let mut new_instructions: Vec<MirInstruction> = Vec::with_capacity(block.instructions.len());
                for inst in block.instructions.drain(..) {
                    match inst {
                        MirInstruction::BoxCall { dst, box_val, method, method_id: _, args, effects } => {
                            let lowered = Self::transform_boxcall(&out, dst, box_val, &method, &args, effects);
                            new_instructions.push(lowered);
                        }
                        other => new_instructions.push(other),
                    }
                }
                block.instructions = new_instructions;
            }
        }
        out
    }

    /// Transform a BoxCall instruction based on receiver type and method.
    fn transform_boxcall(
        func: &MirFunction,
        dst: Option<ValueId>,
        box_val: ValueId,
        method: &str,
        args: &[ValueId],
        effects: crate::mir::EffectMask,
    ) -> MirInstruction {
        // Determine receiver type from function metadata (best-effort)
        let receiver_type = func.metadata.value_types.get(&box_val);

        // Phase 1: ArrayBox support → prefer Method callee with explicit receiver
        if Self::is_box_type(receiver_type, "ArrayBox") {
            return MirInstruction::Call {
                dst,
                func: ValueId::new(0),
                callee: Some(Callee::Method {
                    box_name: "ArrayBox".to_string(),
                    method: method.to_string(),
                    receiver: Some(box_val),
                    certainty: TypeCertainty::Known,
                }),
                args: args.to_vec(),
                effects,
            };
        }

        // Phase 2: MapBox support → Extern("nybox.map.<method>")
        if Self::is_box_type(receiver_type, "MapBox") {
            return MirInstruction::Call {
                dst,
                func: ValueId::new(0),
                callee: Some(Callee::Extern(format!("nybox.map.{}", method))),
                // Convention: first arg is receiver handle
                args: {
                    let mut v = Vec::with_capacity(1 + args.len());
                    v.push(box_val);
                    v.extend_from_slice(args);
                    v
                },
                effects,
            };
        }

        // Phase 2: any.toString → Extern("nybox.any.toString")
        if method == "toString" {
            return MirInstruction::Call {
                dst,
                func: ValueId::new(0),
                callee: Some(Callee::Extern("nybox.any.toString".to_string())),
                args: vec![box_val],
                effects,
            };
        }

        // Default: leave as-is to preserve semantics (caller may handle BoxCall)
        MirInstruction::BoxCall {
            dst,
            box_val,
            method: method.to_string(),
            method_id: None,
            args: args.to_vec(),
            effects,
        }
    }

    fn is_box_type(ty: Option<&crate::mir::MirType>, name: &str) -> bool {
        matches!(ty, Some(crate::mir::MirType::Box(bt)) if bt == name)
    }
}

impl BoxCallEliminator {
    /// Lower an entire module (helper)
    #[allow(dead_code)]
    pub fn lower_module(module: &MirModule) -> MirModule {
        let mut out = module.clone();
        for (_name, f) in out.functions.iter_mut() {
            let lowered = Self::lower_function(f);
            *f = lowered;
        }
        out
    }
}

