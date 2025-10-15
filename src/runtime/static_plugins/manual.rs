//! Manual invoke pointer registration for static plugins (Phase 31-A experimental)
//!
//! This registers per-Box invoke function pointers that were compiled statically.
//! Phase A: StringBox only (minimal validation)
//! Phase B: Auto-generate from hako_box.toml

#[cfg(feature = "static-string-plugin")]
extern "C" {
    fn nyash_string_plugin_invoke_static(
        instance_id: u32,
        method_id: u32,
        args: *const u8,
        args_len: usize,
        result: *mut u8,
        result_len: *mut usize,
    ) -> i32;
}

/// Register invoke pointers for static-linked plugins.
/// Phase A: StringBox only (experimental)
pub fn register_manual_invoke_pointers() {
    #[cfg(feature = "static-string-plugin")]
    {
        if let Ok(h) = crate::runtime::get_global_plugin_host().read() {
            h.register_static_with_spec(
                "nyash-string-plugin",
                "StringBox",
                Some(13), // type_id from hako_box.toml
                None,     // capabilities
                Some(u32::MAX), // fini_method_id
                &[
                    ("birth", 0, false),
                    ("size", 1, false),
                    ("isEmpty", 2, false),
                    ("charCodeAt", 3, false),
                    ("concat", 4, false),
                    ("fromUtf8", 5, false),
                    ("toUtf8", 6, false),
                    ("substring", 7, false),
                    ("indexOf", 8, false),
                    ("lastIndexOf", 9, false),
                    ("charAt", 10, false),
                    ("fini", u32::MAX, false),
                ],
                Some(nyash_string_plugin_invoke_static),
            );
        }
    }
}
