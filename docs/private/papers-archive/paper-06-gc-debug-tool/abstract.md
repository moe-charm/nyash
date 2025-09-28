# Abstract / アブストラクト

## English

We present a novel approach to memory management in programming languages where Garbage Collection (GC) is redefined not as a runtime memory management mechanism, but as a development-time quality assurance tool. In our language Nyash, developers use GC during development for safe exploratory programming and leak detection, then disable it for production deployment, achieving zero-overhead memory management through deterministic destruction patterns.

Our key contribution is the concept of "Ownership Forests" - a structural constraint ensuring that programs maintain identical behavior with GC enabled or disabled. This semantic equivalence is achieved through: (1) prohibition of circular references, maintaining forest structure in the object graph, (2) unified Arc<Mutex> architecture providing thread-safe reference counting, and (3) DebugBox infrastructure for comprehensive leak detection and visualization.

Preliminary results show that this approach maintains development productivity comparable to GC languages while achieving performance characteristics of manual memory management systems. The "Debug-Only GC" paradigm enables a progressive quality assurance process where programs "graduate" from GC-assisted development to deterministic production execution.

## 日本語

本研究では、ガベージコレクション（GC）を実行時のメモリ管理機構としてではなく、開発時の品質保証ツールとして再定義する革新的なアプローチを提示する。我々の開発したプログラミング言語Nyashでは、開発者は開発時にGCを使用して安全な探索的プログラミングとリーク検出を行い、本番デプロイ時にはGCを無効化することで、決定的な破棄パターンによるゼロオーバーヘッドのメモリ管理を実現する。

本研究の主要な貢献は「所有権森（Ownership Forests）」の概念である。これは、GCの有効/無効に関わらず同一の動作を保証する構造的制約である。この意味論的等価性は以下により実現される：(1) 循環参照の禁止によるオブジェクトグラフの森構造維持、(2) スレッドセーフな参照カウントを提供する統一Arc<Mutex>アーキテクチャ、(3) 包括的なリーク検出と可視化のためのDebugBoxインフラストラクチャ。

初期評価の結果、このアプローチはGC言語と同等の開発生産性を維持しながら、手動メモリ管理システムの性能特性を達成することが示された。「Debug-Only GC」パラダイムは、プログラムがGC支援開発から決定的な本番実行へと「卒業」する漸進的な品質保証プロセスを可能にする。

## Keywords / キーワード

- Garbage Collection
- Memory Management  
- Quality Assurance
- Ownership
- Programming Language Design
- ガベージコレクション
- メモリ管理
- 品質保証
- 所有権
- プログラミング言語設計