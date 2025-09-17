use std::io::Read;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

pub struct ChildOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub status_ok: bool,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
}

/// Spawn command with timeout (ms), capture stdout/stderr, and return ChildOutput.
pub fn spawn_with_timeout(mut cmd: Command, timeout_ms: u64) -> std::io::Result<ChildOutput> {
    let mut cmd = cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn()?;
    let mut ch_stdout = child.stdout.take();
    let mut ch_stderr = child.stderr.take();
    let start = Instant::now();
    let mut timed_out = false;
    let mut exit_status: Option<std::process::ExitStatus> = None;
    loop {
        match child.try_wait()? {
            Some(status) => { exit_status = Some(status); break },
            None => {
                if start.elapsed() >= Duration::from_millis(timeout_ms) {
                    let _ = child.kill();
                    let _ = child.wait();
                    timed_out = true;
                    break;
                }
                sleep(Duration::from_millis(10));
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
    let (status_ok, exit_code) = if let Some(st) = exit_status {
        (st.success(), st.code())
    } else { (false, None) };
    Ok(ChildOutput {
        stdout: out_buf,
        stderr: err_buf,
        status_ok,
        exit_code,
        timed_out,
    })
}
