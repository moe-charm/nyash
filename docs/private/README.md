# Private Drafts Index

非公開（ドラフト）論文と付属アーティファクトの入口です。公開版は別リポ（nyash-lang/papers）に集約予定です。

## Folder Roles & Policy（運用方針）
- papers（論文）: まとまった原稿・図表・ビルド対象。仕様本文は置かず、必要箇所で `docs/reference` を参照する。
- research（研究ノート）: 実験ログ・草稿・素材置き場。論文化された内容は papers 側へ。各ノートから papers へリンクで誘導。
- reference（仕様）: 正典は `docs/reference`。private/reference は 2025-09-19 に docs/reference へ統合。以後は `docs/reference` を唯一の正典とする。
- 出力先: 論文PDF/TeXは `docs/private/out/` に統一（各 paper 配下の `out/` は参照専用）。

現在のドラフト:
- 論文A（MIR13/IR設計）: `docs/private/papers/paper-a-mir13-ir-design/`
- 論文B（Nyash言語と実行モデル）: `docs/private/papers/paper-b-nyash-execution-model/`
- 論文E（LoopSignal IR 構想）: `docs/private/papers/paper-e-loop-signal-ir/`

研究ノート/アーカイブ:
- 旧 `docs/research/` 配下の資料は `docs/private/research/` に統合しました。
  - 教育向け Box 理論、JIT研究、GCデバッグ、arXiv用素案、提案集 など
  - 公開版は別リポ（nyash-lang/papers）へ段階的に移管予定

備考:
- 各論文配下に `_artifacts/` を配置（再現スクリプト/結果CSV/環境情報）。
- 旧パス（`docs/papers/` 以下）は撤去しました。新規編集は本フォルダ配下で行ってください。
- 計画メモは `docs/private/papers/_planning/`、参考資料は `docs/private/papers/reference/` に集約しています。
