# Abstract

## Practical SSA Construction for a Box-Oriented Language

Building Static Single Assignment (SSA) form for dynamically-typed languages presents unique challenges, particularly when the language philosophy mandates that "Everything is Box." This paper presents our practical experience constructing SSA form for Nyash, a Box-oriented language targeting LLVM IR.

We identify three main challenges: (1) PHI node placement in complex control flow with mixed handle/pointer types, (2) maintaining correct insertion points across nested scopes and loops, and (3) ensuring dominance properties while supporting dynamic Box operations. 

Our contributions include:
- **BuilderCursor**: A structured approach to LLVM builder position management that prevents post-terminator insertions
- **Sealed SSA with snapshots**: Block-end value snapshots for reliable PHI wiring in gradually-constructed CFGs  
- **Type normalization strategy**: Unified i64 handle representation with on-demand pointer conversions

We evaluate our approach on real-world Nyash programs, including a self-hosting compiler component (`dep_tree_min_string.nyash`). While achieving functional SSA construction, we identify remaining challenges and propose LoopForm IR as a future direction for simplifying control flow representation.

This work provides practical insights for language implementers facing similar challenges when bridging high-level Box abstractions to low-level SSA form.

---

## 和文要旨

動的型付け言語、特に「Everything is Box」を哲学とする言語において、静的単一代入（SSA）形式を構築することは独特の課題を提示する。本稿では、LLVM IRを対象とするBox指向言語Nyashにおける、SSA形式構築の実践的経験を報告する。

主要な課題として、(1)ハンドル/ポインタ混在型での複雑な制御フローにおけるPHIノード配置、(2)ネストしたスコープとループにまたがる挿入位置の維持、(3)動的Box操作をサポートしながらのdominance特性の保証、の3点を特定した。

本稿の貢献は以下の通りである：
- **BuilderCursor**：終端後挿入を防ぐ構造化されたLLVMビルダー位置管理
- **スナップショット付きSealed SSA**：段階的に構築されるCFGでの確実なPHI配線のためのブロック終端値スナップショット
- **型正規化戦略**：オンデマンドポインタ変換を伴う統一的i64ハンドル表現

実世界のNyashプログラム（セルフホスティングコンパイラコンポーネント`dep_tree_min_string.nyash`を含む）で評価を行った。機能的なSSA構築を達成する一方で、残存する課題を特定し、制御フロー表現を簡素化する将来の方向性としてLoopForm IRを提案する。

本研究は、高レベルBox抽象を低レベルSSA形式に橋渡しする際に同様の課題に直面する言語実装者に、実践的な洞察を提供する。