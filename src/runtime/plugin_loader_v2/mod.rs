//! Nyash v2 Plugin Loader (split)

#[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
mod enabled;
#[cfg(any(not(feature = "plugins"), target_arch = "wasm32"))]
mod stub;

#[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
pub use enabled::*;
#[cfg(any(not(feature = "plugins"), target_arch = "wasm32"))]
pub use stub::*;
