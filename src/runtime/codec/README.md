# Runtime Codec Layer

This module owns the TLV encode/decode boundary that the runtime uses when
speaking to plugins.  All helpers should flow through `TlvCodecBox` so the
policy stays concentrated and testable.

- `codec_box.rs`: minimal implementation that maps `NyashBox` values into TLV
  payloads (with host/plugin handle shortcuts).
- Additional helpers can live beside it, but they must keep the TLV
  interpretation aligned with `plugin_ffi_common`.
