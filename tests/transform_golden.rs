use std::{env, fs, path::Path};

fn read_json(path: &Path) -> serde_json::Value {
    let s = fs::read_to_string(path).expect("read json");
    serde_json::from_str(&s).expect("parse json")
}

fn write_pretty(path: &Path, v: &serde_json::Value) {
    let s = serde_json::to_string_pretty(v).expect("pretty json");
    fs::write(path, s).expect("write golden");
}

fn apply_env(env_path: &Path) -> Vec<(String, Option<String>)> {
    if !env_path.exists() {
        return Vec::new();
    }
    let v: serde_json::Value = read_json(env_path);
    let obj = v.as_object().expect("env.json must be object");
    let mut prev = Vec::new();
    for (k, vv) in obj.iter() {
        let val = vv.as_str().expect("env values must be strings").to_string();
        prev.push((k.clone(), env::var(k).ok()));
        env::set_var(k, val);
    }
    prev
}

fn restore_env(prev: Vec<(String, Option<String>)>) {
    for (k, v) in prev.into_iter() {
        match v {
            Some(val) => env::set_var(k, val),
            None => env::remove_var(k),
        }
    }
}

#[cfg(feature = "all-examples")]
#[test]
fn golden_transforms() {
    // To avoid env races across tests when using env toggles
    // run with: RUST_TEST_THREADS=1 cargo test --test transform_golden
    let root = Path::new("tests/golden/transforms");
    assert!(root.exists(), "missing tests/golden/transforms directory");
    for entry in fs::read_dir(root).expect("scan golden dirs") {
        let entry = entry.expect("dir entry");
        if !entry.file_type().expect("ft").is_dir() {
            continue;
        }
        let case_dir = entry.path();
        let in_path = case_dir.join("in.json");
        let out_path = case_dir.join("out.golden.json");
        let env_path = case_dir.join("env.json");
        assert!(in_path.exists(), "{}: missing in.json", case_dir.display());

        let prev_env = apply_env(&env_path);
        let input_v = read_json(&in_path);

        // Convert JSON v0 -> AST -> normalize -> JSON v0
        let ast = nyash_rust::r#macro::ast_json::json_to_ast(&input_v).expect("json_to_ast");
        let ast2 = nyash_rust::runner::modes::macro_child::normalize_core_pass(&ast);
        let out_json = nyash_rust::r#macro::ast_json::ast_to_json(&ast2);
        restore_env(prev_env);

        if std::env::var("BLESS").ok().as_deref() == Some("1") {
            write_pretty(&out_path, &out_json);
            continue;
        }
        let expected = read_json(&out_path);
        if expected != out_json {
            let got_s = serde_json::to_string_pretty(&out_json).unwrap();
            let exp_s = serde_json::to_string_pretty(&expected).unwrap();
            panic!(
                "Golden mismatch in {}\n--- expected\n{}\n--- got\n{}\n",
                case_dir.display(),
                exp_s,
                got_s
            );
        }
    }
}
