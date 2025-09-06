# Abstract

## English Version

We present MIR-13, an ultra-minimal intermediate representation that reduces a traditional 26-instruction set to just 13 instructions while maintaining full computational capability and practical performance. This 50% reduction is achieved through our novel BoxCall unification principle, where array operations (ArrayGet/ArraySet) and field accesses are absorbed into a single, universal BoxCall instruction. 

Our key contributions are: (1) systematic instruction set reduction from Core-26 → Core-15 → Core-13 through empirical validation; (2) the BoxCall unification architecture that elegantly handles all data access patterns; (3) optimization strategies including inline caching (33x speedup), AOT compilation, and typed array specialization that compensate for the minimal instruction set; (4) proof that the "Everything is Box" philosophy can be effectively realized at the IR level without performance penalties.

Implementation results show that despite halving the instruction count, our benchmarks maintain performance within ±5% of the baseline while reducing MIR code size by 20-50%. The system successfully compiles complex applications including GUI programs, web servers, and distributed systems. This work demonstrates that IR minimalism, when coupled with strategic optimization placement, can achieve both extreme simplicity and production-level performance.

Our approach challenges the trend toward increasingly complex intermediate representations (e.g., LLVM's 60+ opcodes), showing that careful design can achieve more with less. We believe MIR-13 opens new possibilities for compiler construction, optimization research, and language implementation education.

## 日本語版

本研究では、従来の26命令セットをわずか13命令まで削減しながら、完全な計算能力と実用的な性能を維持する超最小中間表現MIR-13を提示する。この50%の削減は、配列操作（ArrayGet/ArraySet）やフィールドアクセスを単一の汎用BoxCall命令に吸収する、新規のBoxCall統一原理により実現された。

本研究の主要な貢献は以下の通りである：（1）Core-26 → Core-15 → Core-13への段階的な命令セット削減の実証的検証、（2）すべてのデータアクセスパターンをエレガントに処理するBoxCall統一アーキテクチャ、（3）最小命令セットを補完するインラインキャッシング（33倍高速化）、AOTコンパイル、型付き配列特化などの最適化戦略、（4）「Everything is Box」哲学がIRレベルで性能ペナルティなしに効果的に実現可能であることの証明。

実装結果は、命令数を半減させたにもかかわらず、ベンチマークがベースラインの±5%以内の性能を維持し、MIRコードサイズを20-50%削減することを示している。このシステムはGUIプログラム、Webサーバー、分散システムを含む複雑なアプリケーションのコンパイルに成功している。本研究は、IRミニマリズムが戦略的な最適化配置と組み合わされることで、極端なシンプルさと本番レベルの性能の両立が可能であることを実証した。

我々のアプローチは、ますます複雑化する中間表現（例：LLVMの60以上のオペコード）の傾向に挑戦し、慎重な設計により「より少ないものでより多くを達成できる」ことを示している。MIR-13はコンパイラ構築、最適化研究、言語実装教育に新たな可能性を開くと考えられる。

## Keywords / キーワード

Intermediate representation, Instruction set reduction, BoxCall unification, Compiler optimization, Inline caching, AOT compilation

中間表現、命令セット削減、BoxCall統一、コンパイラ最適化、インラインキャッシング、AOTコンパイル