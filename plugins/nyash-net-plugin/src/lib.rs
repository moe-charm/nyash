//! Nyash Net Plugin (HTTP minimal) — TypeBox v2
//! Provides ServerBox/RequestBox/ResponseBox/ClientBox and socket variants.
//! Pure in-process HTTP over localhost for E2E of BoxRef args/returns.

mod logging;
pub(crate) use logging::net_log;

macro_rules! netlog {
    ($($arg:tt)*) => {{
        let s = format!($($arg)*);
        crate::net_log(&s);
    }};
}

mod abi;
mod boxes;
mod consts;
mod ffi;
mod http_helpers;
mod sockets;
mod state;
mod tlv;

pub use abi::NyashTypeBoxFfi;
pub use boxes::*;
