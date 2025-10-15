//! Ny compiler configuration

pub fn use_ny_compiler_exe() -> bool {
    std::env::var("NYASH_USE_NY_COMPILER_EXE").ok().as_deref() == Some("1")
}

pub fn ny_compiler_exe_path() -> String {
    std::env::var("NYASH_NY_COMPILER_EXE_PATH")
        .unwrap_or_else(|_| "./target/release/ny-compiler".to_string())
}

pub fn ny_compiler_timeout_ms() -> u64 {
    std::env::var("NYASH_NY_COMPILER_TIMEOUT_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30000)
}

pub fn ny_compiler_child_args() -> Option<String> {
    std::env::var("NYASH_NY_COMPILER_CHILD_ARGS").ok()
}

pub fn ny_compiler_use_tmp_only() -> bool {
    std::env::var("NYASH_NY_COMPILER_USE_TMP_ONLY").ok().as_deref() == Some("1")
}

pub fn ny_compiler_skip_py() -> bool {
    std::env::var("NYASH_NY_COMPILER_SKIP_PY").ok().as_deref() == Some("1")
}

pub fn ny_compiler_emit_only() -> bool {
    std::env::var("NYASH_NY_COMPILER_EMIT_ONLY").ok().as_deref() == Some("1")
}

pub fn ny_compiler_stage3() -> bool {
    std::env::var("NYASH_NY_COMPILER_STAGE3").ok().as_deref() == Some("1")
}

pub fn ny_compiler_min_json() -> bool {
    std::env::var("NYASH_NY_COMPILER_MIN_JSON").ok().as_deref() == Some("1")
}
