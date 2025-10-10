# Hako ABI Plugin Capabilities

Defines standard capability bits for plugin-provided TypeBoxes. Capabilities are used by the runtime for policy decisions (e.g., deterministic mode) and observability.

Bit layout (low 8 bits reserved, extend as needed):

- 0 (1<<0): IO — Filesystem I/O, console, non-deterministic storage
- 1 (1<<1): NET — Networking (TCP/UDP/HTTP, sockets)
- 2 (1<<2): ENV — Environment access (process env, cwd)
- 3 (1<<3): TIME — Wall-clock/time measurement timers
- 4 (1<<4): PROC — OS process/spawn/exec
- 5 (1<<5): UI — Display/UI, windowing/graphics
- 6 (1<<6): GPU — GPU/accelerators
- 7 (1<<7): PRIV — Elevated/privileged operations

Conventions

- A TypeBox sets its capabilities via `NyashTypeBoxFfi.capabilities` (u64).
- Multiple bits may be set if a box spans several areas.
- Boxes that are pure and deterministic should set 0.

Deterministic Mode (HAKO_DETERMINISTIC=1)

- Runtime denies creation of IO and NET capability boxes in deterministic mode.
- On-demand provider reprobe is disabled in deterministic mode.
- Other caps are currently allowed; future policies may add gates (documented in this file before activation).

Examples

- FileBox: `capabilities = 1<<0` (IO)
- Net family (Server/Request/Response/Sockets): `capabilities = 1<<1` (NET)
- PathBox: `capabilities = 0` (pure path manipulation)

Notes

- Capabilities are advisory but enforced where policy applies (e.g., deterministic runs, CI profiles).
- If a plugin cannot yet set caps (legacy), the host may apply heuristics for critical boxes (e.g., FileBox).
