Integration/WASM — wasm parity/integration smokes (gated)

Purpose
- House WASM parity/integration tests separately to keep core/integration clean.

Gating (default SKIP)
- Set either `SMOKES_ENABLE_WASM=1` or `NYASH_WASM_USE=1` to enable.

Guidelines
- Keep each test short and with clear SKIP messages when harness is missing.
- Prefer comparing Nyash VM output vs WASM harness when available.

