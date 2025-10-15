use std::{env, fs, path::PathBuf};

fn main() {
    // Phase 31-A: Static plugin linking
    if std::env::var_os("CARGO_FEATURE_STATIC_STRING_PLUGIN").is_some() {
        println!("cargo:rustc-link-search=native=target/release");
        println!("cargo:rustc-link-lib=static=nyash_string_plugin");
    }

    // Embed basic build metadata for --version in runtime (if present)
    // Safe best-effort: fall back to "unknown" when git/date are unavailable.
    // Embed minimal build metadata (optional). Disabled if helper missing.
    // Commented to avoid build-script failures in restricted environments.
    // embed_build_metadata();
    // Stage-4 minimal C harness (parser-c-abi) — compile when feature is enabled
    if std::env::var_os("CARGO_FEATURE_PARSER_C_ABI").is_some() {
        #[allow(unused)]
        {
            // Guard cc dependency presence; if missing, show a helpful note
            #[allow(unused_imports)]
            use cc as _;
            println!("cargo:rerun-if-changed=src/parser_harness/parser_harness.c");
            println!("cargo:rerun-if-changed=src/parser_harness/parser_harness.h");
            cc::Build::new()
                .file("src/parser_harness/parser_harness.c")
                .include("src/parser_harness")
                .compile("parser_harness");
        }
    }
    // Opt-in export of host C ABI symbols and dynamic symbol table
    // Set HAKO_EXPORT_HOST=1 to enable (used by Stage-2 plugin array return)
    if std::env::var("HAKO_EXPORT_HOST").ok().as_deref() == Some("1")
        || std::env::var("NYASH_EXPORT_HOST").ok().as_deref() == Some("1")
    {
        // Export host C ABI (cfg) and make symbols visible to dlopen()ed plugins
        println!("cargo:rustc-cfg=export_host_c_abi");
        if cfg!(target_os = "linux") || cfg!(target_os = "android") || cfg!(target_os = "freebsd") {
            println!("cargo:rustc-link-arg=-rdynamic");
        } else if cfg!(target_os = "macos") {
            // On macOS, export_all is default for executables; keep as-is.
            // If needed in the future: println!("cargo:rustc-link-arg=-Wl,-export_dynamic");
        } else if cfg!(target_os = "windows") {
            // Windows uses import libraries; no action here (plugins call back via host API entry points).
        }
    }
    // Path to grammar spec
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let grammar_dir = manifest_dir.join("grammar");
    let grammar_file = grammar_dir.join("unified-grammar.toml");

    // Ensure output dir exists
    let out_dir = manifest_dir.join("src").join("grammar");
    fs::create_dir_all(&out_dir).ok();
    let out_file = out_dir.join("generated.rs");

    // If grammar file is missing, create a minimal one
    if !grammar_file.exists() {
        fs::create_dir_all(&grammar_dir).ok();
        let minimal = r#"
[keywords.me]
token = "ME"

[keywords.from]
token = "FROM"

[keywords.loop]
token = "LOOP"

[operators.add]
symbol = "+"
coercion_strategy = "string_priority"
type_rules = [
  { left = "String", right = "String", result = "String", action = "concat" },
  { left = "String", right = "Integer", result = "String", action = "concat" },
  { left = "Integer", right = "String", result = "String", action = "concat" },
  { left = "String", right = "Bool", result = "String", action = "concat" },
  { left = "Bool", right = "String", result = "String", action = "concat" },
  { left = "String", right = "Other", result = "String", action = "concat" },
  { left = "Other", right = "String", result = "String", action = "concat" },
  { left = "Integer", right = "Integer", result = "Integer", action = "add_i64" },
  { left = "Float", right = "Float", result = "Float", action = "add_f64" }
]
"#;
        fs::write(&grammar_file, minimal).expect("write minimal unified-grammar.toml");
        println!(
            "cargo:warning=Created minimal grammar at {}",
            grammar_file.display()
        );
    }

    // Read and very light parse: collect
    // - keywords.<name>.token
    // - operators.{add,sub,mul,div}.{coercion_strategy,type_rules}
    // - syntax.statements.allow = [..]
    // - syntax.expressions.allow_binops = [..]
    let content = fs::read_to_string(&grammar_file).expect("read unified-grammar.toml");

    // Naive line scan to avoid build-deps; supports lines like: [keywords.xxx] then token = "YYY"
    let mut current_key: Option<String> = None;
    let mut in_operators_add = false;
    let mut in_operators_sub = false;
    let mut in_operators_mul = false;
    let mut in_operators_div = false;
    let mut add_coercion: Option<String> = None;
    let mut sub_coercion: Option<String> = None;
    let mut mul_coercion: Option<String> = None;
    let mut div_coercion: Option<String> = None;
    let mut entries: Vec<(String, String)> = Vec::new();
    let mut in_type_rules = false;
    let mut add_rules: Vec<(String, String, String, String)> = Vec::new();
    let mut sub_rules: Vec<(String, String, String, String)> = Vec::new();
    let mut mul_rules: Vec<(String, String, String, String)> = Vec::new();
    let mut div_rules: Vec<(String, String, String, String)> = Vec::new();
    for line in content.lines() {
        let s = line.trim();
        if s.starts_with("[keywords.") && s.ends_with("]") {
            let name = s
                .trim_start_matches("[keywords.")
                .trim_end_matches("]")
                .to_string();
            current_key = Some(name);
            in_operators_add = false;
            in_operators_sub = false;
            in_operators_mul = false;
            in_operators_div = false;
            continue;
        }
        if s == "[operators.add]" {
            current_key = None;
            in_operators_add = true;
            in_operators_sub = false;
            in_operators_mul = false;
            in_operators_div = false;
            in_type_rules = false;
            continue;
        }
        if s == "[operators.sub]" {
            current_key = None;
            in_operators_add = false;
            in_operators_sub = true;
            in_operators_mul = false;
            in_operators_div = false;
            in_type_rules = false;
            continue;
        }
        if s == "[operators.mul]" {
            current_key = None;
            in_operators_add = false;
            in_operators_sub = false;
            in_operators_mul = true;
            in_operators_div = false;
            in_type_rules = false;
            continue;
        }
        if s == "[operators.div]" {
            current_key = None;
            in_operators_add = false;
            in_operators_sub = false;
            in_operators_mul = false;
            in_operators_div = true;
            in_type_rules = false;
            continue;
        }
        if let Some(ref key) = current_key {
            if let Some(rest) = s.strip_prefix("token") {
                if let Some(eq) = rest.find('=') {
                    let val = rest[eq + 1..].trim().trim_matches('"').to_string();
                    entries.push((key.clone(), val));
                }
            }
        }
        if in_operators_add || in_operators_sub || in_operators_mul || in_operators_div {
            if s.starts_with("type_rules") && s.contains('[') {
                in_type_rules = true;
                continue;
            }
            if in_type_rules {
                if s.starts_with(']') {
                    in_type_rules = false;
                    continue;
                }
                // Expect lines like: { left = "String", right = "String", result = "String", action = "concat" },
                if s.starts_with('{') && s.ends_with("},") || s.ends_with('}') {
                    let inner = s
                        .trim_start_matches('{')
                        .trim_end_matches('}')
                        .trim_end_matches(',');
                    let mut left = String::new();
                    let mut right = String::new();
                    let mut result = String::new();
                    let mut action = String::new();
                    for part in inner.split(',') {
                        let kv = part.trim();
                        if let Some(eq) = kv.find('=') {
                            let key = kv[..eq].trim();
                            let val = kv[eq + 1..].trim().trim_matches('"').to_string();
                            match key {
                                "left" => left = val,
                                "right" => right = val,
                                "result" => result = val,
                                "action" => action = val,
                                _ => {}
                            }
                        }
                    }
                    if !left.is_empty()
                        && !right.is_empty()
                        && !result.is_empty()
                        && !action.is_empty()
                    {
                        if in_operators_add {
                            add_rules.push((left, right, result, action));
                        } else if in_operators_sub {
                            sub_rules.push((left, right, result, action));
                        } else if in_operators_mul {
                            mul_rules.push((left, right, result, action));
                        } else if in_operators_div {
                            div_rules.push((left, right, result, action));
                        }
                    }
                }
            }
            if let Some(rest) = s.strip_prefix("coercion_strategy") {
                if let Some(eq) = rest.find('=') {
                    let val = rest[eq + 1..].trim().trim_matches('"').to_string();
                    if in_operators_add {
                        add_coercion = Some(val.clone());
                    } else if in_operators_sub {
                        sub_coercion = Some(val.clone());
                    } else if in_operators_mul {
                        mul_coercion = Some(val.clone());
                    } else if in_operators_div {
                        div_coercion = Some(val.clone());
                    }
                }
            }
        }
    }

    // Default rules if none present in TOML (keep codegen deterministic)
    if add_rules.is_empty() {
        add_rules.push((
            "String".into(),
            "String".into(),
            "String".into(),
            "concat".into(),
        ));
        add_rules.push((
            "String".into(),
            "Integer".into(),
            "String".into(),
            "concat".into(),
        ));
        add_rules.push((
            "Integer".into(),
            "String".into(),
            "String".into(),
            "concat".into(),
        ));
        add_rules.push((
            "String".into(),
            "Bool".into(),
            "String".into(),
            "concat".into(),
        ));
        add_rules.push((
            "Bool".into(),
            "String".into(),
            "String".into(),
            "concat".into(),
        ));
        add_rules.push((
            "String".into(),
            "Other".into(),
            "String".into(),
            "concat".into(),
        ));
        add_rules.push((
            "Other".into(),
            "String".into(),
            "String".into(),
            "concat".into(),
        ));
        add_rules.push((
            "Integer".into(),
            "Integer".into(),
            "Integer".into(),
            "add_i64".into(),
        ));
        add_rules.push((
            "Float".into(),
            "Float".into(),
            "Float".into(),
            "add_f64".into(),
        ));
    }
    if sub_rules.is_empty() {
        sub_rules.push((
            "Integer".into(),
            "Integer".into(),
            "Integer".into(),
            "sub_i64".into(),
        ));
        sub_rules.push((
            "Float".into(),
            "Float".into(),
            "Float".into(),
            "sub_f64".into(),
        ));
    }
    if mul_rules.is_empty() {
        mul_rules.push((
            "Integer".into(),
            "Integer".into(),
            "Integer".into(),
            "mul_i64".into(),
        ));
        mul_rules.push((
            "Float".into(),
            "Float".into(),
            "Float".into(),
            "mul_f64".into(),
        ));
    }
    if div_rules.is_empty() {
        div_rules.push((
            "Integer".into(),
            "Integer".into(),
            "Integer".into(),
            "div_i64".into(),
        ));
        div_rules.push((
            "Float".into(),
            "Float".into(),
            "Float".into(),
            "div_f64".into(),
        ));
    }

    // Generate Rust code
    let mut code = String::new();
    code.push_str("// Auto-generated from grammar/unified-grammar.toml\n");
    code.push_str("pub static KEYWORDS: &[(&str, &str)] = &[\n");
    for (k, t) in &entries {
        code.push_str(&format!("    (\"{}\", \"{}\"),\n", k, t));
    }
    code.push_str("];");
    let add_coercion_val = add_coercion.unwrap_or_else(|| "string_priority".to_string());
    let sub_coercion_val = sub_coercion.unwrap_or_else(|| "numeric_only".to_string());
    let mul_coercion_val = mul_coercion.unwrap_or_else(|| "numeric_only".to_string());
    let div_coercion_val = div_coercion.unwrap_or_else(|| "numeric_only".to_string());
    code.push_str(&format!(
        "\npub static OPERATORS_ADD_COERCION: &str = \"{}\";\n",
        add_coercion_val
    ));
    code.push_str(&format!(
        "pub static OPERATORS_SUB_COERCION: &str = \"{}\";\n",
        sub_coercion_val
    ));
    code.push_str(&format!(
        "pub static OPERATORS_MUL_COERCION: &str = \"{}\";\n",
        mul_coercion_val
    ));
    code.push_str(&format!(
        "pub static OPERATORS_DIV_COERCION: &str = \"{}\";\n",
        div_coercion_val
    ));
    // Emit add rules
    code.push_str("pub static OPERATORS_ADD_RULES: &[(&str, &str, &str, &str)] = &[\n");
    for (l, r, res, act) in &add_rules {
        code.push_str(&format!(
            "    (\"{}\", \"{}\", \"{}\", \"{}\"),\n",
            l, r, res, act
        ));
    }
    code.push_str("];");
    // Emit sub rules
    code.push_str("\npub static OPERATORS_SUB_RULES: &[(&str, &str, &str, &str)] = &[\n");
    for (l, r, res, act) in &sub_rules {
        code.push_str(&format!(
            "    (\"{}\", \"{}\", \"{}\", \"{}\"),\n",
            l, r, res, act
        ));
    }
    code.push_str("];");
    // Emit mul rules
    code.push_str("\npub static OPERATORS_MUL_RULES: &[(&str, &str, &str, &str)] = &[\n");
    for (l, r, res, act) in &mul_rules {
        code.push_str(&format!(
            "    (\"{}\", \"{}\", \"{}\", \"{}\"),\n",
            l, r, res, act
        ));
    }
    code.push_str("];");
    // Emit div rules
    code.push_str("\npub static OPERATORS_DIV_RULES: &[(&str, &str, &str, &str)] = &[\n");
    for (l, r, res, act) in &div_rules {
        code.push_str(&format!(
            "    (\"{}\", \"{}\", \"{}\", \"{}\"),\n",
            l, r, res, act
        ));
    }
    code.push_str("];");
    code.push_str(
        r#"
pub fn lookup_keyword(word: &str) -> Option<&'static str> {
    for (k, t) in KEYWORDS {
        if *k == word { return Some(*t); }
    }
    None
}
"#,
    );

    // --- Naive parse for syntax rules (statements/expressions) ---
    let mut syntax_statements: Vec<String> = Vec::new();
    let mut syntax_binops: Vec<String> = Vec::new();
    let mut in_syntax_statements = false;
    let mut in_syntax_expressions = false;
    for line in content.lines() {
        let s = line.trim();
        if s == "[syntax.statements]" {
            in_syntax_statements = true;
            in_syntax_expressions = false;
            continue;
        }
        if s == "[syntax.expressions]" {
            in_syntax_statements = false;
            in_syntax_expressions = true;
            continue;
        }
        if s.starts_with('[') {
            in_syntax_statements = false;
            in_syntax_expressions = false;
        }
        if in_syntax_statements {
            if let Some(rest) = s.strip_prefix("allow") {
                if let Some(eq) = rest.find('=') {
                    let arr = rest[eq + 1..].trim();
                    // Expect [ "if", "loop", ... ] possibly spanning multiple lines; simple split for this snapshot
                    for part in arr.trim_matches(&['[', ']'][..]).split(',') {
                        let v = part.trim().trim_matches('"');
                        if !v.is_empty() {
                            syntax_statements.push(v.to_string());
                        }
                    }
                }
            }
        }
        if in_syntax_expressions {
            if let Some(rest) = s.strip_prefix("allow_binops") {
                if let Some(eq) = rest.find('=') {
                    let arr = rest[eq + 1..].trim();
                    for part in arr.trim_matches(&['[', ']'][..]).split(',') {
                        let v = part.trim().trim_matches('"');
                        if !v.is_empty() {
                            syntax_binops.push(v.to_string());
                        }
                    }
                }
            }
        }
    }
    if syntax_statements.is_empty() {
        syntax_statements = vec![
            "box".into(),
            "global".into(),
            "function".into(),
            "static".into(),
            "if".into(),
            "loop".into(),
            "break".into(),
            "return".into(),
            "print".into(),
            "nowait".into(),
            "include".into(),
            "local".into(),
            "outbox".into(),
            "try".into(),
            "throw".into(),
            "using".into(),
            "from".into(),
        ];
    }
    if syntax_binops.is_empty() {
        syntax_binops = vec!["add".into(), "sub".into(), "mul".into(), "div".into()];
    }
    // Emit syntax arrays
    code.push_str("\npub static SYNTAX_ALLOWED_STATEMENTS: &[&str] = &[\n");
    for k in &syntax_statements {
        code.push_str(&format!("    \"{}\",\n", k));
    }
    code.push_str("];");
    code.push_str("\npub static SYNTAX_ALLOWED_BINOPS: &[&str] = &[\n");
    for k in &syntax_binops {
        code.push_str(&format!("    \"{}\",\n", k));
    }
    code.push_str("];");

    fs::write(&out_file, code).expect("write generated.rs");
    println!("cargo:rerun-if-changed={}", grammar_file.display());

    // Generate externs registry from specs (SSOT)
    gen_externs_registry();
    // --- SSOT registry (TypeRegistry) codegen stub ---
    // Always write a small generated file that either includes specs/type_registry.toml
    // or defines None when absent. Runtime can opt to read this later.
    {
        let gen_path =
            PathBuf::from(env::var("OUT_DIR").unwrap()).join("registry_ssot_generated.rs");
        let ssot_spec = manifest_dir.join("specs").join("type_registry.toml");
        let gen_src = if ssot_spec.exists() {
            r#"// Auto-generated: SSOT registry raw TOML presence
pub const REGISTRY_SSOT_RAW: Option<&'static str> = Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/specs/type_registry.toml")));
"#.to_string()
        } else {
            "// Auto-generated: SSOT registry not present
pub const REGISTRY_SSOT_RAW: Option<&'static str> = None;
"
            .to_string()
        };
        std::fs::write(&gen_path, gen_src).expect("write registry_ssot_generated.rs");
        println!("cargo:rerun-if-changed={}", ssot_spec.display());
    }
}

