# MIR JSON Validator — Harness‑First Fail‑Fast

目的
- ハーネス経路を第一としつつ、MIR→JSON 変換時に必須フィールド欠落を早期検出し、Python 側での曖昧な例外を防ぐ。

概要
- Rust 側で JSON ルート（v0/v1 どちらも）を走査し、命令ごとに必須キーの存在と型を確認する。
- 代表チェック（Phase‑B 時点）
  - `unop`: `kind`, `src`, `dst`
  - `binop`: `operation`, `lhs`, `rhs`, `dst`
  - `compare`: `operation`, `lhs`, `rhs`, `dst`
  - `externcall`: `name` または v1 の `callee`
  - `typeop`: `operation`, `src`, `dst`, `target_type`
  - `newbox`: `type`, `args`, `dst`
  - `boxcall`: `box`, `method`, `args`
  - `call`: `func`, `args`, `dst`（null 許容）
  - `branch`: `cond`, `then`, `else`
  - `jump`: `target`
  - `ret`: `value`（null/省略許容）
  - `copy`: `dst`, `src`
- 失敗時は `Err("MIR JSON validation failed: …")` を返して CLI が即時停止（Fail‑Fast）。

設置
- 実装: `src/runner/mir_json_validate.rs`
- 呼び出し: `src/runner/mir_json_emit.rs`（v0/v1 のルート生成後、書き込み前に必ず実行）

方針
- 既定で ON。ENV による OFF 切替は導入しない（仕様固定）。
- スコープは最小限の必須キーに限定し、拡張は段階的に行う。
  - デバッグ用途として `NYASH_MIR_JSON_SKIP_VALIDATOR=1` を指定すると検証を一時的にスキップできる（CI/本番では利用禁止）。

今後
- 不整合の詳細ログ（関数名/ブロック/インデックス）の整備（現状でも出力済み）
- `safepoint` / `load` / `store` 等への検証拡張（登場時に段階追加）
