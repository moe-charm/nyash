Console Adapter

Responsibility
- Provide a single place to print VM values consistently (stdout).
- Normalize Void/null/String/BoxRef(String) handling without leaking internals.

Inputs/Outputs
- Input: &VMValue
- Output: printed line to stdout (one line per call)

Guards
- No plugin/builtin direct calls here; purely formatting and println.
