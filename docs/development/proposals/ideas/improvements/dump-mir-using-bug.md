# --dump-mir Flag Bug with `using` Statements

## Issue
`./hako --dump-mir file.hako` fails with parse error when file contains `using` statements, but normal execution works correctly.

## Reproduction
```bash
# This FAILS:
./target/release/hako --dump-mir apps/selfhost/test_string_helpers.hako
# Output: ❌ Parse error: Expected identifier at line 2

# This WORKS:
./target/release/hako apps/selfhost/test_string_helpers.hako
# Output: Executes successfully
```

## File Example
```hako
// Test StringHelpers common library
using "apps/selfhost/common/string_helpers.hako" as Helpers

static box TestStringHelpers {
  main() {
    local s = Helpers.int_to_str(42)
    print(s)
    return 0
  }
}
```

## Root Cause
The `--dump-mir` code path has a different parser initialization or configuration that doesn't properly handle `using` statements.

## Impact
- **Severity**: Low (workaround exists)
- **Workaround**: Use normal execution instead of `--dump-mir`
- **Affects**: Development/debugging workflow only

## Priority
**20% category** - Implement only if:
- Frequent need for MIR inspection during selfhost development
- No easy alternative for debugging

## Discovered
Phase 15.11 (2025-10-05) during StringHelpers common library integration

## Related
- Phase 15.11 commit: `6ba6b026`
- All affected files compile and execute correctly without `--dump-mir`
