# MIR JSON Validator — Harness‑First Fail‑Fast

目的
- ハーネス経路を第一としつつ、MIR→JSON 変換時に必須フィールド欠落を早期検出し、Python 側での曖昧な例外を防ぐ。

概要
- Rust 側で JSON ルート（v0/v1 どちらも）を走査し、命令ごとに必須キーの存在と型を確認する。
- 代表チェック（MVP）
  - `unop`: `kind`, `src`, `dst`
  - `binop`: `operation`, `lhs`, `rhs`, `dst`
  - `compare`: `operation`, `lhs`, `rhs`, `dst`
  - `externcall`: `name` または v1 の `callee`
- 失敗時は `Err("MIR JSON validation failed: …")` を返して CLI が即時停止（Fail‑Fast）。

設置
- 実装: `src/runner/mir_json_validate.rs`
- 呼び出し: `src/runner/mir_json_emit.rs`（v0/v1 のルート生成後、書き込み前に必ず実行）

方針
- 既定で ON。ENV による OFF 切替は導入しない（仕様固定）。
- スコープは最小限の必須キーに限定し、拡張は段階的に行う。
  - デバッグ用途として `NYASH_MIR_JSON_SKIP_VALIDATOR=1` を指定すると検証を一時的にスキップできる（CI/本番では利用禁止）。

今後
- 追加命令（`typeop`, `newbox`, `boxcall`）への拡張
- 不整合の詳細ログ（関数名/ブロック/インデックス）の整備（現状でも出力済み）
