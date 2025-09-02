# Abstract

## English Version

We present Nyash, a programming language that achieves full-stack application development—including GUI applications on Ubuntu and Windows—using only 15 intermediate representation (IR) instructions. This unprecedented minimalism is enabled by the "Everything is Box" philosophy, where all language features including variables, functions, concurrency, garbage collection, plugins, and even GUI components are unified under a single Box abstraction. 

Our key contributions are: (1) the design of MIR15, a minimal instruction set that serves as the "atomic elements" of computation; (2) the Box Theory, which provides a mathematical foundation for composing complex behaviors from these atoms; (3) empirical validation showing that the same 15-instruction MIR can drive a VM interpreter, JIT compiler, AOT compiler, and WebAssembly backend with identical semantics; and (4) demonstration of real-world GUI applications running on multiple operating systems using this minimal foundation.

We implemented the entire system—including all four backends—in 30 days with approximately 4,000 lines of code, suggesting that extreme minimalism can paradoxically accelerate development. Performance evaluation shows that despite the minimal instruction set, the system achieves competitive performance through strategic optimization placement within Box boundaries rather than IR complexity.

This work challenges the conventional wisdom that practical programming languages require large instruction sets, demonstrating instead that careful abstraction design can achieve both extreme simplicity and full functionality. We believe this approach opens new possibilities for language design, implementation, and education.

## 日本語版

本研究では、わずか15個の中間表現（IR）命令でUbuntuおよびWindows上のGUIアプリケーションを含むフルスタック開発を実現するプログラミング言語Nyashを提示する。この前例のないミニマリズムは「Everything is Box」哲学により可能となった。変数、関数、並行性、ガベージコレクション、プラグイン、さらにはGUIコンポーネントまで、すべての言語機能が単一のBox抽象化の下に統一されている。

本研究の主要な貢献は以下の通りである：（1）計算の「原子要素」として機能する最小命令セットMIR15の設計、（2）これらの原子から複雑な振る舞いを構成するための数学的基礎を提供するBox理論、（3）同一の15命令MIRがVMインタープリタ、JITコンパイラ、AOTコンパイラ、WebAssemblyバックエンドを同一のセマンティクスで駆動できることの実証的検証、（4）この最小基盤上で複数のOSで動作する実用的なGUIアプリケーションのデモンストレーション。

我々は4つのバックエンドを含む全システムを30日間、約4,000行のコードで実装した。これは極端なミニマリズムが逆説的に開発を加速させる可能性を示唆している。性能評価により、最小命令セットにもかかわらず、IRの複雑性ではなくBox境界内での戦略的な最適化配置により競争力のある性能を達成できることが示された。

本研究は、実用的なプログラミング言語には大規模な命令セットが必要という従来の常識に挑戦し、慎重な抽象化設計により極端なシンプルさと完全な機能性の両立が可能であることを実証した。このアプローチは言語設計、実装、教育に新たな可能性を開くと考えられる。

## Keywords / キーワード

Minimal instruction set, Intermediate representation, Box abstraction, Full-stack development, Cross-platform GUI, Language design

最小命令セット、中間表現、Box抽象化、フルスタック開発、クロスプラットフォームGUI、言語設計