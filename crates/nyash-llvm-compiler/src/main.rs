use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use clap::{ArgAction, Parser};

#[derive(Parser, Debug)]
#[command(name = "ny-llvmc", about = "Nyash LLVM compiler (llvmlite harness wrapper)")]
struct Args {
    /// MIR JSON input file path (use '-' to read from stdin). When omitted with --dummy, a dummy ny_main is emitted.
    #[arg(long = "in", value_name = "FILE", default_value = "-")]
    infile: String,

    /// Output object file (.o)
    #[arg(long, value_name = "FILE")]
    out: PathBuf,

    /// Generate a dummy object (ny_main -> i32 0). Ignores --in when set.
    #[arg(long, action = ArgAction::SetTrue)]
    dummy: bool,

    /// Path to Python harness script (defaults to tools/llvmlite_harness.py in CWD)
    #[arg(long, value_name = "FILE")]
    harness: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Ensure parent dir exists
    if let Some(parent) = args.out.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    // Resolve harness path
    let harness_path = if let Some(p) = args.harness.clone() {
        p
    } else {
        PathBuf::from("tools/llvmlite_harness.py")
    };

    if args.dummy {
        run_harness_dummy(&harness_path, &args.out)
            .with_context(|| "failed to run harness in dummy mode")?;
        println!("[ny-llvmc] dummy object written: {}", args.out.display());
        return Ok(());
    }

    // Prepare input JSON path: either from file or stdin -> temp file
    let mut temp_path: Option<PathBuf> = None;
    let input_path = if args.infile == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .context("reading MIR JSON from stdin")?;
        // Basic sanity check that it's JSON
        let _: serde_json::Value = serde_json::from_str(&buf)
            .context("stdin does not contain valid JSON")?;
        let tmp = std::env::temp_dir().join("ny_llvmc_stdin.json");
        let mut f = File::create(&tmp).context("create temp json file")?;
        f.write_all(buf.as_bytes()).context("write temp json")?;
        temp_path = Some(tmp.clone());
        tmp
    } else {
        PathBuf::from(&args.infile)
    };

    if !input_path.exists() {
        bail!("input JSON not found: {}", input_path.display());
    }

    run_harness_in(&harness_path, &input_path, &args.out)
        .with_context(|| format!("failed to compile MIR JSON via harness: {}", input_path.display()))?;
    println!("[ny-llvmc] object written: {}", args.out.display());

    // Cleanup temp file if used
    if let Some(p) = temp_path {
        let _ = std::fs::remove_file(p);
    }

    Ok(())
}

fn run_harness_dummy(harness: &Path, out: &Path) -> Result<()> {
    ensure_python()?;
    let status = Command::new("python3")
        .arg(harness)
        .arg("--out")
        .arg(out)
        .status()
        .context("failed to execute python harness (dummy)")?;
    if !status.success() {
        bail!("harness exited with status: {:?}", status.code());
    }
    Ok(())
}

fn run_harness_in(harness: &Path, input: &Path, out: &Path) -> Result<()> {
    ensure_python()?;
    let status = Command::new("python3")
        .arg(harness)
        .arg("--in")
        .arg(input)
        .arg("--out")
        .arg(out)
        .status()
        .context("failed to execute python harness")?;
    if !status.success() {
        bail!("harness exited with status: {:?}", status.code());
    }
    Ok(())
}

fn ensure_python() -> Result<()> {
    match Command::new("python3").arg("--version").output() {
        Ok(out) if out.status.success() => Ok(()),
        _ => bail!("python3 not found in PATH (required for llvmlite harness)"),
    }
}

