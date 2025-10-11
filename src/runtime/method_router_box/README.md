Method Router Box (Facade)

Responsibility
- Classify receiver and delegate method calls to the right path (builtin/core vs plugin).
- Do not implement ad‑hoc fallbacks inline; any migration shims must live in small adapter boxes.

Inputs/Outputs
- Input: VMValue receiver, method name, args (VMValue[])
- Output: VMValue or VMError

Guards
- Stage‑1 fallback for Map.keys()/values() is delegated to adapters/map_keys_values_stage1.rs.
- HostHandle resolution is handled before plugin/builtin routing (see host_handle_router).

Non‑goals
- No string/array/map special‑case implementations here.
