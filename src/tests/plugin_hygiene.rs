#[test]
fn plugin_invoke_hygiene_prefers_hostcall_for_mapped() {
    use crate::jit::policy::invoke::{decide_box_method, InvokeDecision};
    use crate::jit::r#extern::collections as c;

    // Ensure plugin builtins are not forced
    std::env::remove_var("NYASH_USE_PLUGIN_BUILTINS");

    // For ArrayBox.get, policy should map to hostcall symbol, not plugin invoke
    let decision = decide_box_method("ArrayBox", "get", 2, true);
    match decision {
        InvokeDecision::HostCall { symbol, reason, .. } => {
            assert_eq!(symbol, c::SYM_ARRAY_GET_H);
            assert_eq!(reason, "mapped_symbol");
        }
        other => panic!("expected HostCall(mapped_symbol), got: {:?}", other),
    }
}

#[test]
fn plugin_invoke_hygiene_string_len_is_hostcall() {
    use crate::jit::policy::invoke::{decide_box_method, InvokeDecision};
    use crate::jit::r#extern::collections as c;

    std::env::remove_var("NYASH_USE_PLUGIN_BUILTINS");
    let decision = decide_box_method("StringBox", "len", 1, true);
    match decision {
        InvokeDecision::HostCall { symbol, reason, .. } => {
            assert_eq!(symbol, c::SYM_STRING_LEN_H);
            assert_eq!(reason, "mapped_symbol");
        }
        other => panic!(
            "expected HostCall(mapped_symbol) for String.len, got: {:?}",
            other
        ),
    }
}
