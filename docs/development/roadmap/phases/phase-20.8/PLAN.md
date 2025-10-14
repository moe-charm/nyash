# Phase 20.8: GC + Rust Deprecation - Implementation Plan

**Duration**: 6 weeks (2026-07-20 → 2026-08-30)
**Status**: Not Started
**Prerequisites**: Phase 20.7 (Collections in Hakorune) completed

---

## Executive Summary

Phase 20.8 is the final phase of the "Pure Hakorune" initiative, completing the vision of "Rust=floor, Hakorune=house". It consists of two sub-phases:

1. **Phase E: GC v0** (Week 1-4) - Implement Mark & Sweep garbage collection in Hakorune
2. **Phase F: Rust VM Deprecation** (Week 5-6) - Deprecate Rust VM and achieve true self-hosting

Upon completion, Hakorune will be a fully self-hosted language with:
- **Rust layer**: ≤ 100 lines (HostBridge API only)
- **Hakorune layer**: Everything else (VM, parser, collections, GC, stdlib)

---

## Week 1-4: Phase E - GC v0 (Mark & Sweep)

### Overview

Implement a minimal stop-the-world Mark & Sweep garbage collector in Hakorune.

**Key Principles**:
- **Simplicity**: No generational GC, no incremental GC
- **Correctness**: Zero memory leaks
- **Observability**: Full GC tracing via `HAKO_GC_TRACE=1`
- **Fail-Fast**: Invalid GC states panic immediately

---

### Week 1-2: Mark Phase Implementation

#### Task 1.1: GC Roots Detection

Implement detection of GC roots (entry points for reachability analysis):

```hakorune
// GcBox implementation
box GcBox {
    // Root set
    stack_roots: ArrayBox      // Stack frame locals
    global_roots: ArrayBox     // Global static boxes
    handle_roots: ArrayBox     // HandleRegistry handles

    birth() {
        from Box.birth()
        me.stack_roots = new ArrayBox()
        me.global_roots = new ArrayBox()
        me.handle_roots = new ArrayBox()
    }

    collect_roots() {
        // 1. Scan stack frames for local variables
        me.scan_stack_frames()

        // 2. Scan global static boxes
        me.scan_global_boxes()

        // 3. Scan HandleRegistry for C-ABI handles
        me.scan_handle_registry()
    }

    scan_stack_frames() {
        // Iterate through VM stack frames
        local frame = ExecBox.current_frame()
        loop(frame != null) {
            local locals = frame.get_locals()
            locals.foreach(func(local) {
                me.stack_roots.push(local)
            })
            frame = frame.parent()
        }
    }

    scan_global_boxes() {
        // Scan all static boxes
        local globals = GlobalRegistry.all_static_boxes()
        globals.foreach(func(box) {
            me.global_roots.push(box)
        })
    }

    scan_handle_registry() {
        // Scan HandleRegistry for live handles
        local handles = HandleRegistry.all_handles()
        handles.foreach(func(handle) {
            me.handle_roots.push(handle)
        })
    }
}
```

**Deliverables**:
- [ ] GC roots detection implemented
- [ ] Stack frame scanning
- [ ] Global box scanning
- [ ] HandleRegistry scanning
- [ ] Tests: Verify all roots found

#### Task 1.2: Mark Algorithm

Implement mark phase (trace reachable objects):

```hakorune
box GcBox {
    marked: MapBox  // object_id -> true

    mark() {
        me.marked = new MapBox()

        // Mark all objects reachable from roots
        me.stack_roots.foreach(func(root) {
            me.mark_object(root)
        })
        me.global_roots.foreach(func(root) {
            me.mark_object(root)
        })
        me.handle_roots.foreach(func(root) {
            me.mark_object(root)
        })
    }

    mark_object(obj) {
        local obj_id = obj.object_id()

        // Already marked?
        if (me.marked.has(obj_id)) {
            return
        }

        // Mark this object
        me.marked.set(obj_id, true)

        // Recursively mark children
        local children = obj.get_children()
        children.foreach(func(child) {
            me.mark_object(child)
        })
    }
}
```

