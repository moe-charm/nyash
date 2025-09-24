use nyash_rust::parser::NyashParser;

fn with_env<K: AsRef<str>, V: AsRef<str>, F: FnOnce()>(key: K, val: Option<V>, f: F) {
    let k = key.as_ref().to_string();
    let prev = std::env::var(&k).ok();
    match val {
        Some(v) => std::env::set_var(&k, v.as_ref()),
        None => std::env::remove_var(&k),
    }
    f();
    match prev {
        Some(p) => std::env::set_var(&k, p),
        None => std::env::remove_var(&k),
    }
}

#[test]
fn stage3_disabled_rejects_try_and_throw() {
    with_env("NYASH_PARSER_STAGE3", None::<&str>, || {
        let code_try = "try { local x = 1 } catch () { }";
        let res_try = NyashParser::parse_from_string(code_try);
        assert!(res_try.is_err(), "try should be rejected when gate is off");

        let code_throw = "throw 1";
        let res_throw = NyashParser::parse_from_string(code_throw);
        assert!(
            res_throw.is_err(),
            "throw should be rejected when gate is off"
        );
    });
}

#[test]
fn stage3_enabled_accepts_throw() {
    with_env("NYASH_PARSER_STAGE3", Some("1"), || {
        let code = "throw (1 + 2)";
        let res = NyashParser::parse_from_string(code);
        assert!(
            res.is_ok(),
            "throw should parse when gate is on: {:?}",
            res.err()
        );
    });
}

#[test]
fn stage3_enabled_accepts_try_catch_variants() {
    with_env("NYASH_PARSER_STAGE3", Some("1"), || {
        // (Type var)
        let code1 = r#"
            try { local a = 1 }
            catch (Error e) { local b = 2 }
            finally { local z = 3 }
        "#;
        assert!(NyashParser::parse_from_string(code1).is_ok());

        // (var) only
        let code2 = r#"
            try { local a = 1 }
            catch (e) { local b = 2 }
        "#;
        assert!(NyashParser::parse_from_string(code2).is_ok());

        // () empty
        let code3 = r#"
            try { local a = 1 }
            catch () { local b = 2 }
        "#;
        assert!(NyashParser::parse_from_string(code3).is_ok());
    });
}
