//! TLV Codec facade.
//!
//! The codec implementation lives in [`codec_box`]; this module only re-exports
//! the primary entry so other crates keep a short import path.

mod codec_box;

pub use codec_box::TlvCodecBox;
