use super::sanitize_symbol;
use crate::backend::llvm::context::CodegenContext;
use inkwell::values::BasicValueEnum;

pub(super) fn emit_wrapper_and_object<'ctx>(
    codegen: &CodegenContext<'ctx>,
    entry_name: &str,
    output_path: &str,
) -> Result<(), String> {
    let i64t = codegen.context.i64_type();
    let ny_main_ty = i64t.fn_type(&[], false);
    let ny_main = codegen.module.add_function("ny_main", ny_main_ty, None);
    let entry_bb = codegen.context.append_basic_block(ny_main, "entry");
    codegen.builder.position_at_end(entry_bb);
    let entry_sym = format!("ny_f_{}", sanitize_symbol(entry_name));
    let entry_fn = codegen
        .module
        .get_function(&entry_sym)
        .ok_or_else(|| format!("entry function symbol not found: {}", entry_sym))?;
    let call = codegen
        .builder
        .build_call(entry_fn, &[], "call_main")
        .map_err(|e| e.to_string())?;
    let rv = call.try_as_basic_value().left();
    let ret_v = if let Some(v) = rv {
        match v {
            BasicValueEnum::IntValue(iv) => {
                if iv.get_type().get_bit_width() == 64 {
                    iv
                } else {
                    codegen
                        .builder
                        .build_int_z_extend(iv, i64t, "ret_zext")
                        .map_err(|e| e.to_string())?
                }
            }
            BasicValueEnum::PointerValue(pv) => codegen
                .builder
                .build_ptr_to_int(pv, i64t, "ret_p2i")
                .map_err(|e| e.to_string())?,
            BasicValueEnum::FloatValue(fv) => codegen
                .builder
                .build_float_to_signed_int(fv, i64t, "ret_f2i")
                .map_err(|e| e.to_string())?,
            _ => i64t.const_zero(),
        }
    } else {
        i64t.const_zero()
    };
    codegen
        .builder
        .build_return(Some(&ret_v))
        .map_err(|e| e.to_string())?;
    if !ny_main.verify(true) {
        return Err("ny_main verification failed".to_string());
    }
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
            if std::fs::metadata(output_path).is_err() {
                let buf = codegen
                    .target_machine
                    .write_to_memory_buffer(&codegen.module, inkwell::targets::FileType::Object)
                    .map_err(|e| format!("Failed to get object buffer: {}", e))?;
                std::fs::write(output_path, buf.as_slice())
                    .map_err(|e| format!("Failed to write object to '{}': {}", output_path, e))?;
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
                eprintln!(
                    "[LLVM] emit complete (Err branch handled) for {}",
                    output_path
                );
            }
            Ok(())
        }
    }
}
