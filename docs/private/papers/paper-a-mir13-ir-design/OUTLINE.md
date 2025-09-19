# 論文A アウトライン（MIR14/最小IR設計）

## 0. タイトル候補
- From Interpreter to Native Apps: A Universal Execution with 14 Core Instructions
- Nyash MIR14: A Minimal but Practical IR for Unified Backends

## 1. 問題設定 / 貢献
- 問題: 実用言語で Interpreter/VM/JIT/AOT/GUI を一貫する最小IR設計は難しい。
- 核となる貢献:
  - Core‑13 + UnaryOp による MIR14 命令セットの設計と根拠。
  - BoxCall を軸とした一貫した呼出しモデルと ABI 境界の単純化。
  - PyVM/llvmlite/JIT/Interpreter 間のパリティ維持手法（観測・スモーク・ハーネス）。
  - LLVM Harness での PHI 不変条件（ブロック先頭集約、型付き incoming）の確立。
  - 実アプリ（GUI含む）を動かす実用性の実証。

## 2. 背景と設計原理
- Everything is Box / BoxCall 統一の哲学。
- 命令縮約の歴史: 27 → 13 → 14（UnaryOp の復活理由）。
- MIR 命令の役割と境界（const/binop/compare/branch/jump/ret/phi/call 系）。
- プラグイン/Box ABI（TypeBox, BID‑FFI）の要点。

## 3. 実装概要
- 実行系レイヤ: Interpreter, VM(PyVM), JIT(Cranelift), LLVM(AOT)。
- Lowering/Builder と Harness（PHI 配線、短絡表現、観測トレース）。
- 仕様差の扱い: llvmlite を規範とし、PyVM を意味論リファレンスとする方針。

## 4. 評価設計
- パリティ検証: tools/parity.sh による PyVM ↔ llvmlite の一致率。
- 性能比較: Interpreter/VM/JIT/AOT の速度・起動時間・メモリ。
- 実アプリ評価: GUI 応答性（<16ms）、代表スモークの通過。
- 安定性: PHI 不変条件違反の検出率、短絡（&&/||）の副作用検証。

## 5. 関連研究
- 汎用 IR（LLVM/MLIR）とミニマル IR の位置付け。
- Truffle/Graal など多層実行系との比較。

## 6. 結果と考察
- 14 命令の妥当性（拡張を要さない範囲、限界）。
- BoxCall を中心に据えた設計の利点/欠点。
- 将来: LoopForm（MIR17）への道筋。

## 7. まとめ
- 最小 IR で多様なバックエンドを統一できる設計指針を提示。

## 付録候補
- 命令セット一覧（参照: ../../../../reference/mir/INSTRUCTION_SET.md）。
- Harness の PHI 仕様（不変条件と実装ノート）。

---

## 執筆メモ（短期タスク）
- MIR13/14 呼称の整合を本文で明示（歴史 → 現在は14）。
- 図表: 命令縮約の年表 / BoxCall 呼出し経路 / PHI 配線模式図。
- スモーク: tools/llvm_smoke.sh, tools/pyvm_stage2_smoke.sh の代表結果を引用。
