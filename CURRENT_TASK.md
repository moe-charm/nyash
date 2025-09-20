# Current Task — Freeze Polish (Concise)

Updated: 2025‑09‑21

## Compressed Snapshot (Short)
- Strings (UTF‑8/CP vs Byte): baseline done
  - [x] PyVM CP smokes (length/indexOf/lastIndexOf/substring)
  - [x] ASCII Byte smoke
  - [x] Rust CP gate (`NYASH_STR_CP=1`) for length/indexOf/lastIndexOf
  - [x] Docs: blueprint updated with CP gate
- Mini‑VM BinOp(+): stabilization in progress（safe pathのみで緑化へ）
  - [x] Removed global digit-sum fallbacks（ハング源を除去）
  - [x] Added typed/token/value-pair probes（仕様不変）
  - [x] Expression‑bounded extractor（Print.expression の `{…}` で value×2 決定的抽出）
  - [x] Main.fast‑path: BinaryOp('+') を早期に 2 値抽出→加算→即 return
  - [x] PyVM: `__me__` ディスパッチ（同一Box内メソッド呼びの安全化）
  - [x] PyVM: `String.substring` の None 引数を安全化（None→既定値）
  - [ ] Mini‑VM 内の me 呼びを完全撤去（関数呼びに統一）/ substring 前の index ガード徹底
  - [ ] 代表スモーク（int+int=46）を緑化（print_prints_in_slice の無限ループ回避を含む）
- CI: keep min-gate light (MacroCtx/selfhost-preexpand/UTF‑8/ScopeBox) — all green

This page is trimmed to reflect the active work only. The previous long form has been archived at `CURRENT_TASK_restored.md`.

Principles (freeze)
- Self‑hosting first. Macro normalization pre‑MIR; PyVM semantics are authoritative.
- New features are paused; allow only bug fixes, docs, smokes/goldens, CI polish.
- Keep changes minimal/local; no spec changes unless to fix critical issues.

### Delta (since last update)
- Docs
  - Added strings blueprint: `docs/blueprints/strings-utf8-byte.md`
  - Refreshed docs index with clear "Start here" links (blueprints/strings, EBNF, strings reference)
  - Clarified operator/loop sugar policy in `guides/language-core-and-sugar.md` ("!" adopted, do‑while not adopted)
- CI/Smokes
  - Added UTF‑8 CP smoke (PyVM): `tools/test/smoke/strings/utf8_cp_smoke.sh` using `apps/tests/strings/utf8_cp_demo.nyash` (green)
  - Wired into min‑gate CI alongside MacroCtx smoke (green)
- Runtime (Rust)
  - StringBox.length: CP/Byte gate via env `NYASH_STR_CP=1` (default remains byte length; freeze‑safe)
  - StringBox.indexOf/lastIndexOf: CP gate via env `NYASH_STR_CP=1`（既定はByte index; PyVMはCP挙動）

Notes / Risks
- 現在の赤は 2 系統の複合が原因：
  1) Nyash 側の `||` 連鎖短絡による digit 判定崩れ（→ if チェーン化で解消）
  2) 同一 Box 内の `me.*` 呼びが PyVM で未解決（→ `__me__` ディスパッチ導入）。
  付随して、`substring(None, …)` 例外や `print_prints_in_slice` のステップ超過が発生。
  ここを「me 撤去＋index ガード＋ループ番兵」で仕上げる。

### Design Decision（Strings & Delegation）
- Keep `StringBox` as the canonical text type (UTF‑8 / Code Point semantics). Do NOT introduce a separate "AnsiStringBox".
- Separate bytes explicitly via `ByteCursorBox`/buffer types (byte semantics only).
- Unify operations through a cursor/strategy interface (length/indexOf/lastIndexOf/substring). `StringBox` delegates to `Utf8Cursor` internally; byte paths use `ByteCursor` explicitly.
- Gate for transition (Rust only): `NYASH_STR_CP=1` enables CP semantics where legacy byte behavior exists.

## Implementation Order（1–2 days）

