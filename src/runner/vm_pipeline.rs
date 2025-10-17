//! vm_pipeline: Helper module for the main VM execution pipeline.
//! Breaks down the monolithic execute_vm_engine into smaller, testable stages.

use crate::ast::ASTNode;
use crate::runner::NyashRunner;
use std::collections::{HashMap, HashSet};
use std::{fs, process};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("MIR compilation error: {0}")]
    MirCompile(String),
    #[error("`using` resolution error: {0}")]
    UsingResolution(String),
    #[error("VM execution error: {0}")]
    VmExecution(String),
    #[error("AST processing error: {0}")]
    AstProcessing(String),
}

type Result<T> = std::result::Result<T, PipelineError>;

/// Stage 1: Load source file and apply pre-lexical normalization.
pub fn load_and_preprocess_source(filename: &str) -> Result<String> {
    let code = fs::read_to_string(filename)?;
    let code = crate::runner::modes::common_util::resolve::preexpand_at_local(&code);
    let code = crate::runner::modes::common_util::prelex::prelex_normalize(&code);
    Ok(code)
}

/// Stage 2: Resolve `using` statements, parse preludes, and prepare aliases.
pub fn resolve_preludes_and_aliases(
    runner: &NyashRunner,
    code: &str,
    filename: &str,
) -> Result<(
    String,
    Vec<ASTNode>,
    HashSet<String>,
    std::collections::HashMap<String, Vec<String>>,
)> {
    if !crate::config::env::enable_using() {
        return Ok((
            code.to_string(),
            Vec::new(),
            HashSet::new(),
            std::collections::HashMap::new(),
        ));
    }

    let use_ast = crate::config::env::using_ast_enabled();
    // Quiet child-pipeline mode (e.g., JSON-only emit for selfhost compiler)
    // In this mode, we allow skipping AST prelude merge without treating it as an error.
    let quiet_pipe = std::env::var("NYASH_JSON_ONLY").ok().as_deref() == Some("1");
    // Also allow skipping when script args include "--min-json" (selfhost header path)
    let min_json_req = std::env::var("NYASH_SCRIPT_ARGS_JSON")
        .ok()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .and_then(|v| v.as_array().cloned())
        .map(|arr| arr.iter().any(|x| x.as_str() == Some("--min-json")))
        .unwrap_or(false);
    let mut prelude_asts: Vec<ASTNode> = Vec::new();
    let mut alias_names: HashSet<String> = HashSet::new();
    let mut alias_map: HashMap<String, String> = HashMap::new();
    let mut alias_top_map: HashMap<String, Vec<String>> = HashMap::new();

    let (clean_code, paths, alias_pairs) =
        crate::runner::modes::common_util::resolve::resolve_prelude_paths_profiled(
            runner, code, filename,
        )
        .map_err(PipelineError::UsingResolution)?;

    for (alias, canon) in alias_pairs.iter() {
        alias_names.insert(alias.clone());
        alias_map.insert(canon.clone(), alias.clone());
    }

    if !paths.is_empty() && !use_ast {
        if quiet_pipe || min_json_req {
            // Quiet pipeline: proceed without AST merge.
            // Aliases were already collected above; module registration is handled by the runner preprocessor.
            if crate::config::env::resolve_trace() {
                crate::runner::trace::log(
                    "[using] AST merge disabled but quiet pipeline active — skipping prelude merge"
                        .to_string(),
                );
            }
        } else {
            return Err(PipelineError::UsingResolution(
                "AST prelude merge is disabled in this profile. Enable NYASH_USING_AST=1 or remove 'using' lines.".to_string(),
            ));
        }
    }

    if use_ast && !paths.is_empty() {
        let parsed_preludes =
            crate::runner::modes::common_util::resolve::parse_preludes_to_asts(runner, &paths)
                .map_err(PipelineError::UsingResolution)?;

        let mut used_prefixed: HashSet<String> = HashSet::new();
        for (path, ast) in parsed_preludes.into_iter() {
            let canon = std::fs::canonicalize(&path)
                .ok()
                .map(|pb| pb.to_string_lossy().to_string())
                .unwrap_or(path.clone());

            if let Some(alias) = alias_map.get(&canon) {
                let renamed = crate::runner::modes::common_util::resolve::alias_tools::rename_with_collision_guard(
                    &ast, alias, &mut used_prefixed, &canon,
                )
                .map_err(PipelineError::UsingResolution)?;
                prelude_asts.push(renamed);
            } else {
                prelude_asts.push(ast);
            }
        }
    }

    if !alias_names.is_empty() && !prelude_asts.is_empty() {
        for ast in prelude_asts.iter() {
            let tops =
                crate::runner::modes::common_util::resolve::alias_tools::collect_prelude_top_names(
                    ast,
                );
            for alias in alias_names.iter() {
                let prefix = format!("{}_", alias);
                let entry = alias_top_map.entry(alias.clone()).or_default();
                for name in tops.iter() {
                    if let Some(rest) = name.strip_prefix(&prefix) {
                        if !entry.iter().any(|s| s == rest) {
                            entry.push(rest.to_string());
                        }
                    }
                }
            }
        }
    }

    Ok((clean_code, prelude_asts, alias_names, alias_top_map))
}

