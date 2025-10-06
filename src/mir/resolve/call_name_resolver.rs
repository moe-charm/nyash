
//! CallNameResolverBox — thin wrapper over core SSOT

pub struct CallNameResolverBox;

impl CallNameResolverBox {
    /// Return true when an identifier is valid for Box/Method segments.
    /// Policy: leading char is [A-Za-z_], tail chars are [A-Za-z0-9_].
    pub fn is_valid_ident(s: &str) -> bool {
        let mut chars = s.chars();
        match chars.next() {
            Some(c0) if c0.is_ascii_alphabetic() || c0 == '_' => {}
            _ => return false,
        }
        for c in chars {
            if !(c.is_ascii_alphanumeric() || c == '_') { return false; }
        }
        true
    }

    /// Build a fully-qualified static call name "Box.method/Arity".
    /// This preserves underscores and rejects invalid identifiers to Fail‑Fast.
    pub fn static_name(class: &str, method: &str, arity: usize) -> Result<String, String> {
        if !Self::is_valid_ident(class) {
            return Err(format!("invalid box name: '{}'", class));
        }
        if !Self::is_valid_ident(method) {
            return Err(format!("invalid method name: '{}'", method));
        }
        Ok(format!("{}.{}{}", class, method, format!("/{}", arity)))
    }


    /// Build fully qualified birth function name: Class.birth/Arity
    pub fn make_birth_name(class: &str, arity: usize) -> String {
        format!("{}.birth/{}", class, arity)
    }

    /// Return true if name is fully qualified (Class.method/Arity)
    pub fn is_fully_qualified(name: &str) -> bool {
        crate::mir::resolve::call_resolver_core::is_fully_qualified(name)
    }

    /// Require full name; return Err with context when not fully qualified
    pub fn require_full_name(name: &str, context: &str) -> Result<String, String> {
        if Self::is_fully_qualified(name) {
            Ok(name.to_string())
        } else {
            Err(format!(
                "{}: incomplete call name '{}' (must be Class.method/N)",
                context, name
            ))
        }
    }

    /// Normalize arbitrary name into fully qualified form using arity.
    pub fn normalize(raw_name: &str, argc: usize) -> Result<String, String> {
        crate::mir::resolve::call_resolver_core::normalize(raw_name, argc)
    }
}
