# arXiv Abstract (English)

## Title
Everything is Box meets MIR-15: A Minimal, Equivalence-Preserving Path from Language to Native in 30 Days

## Abstract
We present Nyash, a language architecture centered on "Everything is a Box." A 15-instruction MIR suffices to implement VM, JIT, AOT, GC, and async—without extending the IR. All high-level features funnel through Box and a unified plugin boundary via `ExternCall`, while the Lowerer/JIT remain world-agnostic. We validate semantic equivalence by matching end-to-end I/O traces across `{VM,JIT,AOT} × {GC on,off}` and report a ~4 KLoC reference implementation leveraging Cranelift. Nyash shows that a minimal, consistent core can deliver native executables, strong extensibility (e.g., GPU/quantum/FFI), and practical performance, offering a short, principled route from language design to deployable binaries.

(~150 words)