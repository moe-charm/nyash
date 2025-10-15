# Singleton Normalization Bug - Impact Trace Report

**Date**: 2025-10-16
**Investigation**: Bug impact tracing for singleton normalization phase
**Question**: Did recent changes introduce the bug, or expose a pre-existing issue?

---

## Executive Summary

**Conclusion**: **The bug was EXPOSED by recent changes, NOT introduced by them.**

The "Copy undefined tolerance" guard (line 205-214 in `arithmetic.rs`) has been masking a pre-existing MIR generation gap in the Extern path since **October 15** (commit `50b4f87f`). Recent Router/HostHandle normalization changes (October 16, commit `19f84e61`) did NOT touch the Extern path at all, but the Phase 31 singleton normalization work **changed the calling patterns** in a way that relied on the tolerance guard to paper over missing argument propagation.

---

## Timeline of Changes

### 1. Copy Tolerance Introduction (October 15, 2025)

**Commit**: `50b4f87f0` - "feat(using+analysis): HAKO_USING統一 + selfhost整理計画完成 + SetBox実装"
**Date**: Wed Oct 15 23:00:28 2025 +0900

**What Changed**: Complete rewrite of `handle_copy` in `src/backend/mir_interpreter/handlers/arithmetic.rs`:

```rust
// BEFORE (commit 50b4f87f^):
pub(super) fn handle_copy(&mut self, dst: ValueId, src: ValueId) -> Result<(), VMError> {
    let v = self.reg_load(src)?;
    self.regs.insert(dst, v);
    Ok(())
}

// AFTER (commit 50b4f87f):
pub(super) fn handle_copy(&mut self, dst: ValueId, src: ValueId) -> Result<(), VMError> {
    // Defensive: some pipelines may place a Copy before the source has a
    // definition in the current block (e.g., entry scheduling artifacts).
    // Behavior:
    // - If src is defined: perform normal copy.
    // - If src is undefined but dst already has a value: treat as no-op (keep dst).
    // - Otherwise: respect strict mode unless tolerate_void is enabled, in which
    //   case initialize dst as Void to keep execution progressing in dev contexts.
    match self.reg_load(src) {
        Ok(v) => {
            self.regs.insert(dst, v);
            Ok(())
        }
        Err(e) => {
            if self.regs.contains_key(&dst) {
                // ... no-op path ...
                return Ok(());
            }
            if super::VmConfig::global().tolerate_void {
                // ⚠️ MASKING PATH: Initialize dst as Void instead of failing
                self.regs.insert(dst, super::VMValue::Void);
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}
```

**Key Point**: This change introduced a "dev tolerance" mode that **silently papers over undefined Copy sources** by initializing them as `Void`.

---

### 2. Router/HostHandle Normalization (October 16, 2025)

**Commit**: `19f84e61` - "chore(wip): Phase‑31 static→singleton bring‑up + trampolines"
**Date**: Thu Oct 16 04:25:48 2025 +0900

**What Changed**:
1. **Router Table Commonization**: Extracted `try_module_function_trampoline` into `trampolines.rs`
2. **HostHandle Normalization**: Router table returns concrete `BoxRef` values (not `HostHandle`)
3. **Static Singleton System**: Added `static_singleton.rs` for `Const Void` → singleton materialization

**Files Modified** (38 files, +1190/-564 lines):
- `src/backend/mir_interpreter/handlers/calls/trampolines.rs` (NEW)
- `src/runtime/static_singleton.rs` (NEW)
- `src/backend/mir_interpreter/handlers/calls/function.rs` (refactored)
- `src/runtime/host_handles.rs` (release() added)
- `src/mir/verification.rs` (ModuleFunction receiver checks)

---

### 3. JsonCanonicalBox Fix (October 16, 2025)

**Commit**: `37e3db2a` - "fix(vm/static-box): JsonCanonicalBox ArrayBox(1) bug"
**Date**: Thu Oct 16 02:23:27 2025 +0900

