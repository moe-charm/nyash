// ---- ExternCall helpers for LLVM lowering ----
// Exported as: nyash.console.log(i8* cstr) -> i64
#[export_name = "nyash.console.log"]
pub extern "C" fn nyash_console_log_export(ptr: *const i8) -> i64 {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        let c = std::ffi::CStr::from_ptr(ptr);
        if let Ok(s) = c.to_str() {
            println!("{}", s);
        }
    }
    0
}

// Exported as: nyash.console.log_handle(i64 handle) -> i64
#[export_name = "nyash.console.log_handle"]
pub extern "C" fn nyash_console_log_handle(handle: i64) -> i64 {
    use nyash_rust::runtime::host_handles as handles;
    eprintln!("DEBUG: handle={}", handle);
    if let Some(obj) = handles::get(handle as u64) {
        let s = obj.to_string_box().value;
        println!("{}", s);
    } else {
        eprintln!("DEBUG: handle {} not found in registry", handle);
        println!("{}", handle);
    }
    0
}

// Exported as: nyash.console.warn_handle(i64 handle) -> i64
#[export_name = "nyash.console.warn_handle"]
pub extern "C" fn nyash_console_warn_handle(handle: i64) -> i64 {
    if handle <= 0 {
        return 0;
    }

    if let Some(obj) = nyash_rust::runtime::host_handles::get(handle as u64) {
        let s = obj.to_string_box().value;
        eprintln!("WARN: {}", s);
    } else {
        eprintln!("WARN: {}", handle);
    }
    0
}

// Exported as: nyash.console.error_handle(i64 handle) -> i64
#[export_name = "nyash.console.error_handle"]
pub extern "C" fn nyash_console_error_handle(handle: i64) -> i64 {
    if handle <= 0 {
        return 0;
    }

    if let Some(obj) = nyash_rust::runtime::host_handles::get(handle as u64) {
        let s = obj.to_string_box().value;
        eprintln!("ERROR: {}", s);
    } else {
        eprintln!("ERROR: {}", handle);
    }
    0
}

// Exported as: nyash.debug.trace_handle(i64 handle) -> i64
#[export_name = "nyash.debug.trace_handle"]
pub extern "C" fn nyash_debug_trace_handle(handle: i64) -> i64 {
    if handle <= 0 {
        return 0;
    }

    if let Some(obj) = nyash_rust::runtime::host_handles::get(handle as u64) {
        let s = obj.to_string_box().value;
        eprintln!("TRACE: {}", s);
    } else {
        eprintln!("TRACE: {}", handle);
    }
    0
}

// Exported as: nyash.console.warn(i8* cstr) -> i64
#[export_name = "nyash.console.warn"]
pub extern "C" fn nyash_console_warn_export(ptr: *const i8) -> i64 {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        let c = std::ffi::CStr::from_ptr(ptr);
        if let Ok(s) = c.to_str() {
            eprintln!("[warn] {}", s);
        }
    }
    0
}

// Exported as: nyash.console.error(i8* cstr) -> i64
#[export_name = "nyash.console.error"]
pub extern "C" fn nyash_console_error_export(ptr: *const i8) -> i64 {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        let c = std::ffi::CStr::from_ptr(ptr);
        if let Ok(s) = c.to_str() {
            eprintln!("[error] {}", s);
        }
    }
    0
}

// Exported as: nyash.debug.trace(i8* cstr) -> i64
#[export_name = "nyash.debug.trace"]
pub extern "C" fn nyash_debug_trace_export(ptr: *const i8) -> i64 {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        let c = std::ffi::CStr::from_ptr(ptr);
        if let Ok(s) = c.to_str() {
            eprintln!("[trace] {}", s);
        }
    }
    0
}

// Exported as: nyash.console.readline() -> i8*
#[export_name = "nyash.console.readline"]
pub extern "C" fn nyash_console_readline_export() -> *mut i8 {
    use std::io;
    // Read a line from stdin; normalize to UTF-8 and strip trailing CR/LF
    let mut input = String::new();
    // Use read_to_end if stdin is not a TTY? Simpler: read_line through BufRead
    // For simplicity, read from stdin into buffer until newline or EOF
    let mut buf = String::new();
    // Note: use std::io::stdin() directly without an unused handle binding
    // On failure or EOF, return empty string
    match io::stdin().read_line(&mut buf) {
        Ok(_n) => {
            input = buf;
        }
        Err(_) => {
            input.clear();
        }
    }
    while input.ends_with('\n') || input.ends_with('\r') {
        input.pop();
    }
    // Allocate C string (null-terminated)
    let mut bytes = input.into_bytes();
    bytes.push(0);
    let boxed = bytes.into_boxed_slice();
    let raw = Box::into_raw(boxed) as *mut u8;
    raw as *mut i8
}
