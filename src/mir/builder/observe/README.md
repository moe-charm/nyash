# observe — Builder 観測（dev‑only/既定OFF）

目的
- Builder 内部の決定（resolve.try/choose, ssa.phi など）を JSONL で観測する。
- 環境変数で明示有効化された時のみ動作（既定OFF）。言語仕様・実行結果は不変。

責務
- ssa.rs: `emit_phi` — PHI の決定（pred の type/origin、dst の決定）を DebugHub へ emit。
- resolve.rs: `emit_try` / `emit_choose` — メソッド解決の候補/最終選択を emit。

非責務（禁止）
- MIR 命令の生成/変更は行わない（副作用なし）。
- 起源付与や型推論は origin 層に限定。

トグル（DebugHub 側でガード）
- `NYASH_DEBUG_ENABLE=1`
- `NYASH_DEBUG_KINDS=resolve,ssa`
- `NYASH_DEBUG_SINK=/path/to/file.jsonl`

レイヤールール
- Allowed: DebugHub emit、Builder の読み取り（関数名/region_id/メタ）。
- Forbidden: rewrite/origin の機能をここに持ち込まない。

