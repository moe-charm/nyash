# Phase 18: LifeBox 原則と LoopForm 設計（文書化＋最小インタフェース）

目的（Why）
- 「全てはループ」の統一観に基づき、スコープ/関数/例外/Box を Loop＋Signal で語る設計を文書として固定。
- 実装は最小（インタフェースの骨組み）。Core‑13 の実装は維持しつつ、LoopForm はまだ“思想IR”として扱う。

コア概念（最小）
- LifeBox 原則: Box＝Loopのインスタンス。`birth → (next)* → signal → fini`。通常Boxは退化（next省略）。
- ScopeLoop（ループしないループ）: 反復1回/Next禁止/単一入出口。`end` 直前に LIFO で fini を直列化。
- LoopSignal: `{ Next, Break, Return, Err, (必要なら Yield) }`。例外は値+Signal で運ぶ。

LoopForm（思想IR）
- スコープ: `loop.begin → 本体 → loop.signal(Break|Return) → loop.end`
- 例外/try: 本体が `Result` を返し、`is_err` 分岐→ catch/finally 集約。finally は `end` 直前に一括。
- 状態束ね: 局所変数は state tuple（ループ状態）に束ね、`end` で一括 fini。

Core‑13 への写像（退化）
- ScopeLoop → 既存の `Jump/Branch/Phi`。追加の call/alloc を発生させない（ゼロコスト退化）。
- 例外統一 → `Result` + `TypeOp(is_err)` + `Branch` + `Phi`。`?` は Err→Return の定型展開。
- Box終了 → 合流ブロックで `fini` を LIFO 直列（検証器で一回性/順序確認）。

決定トグル（提案）
- ASTは単一・IRで分岐: はい（Phase‑17に準拠）
- ScopeLoopを正式モデル化: はい（名称は LifeBox に統合。ScopeLoopは表現上の別名）
- 例外は Result＋分岐で統一: はい
- ゼロコスト条件の監視: Phase‑19 でCI導入（ここでは規約と受入れ基準を明記）

受入れ基準（Docs）
- LifeBox/ScopeLoop/LoopSignal の定義が本文にあり、Core‑13 への退化ルールが列挙されている。
- 検証規約（単一入出口/Next禁止/LIFO fini/追加call禁止）が箇条書きで提示されている。
- Q&A に“なぜLoop化するのか／ゼロコスト条件は何か／例外統一の利点”がまとまっている。

アウトオブスコープ（Phase‑18）
- LoopForm の実装（変換/実行）：Phase‑19 以降
- 詳細トレースAPI：Phase‑19 以降

次フェーズへの受け渡し
- Phase‑19: LoopForm ⇄ Core‑13 の往復 PoC、可逆性テストと禁則CI、必要最小の LoopBox 実装（feature off 既定）。

