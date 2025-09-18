use crate::NyashRunner;

/// Strip `using` lines and register modules/aliases into the runtime registry.
/// Returns cleaned source. No-op when `NYASH_ENABLE_USING` is not set.
#[allow(dead_code)]
pub fn strip_using_and_register(
    runner: &NyashRunner,
    code: &str,
    filename: &str,
) -> Result<String, String> {
    if !crate::config::env::enable_using() {
        return Ok(code.to_string());
    }
    let mut out = String::with_capacity(code.len());
    let mut used_names: Vec<(String, Option<String>)> = Vec::new();
    for line in code.lines() {
        let t = line.trim_start();
        if t.starts_with("using ") {
            crate::cli_v!("[using] stripped line: {}", line);
            let rest0 = t.strip_prefix("using ").unwrap().trim();
            let rest0 = rest0.strip_suffix(';').unwrap_or(rest0).trim();
            let (target, alias) = if let Some(pos) = rest0.find(" as ") {
                (rest0[..pos].trim().to_string(), Some(rest0[pos + 4..].trim().to_string()))
            } else {
                (rest0.to_string(), None)
            };
            let is_path = target.starts_with('"')
                || target.starts_with("./")
                || target.starts_with('/')
                || target.ends_with(".nyash");
            if is_path {
                let path = target.trim_matches('"').to_string();
                let name = alias.clone().unwrap_or_else(|| {
                    std::path::Path::new(&path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("module")
                        .to_string()
                });
                used_names.push((name, Some(path)));
            } else {
                used_names.push((target, alias));
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }

    // Register modules with resolver (aliases/modules/paths)
    let using_ctx = runner.init_using_context();
    let strict = std::env::var("NYASH_USING_STRICT").ok().as_deref() == Some("1");
    let verbose = crate::config::env::cli_verbose();
    let ctx_dir = std::path::Path::new(filename).parent();
    for (ns_or_alias, alias_or_path) in used_names {
        if let Some(path) = alias_or_path {
            let sb = crate::box_trait::StringBox::new(path);
            crate::runtime::modules_registry::set(ns_or_alias, Box::new(sb));
        } else {
            match crate::runner::pipeline::resolve_using_target(
                &ns_or_alias,
                false,
                &using_ctx.pending_modules,
                &using_ctx.using_paths,
                &using_ctx.aliases,
                ctx_dir,
                strict,
                verbose,
            ) {
                Ok(value) => {
                    let sb = crate::box_trait::StringBox::new(value);
                    crate::runtime::modules_registry::set(ns_or_alias, Box::new(sb));
                }
                Err(e) => {
                    return Err(format!("using: {}", e));
                }
            }
        }
    }
    Ok(out)
}