1) Strings CP/Byte baseline（foundation）
- [x] CP smoke（PyVM）: length/indexOf/lastIndexOf/substring（apps/tests/strings/utf8_cp_demo.nyash）
- [x] ASCII byte smoke（PyVM）: length/indexOf/substring（apps/tests/strings/byte_ascii_demo.nyash）
- [x] Rust CP gate: length/indexOf/lastIndexOf → `NYASH_STR_CP=1` でCP（既定はByte）
 - [x] Docs note: CP gate env (`NYASH_STR_CP=1`) を strings blueprint に明記

2) Mini‑VM BinOp(+）安定化（Stage‑B 内部置換）
- [x] 広域フォールバック（全数値合算）を削除（安全化）
- [x] typed/token/value-pair の局所探索を導入（非破壊）
- [x] 式境界（Print.expression）の `{…}` 内で `value`×2 を確実抽出→加算（決定化）
- [x] Main.fast‑path を追加（先頭で確定→即 return）
- [x] PyVM：`__me__` ディスパッチ追加／`substring` None ガード
- [ ] Mini‑VM：me 呼びの全面撤去（関数呼びへ統一）
- [ ] `substring` 呼び前の index>=0 ガード徹底・`print_prints_in_slice` に番兵を追加
- [ ] 代表スモーク（int+int=46）を緑化

3) JSON ローダ分離（導線のみ・互換維持）
- [ ] MiniJsonLoader（薄ラッパ）経由に集約→後で `apps/libs/json_cur.nyash` に差し替え可能に
- [ ] スモークは現状維持（既存の stdin/argv 供給で緑）

4) Nyash 箱の委譲（内部・段階導入）
- [ ] StringBox 公開API → Utf8CursorBox（length/indexOf/substring）へ委譲（互換を保つ）
- [ ] Byte 系は ByteCursorBox を明示利用（混線防止）

5) CI/Docs polish（軽量維持）
- [x] README/blueprint に CP gate を追記
- [ ] Min-gate は軽量を維持（MacroCtx/前展開/UTF‑8/Scope系のみ）

## Mini‑VM（Stage‑B 進行中）

目的: Nyash で書かれた極小VM（Mini‑VM）を段階育成し、PyVM 依存を徐々に薄める。まずは “読む→解釈→出力” の最小パイプを安定化。

現状（達成）
- stdin ローダ（ゲート）: `NYASH_MINIVM_READ_STDIN=1`
- Print(Literal/FunctionCall)、BinaryOp("+") の最小実装とスモーク
- Compare 6種（<, <=, >, >=, ==, !=）と int+int の厳密抽出
- If（リテラル条件）片側分岐の走査

 次にやる（順）
0) BinOp(+）: me 撤去＋index ガード徹底＋print_prints 番兵（ハング防止）
1) JSON ローダの分離（`apps/libs/json_cur.nyash` 採用準備）
2) if/loop の代表スモークを 1–2 本追加（PyVM と出力一致）
3) Mini‑VM を関数形式へ移行（トップレベル MVP から段階置換）

受け入れ（Stage‑B）
- stdin/argv から供給した JSON 入力で Print/分岐が正しく動作（スモーク緑）

## UTF‑8 計画（UTF‑8 First, Bytes Separate）

目的: String の公開 API を UTF‑8 カーソルへ委譲し、文字列処理の一貫性と可観測性を確保（性能最適化は後続）。

現状
- Docs: `docs/reference/language/strings.md`
- MVP Box: `apps/libs/utf8_cursor.nyash`, `apps/libs/byte_cursor.nyash`

段階導入（内部置換のみ）
1) StringBox の公開 API を段階的に `Utf8CursorBox` 委譲（`length/indexOf/substring`）
2) Mini‑VM/macro 内の簡易走査を `Utf8CursorBox`/`ByteCursorBox` に置換（機能同値、内部のみ）
3) Docs/スモークの更新（出力は不変；必要時のみ観測ログを追加）

Nyash スクリプトの基本ボックス（標準 libs）
- 既存: `json_cur.nyash`, `string_ext.nyash`, `array_ext.nyash`, `string_builder.nyash`, `test_assert.nyash`, `utf8_cursor.nyash`, `byte_cursor.nyash`
- 追加候補（凍結順守: libs 配下・任意採用・互換保持）
  - MapExtBox（keys/values/entries）
  - PathBox mini（dirname/join の最小）
  - PrintfExt（`StringBuilderBox` 補助）

