use super::LLVMCompiler;
use crate::backend::llvm::context::CodegenContext;
use crate::mir::function::MirModule;
use crate::mir::instruction::MirInstruction;
use crate::mir::ValueId;
use inkwell::context::Context;
use inkwell::types::BasicType; // for as_basic_type_enum()
use inkwell::{
    types::BasicTypeEnum,
    values::{BasicValueEnum, PhiValue},
};
use std::collections::HashMap;

mod instructions;
mod types;

use instructions::{lower_instruction, lower_terminator};
use types::{map_mirtype_to_basic, map_type};

impl LLVMCompiler {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            values: HashMap::new(),
        })
    }

    pub fn compile_module(&self, mir_module: &MirModule, output_path: &str) -> Result<(), String> {
        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
            eprintln!(
                "[LLVM] compile_module start: functions={}, out={}",
                mir_module.functions.len(),
                output_path
            );
        }
        let context = Context::create();
        let codegen = CodegenContext::new(&context, "nyash_module")?;
        // Lower only Main.main for now
        // Find entry function
        let func = if let Some((_n, f)) = mir_module
            .functions
            .iter()
            .find(|(_n, f)| f.metadata.is_entry_point)
        {
            f
        } else if let Some(f) = mir_module.functions.get("Main.main") {
            f
        } else if let Some(f) = mir_module.functions.get("main") {
            f
        } else if let Some((_n, f)) = mir_module.functions.iter().next() {
            f
        } else {
            return Err("Main.main function not found in module".to_string());
        };

        // Map MIR types to LLVM types via helpers

        // Load box type-id mapping from nyash_box.toml (central plugin registry)
        let box_type_ids = crate::backend::llvm::box_types::load_box_type_ids();

        // Function type
        let ret_type = match func.signature.return_type {
            crate::mir::MirType::Void => None,
            ref t => Some(map_type(codegen.context, t)?),
        };
        let fn_type = match ret_type {
            Some(BasicTypeEnum::IntType(t)) => t.fn_type(&[], false),
            Some(BasicTypeEnum::FloatType(t)) => t.fn_type(&[], false),
            Some(BasicTypeEnum::PointerType(t)) => t.fn_type(&[], false),
            Some(_) => return Err("Unsupported return basic type".to_string()),
            None => codegen.context.void_type().fn_type(&[], false),
        };
        let llvm_func = codegen.module.add_function("ny_main", fn_type, None);

        // Create LLVM basic blocks: ensure entry is created first to be function entry
        let mut bb_map: HashMap<crate::mir::BasicBlockId, inkwell::basic_block::BasicBlock> =
            HashMap::new();
        let entry_first = func.entry_block;
        let entry_bb = codegen
            .context
            .append_basic_block(llvm_func, &format!("bb{}", entry_first.as_u32()));
        bb_map.insert(entry_first, entry_bb);
        for bid in func.block_ids() {
            if bid == entry_first {
                continue;
            }
            let name = format!("bb{}", bid.as_u32());
            let bb = codegen.context.append_basic_block(llvm_func, &name);
            bb_map.insert(bid, bb);
        }

        // Position at entry
        codegen.builder.position_at_end(entry_bb);

        // SSA value map
        let mut vmap: HashMap<ValueId, BasicValueEnum> = HashMap::new();

        // Pre-create allocas for locals on demand (entry-only builder)
        let mut allocas: HashMap<ValueId, PointerValue> = HashMap::new();
        let mut entry_builder = codegen.context.create_builder();
        entry_builder.position_at_end(entry_bb);

        // Helper: map MirType to LLVM basic type (value type)

        // Helper: create (or get) an alloca for a given pointer-typed SSA value id
        let mut alloca_elem_types: HashMap<ValueId, BasicTypeEnum> = HashMap::new();

        // Pre-create PHI nodes for all blocks (so we can add incoming from predecessors)
        let mut phis_by_block: HashMap<
            crate::mir::BasicBlockId,
            Vec<(ValueId, PhiValue, Vec<(crate::mir::BasicBlockId, ValueId)>)>,
        > = HashMap::new();
        for bid in func.block_ids() {
            let bb = *bb_map.get(&bid).ok_or("missing bb in map")?;
            // Position at start of the block (no instructions emitted yet)
            codegen.builder.position_at_end(bb);
            let block = func.blocks.get(&bid).unwrap();
            for inst in block
                .instructions
                .iter()
                .take_while(|i| matches!(i, MirInstruction::Phi { .. }))
            {
                if let MirInstruction::Phi { dst, inputs } = inst {
                    // Decide PHI type: prefer annotated value type; fallback to first input's annotated type; finally i64
                    let mut phi_ty: Option<BasicTypeEnum> = None;
                    if let Some(mt) = func.metadata.value_types.get(dst) {
                        phi_ty = Some(map_mirtype_to_basic(codegen.context, mt));
                    } else if let Some((_, iv)) = inputs.first() {
                        if let Some(mt) = func.metadata.value_types.get(iv) {
                            phi_ty = Some(map_mirtype_to_basic(codegen.context, mt));
                        }
                    }
                    let phi_ty = phi_ty.unwrap_or_else(|| codegen.context.i64_type().into());
                    let phi = codegen
                        .builder
                        .build_phi(phi_ty, &format!("phi_{}", dst.as_u32()))
                        .map_err(|e| e.to_string())?;
                    vmap.insert(*dst, phi.as_basic_value());
                    phis_by_block
                        .entry(bid)
                        .or_default()
                        .push((*dst, phi, inputs.clone()));
                }
            }
        }

        // Lower in block order
        for bid in func.block_ids() {
            let bb = *bb_map.get(&bid).unwrap();
            if codegen
                .builder
                .get_insert_block()
                .map(|b| b != bb)
                .unwrap_or(true)
            {
                codegen.builder.position_at_end(bb);
            }
            let block = func.blocks.get(&bid).unwrap();
            for inst in &block.instructions {
                lower_instruction(
                    inst,
                    &codegen,
                    func,
                    bid,
                    &mut vmap,
                    &mut allocas,
                    &entry_builder,
                    &mut alloca_elem_types,
                    &phis_by_block,
                    &bb_map,
                    &box_type_ids,
                )?;
            }
            if let Some(term) = &block.terminator {
                lower_terminator(
                    term,
                    &codegen,
                    func,
                    bid,
                    &mut vmap,
                    &phis_by_block,
                    &bb_map,
                )?;
            }
        }

        // Verify and emit
        if !llvm_func.verify(true) {
            return Err("Function verification failed".to_string());
        }
        // Try writing via file API first; if it succeeds but file is missing due to env/FS quirks,
        // also write via memory buffer as a fallback to ensure presence.
        let verbose = std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1");
        if verbose {
            eprintln!("[LLVM] emitting object to {} (begin)", output_path);
        }
        match codegen.target_machine.write_to_file(
            &codegen.module,
            inkwell::targets::FileType::Object,
            std::path::Path::new(output_path),
        ) {
            Ok(_) => {
                // Verify; if missing, fallback to memory buffer write
                if std::fs::metadata(output_path).is_err() {
                    let buf = codegen
                        .target_machine
                        .write_to_memory_buffer(&codegen.module, inkwell::targets::FileType::Object)
                        .map_err(|e| format!("Failed to get object buffer: {}", e))?;
                    std::fs::write(output_path, buf.as_slice()).map_err(|e| {
                        format!("Failed to write object to '{}': {}", output_path, e)
                    })?;
                    if verbose {
                        eprintln!(
                            "[LLVM] wrote object via memory buffer fallback: {} ({} bytes)",
                            output_path,
                            buf.get_size()
                        );
                    }
                } else if verbose {
                    if let Ok(meta) = std::fs::metadata(output_path) {
                        eprintln!(
                            "[LLVM] wrote object via file API: {} ({} bytes)",
                            output_path,
                            meta.len()
                        );
                    }
                }
                if verbose {
                    eprintln!("[LLVM] emit complete (Ok branch) for {}", output_path);
                }
                Ok(())
            }
            Err(e) => {
                // Fallback: memory buffer
                let buf = codegen
                    .target_machine
                    .write_to_memory_buffer(&codegen.module, inkwell::targets::FileType::Object)
                    .map_err(|ee| {
                        format!(
                            "Failed to write object ({}); and memory buffer failed: {}",
                            e, ee
                        )
                    })?;
                std::fs::write(output_path, buf.as_slice()).map_err(|ee| {
                    format!(
                        "Failed to write object to '{}': {} (original error: {})",
                        output_path, ee, e
                    )
                })?;
                if verbose {
                    eprintln!(
                        "[LLVM] wrote object via error fallback: {} ({} bytes)",
                        output_path,
                        buf.get_size()
                    );
                }
                if verbose {
                    eprintln!(
                        "[LLVM] emit complete (Err branch handled) for {}",
                        output_path
                    );
                }
                Ok(())
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_creation() {
        let compiler = LLVMCompiler::new();
        assert!(compiler.is_ok());
    }
}
