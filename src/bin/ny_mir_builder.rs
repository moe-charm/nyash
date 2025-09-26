use clap::{Arg, ArgAction, Command};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command as PCommand, Stdio};

fn main() {
    env_logger::init();
    let matches = Command::new("ny_mir_builder")
        .about("MIR Builder CLI (Phase-15 EXE-first) — consumes Nyash JSON IR and emits {obj|exe|ll|json}")
        .arg(Arg::new("in").long("in").value_name("FILE").help("Input JSON IR file (v0/v1)").required(false))
        .arg(Arg::new("stdin").long("stdin").action(ArgAction::SetTrue).help("Read JSON IR from stdin (default)"))
        .arg(Arg::new("emit").long("emit").value_name("KIND").default_value("obj").value_parser(["obj","exe","ll","json"]))
        .arg(Arg::new("out").short('o').value_name("OUT").required(false))
        .arg(Arg::new("target").long("target").value_name("TRIPLE").required(false))
        .arg(Arg::new("nyrt").long("nyrt").value_name("DIR").required(false))
        .arg(Arg::new("verify_llvm").long("verify-llvm").action(ArgAction::SetTrue))
        .arg(Arg::new("quiet").long("quiet").action(ArgAction::SetTrue))
        .get_matches();

    // Resolve input
    let stdin_mode = matches.get_flag("stdin") || !matches.contains_id("in");
    let in_file: PathBuf = if stdin_mode {
        // Read stdin to tmp/ny_mir_builder_input.json for re-use
        let mut buf = Vec::new();
        io::stdin().read_to_end(&mut buf).expect("read stdin");
        if buf.is_empty() {
            eprintln!("error: no input on stdin");
            std::process::exit(2);
        }
        let cwd_tmp = Path::new("tmp");
        let _ = fs::create_dir_all(cwd_tmp);
        let cwd_path = cwd_tmp.join("ny_mir_builder_input.json");
        fs::write(&cwd_path, &buf).expect("write cwd tmp json");
        cwd_path
    } else {
        let p = PathBuf::from(matches.get_one::<String>("in").unwrap());
        if !p.exists() {
            eprintln!("error: input not found: {}", p.display());
            std::process::exit(2);
        }
        p
    };

    let emit = matches.get_one::<String>("emit").unwrap().as_str();
    let out_path = matches
        .get_one::<String>("out")
        .map(|s| s.to_string())
        .unwrap_or_else(|| match emit {
            "obj" => format!(
                "{}/target/aot_objects/a.o",
                std::env::current_dir().unwrap().display()
            ),
            "ll" => format!(
                "{}/target/aot_objects/a.ll",
                std::env::current_dir().unwrap().display()
            ),
            "exe" => "a.out".to_string(),
            "json" => "/dev/stdout".to_string(),
            _ => unreachable!(),
        });
    let verify = matches.get_flag("verify_llvm");
    let quiet = matches.get_flag("quiet");
    let nyrt_dir = matches
        .get_one::<String>("nyrt")
        .map(|s| s.to_string())
        .unwrap_or("crates/nyash_kernel".to_string());

    // Determine sibling nyash binary path (target dir)
    let nyash_bin = current_dir_bin("nyash");
    if emit == "json" {
        if out_path == "/dev/stdout" {
            let mut f = File::open(&in_file).expect("open in");
            io::copy(&mut f, &mut io::stdout()).ok();
        } else {
            fs::copy(&in_file, &out_path).expect("copy json");
        }
        if !quiet {
            println!("OK json:{}", out_path);
        }
        return;
    }

    // Ensure build dirs
    let aot_dir = Path::new("target/aot_objects");
    let _ = fs::create_dir_all(aot_dir);

    match emit {
        "ll" => {
            std::env::set_var("NYASH_LLVM_DUMP_LL", "1");
            std::env::set_var("NYASH_LLVM_LL_OUT", &out_path);
            if verify {
                std::env::set_var("NYASH_LLVM_VERIFY", "1");
            }
            std::env::set_var("NYASH_LLVM_USE_HARNESS", "1");
            run_nyash_pipe(&nyash_bin, &in_file);
            if !Path::new(&out_path).exists() {
                eprintln!("error: failed to produce {}", out_path);
                std::process::exit(4);
            }
            if !quiet {
                println!("OK ll:{}", out_path);
            }
        }
        "obj" => {
            std::env::set_var("NYASH_LLVM_OBJ_OUT", &out_path);
            if verify {
                std::env::set_var("NYASH_LLVM_VERIFY", "1");
            }
            std::env::set_var("NYASH_LLVM_USE_HARNESS", "1");
            // remove stale
            let _ = fs::remove_file(&out_path);
            run_nyash_pipe(&nyash_bin, &in_file);
            if !Path::new(&out_path).exists() {
                eprintln!("error: failed to produce {}", out_path);
                std::process::exit(4);
            }
            if !quiet {
                println!("OK obj:{}", out_path);
            }
        }
        "exe" => {
            let obj_path = format!(
                "{}/target/aot_objects/__tmp_mir_builder.o",
                std::env::current_dir().unwrap().display()
            );
            std::env::set_var("NYASH_LLVM_OBJ_OUT", &obj_path);
            if verify {
                std::env::set_var("NYASH_LLVM_VERIFY", "1");
            }
            std::env::set_var("NYASH_LLVM_USE_HARNESS", "1");
            let _ = fs::remove_file(&obj_path);
            run_nyash_pipe(&nyash_bin, &in_file);
            if !Path::new(&obj_path).exists() {
                eprintln!("error: failed to produce object {}", obj_path);
                std::process::exit(4);
            }
            // Link with NyRT
            if let Err(e) = link_exe(&obj_path, &out_path, &nyrt_dir) {
                eprintln!("error: link failed: {}", e);
                std::process::exit(5);
            }
            if !quiet {
                println!("OK exe:{}", out_path);
            }
        }
        _ => unreachable!(),
    }
}

