# hakorune-std (draft)

Hakorune standard boxes implemented in .hako, designed to run over a minimal WASM ABI.

Conventions
- Use extern_call("nykernel.*", ...) for memory operations.
- Validate inputs and throw on invalid state (Fail-Fast). No silent fallbacks.
- Keep hot paths simple; push complex logic into small helpers.

Status
- ArrayBox skeleton available; Map/String planned next.

