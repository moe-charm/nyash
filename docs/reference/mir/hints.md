# MIR Hints — Zero‑Cost Structural Guidance

目的
- 構造を変えずに最適な IR を導くための“軽量ヒント”集合。Release では完全に剥離（ゼロコスト）。

原則
- ヒントは意味論を持たない（最適化・検証の補助のみ）。
- 生成器はヒントなしでも正しい MIR/IR を出す。ヒントは安定化・検証・最適化誘導のために用いる。

ヒント一覧（MVP 案）
- hint.scope_enter(id), hint.scope_leave(id)
  - スコープ境界を指示（cleanup 合流の挿入点検討に使用）。
- hint.defer(call-list)
  - defer 呼出し列の静的展開に用いる（例外未導入の間は分岐/return/loop-exit 経路へ複製）。
- hint.join_result(var)
  - If/Match 式の合流結果（join 変数）を明示。空 PHI 抑止とブロック先頭 PHI を誘導。
- hint.loop_carrier(vars…)
  - ループヘッダで同一グループ PHI へ揃える対象変数集合（LoopForm と整合）。
- hint.loop_header, hint.loop_latch
  - 自然ループの境界指示（コードレイアウト/最適化の補助）。
- hint.no_empty_phi（検証）
  - 空 PHI 禁止の検証を有効化（開発/CI向け）。

パイプラインでの扱い
1) Macro: If/Match 正規化・Scope 属性付与・LoopForm（while/for/foreach）整形後に、
2) Lowering: 上記ヒントを埋める（構造は不変）。
3) Verify: 空 PHI 不在・PHI は合流先頭・ループヘッダの PHI 整列などを確認。
4) Strip: Release ではヒントを完全剥離（IRには一切痕跡なし）。

注意
- 既存の機能（マクロ・正規化）で構造を整えた上で使う。ヒントのみでは誤構造は正せない。
- CI の軽量ゲートでは `hint.no_empty_phi` 相当のスモークで IR 健全性を監視する。

