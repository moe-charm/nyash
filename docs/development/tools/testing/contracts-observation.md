# Contracts Observation (dev-only)

This note documents the development-only observation logs emitted by the VM and plugin loader when the following envs are enabled. Behavior remains unchanged; logs are printed as single-line JSON to stderr to aid parity and correctness checks.

- NYASH_CHECK_CONTRACTS=1
  - NewBox → birth relation
    - contracts_newbox: {"kind":"contracts_newbox","class":"<Box>","argc":N,"key":<id>}
    - contracts_birth:  {"kind":"contracts_birth","seen_new":0|1,"duplicate":0|1,"argc_new":N,"argc_birth":M,"argc_match":0|1,"key":<id>}
  - Arity checks (Box methods)
    - contracts_arity:       {"kind":"contracts_arity","box":"ArrayBox","method":"set","expected":2,"got":M}
    - contracts_arity_min:   {"kind":"contracts_arity_min","box":"StringBox","method":"indexOf","min_expected":1,"got":0}
    - contracts_arity_range: {"kind":"contracts_arity_range","box":"StringBox","method":"is_digit_char","min":0,"max":1,"got":K}
  - Type/index hints
    - contracts_type:  {"kind":"contracts_type","box":"MapBox","method":"get","expected":"String","actual":"<Type>"}
    - contracts_index: {"kind":"contracts_index","box":"ArrayBox","method":"get","neg":0|1,"oob":0|1,"idx":I,"len":L}
  - Plugin route warnings
    - contracts_warn:  {"kind":"contracts_warn","what":"plugin_invoke_non_plugin","method":"..."}

- NYASH_TRACE_EFFECTS=1
  - Plugin call/return traces (Final/v2)
    - plugin_call: {"kind":"plugin_call","box":"<type>","method":"<name>","argc":N}
    - plugin_ret:  {"kind":"plugin_ret","box":"<type>","method":"<name>","tag":"<NyashBox|<none>|<error>>"}

- NYASH_CALL_TRACE=1
  - Unified call trace (VM runtime) — see smoke-tests-v2.md for `call_trace_diff.sh`

Notes
- Keys and indices are observed best-effort; behavior remains unchanged.
- Logs are development-only; keep them env-gated in normal runs/CI.
