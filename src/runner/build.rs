use super::NyashRunner;
use std::path::{Path, PathBuf};

pub(super) fn run_build_mvp_impl(runner: &NyashRunner, cfg_path: &str) -> Result<(), String> {
    let cwd = std::env::current_dir().unwrap_or(PathBuf::from("."));
    let cfg_abspath = if Path::new(cfg_path).is_absolute() {
        PathBuf::from(cfg_path)
    } else {
        cwd.join(cfg_path)
    };
    // 1) Load nyash.toml
    let text = std::fs::read_to_string(&cfg_abspath)
        .map_err(|e| format!("read {}: {}", cfg_abspath.display(), e))?;
    let doc = toml::from_str::<toml::Value>(&text)
        .map_err(|e| format!("parse {}: {}", cfg_abspath.display(), e))?;
    // 2) Apply [env]
    if let Some(env_tbl) = doc.get("env").and_then(|v| v.as_table()) {
        for (k, v) in env_tbl.iter() {
            if let Some(s) = v.as_str() {
                std::env::set_var(k, s);
            }
        }
    }
    // Derive options
    let profile = runner
        .config
        .build_profile
        .clone()
        .unwrap_or_else(|| "release".into());
    let aot = runner
        .config
        .build_aot
        .clone()
        .unwrap_or_else(|| "cranelift".into());
    let out = runner.config.build_out.clone();
    let target = runner.config.build_target.clone();
    // 3) Build plugins: read [plugins] values as paths and build each
    if let Some(pl_tbl) = doc.get("plugins").and_then(|v| v.as_table()) {
        for (name, v) in pl_tbl.iter() {
            if let Some(path) = v.as_str() {
                let p = if Path::new(path).is_absolute() {
                    PathBuf::from(path)
                } else {
                    cwd.join(path)
                };
                let mut cmd = std::process::Command::new("cargo");
                cmd.arg("build");
                if profile == "release" {
                    cmd.arg("--release");
                }
                if let Some(t) = &target {
                    cmd.args(["--target", t]);
                }
                cmd.current_dir(&p);
                println!("[build] plugin {} at {}", name, p.display());
                let status = cmd
                    .status()
                    .map_err(|e| format!("spawn cargo (plugin {}): {}", name, e))?;
                if !status.success() {
                    return Err(format!(
                        "plugin build failed: {} (dir={})",
                        name,
                        p.display()
                    ));
                }
            }
        }
    }
    // 4) Build nyash core (features)
    {
        let mut cmd = std::process::Command::new("cargo");
        cmd.arg("build");
        if profile == "release" {
            cmd.arg("--release");
        }
        match aot.as_str() {
            "llvm" => {
                cmd.args(["--features", "llvm"]);
            }
            _ => {
                cmd.args(["--features", "cranelift-jit"]);
            }
        }
        if let Some(t) = &target {
            cmd.args(["--target", t]);
        }
        println!(
            "[build] nyash core ({}, features={})",
            profile,
            if aot == "llvm" {
                "llvm"
            } else {
                "cranelift-jit"
            }
        );
        let status = cmd
            .status()
            .map_err(|e| format!("spawn cargo (core): {}", e))?;
        if !status.success() {
            return Err("nyash core build failed".into());
        }
    }
    // 5) Determine app entry
    let app = if let Some(a) = runner.config.build_app.clone() {
        a
    } else {
        // try [build].app, else suggest
        if let Some(tbl) = doc.get("build").and_then(|v| v.as_table()) {
            if let Some(s) = tbl.get("app").and_then(|v| v.as_str()) {
                s.to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    };
    let app = if !app.is_empty() {
        app
    } else {
        // collect candidates under apps/**/main.nyash
        let mut cand: Vec<String> = Vec::new();
        fn walk(dir: &Path, acc: &mut Vec<String>) {
            if let Ok(rd) = std::fs::read_dir(dir) {
                for e in rd.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        walk(&p, acc);
                    } else if p.file_name().map(|n| n == "main.nyash").unwrap_or(false) {
                        acc.push(p.display().to_string());
                    }
                }
            }
        }
        walk(&cwd.join("apps"), &mut cand);
        let msg = if cand.is_empty() {
            "no app specified (--app) and no apps/**/main.nyash found".to_string()
        } else {
            format!(
                "no app specified (--app). Candidates:\n  - {}",
                cand.join("\n  - ")
            )
        };
        return Err(msg);
    };
    // 6) Emit object
    let obj_dir = cwd.join("target").join("aot_objects");
    let _ = std::fs::create_dir_all(&obj_dir);
    let obj_path = obj_dir.join("main.o");
    if aot == "llvm" {
        // llvmliteハーネス使用によりLLVM_SYS_180_PREFIX不要
        std::env::set_var("NYASH_LLVM_OBJ_OUT", &obj_path);
        println!("[emit] LLVM object → {}", obj_path.display());
        let status = std::process::Command::new(
            cwd.join("target")
                .join(profile.clone())
                .join(if cfg!(windows) { "nyash.exe" } else { "nyash" }),
        )
        .args(["--backend", "llvm", &app])
        .status()
        .map_err(|e| format!("spawn nyash llvm: {}", e))?;
        if !status.success() {
            return Err("LLVM emit failed".into());
        }
    } else {
        std::env::set_var("NYASH_AOT_OBJECT_OUT", &obj_dir);
        println!(
            "[emit] Cranelift object → {} (directory)",
            obj_dir.display()
        );
        let status = std::process::Command::new(
            cwd.join("target")
                .join(profile.clone())
                .join(if cfg!(windows) { "nyash.exe" } else { "nyash" }),
        )
        .args(["--backend", "vm", &app])
        .status()
        .map_err(|e| format!("spawn nyash jit-aot: {}", e))?;
        if !status.success() {
            return Err("Cranelift emit failed".into());
        }
    }
    if !obj_path.exists() {
        // In Cranelift path we produce target/aot_objects/<name>.o; fall back to main.o default
        if !obj_dir.join("main.o").exists() {
            return Err(format!("object not generated under {}", obj_dir.display()));
        }
    }
    let out_path = if let Some(o) = out {
        PathBuf::from(o)
    } else {
        if cfg!(windows) {
            cwd.join("app.exe")
        } else {
            cwd.join("app")
        }
    };
    // 7) Link
    println!("[link] → {}", out_path.display());
    #[cfg(windows)]
    {
        // Prefer MSVC link.exe, then clang fallback
        if let Ok(link) = which::which("link") {
            let status = std::process::Command::new(&link)
                .args([
                    "/NOLOGO",
                    &format!("/OUT:{}", out_path.display().to_string()),
                ])
                .arg(&obj_path)
                .arg(cwd.join("target").join("release").join("nyrt.lib"))
                .status()
                .map_err(|e| format!("spawn link.exe: {}", e))?;
            if status.success() {
                println!("OK");
                return Ok(());
            }
        }
        if let Ok(clang) = which::which("clang") {
            let status = std::process::Command::new(&clang)
                .args([
                    "-o",
                    &out_path.display().to_string(),
                    &obj_path.display().to_string(),
                ])
                .arg(
                    cwd.join("target")
                        .join("release")
                        .join("nyrt.lib")
                        .display()
                        .to_string(),
                )
                .arg("-lntdll")
                .status()
                .map_err(|e| format!("spawn clang: {}", e))?;
            if status.success() {
                println!("OK");
                return Ok(());
            }
            return Err("link failed on Windows (tried link.exe and clang)".into());
        }
        return Err("no linker found (need Visual Studio link.exe or LLVM clang)".into());
    }
    #[cfg(not(windows))]
    {
        let status = std::process::Command::new("cc")
            .arg(&obj_path)
            .args([
                "-L",
                &cwd.join("target").join("release").display().to_string(),
            ])
            .args([
                "-Wl,--whole-archive",
                "-lnyrt",
                "-Wl,--no-whole-archive",
                "-lpthread",
                "-ldl",
                "-lm",
            ])
            .args(["-o", &out_path.display().to_string()])
            .status()
            .map_err(|e| format!("spawn cc: {}", e))?;
        if !status.success() {
            return Err("link failed (cc)".into());
        }
    }
    println!("✅ Success: {}", out_path.display());
    Ok(())
}
