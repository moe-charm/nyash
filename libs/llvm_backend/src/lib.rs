use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn llvm_compile_mir_to_object(
    mir_json_path: *const c_char,
    output_path: *const c_char,
) -> i64 {
    // Safety: validate pointers
    if mir_json_path.is_null() || output_path.is_null() {
        return -1;
    }
    let mir_path = unsafe { match CStr::from_ptr(mir_json_path).to_str() { Ok(s) => s.to_string(), Err(_) => return -1 } };
    let out_path = unsafe { match CStr::from_ptr(output_path).to_str() { Ok(s) => s.to_string(), Err(_) => return -1 } };

    // Call the Python llvmlite harness to compile MIR JSON → object file
    // Expect script at tools/llvmlite_harness.py relative to repository root
    // Use NYASH_ROOT when available to resolve the script path
    let harness = std::env::var("NYASH_ROOT")
        .map(|r| format!("{}/tools/llvmlite_harness.py", r))
        .unwrap_or_else(|_| "tools/llvmlite_harness.py".to_string());

    let status = std::process::Command::new("python3")
        .arg(&harness)
        .arg(&mir_path)
        .arg("-o")
        .arg(&out_path)
        .status();

    match status {
        Ok(s) if s.success() => 0,
        _ => -1,
    }
}

#[no_mangle]
pub extern "C" fn llvm_compile_mir_to_ll(
    mir_json_path: *const c_char,
    output_ll_path: *const c_char,
) -> i64 {
    // Safety: validate pointers
    if mir_json_path.is_null() || output_ll_path.is_null() {
        return -1;
    }
    let mir_path = unsafe {
        match CStr::from_ptr(mir_json_path).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return -1,
        }
    };
    let out_ll = unsafe {
        match CStr::from_ptr(output_ll_path).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return -1,
        }
    };

    // Call the Python llvmlite harness to compile MIR JSON → LLVM IR (.ll)
    let harness = std::env::var("NYASH_ROOT")
        .map(|r| format!("{}/tools/llvmlite_harness.py", r))
        .unwrap_or_else(|_| "tools/llvmlite_harness.py".to_string());

    let status = std::process::Command::new("python3")
        .arg(&harness)
        .arg("--in")
        .arg(&mir_path)
        .arg("--emit-ll")
        .arg(&out_ll)
        .status();

    match status {
        Ok(s) if s.success() => 0,
        _ => -1,
    }
}