fn current_dir_bin(name: &str) -> PathBuf {
    // Resolve sibling binary in target/<profile>
    // Try current_exe parent dir first
    if let Ok(cur) = std::env::current_exe() {
        if let Some(dir) = cur.parent() {
            let cand = dir.join(name);
            if cand.exists() {
                return cand;
            }
            #[cfg(windows)]
            {
                let cand = dir.join(format!("{}.exe", name));
                if cand.exists() {
                    return cand;
                }
            }
        }
    }
    // Fallback to target/release
    let cand = PathBuf::from("target/release").join(name);
    if cand.exists() {
        return cand;
    }
    #[cfg(windows)]
    {
        let cand = PathBuf::from("target/release").join(format!("{}.exe", name));
        return cand;
    }
    cand
}

fn run_nyash_pipe(nyash_bin: &Path, json_file: &Path) {
    // Pipe the JSON into nyash with --ny-parser-pipe and llvm backend
    let file = File::open(json_file).expect("open json");
    let mut cmd = PCommand::new(nyash_bin);
    cmd.arg("--backend").arg("llvm").arg("--ny-parser-pipe");
    cmd.stdin(Stdio::from(file));
    // Quiet child output
    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() != Some("1") {
        cmd.stdout(Stdio::null());
    }
    let status = cmd.status().expect("run nyash");
    if !status.success() {
        eprintln!("error: nyash harness failed (status {:?})", status.code());
        std::process::exit(4);
    }
}

fn link_exe(obj_path: &str, out_path: &str, nyrt_dir: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // Prefer lld-link, then link.exe, fallback to cc
        let nyrt_release = format!("{}/target/release", nyrt_dir.replace('\\', "/"));
        let lib_nyrt_lib = format!("{}/nyrt.lib", nyrt_release);
        let lib_nyrt_a = format!("{}/libnyash_kernel.a", nyrt_release);
        if which::which("lld-link").is_ok() {
            let mut args: Vec<String> = Vec::new();
            args.push(format!("/OUT:{}", out_path));
            args.push(obj_path.to_string());
            // Provide LIBPATH and library name (prefer nyrt.lib)
            args.push(format!("/LIBPATH:{}", nyrt_release));
            if std::path::Path::new(&lib_nyrt_lib).exists() {
                args.push("nyrt.lib".to_string());
            }
            // lld-link cannot consume .a directly; rely on .lib
            let status = PCommand::new("lld-link")
                .args(args.iter().map(|s| s.as_str()))
                .status()
                .map_err(|e| e.to_string())?;
            if status.success() {
                return Ok(());
            }
            return Err(format!("lld-link failed: status {:?}", status.code()));
        }
        if which::which("link").is_ok() {
            let mut args: Vec<String> = Vec::new();
            args.push(format!("/OUT:{}", out_path));
            args.push(obj_path.to_string());
            args.push(format!("/LIBPATH:{}", nyrt_release));
            if std::path::Path::new(&lib_nyrt_lib).exists() {
                args.push("nyrt.lib".to_string());
            }
            let status = PCommand::new("link")
                .args(args.iter().map(|s| s.as_str()))
                .status()
                .map_err(|e| e.to_string())?;
            if status.success() {
                return Ok(());
            }
            return Err(format!("link.exe failed: status {:?}", status.code()));
        }
        // Fallback: try cc with MinGW-like flags
        let status = PCommand::new("cc")
            .args([obj_path])
            .args(["-L", &format!("{}/target/release", nyrt_dir)])
            .args(["-lnyash_kernel", "-o", out_path])
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            return Ok(());
        }
        return Err(format!("cc link failed: status {:?}", status.code()));
    }
    #[cfg(not(target_os = "windows"))]
    {
        let status = PCommand::new("cc")
            .args([obj_path])
            .args(["-L", "target/release"])
            .args(["-L", &format!("{}/target/release", nyrt_dir)])
            .args([
                "-Wl,--whole-archive",
                "-lnyash_kernel",
                "-Wl,--no-whole-archive",
            ])
            .args(["-lpthread", "-ldl", "-lm", "-o", out_path])
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("cc failed: status {:?}", status.code()))
        }
    }
}
