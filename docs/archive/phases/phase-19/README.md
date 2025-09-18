# Phase 19: LoopForm ⇄ Core‑13 PoC と検証・ゼロコストCI

目的（Why）
- Phase‑18 の設計を具体化し、限定範囲で LoopForm ⇄ Core‑13 の往復を成立させる。
- 可逆性と“ゼロコスト退化”の検証を自動化（CI）し、設計の健全性を継続的に保証する。

やること（PoC）
- 変換: `lower_loop(ast) -> LoopModule`（限定サブセット）
- 実行: `interp-loop`（限定サブセット）
- ラウンドトリップ: `LoopForm -> Core‑13` と `Core‑13 -> LoopForm`（退化ScopeLoopは等価）

検証と禁則（CI）
- 可逆性テスト: 値・分岐・fini順序・効果が一致。
- ゼロコスト条件: 退化ScopeLoopの Core‑13 で “余計な `ExternCall/alloc` がゼロ”。
  - 手順: IR差分/統計で call/alloc 新規発生が無いことをチェック。
  - 将来: AOT/ASM差分の軽量比較（変換に伴う命令増なし）。

LoopBox（最小・非既定）
- 目的: 第一級ループとして外部へ渡す/中断再開/反射トレース時のみ具現化。
- API: `birth/next/signal/fini` + `TypeOp is_signal_kind` + unwrap_*。
- 既定: feature off（本体は退化ScopeLoopで十分）。

段階的導入
- M1: 退化ScopeLoopの往復可逆性テスト + ゼロコストCI（IR差分）
- M2: LoopBox最小実装 + verify（birth→(next)*→fini 一回性）
- M3: NDJSON拡張（trace/ping/subscribe）と観測ポイント

アウトオブスコープ（Phase‑19）
- 最適化・高度なスケジューリング（Phase‑20+）

備考
- 例外は Result＋分岐で統一（引き続き）
- ASTは単一・IR分岐のまま維持