/// Stage 3: Parse the main source code and merge it with prelude ASTs.
pub fn parse_and_merge_ast(
    code: &str,
    prelude_asts: Vec<ASTNode>,
    alias_top_map: &std::collections::HashMap<String, Vec<String>>,
) -> Result<ASTNode> {
    // Opt-in dev flag: prefer front::parser_layer facade for parsing
    let use_facade = std::env::var("HAKO_FRONT_USE_FACADE")
        .ok()
        .map(|v| v == "1" || v == "true" || v == "on")
        .unwrap_or(false);
    let main_ast = if use_facade {
        crate::front::parser_layer::facade::parse_source_to_ast(code)
            .map_err(|e| PipelineError::Parse(e.message))?
    } else {
        use crate::parser::NyashParser;
        NyashParser::parse_from_string(code).map_err(|e| PipelineError::Parse(e.to_string()))?
    };

    let merged = if crate::config::env::using_ast_enabled() && !prelude_asts.is_empty() {
        crate::runner::modes::common_util::resolve::merge_prelude_asts_with_main(
            prelude_asts,
            &main_ast,
        )
    } else {
        main_ast
    };

    let merged = if !alias_top_map.is_empty() {
        crate::runner::modes::common_util::resolve::alias_tools::rewrite_main_alias_refs(
            &merged,
            alias_top_map,
        )
    } else {
        merged
    };

    Ok(merged)
}

/// Stage 4: Apply alias desugaring and macro expansion to the AST.
pub fn process_ast_macros_and_aliases(
    ast: ASTNode,
    alias_names: &HashSet<String>,
) -> Result<ASTNode> {
    let ast = crate::runner::modes::common_util::resolve::alias_tools::desugar_alias_field_access(
        &ast,
        alias_names,
        true,
    );
    let ast = crate::r#macro::maybe_expand_and_dump(&ast, false);
    Ok(ast)
}

