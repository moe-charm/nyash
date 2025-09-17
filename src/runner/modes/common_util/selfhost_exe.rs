use std::io::Read;
use std::process::Stdio;
use std::thread::sleep;
use std::time::{Duration, Instant};

/// Try external selfhost compiler EXE to parse Ny -> JSON v0 and return MIR module.
/// Returns Some(module) on success, None on failure (timeout/invalid output/missing exe)
pub fn exe_try_parse_json_v0(filename: &str, timeout_ms: u64) -> Option<crate::mir::MirModule> {
    // Resolve parser EXE path
    let exe_path = if let Ok(p) = std::env::var("NYASH_NY_COMPILER_EXE_PATH") {
        std::path::PathBuf::from(p)
    } else {
        let mut p = std::path::PathBuf::from("dist/nyash_compiler");
        #[cfg(windows)]
        {
            p.push("nyash_compiler.exe");
        }
        #[cfg(not(windows))]
        {
            p.push("nyash_compiler");
        }
        if !p.exists() {
            if let Ok(w) = which::which("nyash_compiler") {
                w
            } else {
                p
            }
        } else {
            p
        }
    };
    if !exe_path.exists() {
        crate::cli_v!("[ny-compiler] exe not found at {}", exe_path.display());
        return None;
    }
    // Build command
    let mut cmd = std::process::Command::new(&exe_path);
    cmd.arg(filename);
    if crate::config::env::ny_compiler_min_json() {
        cmd.arg("--min-json");
    }
    if crate::config::env::selfhost_read_tmp() {
        cmd.arg("--read-tmp");
    }
    if let Some(raw) = crate::config::env::ny_compiler_child_args() {
        for tok in raw.split_whitespace() {
            cmd.arg(tok);
        }
    }
    let mut cmd = cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[ny-compiler] exe spawn failed: {}", e);
            return None;
        }
    };
    let mut ch_stdout = child.stdout.take();
    let mut ch_stderr = child.stderr.take();
    let start = Instant::now();
    let mut timed_out = false;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if start.elapsed() >= Duration::from_millis(timeout_ms) {
                    let _ = child.kill();
                    let _ = child.wait();
                    timed_out = true;
                    break;
                }
                sleep(Duration::from_millis(10));
            }
            Err(e) => {
                eprintln!("[ny-compiler] exe wait error: {}", e);
                return None;
            }
        }
    }
    let mut out_buf = Vec::new();
    let mut err_buf = Vec::new();
    if let Some(mut s) = ch_stdout {
        let _ = s.read_to_end(&mut out_buf);
    }
    if let Some(mut s) = ch_stderr {
        let _ = s.read_to_end(&mut err_buf);
    }
    if timed_out {
        let head = String::from_utf8_lossy(&out_buf)
            .chars()
            .take(200)
            .collect::<String>();
        eprintln!(
            "[ny-compiler] exe timeout after {} ms; stdout(head)='{}'",
            timeout_ms,
            head.replace('\n', "\\n")
        );
        return None;
    }
    let stdout = match String::from_utf8(out_buf) {
        Ok(s) => s,
        Err(_) => String::new(),
    };
    let mut json_line = String::new();
    for line in stdout.lines() {
        let t = line.trim();
        if t.starts_with('{') && t.contains("\"version\"") && t.contains("\"kind\"") {
            json_line = t.to_string();
            break;
        }
    }
    if json_line.is_empty() {
        if crate::config::env::cli_verbose() {
            let head: String = stdout.chars().take(200).collect();
            let errh: String = String::from_utf8_lossy(&err_buf).chars().take(200).collect();
            crate::cli_v!(
                "[ny-compiler] exe produced no JSON; stdout(head)='{}' stderr(head)='{}'",
                head.replace('\n', "\\n"),
                errh.replace('\n', "\\n")
            );
        }
        return None;
    }
    match crate::runner::json_v0_bridge::parse_json_v0_to_module(&json_line) {
        Ok(module) => Some(module),
        Err(e) => {
            eprintln!("[ny-compiler] JSON parse failed: {}", e);
            None
        }
    }
}
