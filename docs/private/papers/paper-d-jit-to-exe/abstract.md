# Abstract

Modern programming languages face a fundamental trade-off between execution flexibility and deployment simplicity. Languages with JIT compilation offer excellent runtime performance but require complex runtime environments, while ahead-of-time compiled languages produce simple binaries but lack runtime optimization opportunities. We present a unified compilation pipeline that bridges this gap through an extremely minimal intermediate representation (MIR) consisting of only 13 instructions.

Our approach, implemented in the Nyash programming language, demonstrates that a single IR can efficiently target multiple execution backends: interpreter, JIT, AOT, and WebAssembly. The key innovation lies in the "Everything is Box" philosophy, which provides a uniform memory model that simplifies both compilation and runtime behavior. By integrating Cranelift for code generation and embedding lld as the linker, we achieve complete independence from external toolchains while maintaining competitive performance.

We introduce three novel contributions: (1) MIR13, an extremely minimal IR that captures the full semantics of a dynamic language in just 13 instructions, (2) a unified execution pipeline that seamlessly transitions from interpretation to JIT to native code generation, and (3) a C ABI facade that enables clean integration with existing systems while preserving the simplicity of the Box model.

Our evaluation shows that programs compiled through this pipeline achieve performance within 15% of hand-optimized C code while maintaining the development productivity of dynamic languages. The JIT-to-native transition is completely transparent, allowing developers to start with rapid prototyping and seamlessly move to production deployment. Binary sizes are competitive with Go, typically 2-5MB for real-world applications.

This work demonstrates that the traditional boundaries between JIT and AOT compilation are artificial constraints that can be eliminated through careful IR design and unified runtime architecture. The resulting system is not only technically elegant but also practically useful, as evidenced by the self-hosting Nyash compiler written in just 20,000 lines of Nyash code (compared to 80,000 lines in the original Rust implementation).

## Keywords
Programming Languages, Compiler Design, Intermediate Representation, Just-In-Time Compilation, Ahead-of-Time Compilation, Code Generation, Nyash, Box Model