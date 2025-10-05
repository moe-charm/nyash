
//! CallNameResolverBox — thin wrapper over core SSOT

pub struct CallNameResolverBox;

impl CallNameResolverBox {
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
