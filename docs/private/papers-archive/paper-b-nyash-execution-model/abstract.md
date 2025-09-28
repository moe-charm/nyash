# Abstract

## English Version

We present Nyash, a box-first programming language that introduces symmetric memory management through init/fini pairs and a novel P2P Intent communication model. Unlike traditional object-oriented or functional languages, Nyash's "Everything is Box" philosophy unifies all computational entities—from primitive values to distributed nodes—under a single abstraction. This design enables unprecedented flexibility: seamless transitions between garbage-collected and manual memory management, transparent plugin integration via TypeBox ABI, and natural expression of distributed applications.

Our key contributions are: (1) the init/fini symmetric lifecycle model that guarantees deterministic resource cleanup while supporting both GC and manual modes; (2) the P2P Intent system that elevates message passing to intent-based communication, enabling elegant distributed application design; (3) a multi-tier execution architecture (Interpreter → VM → JIT → AOT → WASM) with identical semantics across all backends; (4) real-world validation through applications including NyashCoin (P2P cryptocurrency), a plugin marketplace, and cross-platform GUI applications.

The language is implemented in ~4,000 lines of Rust, demonstrating that a powerful and flexible language can emerge from simple, orthogonal concepts. Performance evaluation shows that Nyash applications achieve near-native speed in AOT mode while maintaining the development convenience of dynamic languages in interpreter mode. User studies indicate that the Box model significantly reduces cognitive load for concurrent and distributed programming.

This work presents a fresh approach to language design where simplicity and power are not opposing forces but complementary aspects of a unified philosophy.

## 日本語版

本研究では、init/finiペアによる対称的メモリ管理と新規のP2P Intent通信モデルを導入するBox中心プログラミング言語Nyashを提示する。従来のオブジェクト指向言語や関数型言語とは異なり、Nyashの「Everything is Box」哲学は、プリミティブ値から分散ノードまで、すべての計算実体を単一の抽象化の下に統一する。この設計により前例のない柔軟性が実現される：ガベージコレクションと手動メモリ管理のシームレスな切り替え、TypeBox ABIによる透過的なプラグイン統合、分散アプリケーションの自然な表現。

本研究の主要な貢献は以下の通りである：（1）GCモードと手動モードの両方をサポートしながら決定的なリソースクリーンアップを保証するinit/fini対称ライフサイクルモデル、（2）メッセージパッシングを意図ベース通信に昇華させ、エレガントな分散アプリケーション設計を可能にするP2P Intentシステム、（3）すべてのバックエンドで同一のセマンティクスを持つ多層実行アーキテクチャ（インタープリタ → VM → JIT → AOT → WASM）、（4）NyashCoin（P2P暗号通貨）、プラグインマーケットプレイス、クロスプラットフォームGUIアプリケーションを含む実世界アプリケーションによる検証。

言語は約4,000行のRustで実装されており、シンプルで直交する概念から強力で柔軟な言語が生まれることを実証している。性能評価により、NyashアプリケーションはAOTモードでネイティブに近い速度を達成しながら、インタープリタモードでは動的言語の開発利便性を維持することが示された。ユーザー調査は、Boxモデルが並行・分散プログラミングの認知負荷を大幅に削減することを示している。

本研究は、シンプルさと強力さが対立する力ではなく、統一された哲学の補完的な側面である言語設計への新鮮なアプローチを提示する。

## Keywords / キーワード

Box-first programming, Symmetric memory management, P2P Intent model, Multi-tier execution, Plugin architecture, Distributed systems

Box中心プログラミング、対称的メモリ管理、P2P Intentモデル、多層実行、プラグインアーキテクチャ、分散システム