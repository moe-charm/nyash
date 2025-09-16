# 論文E: LifeBox Model と LoopForm IR（制御の値化と統一）

- タイトル（案）: LifeBox Model and LoopForm IR over a Box‑First Runtime
- 略称: LifeBox Model (LBM) / LoopForm IR（別名: LoopSignal IR）
- ステータス: コンセプト草稿（A/B論文後に着手）

## 要旨（短）
LifeBox Model（LBM）は「Box=Loop1」という見方でライフサイクル（birth/fini）と制御（return/break/yield）を統合する概念である。その上で、制御を値（Signal）として扱う LoopForm IR（別名: LoopSignal IR）を導入し、`loop.begin/iter/branch/end` の標準形で高位表現を正規化する。Nyashの「Everything is Box」と整合し、Loweringの一様化とCFGの定型化（合流点の単純化）を実現する。Loop1 は完全インライン化されゼロコスト化でき、generator/async/effect は Signal 拡張で段階的に導入可能である。

## 位置づけ
- 既存近縁: CPS/継続・代数的効果・コルーチン
- 差分: 制御を「値（Signal）」としてMIRレベルで扱い、Box哲学と結合した実装主導の最小集合。汎用効果機構より小さく、導入・評価が容易。

## 関連
- 論文A（MIR13/IR設計）: 本稿は将来の拡張。まずAを優先して仕上げ、その後に独立短論文としてまとめる。
- 論文B（Nyash言語）: birth/fini・async/generator の設計と橋渡し要素。

## 2025-09-16 追記: MIR進化計画
- **MIR14→MIR13→MIR17の段階的移行**: `mir-evolution-plan.md` に詳細記載
- Codexとの協働でLoopForm設計を具体化
- PHI責務のLLVM層移管とLoopForm追加（+4命令）の戦略

---

- 下書き本文: `main-paper-jp.md`
- MIR進化計画: `mir-evolution-plan.md`（新規追加）
- 補助: 擬似MIRとLowering図、評価計画の雛形を本文内に記載