## CI/Gates — Green vs Pending

Always‑on（期待値: 緑）
- rust‑check: `cargo check --all-targets`
- pyvm‑smoke: `tools/pyvm_stage2_smoke.sh`
- macro‑golden: identity/strings/array/map/loopform（key‑order insensitive）
- macro‑smokes‑lite:
  - match guard（literal OR / type minimal）
  - MIR hints（Scope/Join/Loop）
  - ScopeBox（no‑op）
  - MacroCtx ctx JSON（追加済み）
- selfhost‑preexpand‑smoke: upper_string（auto engage 確認）

Pending / Skipped（未導入・任意）
- Match guard: 追加ゴールデン（type最小形）
- LoopForm: break/continue 降下の観測スモーク
- Mini‑VM Stage‑B: JSON ローダ分離 + if/loop 代表スモーク
- Mini‑VM Stage‑B: BinOp(+）緑化（me 撤去＋index ガード＋番兵）
- UTF‑8 委譲: StringBox→Utf8CursorBox の段階置換（内部のみ；ゲート）
- UTF‑8 CP gate (Rust): indexOf/lastIndexOf env‑gated CP semantics（既定OFF）
- LLVM 重テスト: 手動/任意ジョブのみ（常時スキップ）

## 80/20 Plan（小粒で高効果）

Checklist（更新済み）
- [x] Self‑host 前展開の固定スモーク 1 本（upper_string）
- [x] MacroCtx ctx JSON スモーク 1 本（CI 組み込み）
- [ ] Match 正規化: 追加テストは当面維持（必要時にのみ追加）
- [x] プロファイル運用ガイド追記（`--profile dev|lite`）
- [ ] LLVM 重テストは常時スキップ（手動/任意ジョブのみ）
- [ ] 警告掃除は次回リファクタで一括（今回は非破壊）

Acceptance
- 上記 2 本（pre‑expand/MacroCtx）常時緑、既存 smokes/goldens 緑
- README/ガイドにプロファイル説明が反映済み
 - UTF‑8 CP smoke is green under PyVM; Rust CP gate remains opt‑in

## Self‑Hosting — Stage A（要約）

Scope（最小）
- JSON v0 ローダ（ミニセット）/ Mini‑VM（最小命令）/ スモーク 2 本（print / if）

Progress
- [x] Mini‑VM MVP（print literal / if branch）
- [x] PyVM P1: `String.indexOf` 追加
- [x] Entry 統一（`Main.main`）/ ネスト関数リフト

Next（クリーン経路）
- [ ] Mini‑VM: 関数形式＋簡易 JSON ローダへ段階移行
- [ ] Docs: invariants/constraints/testing‑matrix へ反映追加

### Guardrails（active）
- 参照実行: PyVM が常時緑、マクロ正規化は pre‑MIR で一度だけ
- 前展開: `NYASH_MACRO_SELFHOST_PRE_EXPAND=auto`（dev/CI）
- テスト: VM/goldens は軽量維持、IR は任意ジョブ

## Post‑Freeze Backlog（Docs only）
- Language: Scope reuse blocks（design） — docs/proposals/scope-reuse.md
- Language: Flow blocks & `->` piping（design） — docs/design/flow-blocks.md
- Guards: Range/CharClass sugar（reference） — docs/reference/language/match-guards.md
- Strings: `toDigitOrNull` / `toIntOrNull`（design note） — docs/reference/language/strings.md

## Nyash VM めど後 — 機能追加リンク（備忘）
- スコープ再利用ブロック（MVP 提案）: docs/proposals/scope-reuse.md
- 矢印フロー × 匿名ブロック（設計草案）: docs/design/flow-blocks.md
- Match Guard の Range/CharClass（参照・設計）: docs/reference/language/match-guards.md
- String 便利関数（toDigit/Int; 設計）: docs/reference/language/strings.md

Trigger: nyash_vm の安定（主要スモーク緑・自己ホスト経路が日常運用）。達成後に検討→MVP 実装へ。