**Deliverables**:
- [ ] Mark algorithm implemented
- [ ] Recursive marking of children
- [ ] Cycle detection (avoid infinite loops)
- [ ] Tests: Verify mark correctness

---

### Week 3-4: Sweep Phase & Metrics

#### Task 3.1: Sweep Algorithm

Implement sweep phase (free unmarked objects):

```hakorune
box GcBox {
    all_objects: ArrayBox  // All allocated objects

    sweep() {
        local freed_count = 0
        local start_time = TimeBox.now_ms()

        // Iterate through all objects
        local survivors = new ArrayBox()
        me.all_objects.foreach(func(obj) {
            local obj_id = obj.object_id()

            if (me.marked.has(obj_id)) {
                // Survivor: Keep it
                survivors.push(obj)
            } else {
                // Garbage: Free it
                obj.destroy()
                freed_count = freed_count + 1
            }
        })

        // Update object list
        me.all_objects = survivors

        local sweep_time = TimeBox.now_ms() - start_time
        me.log_sweep(freed_count, survivors.size(), sweep_time)
    }

    log_sweep(freed, survivors, time_ms) {
        if (EnvBox.has("HAKO_GC_TRACE")) {
            ConsoleBox.log("[GC] Sweep phase: " + freed + " objects freed (" + time_ms + "ms)")
            ConsoleBox.log("[GC] Survivors: " + survivors + " objects")
        }
    }
}
```

**Deliverables**:
- [ ] Sweep algorithm implemented
- [ ] Object destruction (finalization)
- [ ] Survivor list updated
- [ ] Tests: Verify sweep correctness

#### Task 3.2: GC Metrics Collection

Implement GC metrics for observability:

```hakorune
box GcBox {
    metrics: GcMetricsBox

    birth() {
        from Box.birth()
        me.metrics = new GcMetricsBox()
    }

    collect() {
        me.metrics.increment_collections()

        // Mark phase
        local mark_start = TimeBox.now_ms()
        me.collect_roots()
        me.mark()
        local mark_time = TimeBox.now_ms() - mark_start

        // Sweep phase
        local sweep_start = TimeBox.now_ms()
        me.sweep()
        local sweep_time = TimeBox.now_ms() - sweep_start

        // Record metrics
        me.metrics.record_collection(mark_time, sweep_time, me.marked.size())
    }
}

box GcMetricsBox {
    total_allocations: IntegerBox
    total_collections: IntegerBox
    total_freed: IntegerBox
    peak_handles: IntegerBox

    birth() {
        from Box.birth()
        me.total_allocations = 0
        me.total_collections = 0
        me.total_freed = 0
        me.peak_handles = 0
    }

    increment_allocations() {
        me.total_allocations = me.total_allocations + 1
    }

    increment_collections() {
        me.total_collections = me.total_collections + 1
    }

    record_collection(mark_time, sweep_time, survivors) {
        // Log metrics
        if (EnvBox.has("HAKO_GC_TRACE")) {
            ConsoleBox.log("[GC] Mark phase: " + mark_time + "ms")
            ConsoleBox.log("[GC] Sweep phase: " + sweep_time + "ms")
            ConsoleBox.log("[GC] Survivors: " + survivors)
        }
    }

    print_stats() {
        ConsoleBox.log("[GC Stats] Total allocations: " + me.total_allocations)
        ConsoleBox.log("[GC Stats] Total collections: " + me.total_collections)
        ConsoleBox.log("[GC Stats] Total freed: " + me.total_freed)
        ConsoleBox.log("[GC Stats] Peak handles: " + me.peak_handles)
    }
}
```

**Deliverables**:
- [ ] GcMetricsBox implemented
- [ ] Allocation/collection counters
- [ ] Timing metrics
- [ ] `HAKO_GC_TRACE=1` logging
- [ ] Tests: Verify metrics accuracy

#### Task 3.3: Integration & Testing

Integrate GC with VM execution:

