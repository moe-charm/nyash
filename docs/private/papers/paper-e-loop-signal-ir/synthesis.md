# LoopSignal IR - 外部レビュー統合サマリ

本ドキュメントは `claude_output.md` と `gemini_output.md` の示唆を統合した要点メモである。本文（main-paper-jp.md）の第8節に反映済み。

- Signal 型: `LoopSignal<T> = Next(T) | Break(T) | Yield(T) | Return(T)`（命名は Next を採用）。タグは `i8`、ペイロードはSSA値。
- Return の位置づけ: Signal に含める「統一モデル」を採用。IR直交性/最適化/インライン容易性を優先。
- LoopForm 命令: `loop.begin / loop.iter / loop.branch / loop.end` の4命令。`loop.begin` をPHI配置の標準合流点とする。
- LLVM/ABI 表現: まずは `{ i8 tag, payload }` の素直なstruct。ホットパス（Next）は既存最適化でゼロコスト相当に収束する想定。必要に応じて将来特殊化。
- 逆Lowering: LoopForm → 従来MIR の逆変換を常備し、フラグでON/OFF（段階導入を安全化）。
- 最適化パス: 1) Loop1完全インライン化 2) Yieldなし状態省略 3) 分岐合流点正規化（dispatch集中）。
- 適用の段階化: while/for/scope から開始し、関数/if/generator/async に拡大。例外・effects は将来拡張。
- 文書方針: LoopForm は「中間正規形」であり、最終的に Core‑13 に再Lowering可能であることを明記。

実装ロードマップ（最小到達順）
1) LoopSignal型/loop.*ノード追加（オプトイン）
2) while/for/scope のLowering移行
3) 逆Lowering完成（デフォルトON運用）
4) 最適化3本（Loop1 inline / Yieldなし省略 / dispatch統合）
5) 評価（表現/コンパイラ時間/性能）
6) 適用拡大（関数/if、generator/async）
