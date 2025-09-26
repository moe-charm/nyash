use std::path::Path;

/// Run a Nyash program as a child (`nyash --backend vm <program>`) and capture the first JSON v0 line.
/// - `exe`: path to nyash executable
/// - `program`: path to the Nyash script to run (e.g., apps/selfhost/compiler/compiler.nyash)
/// - `timeout_ms`: kill child after this duration
/// - `extra_args`: additional args to pass after program (e.g., "--", "--read-tmp")
/// - `env_remove`: environment variable names to remove for the child
/// - `envs`: key/value pairs to set for the child
pub fn run_ny_program_capture_json(
    exe: &Path,
    program: &Path,
    timeout_ms: u64,
    extra_args: &[&str],
    env_remove: &[&str],
    envs: &[(&str, &str)],
) -> Option<String> {
    use std::process::Command;
    let mut cmd = Command::new(exe);
    cmd.arg("--backend").arg("vm").arg(program);
    for a in extra_args {
        cmd.arg(a);
    }
    for k in env_remove {
        cmd.env_remove(k);
    }
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let out = match crate::runner::modes::common_util::io::spawn_with_timeout(cmd, timeout_ms) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("[selfhost-child] spawn failed: {}", e);
            return None;
        }
    };
    if out.timed_out {
        let head = String::from_utf8_lossy(&out.stdout).chars().take(200).collect::<String>();
        eprintln!(
            "[selfhost-child] timeout after {} ms; stdout(head)='{}'",
            timeout_ms,
            head.replace('\n', "\\n")
        );
        return None;
    }
    let stdout = match String::from_utf8(out.stdout) {
        Ok(s) => s,
        Err(_) => String::new(),
    };
    crate::runner::modes::common_util::selfhost::json::first_json_v0_line(stdout)
}
