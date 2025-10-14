//! Layer Interfaces — Parser/Resolver contracts
//!
//! Minimal traits to decouple front-end layers. Implementations can be provided
//! by existing modules and gradually migrated.

/// Marker trait: data produced by the parser layer.
pub trait ParserOutput: Send + Sync {}

/// Marker trait: input accepted by the resolver layer.
/// Typically extends `ParserOutput` to ensure pipeline continuity.
pub trait ResolverInput: ParserOutput {}

/// Common error shape for front-end layers (minimal, expandable later).
#[derive(Debug, Clone)]
pub struct FrontendError {
    pub message: String,
}

impl FrontendError {
    pub fn new(message: impl Into<String>) -> Self { Self { message: message.into() } }
}