```hakorune
box MiniVmBox {
    gc: GcBox

    birth() {
        from Box.birth()
        me.gc = new GcBox()
    }

    allocate_object(obj) {
        // Register with GC
        me.gc.register_object(obj)
        me.gc.metrics.increment_allocations()

        // Trigger GC if needed
        if (me.gc.should_collect()) {
            me.gc.collect()
        }

        return obj
    }

    destroy() {
        // Print GC stats before exit
        me.gc.metrics.print_stats()
        from Box.destroy()
    }
}
```

**Deliverables**:
- [ ] GC integrated with VM
- [ ] Allocation hook
- [ ] GC trigger policy
- [ ] Stats printed at exit
- [ ] Tests: End-to-end GC validation

---

## Week 5-6: Phase F - Rust VM Deprecation

### Overview

Deprecate Rust VM and achieve true self-hosting with Hakorune VM as the default backend.

**Goals**:
1. Make Hakorune-VM the default (`--backend vm`)
2. Move Rust-VM to opt-in mode (`--backend vm-rust`, with warning)
3. Verify bit-identical self-compilation (Hako₁ → Hako₂ → Hako₃)
4. Minimize Rust layer to ≤ 100 lines (HostBridge API only)

---

### Week 5: Backend Switching & Deprecation

#### Task 5.1: Make Hakorune-VM Default

Update CLI argument parsing:

```rust
// src/cli.rs

#[derive(Parser)]
struct Cli {
    #[clap(long, default_value = "vm")]
    backend: Backend,
}

enum Backend {
    Vm,        // Hakorune-VM (new default)
    VmRust,    // Rust-VM (deprecated)
    Llvm,      // LLVM backend
}

fn main() {
    let cli = Cli::parse();

    if matches!(cli.backend, Backend::VmRust) {
        eprintln!("Warning: Rust-VM (--backend vm-rust) is deprecated.");
        eprintln!("         It will be removed in Phase 15.82.");
        eprintln!("         Use Hakorune-VM (--backend vm) instead.");
    }

    // Execute with chosen backend
    execute(cli.backend, &cli.input_file);
}
```

**Deliverables**:
- [ ] CLI updated (Hakorune-VM default)
- [ ] Deprecation warning added
- [ ] Documentation updated
- [ ] Tests: Verify default backend

#### Task 5.2: Golden Tests Verification

Verify Rust-VM vs Hakorune-VM parity:

```bash
# Run golden tests
./tools/golden_tests.sh

# Expected output:
# ✅ arithmetic.hako: Rust-VM == Hakorune-VM
# ✅ control_flow.hako: Rust-VM == Hakorune-VM
# ✅ collections.hako: Rust-VM == Hakorune-VM
# ✅ recursion.hako: Rust-VM == Hakorune-VM
# ✅ strings.hako: Rust-VM == Hakorune-VM
# ✅ enums.hako: Rust-VM == Hakorune-VM
# ✅ closures.hako: Rust-VM == Hakorune-VM
# ✅ selfhost_mini.hako: Rust-VM == Hakorune-VM
#
# All golden tests PASSED (8/8)
```

**Deliverables**:
- [ ] Golden test suite passes
- [ ] 100% Rust-VM vs Hakorune-VM parity
- [ ] CI integration
- [ ] Tests: All outputs match exactly

---

### Week 6: Bit-Identical Verification & Rust Minimization

#### Task 6.1: Bit-Identical Self-Compilation

Implement self-compilation chain verification:

