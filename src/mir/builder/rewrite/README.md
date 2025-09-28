# rewrite — Known 経路の関数化 ＋ 特殊規則（P1）

目的
- Known 受け手のメソッド呼び出し `obj.m(a)` を関数呼び出し `Class.m(me,obj,a)` に正規化し、実行系を単純化する。
- 表示系の特殊規則（`toString` / `stringify` → 規範 `str`）を一箇所に集約する（互換維持）。
- 仕様は不変。Union は観測のみで、Known のみ関数化対象。

責務
- known.rs: Known 経路の instance→function 正規化（ユーザー Box のみ、既存ガード尊重）。
- special.rs: `toString`/`stringify` → `str` の早期処理（Class.str/0 を優先、互換で stringify/0）。
  - `equals/1` もここに集約（Known 優先 → 一意候補のみ許容）。
- 観測は observe 層に委譲（resolve.choose など）。

非責務（禁止）
- Union の強引な関数化（Unknown/曖昧なものは扱わない）。
- 起源付与/型推論の実施（origin 層に限定）。
- NYABI 呼び出しや VM 直接呼び出し。

API（呼び出し側から）
- `try_known_rewrite(builder, recv, class, method, args) -> Option<Result<ValueId,String>>`
- `try_unique_suffix_rewrite(builder, recv, method, args) -> Option<Result<ValueId,String>>`
- `try_known_or_unique(builder, recv, class_opt, method, args) -> Option<Result<ValueId,String>>`
- `try_early_str_like(builder, recv, class_opt, method, arity) -> Option<Result<ValueId,String>>`
- `try_special_equals(builder, recv, class_opt, method, args) -> Option<Result<ValueId,String>>`

レイヤールール
- Allowed: Builder のメタ参照/関数名生成、MirInstruction の生成（関数化結果）。
- Forbidden: origin/observe のロジックを混在させない（必要時は呼び出しで連携）。

決定原則
- Known のみ関数化（`value_origin_newbox` が根拠）。
- 表示系は規範 `str` を優先、`stringify` は当面互換として許容。
- すべての決定は dev 観測（resolve.try/choose）で可視化し、挙動は不変。
