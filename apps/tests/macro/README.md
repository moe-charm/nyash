# Macro Tests Layout

- if/: If normalization cases (assign, print, return, chains/guards)
- loopform/: Loop normalization (carrier tail align, two-vars)
- collections/: Array/Map macro examples
- types/: Type checks (is/as)
- strings/: String macros (upper_string)
- identity/: Identity macro
- test_runner/: Macro test runner behavior (filters, args, return policy)

Each file is a thin include to the original sample kept at the root for now. Once stabilized, originals can be removed and bodies moved here.