```bash
#!/bin/bash
# tools/verify_self_compilation.sh

set -e

echo "=== Self-Compilation Verification ==="

# Hako₁: Rust-based compiler (current version)
echo "[1/5] Building Hako₁ (Rust-based compiler)..."
cargo build --release
cp target/release/hako hako_1

# Hako₂: Compiled by Hako₁
echo "[2/5] Building Hako₂ (via Hako₁)..."
./hako_1 apps/selfhost-compiler/main.hako -o hako_2
chmod +x hako_2

# Hako₃: Compiled by Hako₂
echo "[3/5] Building Hako₃ (via Hako₂)..."
./hako_2 apps/selfhost-compiler/main.hako -o hako_3
chmod +x hako_3

# Verify Hako₂ == Hako₃ (bit-identical)
echo "[4/5] Verifying bit-identical: Hako₂ == Hako₃..."
if diff hako_2 hako_3 > /dev/null; then
    echo "✅ SUCCESS: Hako₂ == Hako₃ (bit-identical)"
else
    echo "❌ FAILURE: Hako₂ != Hako₃"
    exit 1
fi

# Verify Hako₁ == Hako₂ (should match after stabilization)
echo "[5/5] Verifying bit-identical: Hako₁ == Hako₂..."
if diff hako_1 hako_2 > /dev/null; then
    echo "✅ SUCCESS: Hako₁ == Hako₂ (bit-identical)"
else
    echo "⚠️  WARNING: Hako₁ != Hako₂ (expected during transition)"
fi

echo ""
echo "=== Self-Compilation Verification PASSED ==="
```

**CI Integration**:

```yaml
# .github/workflows/self_compilation.yml
name: Self-Compilation Verification
on:
  push:
    branches: [main, private/selfhost]
  pull_request:
  schedule:
    - cron: '0 0 * * *'  # Daily at midnight

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Build Hako (Rust-based)
        run: cargo build --release

      - name: Self-Compilation Chain
        run: ./tools/verify_self_compilation.sh

      - name: Upload artifacts
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: self-compilation-failure
          path: |
            hako_1
            hako_2
            hako_3
```

**Deliverables**:
- [ ] `verify_self_compilation.sh` script
- [ ] CI workflow added
- [ ] Daily verification runs
- [ ] Bit-identical verification passes
- [ ] Tests: Hako₂ == Hako₃ confirmed

#### Task 6.2: Rust Layer Audit

Verify Rust layer ≤ 100 lines:

```bash
#!/bin/bash
# tools/audit_rust_layer.sh

echo "=== Rust Layer Audit ==="

# Count lines in HostBridge API
RUST_LINES=$(wc -l src/host_bridge.rs | awk '{print $1}')

echo "Rust layer: $RUST_LINES lines (target: ≤ 100)"

if [ "$RUST_LINES" -le 100 ]; then
    echo "✅ SUCCESS: Rust layer minimized (≤ 100 lines)"
else
    echo "❌ FAILURE: Rust layer too large (> 100 lines)"
    echo "   Please move more logic to Hakorune"
    exit 1
fi

# List Rust files (should be minimal)
echo ""
echo "Rust files:"
find src -name "*.rs" -not -path "src/host_bridge.rs" | while read file; do
    lines=$(wc -l "$file" | awk '{print $1}')
    echo "  $file: $lines lines (should be removed or moved to Hakorune)"
done
```

**Deliverables**:
- [ ] `audit_rust_layer.sh` script
- [ ] Rust layer ≤ 100 lines confirmed
- [ ] Non-HostBridge Rust files identified
- [ ] Migration plan for remaining Rust code
- [ ] Tests: Rust layer audit passes

#### Task 6.3: Final Documentation

Update documentation to reflect completion:

**Update CURRENT_TASK.md**:
```markdown
## ✅ Phase 20.8 Complete (2026-08-30)

- ✅ GC v0 implemented (Mark & Sweep)
- ✅ Hakorune-VM is default backend
- ✅ Rust-VM deprecated (--backend vm-rust)
- ✅ Bit-identical self-compilation verified
- ✅ Rust layer minimized (≤ 100 lines)

**Status**: True Self-Hosting Achieved
**Next**: Phase 15.82 (Advanced GC, Performance Optimization)
```

**Update README.md**:
```markdown
## Hakorune - True Self-Hosted Programming Language

Hakorune is a fully self-hosted language where the compiler, VM, and runtime
are all implemented in Hakorune itself.

**Architecture**:
- Rust layer: ~100 lines (HostBridge API for C-ABI boundary)
- Hakorune layer: Everything else (VM, parser, GC, stdlib)

**Self-Hosting Status**: ✅ Complete (2026-08-30)
```

