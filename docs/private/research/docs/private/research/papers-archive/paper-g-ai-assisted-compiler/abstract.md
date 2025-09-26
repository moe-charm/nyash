# Abstract

## AI-Assisted Compiler Development: Building Nyash LLVM Backend with ChatGPT and Claude

We present the first documented case of building a compiler's LLVM backend through intensive AI assistance over a week-long development period. The Nyash programming language, based on the "Everything is Box" philosophy, achieves a minimalist intermediate representation (MIR14) with just 14 instructions, evolved from an initial 27 through aggressive reduction and pragmatic restoration.

Our development process involved hundreds of AI interactions, with ChatGPT providing deep architectural analysis (including an 8-minute investigation into PHI node issues) and Claude offering continuous implementation support. Key technical contributions include: (1) MIR14 design achieving 27→13→14 instruction evolution, (2) LoopForm IR for control flow normalization, (3) Sealed SSA with Resolver API for unified value resolution, and (4) BuilderCursor for structural terminator safety.

The project faced and overcame significant challenges including PHI wiring complexity, dominance violations, and type system confusion between i64 handles and i8* pointers. A critical insight emerged when LoopForm, initially blamed for introducing problems, actually exposed pre-existing design flaws in value resolution and type conversion placement.

We document the pragmatic decision to introduce Python llvmlite as a verification harness when Rust/inkwell complexity became a bottleneck, demonstrating that "simple is best" - a core Nyash philosophy. The development logs, though incomplete ("I don't remember anymore"), provide valuable insights into the chaotic reality of AI-assisted development.

This work demonstrates that AI can successfully assist in building complex systems like compilers, but human design judgment remains essential. The co-evolution of human design decisions and AI implementation represents a new paradigm in software development, with implications for future compiler construction and computer science education.

## 日本語要旨

ChatGPTとClaudeを活用し、1週間以上の開発期間でLLVMバックエンドを構築した世界初の完全記録を報告する。「Everything is Box」哲学に基づくNyashプログラミング言語は、27命令から13命令への積極的削減と実用的な1命令復活により、わずか14命令の極限的中間表現（MIR14）を実現した。

数百回のAI対話を通じた開発では、ChatGPTがPHIノード問題の8分間にわたる深い調査を含むアーキテクチャ分析を提供し、Claudeが継続的な実装支援を行った。主要な技術的貢献として、(1) 27→13→14命令進化を遂げたMIR14設計、(2) 制御フロー正規化のためのLoopForm IR、(3) 統一的値解決のためのResolver API付きSealed SSA、(4) 構造的終端安全性のためのBuilderCursorが挙げられる。

開発過程では、PHI配線の複雑性、支配関係違反、i64ハンドルとi8*ポインタ間の型システム混乱など、重大な課題に直面した。当初LoopFormが問題の原因と誤解されたが、実際には既存の値解決と型変換配置の設計欠陥を顕在化させただけという重要な洞察を得た。

Rust/inkwellの複雑性がボトルネックとなった際、検証ハーネスとしてPython llvmliteを導入する実用的判断を下した。これは「簡単最高」というNyashの核心哲学の実証でもある。開発ログは不完全（「もう覚えてないにゃー」）ながらも、AI支援開発の混沌とした現実への貴重な洞察を提供する。

本研究は、コンパイラのような複雑なシステム構築においてもAI支援が有効であることを実証したが、人間の設計判断は依然として不可欠である。人間の設計決定とAI実装の共進化は、ソフトウェア開発の新しいパラダイムを示し、将来のコンパイラ構築と計算機科学教育に示唆を与える。