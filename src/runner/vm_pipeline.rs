
//! vm_pipeline: Helper module for the main VM execution pipeline.
//! Breaks down the monolithic execute_vm_engine into smaller, testable stages.

use crate::ast::ASTNode;
use crate::runner::NyashRunner;
use std::collections::{HashMap, HashSet};
use std::io::Write;
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
) -> Result<(String, Vec<ASTNode>, HashSet<String>)> {
    if !crate::config::env::enable_using() {
        return Ok((code.to_string(), Vec::new(), HashSet::new()));
    }

    let use_ast = crate::config::env::using_ast_enabled();
    let mut prelude_asts: Vec<ASTNode> = Vec::new();
    let mut alias_names: HashSet<String> = HashSet::new();
    let mut alias_map: HashMap<String, String> = HashMap::new();

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
        return Err(PipelineError::UsingResolution(
            "AST prelude merge is disabled in this profile. Enable NYASH_USING_AST=1 or remove 'using' lines.".to_string(),
        ));
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

    Ok((clean_code, prelude_asts, alias_names))
}

/// Stage 3: Parse the main source code and merge it with prelude ASTs.
pub fn parse_and_merge_ast(code: &str, prelude_asts: Vec<ASTNode>) -> Result<ASTNode> {
    use crate::parser::NyashParser;

    let main_ast =
        NyashParser::parse_from_string(code).map_err(|e| PipelineError::Parse(e.to_string()))?;

    if crate::config::env::using_ast_enabled() && !prelude_asts.is_empty() {
        Ok(
            crate::runner::modes::common_util::resolve::merge_prelude_asts_with_main(
                prelude_asts,
                &main_ast,
            ),
        )
    } else {
        Ok(main_ast)
    }
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
            } = st {
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
            ) -> std::result::Result<Box<dyn crate::box_trait::NyashBox>, crate::box_factory::RuntimeError> {
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
            fn box_types(&self) -> Vec<&str> { vec![] }
            fn is_available(&self) -> bool { true }
            fn factory_type(&self) -> crate::box_factory::FactoryType { crate::box_factory::FactoryType::User }
        }
        let factory = InlineUserBoxFactory { decls: Arc::new(RwLock::new(decls)) };
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
        },
        Err(e) => Err(PipelineError::VmExecution(e.to_string())),
    }
}
