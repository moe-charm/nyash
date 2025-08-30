# Abstract / アブストラクト

## 日本語版

Nyashは「Everything is Box」という設計哲学に基づき、変数・関数・同期・GC・プラグインをすべてBoxで統一したプログラミング言語である。本研究では、中間表現MIRを従来の26命令から15命令に削減し、それにもかかわらずガベージコレクション、非同期処理、同期処理、プラグインシステム、さらには将来のGPU計算まで表現可能であることを示した。さらに、この15命令MIRを基盤に、インタープリタ（VM）、JITコンパイラ、AOTコンパイルによるネイティブ実行ファイル生成を、わずか30日で実装した。本稿ではMIR命令セットの設計、VM/JIT/AOTの等価性検証（I/Oトレース一致）、および4K行規模での実装経験を報告する。

### キーワード
プログラミング言語設計、中間表現、最小命令セット、Box統一モデル、コンパイラ実装

## English Version

Nyash is a programming language based on the philosophy of "Everything is a Box," unifying variables, functions, concurrency, garbage collection, and plugins under a single abstraction. We reduced the intermediate representation (MIR) from 26 to 15 instructions, while still being able to express garbage collection, asynchronous and synchronous operations, plugin systems, and even potential future GPU computation. Building on this 15-instruction MIR, we implemented an interpreter (VM), a JIT compiler, and an AOT compiler that produces native executables—all within 30 days. This paper presents the design of the MIR instruction set, the equivalence validation between VM/JIT/AOT (via I/O trace matching), and insights from a ~4 KLoC implementation.

### Keywords
Programming language design, Intermediate representation, Minimal instruction set, Unified Box model, Compiler implementation

## 要点整理

### 革新的な点
1. **15命令という極限的シンプルさ**: 実用言語として前例のない少なさ
2. **30日間での全実装**: 通常は年単位のプロジェクトを1ヶ月で
3. **4000行のコンパクトさ**: 教育・学習に最適な規模
4. **完全な機能性**: GC、並行性、プラグインまでカバー

### 実証された内容
- MIR命令数と表現力はトレードオフではない
- シンプルさが開発速度を劇的に向上させる
- 統一モデルが実装の見通しを良くする
- 小規模でも実用的な言語が作れる