**Deliverables**:
- [ ] CURRENT_TASK.md updated
- [ ] README.md updated
- [ ] Phase 20.8 completion report
- [ ] Phase 15.82 planning document

---

## Success Criteria

### Phase E (GC v0)

- [ ] GC v0 implemented and functional
- [ ] Mark & Sweep algorithms correct
- [ ] GC roots detected (stack, global, handles)
- [ ] Metrics collection working
- [ ] `HAKO_GC_TRACE=1` provides detailed logs
- [ ] Zero memory leaks in smoke tests
- [ ] Performance: GC overhead ≤ 10% of total runtime

### Phase F (Rust VM Deprecation)

- [ ] Hakorune-VM is default backend (`--backend vm`)
- [ ] Rust-VM deprecated with clear warning
- [ ] Bit-identical self-compilation verified (Hako₂ == Hako₃)
- [ ] CI daily verification passes
- [ ] Rust layer ≤ 100 lines (HostBridge API only)
- [ ] All smoke tests pass with Hakorune-VM
- [ ] Documentation complete

### Overall

- [ ] **True Self-Hosting**: Hakorune IS Hakorune
- [ ] **Rust=floor, Hakorune=house**: Architecture realized
- [ ] **Production Ready**: All tests pass, no memory leaks
- [ ] **Performance**: ≥ 50% of Rust-VM speed

---

## Risk Mitigation

### Risk 1: GC Bugs (Memory Leaks/Corruption)

**Mitigation**:
- Implement comprehensive tests (golden tests, smoke tests)
- Use `HAKO_GC_TRACE=1` for debugging
- Start with simple Mark & Sweep (no generational/incremental)
- Valgrind integration for leak detection

### Risk 2: Self-Compilation Divergence

**Mitigation**:
- Daily CI verification (Hako₂ == Hako₃)
- Freeze Rust VM after Phase 20.5 (no new features)
- Golden tests ensure Rust-VM vs Hakorune-VM parity
- Bisect on divergence to identify root cause

### Risk 3: Performance Degradation

**Mitigation**:
- Accept slower performance initially (≥ 50% of Rust-VM)
- Profile hot paths and optimize incrementally
- GC tuning (collection frequency, root set optimization)
- Defer advanced optimizations to Phase 15.82

### Risk 4: Incomplete Rust Minimization

**Mitigation**:
- Strict audit (Rust layer ≤ 100 lines)
- Move all logic to Hakorune (VM, collections, GC)
- HostBridge API is stable (no new features)
- Clear boundary: Rust=C-ABI only

---

## Timeline Summary

```
Week 1-2: GC Mark Phase
  - GC roots detection
  - Mark algorithm
  - Basic tracing

Week 3-4: GC Sweep Phase & Metrics
  - Sweep algorithm
  - Metrics collection
  - HAKO_GC_TRACE=1
  - Integration & testing

Week 5: Backend Switching & Deprecation
  - Hakorune-VM default
  - Rust-VM deprecation warning
  - Golden tests verification

Week 6: Bit-Identical Verification & Audit
  - Self-compilation chain
  - CI integration
  - Rust layer audit (≤ 100 lines)
  - Final documentation
```

---

## Related Documents

- **Phase 20.7 (Collections)**: [../phase-20.7/README.md](../phase-20.7/README.md)
- **Phase 15.80 (VM Core)**: [../phase-15.80/README.md](../phase-15.80/README.md)
- **Pure Hakorune Roadmap**: [../phase-20.5/PURE_HAKORUNE_ROADMAP.md](../phase-20.5/PURE_HAKORUNE_ROADMAP.md)
- **HostBridge API**: [../phase-20.5/HOSTBRIDGE_API_DESIGN.md](../phase-20.5/HOSTBRIDGE_API_DESIGN.md)

---

**Status**: Not Started
**Start Date**: 2026-07-20
**Target Completion**: 2026-08-30
**Dependencies**: Phase 20.7 must be complete
