use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use clap::{ArgAction, Parser};

#[derive(Parser, Debug)]
#[command(
    name = "ny-llvmc",
    about = "Nyash LLVM compiler (llvmlite harness wrapper)"
)]
struct Args {
    /// MIR JSON input file path (use '-' to read from stdin). When omitted with --dummy, a dummy ny_main is emitted.
    #[arg(long = "in", value_name = "FILE", default_value = "-")]
    infile: String,

    /// Output path. For `--emit obj`, this is an object (.o). For `--emit exe`, this is an executable path.
    #[arg(long, value_name = "FILE")]
    out: PathBuf,

    /// Generate a dummy object (ny_main -> i32 0). Ignores --in when set.
    #[arg(long, action = ArgAction::SetTrue)]
    dummy: bool,

    /// Path to Python harness script (defaults to tools/llvmlite_harness.py in CWD)
    #[arg(long, value_name = "FILE")]
    harness: Option<PathBuf>,

    /// Emit kind: 'obj' (default) or 'exe'.
    #[arg(long, value_name = "{obj|exe}", default_value = "obj")]
    emit: String,

    /// Path to directory containing libnyash_kernel.a when emitting an executable. If omitted, searches target/release then crates/nyash_kernel/target/release.
    #[arg(long, value_name = "DIR")]
    nyrt: Option<PathBuf>,

    /// Extra linker libs/flags appended when emitting an executable (single string, space-separated).
    #[arg(long, value_name = "FLAGS")]
    libs: Option<String>,
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

    // Determine emit kind
    let emit_exe = matches!(args.emit.as_str(), "exe" | "EXE");

    if args.dummy {
        // Dummy ny_main: always go through harness to produce an object then link if requested
        let obj_path = if emit_exe {
            // derive a temporary .o path next to output
            let mut p = args.out.clone();
            p.set_extension("o");
            p
        } else {
            args.out.clone()
        };
        run_harness_dummy(&harness_path, &obj_path)
            .with_context(|| "failed to run harness in dummy mode")?;
        if emit_exe {
            link_executable(
                &obj_path,
                &args.out,
                args.nyrt.as_ref(),
                args.libs.as_deref(),
            )?;
            println!("[ny-llvmc] executable written: {}", args.out.display());
        } else {
            println!("[ny-llvmc] dummy object written: {}", obj_path.display());
        }
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
        let _: serde_json::Value =
            serde_json::from_str(&buf).context("stdin does not contain valid JSON")?;
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

    // Produce object first
    let obj_path = if emit_exe {
        let mut p = args.out.clone();
        p.set_extension("o");
        p
    } else {
        args.out.clone()
    };

    run_harness_in(&harness_path, &input_path, &obj_path).with_context(|| {
        format!(
            "failed to compile MIR JSON via harness: {}",
            input_path.display()
        )
    })?;
    if emit_exe {
        link_executable(
            &obj_path,
            &args.out,
            args.nyrt.as_ref(),
            args.libs.as_deref(),
        )?;
        println!("[ny-llvmc] executable written: {}", args.out.display());
    } else {
        println!("[ny-llvmc] object written: {}", obj_path.display());
    }

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

fn link_executable(
    obj: &Path,
    out_exe: &Path,
    nyrt_dir_opt: Option<&PathBuf>,
    extra_libs: Option<&str>,
) -> Result<()> {
    // Resolve NyKernel static lib directory (support legacy/new names)
    let nyrt_dir = if let Some(dir) = nyrt_dir_opt {
        dir.clone()
    } else {
        // try target/release then crates/nyash_kernel/target/release, crates/hako_kernel/target/release
        let a = PathBuf::from("target/release");
        let b = PathBuf::from("crates/nyash_kernel/target/release");
        let c = PathBuf::from("crates/hako_kernel/target/release");
        if a.join("libnyash_kernel.a").exists() || a.join("libhako_kernel.a").exists() {
            a
        } else if b.join("libnyash_kernel.a").exists() || b.join("libhako_kernel.a").exists() {
            b
        } else if c.join("libnyash_kernel.a").exists() || c.join("libhako_kernel.a").exists() {
            c
        } else {
            a
        }
    };
    let lib_nyash = nyrt_dir.join("libnyash_kernel.a");
    let lib_hako = nyrt_dir.join("libhako_kernel.a");
    let libnyrt = if lib_nyash.exists() {
        &lib_nyash
    } else {
        &lib_hako
    };
    if !libnyrt.exists() {
        bail!(
            "Kernel archive not found in {} (looked for libnyash_kernel.a, libhako_kernel.a). Use --nyrt to specify directory",
            nyrt_dir.display()
        );
    }

    // Choose a C linker
    let linker = ["cc", "clang", "gcc"]
        .into_iter()
        .find(|c| {
            Command::new(c)
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        })
        .unwrap_or("cc");

    let mut cmd = Command::new(linker);
    cmd.arg("-o").arg(out_exe);
    cmd.arg(obj);
    // Whole-archive NyKernel to ensure all objects are linked
    cmd.arg("-Wl,--whole-archive")
        .arg(libnyrt)
        .arg("-Wl,--no-whole-archive");
    // Common libs on Linux
    cmd.arg("-ldl").arg("-lpthread").arg("-lm");
    if let Some(extras) = extra_libs {
        for tok in extras.split_whitespace() {
            cmd.arg(tok);
        }
    }
    let status = cmd.status().context("failed to invoke system linker")?;
    if !status.success() {
        bail!("linker exited with status: {:?}", status.code());
    }
    Ok(())
}
