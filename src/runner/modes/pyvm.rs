use super::super::NyashRunner;
use nyash_rust::{mir::MirCompiler, parser::NyashParser};
use std::{fs, process};

/// Execute using PyVM only (no Rust VM runtime). Emits MIR(JSON) and invokes tools/pyvm_runner.py.
pub fn execute_pyvm_only(runner: &NyashRunner, filename: &str) {
    if std::env::var("NYASH_PYVM_TRACE").ok().as_deref() == Some("1") { eprintln!("[pyvm] entry"); }
    // Read the file
    let code = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file {}: {}", filename, e);
            process::exit(1);
        }
    };

    // Optional using pre-processing (strip lines and register modules)
    let mut code = if crate::config::env::enable_using() {
        match crate::runner::modes::common_util::resolve::strip_using_and_register(runner, &code, filename) {
            Ok(s) => s,
            Err(e) => { eprintln!("❌ {}", e); process::exit(1); }
        }
    } else { code };

    // Dev sugar pre-expand: line-head @name[:T] = expr → local name[:T] = expr
    code = crate::runner::modes::common_util::resolve::preexpand_at_local(&code);

    // Normalize logical operators for Stage-2 parser: translate '||'/'&&' to 'or'/'and' outside strings/comments
    fn normalize_logical_ops(src: &str) -> String {
        let mut out = String::with_capacity(src.len());
        let mut it = src.chars().peekable();
        let mut in_str = false;
        let mut in_line = false;
        let mut in_block = false;
        while let Some(c) = it.next() {
            if in_line {
                out.push(c);
                if c == '\n' { in_line = false; }
                continue;
            }
            if in_block {
                out.push(c);
                if c == '*' && matches!(it.peek(), Some('/')) { out.push('/'); it.next(); in_block = false; }
                continue;
            }
            if in_str {
                out.push(c);
                if c == '\\' { if let Some(nc) = it.next() { out.push(nc); } continue; }
                if c == '"' { in_str = false; }
                continue;
            }
            match c {
                '"' => { in_str = true; out.push(c); }
                '/' => {
                    match it.peek() { Some('/') => { out.push('/'); out.push('/'); it.next(); in_line = true; }, Some('*') => { out.push('/'); out.push('*'); it.next(); in_block = true; }, _ => out.push('/') }
                }
                '#' => { in_line = true; out.push('#'); }
                '|' => {
                    if matches!(it.peek(), Some('|')) { out.push_str(" or "); it.next(); } else if matches!(it.peek(), Some('>')) { out.push('|'); out.push('>'); it.next(); } else { out.push('|'); }
                }
                '&' => {
                    if matches!(it.peek(), Some('&')) { out.push_str(" and "); it.next(); } else { out.push('&'); }
                }
                _ => out.push(c),
            }
        }
        out
    }
    code = normalize_logical_ops(&code);

    // Parse to AST
    if std::env::var("NYASH_PYVM_DUMP_CODE").ok().as_deref() == Some("1") {
        eprintln!("[pyvm-code]\n{}", code);
    }
    let ast = match NyashParser::parse_from_string(&code) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("❌ Parse error: {}", e);
            process::exit(1);
        }
    };
    let ast = crate::r#macro::maybe_expand_and_dump(&ast, false);
    let ast = crate::runner::modes::macro_child::normalize_core_pass(&ast);

    // Compile to MIR (respect default optimizer setting)
    let mut mir_compiler = MirCompiler::with_options(true);
    let mut compile_result = match mir_compiler.compile(ast) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("❌ MIR compilation error: {}", e);
            process::exit(1);
        }
    };

    // Optional: VM-only escape analysis elision pass retained for parity with VM path
    if std::env::var("NYASH_VM_ESCAPE_ANALYSIS").ok().as_deref() == Some("1") {
        let removed = nyash_rust::mir::passes::escape::escape_elide_barriers_vm(&mut compile_result.module);
        if removed > 0 { crate::cli_v!("[PyVM] escape_elide_barriers: removed {} barriers", removed); }
    }

    // Delegate to common PyVM harness
    match crate::runner::modes::common_util::pyvm::run_pyvm_harness_lib(&compile_result.module, "pyvm") {
        Ok(code) => { process::exit(code); }
        Err(e) => { eprintln!("❌ PyVM error: {}", e); process::exit(1); }
    }
}
