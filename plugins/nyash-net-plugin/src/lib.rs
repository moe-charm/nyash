//! Nyash Net Plugin (HTTP minimal) — TypeBox v2
//! Provides ServerBox/RequestBox/ResponseBox/ClientBox and socket variants.
//! Pure in-process HTTP over localhost for E2E of BoxRef args/returns.

mod logging;
use logging::net_log;

macro_rules! netlog {
    ($($arg:tt)*) => {{
        let s = format!($($arg)*);
        net_log(&s);
    }};
}

mod abi;
mod consts;
mod ffi;
mod http_helpers;
mod sockets;
mod state;
mod tlv;
mod boxes;

pub use abi::NyashTypeBoxFfi;
pub use boxes::*;