/// Stage 5: Scan the AST for BoxDeclarations and register an inline factory for them.
pub fn register_user_boxes_from_ast(ast: &ASTNode) {
    use std::sync::{Arc, RwLock};

    let mut nonstatic_decls: HashMap<String, crate::core::model::BoxDeclaration> = HashMap::new();
    let mut static_names: Vec<String> = Vec::new();

    if let ASTNode::Program { statements, .. } = ast {
        for st in statements {
            if let ASTNode::BoxDeclaration {
                name,
                fields,
                public_fields,
                private_fields,
                methods,
                constructors,
                init_fields,
                weak_fields,
                is_interface,
                extends,
                implements,
                type_parameters,
                is_static,
                ..
            } = st
            {
                if *is_static {
                    static_names.push(name.clone());
                    continue;
                }
                let decl = crate::core::model::BoxDeclaration {
                    name: name.clone(),
                    fields: fields.clone(),
                    public_fields: public_fields.clone(),
                    private_fields: private_fields.clone(),
                    methods: methods.clone(),
                    constructors: constructors.clone(),
                    init_fields: init_fields.clone(),
                    weak_fields: weak_fields.clone(),
                    is_interface: *is_interface,
                    extends: extends.clone(),
                    implements: implements.clone(),
                    type_parameters: type_parameters.clone(),
                };
                nonstatic_decls.insert(name.clone(), decl);
            }
        }
    }

    let mut decls = nonstatic_decls.clone();
    for s in static_names.into_iter() {
        let inst = format!("{}Instance", s);
        if let Some(d) = nonstatic_decls.get(&inst) {
            decls.insert(s, d.clone());
        }
    }

    if !decls.is_empty() {
        struct InlineUserBoxFactory {
            decls: Arc<RwLock<HashMap<String, crate::core::model::BoxDeclaration>>>,
        }
        impl crate::box_factory::BoxFactory for InlineUserBoxFactory {
            fn create_box(
                &self,
                name: &str,
                args: &[Box<dyn crate::box_trait::NyashBox>],
            ) -> std::result::Result<
                Box<dyn crate::box_trait::NyashBox>,
                crate::box_factory::RuntimeError,
            > {
                let opt = { self.decls.read().unwrap().get(name).cloned() };
                let decl = match opt {
                    Some(d) => d,
                    None => {
                        return Err(crate::box_factory::RuntimeError::InvalidOperation {
                            message: format!("Unknown Box type: {}", name),
                        })
                    }
                };
                let mut inst = crate::instance_v2::InstanceBox::from_declaration(
                    decl.name.clone(),
                    decl.fields.clone(),
                    decl.methods.clone(),
                );
                let _ = inst.init(args);
                Ok(Box::new(inst))
            }
            fn box_types(&self) -> Vec<&str> {
                vec![]
            }
            fn is_available(&self) -> bool {
                true
            }
            fn factory_type(&self) -> crate::box_factory::FactoryType {
                crate::box_factory::FactoryType::User
            }
        }
        let factory = InlineUserBoxFactory {
            decls: Arc::new(RwLock::new(decls)),
        };
        crate::runtime::unified_registry::register_user_defined_factory(Arc::new(factory));
    }
}

