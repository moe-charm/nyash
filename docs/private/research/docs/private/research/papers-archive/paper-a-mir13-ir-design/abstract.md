# Abstract

## English Version

We present MIR-14, a minimal yet practical intermediate representation that evolves from an initial 27-instruction set through aggressive reduction to 13 instructions, then pragmatically adds back one essential operation (UnaryOp) for a final count of 14 core instructions. This evolution demonstrates the tension between theoretical minimalism and practical efficiency in compiler design.

Our key contributions are: (1) systematic instruction set evolution from Core-27 → Core-13 → Core-14 through empirical validation; (2) the BoxCall unification architecture that elegantly handles all data access patterns; (3) recognition that certain primitive operations (UnaryOp for negation and NOT) significantly improve both compilation efficiency and runtime performance; (4) optimization strategies including inline caching (33x speedup), AOT compilation, and typed array specialization that complement the minimal instruction set; (5) proof that the "Everything is Box" philosophy can be effectively realized at the IR level while maintaining practical performance.

Implementation results show that despite halving the instruction count, our benchmarks maintain performance within ±5% of the baseline while reducing MIR code size by 20-50%. The system successfully compiles complex applications including GUI programs, web servers, and distributed systems. This work demonstrates that IR minimalism, when coupled with strategic optimization placement, can achieve both extreme simplicity and production-level performance.

Our approach challenges the trend toward increasingly complex intermediate representations (e.g., LLVM's 60+ opcodes), showing that careful design can achieve more with less. We believe MIR-14 opens new possibilities for compiler construction, optimization research, and language implementation education.

## 日本語版

本研究では、初期の27命令セットから積極的な削減により13命令まで圧縮し、その後実用性を考慮して必須演算（UnaryOp）を追加した14命令の最小限かつ実践的な中間表現MIR-14を提示する。この進化は、コンパイラ設計における理論的ミニマリズムと実践的効率性の間の緊張関係を示している。

本研究の主要な貢献は以下の通りである：（1）Core-27 → Core-13 → Core-14への段階的な命令セット進化の実証的検証、（2）すべてのデータアクセスパターンをエレガントに処理するBoxCall統一アーキテクチャ、（3）特定のプリミティブ演算（否定やNOT演算のためのUnaryOp）がコンパイル効率と実行時性能の両方を大幅に改善するという認識、（4）最小命令セットを補完するインラインキャッシング（33倍高速化）、AOTコンパイル、型付き配列特化などの最適化戦略、（5）「Everything is Box」哲学が実用的な性能を維持しながらIRレベルで効果的に実現可能であることの証明。

実装結果は、命令数を半減させたにもかかわらず、ベンチマークがベースラインの±5%以内の性能を維持し、MIRコードサイズを20-50%削減することを示している。このシステムはGUIプログラム、Webサーバー、分散システムを含む複雑なアプリケーションのコンパイルに成功している。本研究は、IRミニマリズムが戦略的な最適化配置と組み合わされることで、極端なシンプルさと本番レベルの性能の両立が可能であることを実証した。

我々のアプローチは、ますます複雑化する中間表現（例：LLVMの60以上のオペコード）の傾向に挑戦し、慎重な設計により「より少ないものでより多くを達成できる」ことを示している。MIR-14はコンパイラ構築、最適化研究、言語実装教育に新たな可能性を開くと考えられる。

## Keywords / キーワード

Intermediate representation, Instruction set reduction, BoxCall unification, Compiler optimization, Inline caching, AOT compilation

中間表現、命令セット削減、BoxCall統一、コンパイラ最適化、インラインキャッシング、AOTコンパイル