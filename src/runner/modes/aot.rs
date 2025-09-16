use super::super::NyashRunner;
#[cfg(feature = "cranelift-jit")]
use std::{process, process::Command};

impl NyashRunner {
    /// Execute AOT compilation mode (split)
    #[cfg(feature = "cranelift-jit")]
    pub(crate) fn execute_aot_mode(&self, filename: &str) {
        let output = self.config.output_file.as_deref().unwrap_or("app");
        // Prefer using provided helper scripts to ensure link flags and runtime integration
        let status = if cfg!(target_os = "windows") {
            // Use PowerShell helper; falls back to bash if available inside the script
            Command::new("powershell")
                .args([
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                    "tools/build_aot.ps1",
                    "-Input",
                    filename,
                    "-Out",
                    &format!("{}.exe", output),
                ])
                .status()
        } else {
            Command::new("bash")
                .args(["tools/build_aot.sh", filename, "-o", output])
                .status()
        };
        match status {
            Ok(s) if s.success() => {
                println!(
                    "✅ AOT compilation successful!\nExecutable written to: {}",
                    output
                );
            }
            Ok(s) => {
                eprintln!(
                    "❌ AOT compilation failed (exit={} ). See logs above.",
                    s.code().unwrap_or(-1)
                );
                process::exit(1);
            }
            Err(e) => {
                eprintln!("❌ Failed to invoke build_aot.sh: {}", e);
                eprintln!(
                    "Hint: ensure bash is available, or run: bash tools/build_aot.sh {} -o {}",
                    filename, output
                );
                process::exit(1);
            }
        }
    }
}