// --- Externs SSOT generation -------------------------------------------------
fn gen_externs_registry() {
    use std::io::Write;
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let specs_dir = manifest_dir.join("specs").join("externs");
    let specs_file = specs_dir.join("registry.toml");
    println!("cargo:rerun-if-changed={}", specs_file.display());

    if !specs_file.exists() {
        fs::create_dir_all(&specs_dir).ok();
        let minimal = r#"[[extern]]
name = "env.console.log"
argc = 1
io = true
"#;
        fs::write(&specs_file, minimal).expect("write minimal externs registry");
        println!(
            "cargo:warning=Created minimal externs registry at {}",
            specs_file.display()
        );
    }

    let content = fs::read_to_string(&specs_file).expect("read externs registry");
    #[derive(Default)]
    struct Item {
        name: String,
        argc: usize,
        io: bool,
    }
    let mut items: Vec<Item> = Vec::new();
    let mut cur: Option<Item> = None;
    for line in content.lines() {
        let s = line.trim();
        if s.is_empty() || s.starts_with('#') {
            continue;
        }
        if s == "[[extern]]" {
            if let Some(it) = cur.take() {
                items.push(it);
            }
            cur = Some(Item::default());
            continue;
        }
        if let Some(eq) = s.find('=') {
            let key = s[..eq].trim();
            let val = s[eq + 1..].trim().trim_matches('"');
            if let Some(ref mut it) = cur {
                match key {
                    "name" => it.name = val.to_string(),
                    "argc" => it.argc = val.parse().unwrap_or(0),
                    "io" => {
                        let v = val.trim_matches('"').to_ascii_lowercase();
                        it.io = matches!(v.as_str(), "1" | "true" | "on");
                    }
                    _ => {}
                }
            }
        }
    }
    if let Some(it) = cur.take() {
        items.push(it);
    }

    // Generate Rust source
    let out_dir = manifest_dir.join("src").join("mir").join("externs");
    fs::create_dir_all(&out_dir).ok();
    let out_file = out_dir.join("generated.rs");
    let mut buf = String::new();
    buf.push_str("// @generated by build.rs from specs/externs/registry.toml\n");
    buf.push_str(
        "pub fn generated_externs() -> &'static [(&'static str, usize, bool)] {\n    &[\n",
    );
    for it in &items {
        if it.name.is_empty() {
            continue;
        }
        buf.push_str(&format!(
            "        (\"{}\", {}, {}),\n",
            it.name,
            it.argc,
            if it.io { "true" } else { "false" }
        ));
    }
    buf.push_str("    ]\n}\n");

    let mut f = fs::File::create(&out_file).expect("create generated externs file");
    f.write_all(buf.as_bytes())
        .expect("write generated externs");

    // ---- Static plugins codegen (optional, config-driven) ----
    // Read plugin list from static_plugins.toml and generate a small Rust module
    // that registers their hako_box.toml specs at runtime.
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let static_cfg = manifest_dir.join("static_plugins.toml");
    let out_dir_rs = PathBuf::from(env::var("OUT_DIR").unwrap());
    let gen_path = out_dir_rs.join("static_plugins_generated.rs");
    let mut generated = String::new();
    generated.push_str(
        "// Auto-generated by build.rs — static plugins registry
",
    );
    generated.push_str(
        "// Edit static_plugins.toml to change the set.

",
    );
    generated.push_str(
        "pub fn register_static_plugins() {
",
    );
    if static_cfg.exists() {
        println!("cargo:rerun-if-changed={}", static_cfg.display());
        let cfg_str =
            std::fs::read_to_string(&static_cfg).expect("Failed to read static_plugins.toml");
        #[derive(serde::Deserialize)]
        struct PluginCfg {
            path: String,
            #[allow(dead_code)]
            feature: Option<String>,
        }
        #[derive(serde::Deserialize)]
        struct Root {
            plugins: Vec<PluginCfg>,
        }
        let root: Root = toml::from_str(&cfg_str).expect("static_plugins.toml parse error");
        for p in root.plugins {
            let inc = format!(
                r#"    let toml_str: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/{}/hako_box.toml"));
    register_from_toml(toml_str);
"#,
                p.path
            );
            generated.push_str(&inc);
        }
    } else {
        // No config — keep function but do nothing
        println!("cargo:rerun-if-changed=static_plugins.toml");
    }
    generated.push_str(
        "}
",
    );
    std::fs::write(&gen_path, generated).expect("write static_plugins_generated.rs");
}
