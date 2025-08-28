# Abstract

We present **Nyash**, a box-centric language that unifies lifecycles of user-level classes and native plugins under a single contract. Boxes form an ownership forest (single strong edge + weak references), ensuring deterministic `fini` execution and enabling a semantics-preserving **GC on/off switch** (`@must_drop/@gcable`). A thin **C ABI v0** allows plugins to be invoked identically from VM, JIT, AOT, and WASM; AOT uses static linking to eliminate PLT overhead. With a compact MIR (~26 ops) and ~4K LoC implementation, Nyash achieves equal I/O traces across all backends while delivering competitive performance. We show that this unified model simplifies FFI, preserves correctness, and enables box-local optimizations—something previous systems could not simultaneously guarantee.

## 日本語要約

**Nyash**は、ユーザーレベルのクラスとネイティブプラグインのライフサイクルを単一の契約で統一するBox中心の言語である。Boxは所有権の森（単一の強エッジ＋弱参照）を形成し、決定的な`fini`実行を保証し、意味論を保持する**GCオン/オフ切り替え**（`@must_drop/@gcable`）を可能にする。薄い**C ABI v0**により、プラグインはVM、JIT、AOT、WASMから同一に呼び出され、AOTは静的リンクによりPLTオーバーヘッドを排除する。コンパクトなMIR（〜26命令）と〜4K LoCの実装で、Nyashは全バックエンドで等しいI/Oトレースを達成しつつ、競争力のある性能を提供する。この統一モデルがFFIを簡潔にし、正しさを保持し、Box局所最適化を可能にすることを示す—これは既存システムが同時に保証できなかった特性である。