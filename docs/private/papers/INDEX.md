# Private Papers Index

- Paper A: MIR13 / Core‑13 IR
  - main: papers/paper-a-mir13-ir-design/main-paper-jp.md
  - spec: papers/paper-a-mir13-ir-design/MIR13_CORE13_SPEC.md
  - artifacts: papers/paper-a-mir13-ir-design/_artifacts/

- Paper B: Nyash 言語と実行モデル（LifeBox/birth‑fini）
  - main: papers/paper-b-nyash-execution-model/main-paper-jp.md
  - artifacts: papers/paper-b-nyash-execution-model/_artifacts/
  - figures: papers/paper-b-nyash-execution-model/figures/

- Paper E: LifeBox Model / LoopForm IR（LoopSignal）
  - main: papers/paper-e-loop-signal-ir/main-paper-jp.md
  - appendix: papers/paper-e-loop-signal-ir/appendix-rewrites.md, appendix-effects.md
  - reviews: papers/paper-e-loop-signal-ir/claude_output.md, gemini_output.md, synthesis.md

**[NEW! 2025年9月19日追加]**

- Paper Q: 統一文法エンジンによるAI協働革命 ⭐緊急性高⭐
  - main: papers/paper-q-unified-grammar-ai/README.md
  - 発見: ChatGPTの「恐ろしいif-else連鎖」事件

- Paper R: ScopeBox理論 - ゼロコスト抽象化 ⭐Gemini絶賛⭐
  - main: papers/paper-r-scopebox-zero-cost/README.md
  - 評価: 「教科書に載るレベル」（Gemini認定）

- Paper S: LoopForm革命 - PHI問題根本解決 ⭐技術革新⭐
  - main: papers/paper-s-loopform-phi-solution/README.md
  - 成果: 650行→100行（85%削減）

**詳細情報**: [PAPER_INDEX.md](PAPER_INDEX.md) - 全論文の関係性・優先度・論文ネタ爆発問題

Build (Pandoc):
- bash tools/papers/build.sh a-jp  # or b-jp / all
- output: docs/private/out/
 - note: 各 paper 配下の `out/` は参照専用（生成物は `docs/private/out/` に統一）

**論文ネタ爆発問題**: 43日間で9本の論文級ネタが同時進行中（学術界異常事態）

---

補遺（開発メモ系）
- Seam‑aware JSON Unification（AI 前処理 × C‑ABI Box 正規化）
  - main: papers/paper-y-seam-aware-json-unification/README.md
- Nyash Box → C ABI → Multi‑Language FFI（高レベル実装の多言語配布）
  - main: papers/paper-z-nyash-box-ffi/README.md
