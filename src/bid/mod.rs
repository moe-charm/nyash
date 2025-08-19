// BID-FFI: Box Interface Definition with Foreign Function Interface
// Everything is Box philosophy meets practical FFI/ABI design!

pub mod types;
pub mod tlv;
pub mod error;
pub mod metadata;
pub mod plugin_api;
pub mod bridge;
pub mod plugins;
#[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
pub mod loader;
// pub mod registry;  // legacy - v2 plugin system uses BoxFactoryRegistry instead
// pub mod plugin_box;  // legacy - FileBox専用実装
pub mod generic_plugin_box;

pub use types::*;
pub use tlv::*;
pub use error::*;
pub use metadata::*;
pub use plugin_api::*;
pub use bridge::*;
#[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
pub use loader::*;
// pub use registry::*;  // legacy - v2 plugin system uses BoxFactoryRegistry instead
// pub use plugin_box::*;  // legacy
pub use generic_plugin_box::*;

/// BID-1 version constant
pub const BID_VERSION: u16 = 1;

/// Platform-dependent pointer size
#[cfg(target_pointer_width = "32")]
pub type Usize = u32;

#[cfg(target_pointer_width = "64")]
pub type Usize = u64;

/// Standard alignment requirement
pub const BID_ALIGNMENT: usize = 8;