/// Stage 6: Compile the final AST to MIR and execute it with the VM.
pub fn compile_and_execute_mir(ast: ASTNode, no_optimize: bool) -> Result<()> {
    let mut mir_compiler = crate::mir::MirCompiler::with_options(!no_optimize);
    let compile = mir_compiler
        .compile(ast)
        .map_err(|e| PipelineError::MirCompile(e.to_string()))?;

    let mut engine = crate::runner::modes::super_iface::vm_engine_from_env();
    match engine.execute(&compile.module) {
        Ok(code) => {
            // Result printing is handled in the VM engine leaf for robustness.
            process::exit(code)
        }
        Err(e) => Err(PipelineError::VmExecution(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::ASTNode;
    use crate::cli::CliConfig;
    use crate::runner::NyashRunner;
    use tempfile::tempdir;

    struct EnvGuard {
        key: &'static str,
        original: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let original = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, original }
        }
    }

    struct DirGuard {
        original: std::path::PathBuf,
    }

    impl DirGuard {
        fn change_to(path: &std::path::Path) -> Self {
            let original = std::env::current_dir().expect("current dir");
            std::env::set_current_dir(path).expect("set current dir");
            Self { original }
        }
    }

    impl Drop for DirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original);
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(ref value) = self.original {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    fn find_main_assignment_value(ast: &ASTNode) -> &ASTNode {
        if let ASTNode::Program { statements, .. } = ast {
            for st in statements {
                if let ASTNode::BoxDeclaration { name, methods, .. } = st {
                    if name == "Main" {
                        if let Some(ASTNode::FunctionDeclaration { body, .. }) = methods.get("main")
                        {
                            for stmt in body {
                                if let ASTNode::Assignment { value, .. } = stmt {
                                    return value.as_ref();
                                }
                            }
                        }
                    }
                }
            }
        }
        panic!("Main.main assignment not found in AST: {:?}", ast);
    }

    fn assert_alias_call(node: &ASTNode) {
        match node {
            ASTNode::FunctionCall {
                name, arguments, ..
            } => {
                assert_eq!(
                    name, "FU_Utils.add/2",
                    "function call not rewritten: {name}"
                );
                assert_eq!(
                    arguments.len(),
                    2,
                    "unexpected arity for rewritten function call"
                );
            }
            ASTNode::MethodCall {
                object,
                method,
                arguments,
                ..
            } => {
                if let ASTNode::Variable { name, .. } = object.as_ref() {
                    assert_eq!(name, "FU_Utils", "alias receiver not prefixed: {name}");
                } else {
                    panic!("unexpected method receiver shape: {:?}", object);
                }
                assert_eq!(method, "add", "method name changed unexpectedly");
                assert_eq!(arguments.len(), 2, "unexpected arity for method call");
            }
            other => panic!(
                "unexpected assignment form after alias rewrite: {:?}",
                other
            ),
        }
    }

    #[test]
    fn alias_without_explicit_as_prefixed_and_rewritten() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        let _root_guard = EnvGuard::set("NYASH_ROOT", root.to_str().unwrap());
        let _using_guard = EnvGuard::set("NYASH_USING", "1");
        let _using_ast_guard = EnvGuard::set("NYASH_USING_AST", "1");
        let _using_file_guard = EnvGuard::set("NYASH_ALLOW_USING_FILE", "1");
        let _modules_guard = EnvGuard::set("NYASH_MODULES", "");

        let config = r#"
[using.flow_utils]
path = "lib/flow_utils/"
main = "utils.nyash"

[using.aliases]
FU = "flow_utils"

[using]
paths = ["lib"]
"#;
        std::fs::write(root.join("nyash.toml"), config).unwrap();

        let lib_dir = root.join("lib/flow_utils");
        std::fs::create_dir_all(&lib_dir).unwrap();
        std::fs::write(
            lib_dir.join("utils.nyash"),
            r#"flow Utils {
  add(a, b) { return a + b }
}
"#,
        )
        .unwrap();

        let main_path = root.join("main.nyash");
        std::fs::write(
            &main_path,
            r#"using FU
flow Main {
  main() {
    local v
    v = Utils.add(10, 20)
    print(v)
    return 0
  }
}
"#,
        )
        .unwrap();

        let runner = NyashRunner::new(CliConfig::default());
        let _dir_guard = DirGuard::change_to(root);
        let ctx_check = runner.init_using_context();
        assert!(
            ctx_check.aliases.contains_key("FU"),
            "using context aliases missing FU mapping: {:?}",
            ctx_check.aliases.keys().collect::<Vec<_>>()
        );
        let code = std::fs::read_to_string(&main_path).unwrap();
        let (clean, preludes, aliases, alias_top_map) =
            resolve_preludes_and_aliases(&runner, &code, main_path.to_str().unwrap()).unwrap();

        assert!(
            aliases.contains("FU"),
            "alias set should include FU but was {:?}",
            aliases
        );
        assert!(
            !preludes.is_empty(),
            "expected prelude ASTs to be collected for alias"
        );
        let tops = alias_top_map
            .get("FU")
            .expect("expected top-level names for alias FU");
        assert!(
            tops.contains(&"Utils".to_string()),
            "alias tops missing Utils entry: {:?}",
            tops
        );
        let prelude_names: Vec<String> = preludes
            .iter()
            .flat_map(|ast| match ast {
                ASTNode::Program { statements, .. } => statements
                    .iter()
                    .filter_map(|st| match st {
                        ASTNode::BoxDeclaration { name, .. } => Some(name.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>(),
                _ => Vec::new(),
            })
            .collect();
        assert!(
            prelude_names.iter().any(|n| n == "FU_Utils"),
            "preludes missing renamed alias box: {:?}",
            prelude_names
        );

        use crate::parser::NyashParser;
        let raw_main_ast = NyashParser::parse_from_string(&clean).unwrap();
        let rewritten_main =
            crate::runner::modes::common_util::resolve::alias_tools::rewrite_main_alias_refs(
                &raw_main_ast,
                &alias_top_map,
            );
        let rewritten_main_value = find_main_assignment_value(&rewritten_main);
        assert_alias_call(rewritten_main_value);

        let merged = parse_and_merge_ast(&clean, preludes, &alias_top_map).unwrap();
        let merged_value = find_main_assignment_value(&merged);
        assert_alias_call(merged_value);

        let processed = process_ast_macros_and_aliases(merged, &aliases).unwrap();
        let processed_value = find_main_assignment_value(&processed);
        assert_alias_call(processed_value);
    }
}
