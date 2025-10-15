/*!
 * using_core – helpers to flatten [modules] and extract simple using tables
 */

pub fn flatten_modules_from_value(doc: &toml::Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if let Some(tbl) = doc.get("modules").and_then(|v| v.as_table()) {
        out.extend(flatten_modules_table(tbl));
    }
    out
}

pub fn flatten_modules_table(tbl: &toml::value::Table) -> Vec<(String, String)> {
    fn visit(prefix: &str, t: &toml::value::Table, out: &mut Vec<(String, String)>) {
        for (k, v) in t.iter() {
            let name = if prefix.is_empty() { k.to_string() } else { format!("{}.{}", prefix, k) };
            if let Some(s) = v.as_str() {
                out.push((name, s.to_string()));
            } else if let Some(sub) = v.as_table() {
                visit(&name, sub, out);
            }
        }
    }
    let mut out = Vec::new();
    visit("", tbl, &mut out);
    out
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flatten_empty_and_nested() {
        let doc: toml::Value = toml::from_str(r#"[modules]"#).unwrap();
        let v = flatten_modules_from_value(&doc);
        assert!(v.is_empty());
        let doc2: toml::Value = toml::from_str(r#"[modules.a.b]
 c = "x/path""#).unwrap();
        let v2 = flatten_modules_from_value(&doc2);
        assert_eq!(v2, vec![("a.b.c".into(), "x/path".into())]);
    }
    #[test]
    fn flatten_multiple_branches() {
        let doc: toml::Value = toml::from_str(r#"
            [modules.a.b]
            c = "x/path"
            d = "y/path"
            [modules.e]
            f = "z/path"
        "#).unwrap();
        let mut v = flatten_modules_from_value(&doc);
        v.sort();
        assert_eq!(v, vec![
            ("a.b.c".into(), "x/path".into()),
            ("a.b.d".into(), "y/path".into()),
            ("e.f".into(), "z/path".into()),
        ]);
    }
}
