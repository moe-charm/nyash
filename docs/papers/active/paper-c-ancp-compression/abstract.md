# Abstract: Reversible 90% Code Compression via Multi-Stage Syntax Transformation

## English Abstract

Traditional code minification techniques, exemplified by tools like Terser and UglifyJS, achieve compression rates of 50-60% while sacrificing semantic information and variable naming. These approaches optimize for reduced file size rather than machine comprehension. 

In the era of AI-assisted programming, where Large Language Models (LLMs) face severe context limitations, we propose ANCP (AI-Nyash Compact Notation Protocol) - a novel multi-stage reversible code compression technique that achieves 90% token reduction while preserving complete semantic integrity.

Our approach introduces a three-layer transformation pipeline: Pretty (P) for human development, Compact (C) for distribution with 48% compression, and Fusion (F) for AI communication with 90% compression. Each transformation maintains perfect reversibility through bidirectional source maps and symbol tables.

We demonstrate our technique on Nyash, a box-first programming language, achieving compression ratios significantly exceeding existing state-of-the-art while enabling LLMs to process 2-3x larger codebases within context limits. Evaluation on a self-hosting compiler shows consistent 90% reduction across 80,000 lines of code with zero semantic loss.

This work challenges the fundamental assumption that code compression must sacrifice readability, instead proposing AI-optimized compression as a new dimension of language design.

**Keywords**: code compression, AI-assisted programming, reversible transformation, domain-specific languages, Box-first design

---

## 日本語要旨

従来のコード圧縮技術（Terser、UglifyJS等）は50-60%の圧縮率を達成するが、意味情報と変数名を犠牲にしている。これらの手法はファイルサイズ削減に最適化されており、機械理解には最適化されていない。

AI支援プログラミングの時代において、大規模言語モデル（LLM）が深刻なコンテキスト制限に直面する中、我々はANCP（AI-Nyash Compact Notation Protocol）を提案する。これは、完全な意味的整合性を保持しながら90%のトークン削減を達成する、新しい多段階可逆コード圧縮技術である。

我々のアプローチは3層変換パイプラインを導入する：人間開発用のPretty（P）、48%圧縮配布用のCompact（C）、90%圧縮AI通信用のFusion（F）。各変換は双方向ソースマップとシンボルテーブルによる完全可逆性を維持する。

Box-Firstプログラミング言語Nyashでの実証実験により、既存の最先端技術を大幅に上回る圧縮率を達成し、LLMがコンテキスト制限内で2-3倍大きなコードベースを処理可能にした。8万行の自己ホスティングコンパイラでの評価では、意味的損失ゼロで一貫した90%削減を実現した。

本研究は、コード圧縮が可読性を犠牲にしなければならないという根本的仮定に挑戦し、AI最適化圧縮を言語設計の新たな次元として提案する。

**キーワード**: コード圧縮, AI支援プログラミング, 可逆変換, ドメイン固有言語, Box-First設計