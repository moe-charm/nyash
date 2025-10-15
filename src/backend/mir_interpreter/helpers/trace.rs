use super::super::*;

impl MirInterpreter {
    /// Emit a one-line JSON call trace when NYASH_CALL_TRACE=1.
    /// Label examples: "Global:print", "Method:ConsoleBox.println/1", "BoxCall:push".
    pub(in crate::backend::mir_interpreter) fn emit_call_trace_label(&self, label: &str, argc: usize, recv: Option<u32>) {
        if std::env::var("NYASH_CALL_TRACE").ok().as_deref() != Some("1") {
            return;
        }
        let bb = self.last_block.map(|b| b.as_u32()).unwrap_or(0);
        let mut out = String::from("{\"kind\":\"call\",\"callee\":\"");
        let esc = |s: &str| s.replace('"', "\\\"");
        out.push_str(&esc(label));
        out.push('"');
        if let Some(r) = recv {
            out.push_str(",\"recv\":");
            out.push_str(&r.to_string());
        }
        out.push_str(",\"argc\":");
        out.push_str(&argc.to_string());
        out.push_str(",\"bb\":");
        out.push_str(&bb.to_string());
        out.push('}');
        eprintln!("{}", out);
    }
}

// ---- Box trace (dev-only observer) ----
impl MirInterpreter {
    #[inline]
    pub(in crate::backend::mir_interpreter) fn box_trace_enabled() -> bool {
        std::env::var("NYASH_BOX_TRACE").ok().as_deref() == Some("1")
    }

    fn box_trace_filter_match(class_name: &str) -> bool {
        if let Ok(filt) = std::env::var("NYASH_BOX_TRACE_FILTER") {
            let want = filt.trim();
            if want.is_empty() { return true; }
            // comma/space separated tokens; match if any token is contained in class
            for tok in want.split(|c: char| c == ',' || c.is_whitespace()) {
                let t = tok.trim();
                if !t.is_empty() && class_name.contains(t) { return true; }
            }
            false
        } else {
            true
        }
    }

    fn json_escape(s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 8);
        for ch in s.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if c.is_control() => out.push(' '),
                c => out.push(c),
            }
        }
        out
    }

    pub(in crate::backend::mir_interpreter) fn box_trace_emit_new(&self, class_name: &str, argc: usize) {
        if !Self::box_trace_enabled() || !Self::box_trace_filter_match(class_name) { return; }
        eprintln!(
            "{{\"ev\":\"new\",\"class\":\"{}\",\"argc\":{}}}",
            Self::json_escape(class_name), argc
        );
    }

    pub(in crate::backend::mir_interpreter) fn box_trace_emit_call(&self, class_name: &str, method: &str, argc: usize) {
        if !Self::box_trace_enabled() || !Self::box_trace_filter_match(class_name) { return; }
        eprintln!(
            "{{\"ev\":\"call\",\"class\":\"{}\",\"method\":\"{}\",\"argc\":{}}}",
            Self::json_escape(class_name), Self::json_escape(method), argc
        );
    }

    #[allow(dead_code)]
pub(in crate::backend::mir_interpreter) fn box_trace_emit_get(&self, class_name: &str, field: &str, val_kind: &str) {
        if !Self::box_trace_enabled() || !Self::box_trace_filter_match(class_name) { return; }
        eprintln!(
            "{{\"ev\":\"get\",\"class\":\"{}\",\"field\":\"{}\",\"val\":\"{}\"}}",
            Self::json_escape(class_name), Self::json_escape(field), Self::json_escape(val_kind)
        );
    }

    pub(in crate::backend::mir_interpreter) fn box_trace_emit_set(&self, class_name: &str, field: &str, val_kind: &str) {
        if !Self::box_trace_enabled() || !Self::box_trace_filter_match(class_name) { return; }
        eprintln!(
            "{{\"ev\":\"set\",\"class\":\"{}\",\"field\":\"{}\",\"val\":\"{}\"}}",
            Self::json_escape(class_name), Self::json_escape(field), Self::json_escape(val_kind)
        );
    }
}

// ---- Print trace (dev-only) ----
impl MirInterpreter {
    #[inline]
    pub(in crate::backend::mir_interpreter) fn print_trace_enabled() -> bool {
        std::env::var("NYASH_PRINT_TRACE").ok().as_deref() == Some("1")
    }

    pub(in crate::backend::mir_interpreter) fn print_trace_emit(&self, val: &VMValue) {
        if !Self::print_trace_enabled() { return; }
        let (kind, class, nullish): (&'static str, String, Option<&'static str>) = match val {
            VMValue::Integer(_) => ("Integer", "".to_string(), None),
            VMValue::Float(_) => ("Float", "".to_string(), None),
            VMValue::Bool(_) => ("Bool", "".to_string(), None),
            VMValue::String(_) => ("String", "".to_string(), None),
            VMValue::Void => ("Void", "".to_string(), None),
            #[cfg(feature = "legacy-boxes")]
            VMValue::Future(_) => ("Future", "".to_string(), None),
            VMValue::BoxRef(b) => {
                // Prefer InstanceBox.class_name when available
                if let Some(inst) = b.as_any().downcast_ref::<crate::instance_v2::InstanceBox>() {
                    let tag = if crate::config::env::null_missing_box_enabled() {
                        #[cfg(feature = "legacy-boxes")]
                        {
                            if b.as_any().downcast_ref::<crate::boxes::null_box::NullBox>().is_some() { Some("null") }
                            else if b.as_any().downcast_ref::<crate::boxes::missing_box::MissingBox>().is_some() { Some("missing") }
                            else { None }
                        }
                        #[cfg(not(feature = "legacy-boxes"))]
                        { None }
                    } else { None };
                    ("BoxRef", inst.class_name.clone(), tag)
                } else {
                    let tag = if crate::config::env::null_missing_box_enabled() {
                        #[cfg(feature = "legacy-boxes")]
                        {
                            if b.as_any().downcast_ref::<crate::boxes::null_box::NullBox>().is_some() { Some("null") }
                            else if b.as_any().downcast_ref::<crate::boxes::missing_box::MissingBox>().is_some() { Some("missing") }
                            else { None }
                        }
                        #[cfg(not(feature = "legacy-boxes"))]
                        { None }
                    } else { None };
                    ("BoxRef", b.type_name().to_string(), tag)
                }
            }
        };
        if let Some(tag) = nullish {
            eprintln!(
                "{{\"ev\":\"print\",\"kind\":\"{}\",\"class\":\"{}\",\"nullish\":\"{}\"}}",
                kind,
                Self::json_escape(&class),
                tag
            );
        } else {
            eprintln!(
                "{{\"ev\":\"print\",\"kind\":\"{}\",\"class\":\"{}\"}}",
                kind,
                Self::json_escape(&class)
            );
        }
    }
}