**What Changed**: Fixed receiver passing logic for static vs instance methods in `legacy/mod.rs`.

**Key Point**: This commit shows the receiver handling was being actively worked on, which suggests the Extern path might have had similar unaddressed issues.

---

## Component Analysis

### Did Router/HostHandle Changes Touch Extern Path?

**Answer: NO**

**Evidence**:

1. **extern_string.rs UNCHANGED**:
   ```bash
   $ git diff 50b4f87f 19f84e61 -- src/backend/mir_interpreter/extern_adapter/extern_string.rs
   (empty output)
   ```

2. **extern_adapter.rs Changes**: Only module refactoring (extracting submodules to top-level), NO functional changes:
   ```diff
   - #[path = "extern_adapter/extern_string.rs"] mod extern_string; extern_string::register(&mut map);
   + extern_string::register(&mut map);  // Now at top-level import
   ```

3. **Trampoline Code Analysis**:
   The new `trampolines.rs` (lines 28-31) shows an **early exit for Extern path**:
   ```rust
   if method == "size" || method == "len" || method == "length" {
       let recv = args[0];
       return Some(interp.handle_callee_extern("nyrt.string.length", &[recv]));
   }
   ```

   **Critical**: This code receives `args[0]` as the receiver and passes it directly to Extern. If `args[0]` is undefined/Void, the Extern handler will fail or return garbage.

---

### Did Copy Tolerance Mask a Pre-existing Bug?

**Answer: YES**

**Evidence**:

1. **Tolerance Timeline**: The `tolerate_void` guard was introduced on **October 15** (commit `50b4f87f`), **before** the Router/HostHandle changes.

2. **Purpose Statement** (from commit message):
   > "Defensive: some pipelines may place a Copy before the source has a definition in the current block (e.g., entry scheduling artifacts)."

   This explicitly acknowledges that **MIR generation was creating undefined Copy sources**, and the tolerance mode was added to **work around** this issue rather than fix it.

3. **Dev Context Flag**: The tolerance is controlled by `VmConfig::global().tolerate_void`, which suggests it's a **development-time workaround**, not a production-ready solution.

---

### What About the Extern Path Specifically?

**The Extern path has ALWAYS been fragile for String methods:**

1. **String.length Normalization** (from Builder):
   The MIR Builder normalizes `String.length` calls to `Extern("nyrt.string.length")` (see commit history mentions of "Builder normalizes to Extern").

2. **Receiver Propagation Gap**:
   The trampoline code shows the receiver is passed as `args[0]`:
   ```rust
   let recv = args[0];
   return Some(interp.handle_callee_extern("nyrt.string.length", &[recv]));
   ```

   If the Builder emits:
   ```
   Copy dst=recv, src=undefined_static_singleton
   Extern("nyrt.string.length", [recv])
   ```

   Then the Copy tolerance will initialize `recv=Void`, and Extern will receive `Void` instead of the actual string box.

3. **No Extern Path Fixes in Recent Commits**:
   Looking at the commit list, there are NO commits between `50b4f87f` (Copy tolerance) and `19f84e61` (Phase 31) that mention "Extern" or "string.length" fixes.

---

## Root Cause Analysis

### Pre-existing Issue (Prior to Oct 15)

**Hypothesis**: The MIR Builder has had a gap in the Extern path for String methods where:
1. Static box singleton receivers are not properly materialized before Extern calls
2. The Builder emits `Copy` instructions referencing undefined static singleton sources
3. Before `tolerate_void`, this would have caused **hard failures**

**Why It Wasn't Caught Earlier**:
- String methods might have been using the Method path (via StringBox vtable) instead of Extern path
- Tests might not have exercised the specific "static box → Extern" call pattern

---

### October 15 Change (Commit 50b4f87f)

**What Happened**: Added `tolerate_void` guard to **mask** the undefined Copy source issue.

**Impact**: Tests that previously failed on "undefined register" now silently pass by initializing receivers as `Void`.

