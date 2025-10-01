Hako Kernel — static runtime shim (alias of Nyash Kernel)

Scope
- Thin alias crate that builds a static library `libhako_kernel.a` linking to `nyash-rust`.
- Used by ny-llvmc when emitting executables; coexists with legacy `nyash_kernel` during migration.

Build
- cargo build --release -p hako_kernel
- Output: crates/hako_kernel/target/release/libhako_kernel.a

Notes
- This crate does not duplicate runtime logic; it depends on `nyash-rust` directly.
- ny-llvmc searches both `libnyash_kernel.a` and `libhako_kernel.a` (compat).

