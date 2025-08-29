Title: Box-First JIT: Decoupled, Probe-Driven JIT Enablement in Nyash within 24 Hours

Abstract (JP short)
本稿は、Nyash言語にJITを24時間で安全導入するための「箱理論（Box-First）」アプローチを提示する。JITエンジンをVMから疎結合化し、設定・境界・ハンドル・観測をすべて「箱」に封じ込めることで、強い可逆性（戻せる足場）と段階的最適化を両立した。具体的には、JitConfigBoxによるenv排除と実行時プローブ、HandleRegistryによるJIT↔Hostの疎結合化、typed-ABIのスイッチング足場、b1（真偽）内部表現の限定導入、CFG/PHIのDOT可視化、統合JIT統計を実装した。結果として、分岐・PHI・F64演算のJIT経路が稼働し、VMと一致するゴールデンを維持しつつ、JITがVM比1.06〜1.40倍の改善を示した（初期段階、compileコスト込み）。本手法はAI支援開発下でも「力づく最適化」を避け、可視・可逆・切替可能な足場を短時間で積み上げる有効戦略である。

Abstract (EN short)
We present a Box-First approach to land a practical JIT in the Nyash language within 24 hours. By encapsulating configuration, boundaries, handles, and observability as Boxes, the design achieves strong reversibility and incremental optimization. We implemented a JitConfigBox with runtime capability probing, a decoupled JIT/VM boundary (JitValue/HandleRegistry), a typed-ABI switching scaffold, a conservative b1-internal path, CFG/PHI DOT visualization, and unified JIT statistics. Early results show correct execution on branches/PHI/f64 arithmetic and 1.06–1.40× VM speedups (including compile costs). The method avoids brute-force optimization in AI-assisted development by building visible, reversible, and switchable scaffolding first.