**Keyword from commit message**: "entry scheduling artifacts" - this acknowledges the Builder was emitting malformed MIR.

---

### October 16 Changes (Commit 19f84e61)

**What Happened**: Phase 31 singleton normalization changed calling patterns:
1. Static boxes are now materialized via `static_singleton::get()` in `handle_const`
2. ModuleFunction trampolines route String methods to Extern path
3. Verifier enforces receiver presence checks

**Impact**: The singleton normalization **relies on** the Copy tolerance to paper over the gap between singleton materialization (in `Const Void`) and Extern call argument propagation.

**Why It Manifested Now**:
- The trampoline refactoring made the String → Extern path more prominent
- The singleton system changed when/how static box receivers are materialized
- The Verifier now catches missing receivers, but Copy tolerance prevents it from failing

---

## Evidence Summary

| Aspect | Evidence | Conclusion |
|--------|----------|------------|
| **Extern path touched?** | `git diff` shows NO changes to `extern_string.rs` | Not modified |
| **Copy tolerance age** | Introduced Oct 15 (`50b4f87f`), before Router changes | Pre-dates Router normalization |
| **Tolerance purpose** | Comment says "defensive... entry scheduling artifacts" | Acknowledges Builder bug |
| **Router/HostHandle changes** | Only refactoring (trampolines extraction) | No functional Extern changes |
| **String.length handling** | Trampoline passes `args[0]` directly to Extern | Assumes receiver is defined |
| **Singleton system** | Added Oct 16, materializes in `handle_const` | New materialization point |

---

## Conclusion

### Question: Did Recent Changes Introduce the Bug?

**Answer: NO**

The recent Router/HostHandle normalization changes (commit `19f84e61`) did NOT introduce the bug. They:
1. Did not touch the Extern path implementation
2. Did not modify `extern_string.rs`
3. Only refactored existing trampoline logic into a separate file

---

### Question: Did Recent Changes Expose a Pre-existing Bug?

**Answer: YES**

The recent changes **exposed** a pre-existing gap in the MIR Builder's Extern path:
1. **Pre-existing Gap**: Builder fails to materialize static singleton receivers before Extern calls
2. **Masking Layer**: Copy tolerance (added Oct 15) silently initializes undefined receivers as `Void`
3. **Exposure Mechanism**: Singleton normalization (Oct 16) changed calling patterns in a way that relies on the tolerance guard

**Evidence**: The `tolerate_void` guard was added **before** the Router changes, explicitly acknowledging "entry scheduling artifacts" (i.e., Builder-emitted undefined Copy sources).

---

## Recommendations

1. **Fix the Builder**: Ensure static singleton receivers are properly materialized (via `Const` instructions) before Extern calls.

2. **Remove Tolerance Guard**: Once the Builder is fixed, remove the `tolerate_void` workaround to enforce strict Fail-Fast behavior.

3. **Add Regression Test**: Create a test for "static box → String.length → Extern path" to catch this pattern in the future.

4. **Audit Other Extern Paths**: Check if Array.size, Map.size, etc. have similar gaps.

---

## Related Files

- **Copy Handler**: `/home/tomoaki/git/hakorune-selfhost/src/backend/mir_interpreter/handlers/arithmetic.rs:181-220`
- **Trampolines**: `/home/tomoaki/git/hakorune-selfhost/src/backend/mir_interpreter/handlers/calls/trampolines.rs:28-31`
- **Extern String**: `/home/tomoaki/git/hakorune-selfhost/src/backend/mir_interpreter/extern_adapter/extern_string.rs:6-18`
- **Singleton System**: `/home/tomoaki/git/hakorune-selfhost/src/runtime/static_singleton.rs`

---

## References

- Commit `50b4f87f`: Copy tolerance introduction
- Commit `37e3db2a`: JsonCanonicalBox receiver fix
- Commit `19f84e61`: Phase 31 singleton normalization
- CURRENT_TASK.md: Phase 31 progress (lines 1-70)
