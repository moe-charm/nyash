# Current Task — Stability Polish (Concise)

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
  - [x] Mini‑VM 内の me 呼びを完全撤去（関数呼びに統一）/ substring 前の index ガード徹底
  - [x] 代表スモーク（int+int=46）を緑化（print_prints_in_slice の無限ループ回避を含む）
- CI: keep min-gate light (MacroCtx/selfhost-preexpand/UTF‑8/ScopeBox) — all green

This page is trimmed to reflect the active work only. The previous long form has been archived at `CURRENT_TASK_restored.md`.

Principles (feature‑pause)
- Self‑hosting first. Macro normalization pre‑MIR; PyVM semantics are authoritative.
- Big feature additions are paused until Nyash VM bootstrap completes. Bug fixes, docs, smokes/goldens, CI polish, robustness (spec‑preserving) continue.
- Keep changes minimal/local; no spec changes unless to fix critical issues, and guard any optional paths behind default‑OFF flags.

### Delta (since last update)
- JSON provider（yyjsonベンダリング完了・切替はランタイム）
  - `plugins/nyash-json-plugin/c/yyjson/{yyjson.h,yyjson.c,LICENSE}` を同梱し、`build.rs + cc` で自己完結ビルド。
  - env `NYASH_JSON_PROVIDER=serde|yyjson`（既定=serde）。nyash.toml の [env] からも設定可能。
  - yyjson 経路で parse/root/get/size/at/str/int/bool を実装（TLVタグは従来準拠）。失敗時は安全に serde にフォールバック。
  - JSON スモーク（parse_ok/err/nested/collect_prints）は serde/yyjson 両経路で緑。専用の yyjson CI ジョブは追加しない（不要）。
- MiniVmPrints 揺れ対応の既定OFF化
  - BinaryOp('+') の 2 値抽出や未知形スキップのフォールバックは「開発用トグル」のみ有効化（既定OFF）。
  - 本線は JSON Box 経路に統一（collect_prints_mixed は JSON ベースのアプリに切替）。
- Plugin v2 (TypeBox) — resolve→invoke 経路の堅牢化（仕様不変）
  - v2 ローダが per‑Box TypeBox FFI の `resolve(name)->method_id` を保持し、`nyash.toml` 未定義メソッドでも動的解決→キャッシュ
  - Unified Host の `resolve_method` も config→TypeBox.resolve の順で解決
  - 影響範囲はローダ/ホストのみ。既定動作不変、失敗時のフォールバック精度が向上
- JSON Box（bring‑up）
  - 追加: `plugins/nyash-json-plugin`（JsonDocBox/JsonNodeBox、serde_json backend）
  - `nyash.toml` に JsonDocBox/JsonNodeBox の methods を登録（birth/parse/root/error, kind/get/size/at/str/int/bool/fini）
  - PyVM 経路: `src/llvm_py/pyvm/ops_box.py` に最小シム（JsonDoc/JsonNode）を追加（parse/root/get/size/at/str/int/bool）
  - Smoke: `tools/test/smoke/selfhost/jsonbox_collect_prints.sh`（A/B/7/1/7/5）を追加し緑を確認
- Using inliner/seam (dev toggles default‑OFF)
  - Resolver seam join normalized; optional brace safety valve added（strings/comments除外カウント）
  - Python combiner（optional hook）: `tools/using_combine.py` を追加（--fix-braces/--seam-debug など）
  - mini_vm_core の末尾ブレースを整合（未閉じ/余剰の解消）
  - MiniVm.collect_prints の未知形スキップを Print オブジェクト全体に拡張（停滞防止）
- Docs
  - Added strings blueprint: `docs/blueprints/strings-utf8-byte.md`
  - Refreshed docs index with clear "Start here" links (blueprints/strings, EBNF, strings reference)
  - Clarified operator/loop sugar policy in `guides/language-core-and-sugar.md` ("!" adopted, do‑while not adopted)
  - Concurrency docs (design-only): box model, semantics, and patterns/checklist added
    - `docs/proposals/concurrency/boxes.md`
    - `docs/reference/concurrency/semantics.md`
    - `docs/guides/box-patterns.md`, `docs/guides/box-design-checklist.md`
- CI/Smokes
  - Added UTF‑8 CP smoke (PyVM): `tools/test/smoke/strings/utf8_cp_smoke.sh` using `apps/tests/strings/utf8_cp_demo.nyash` (green)
  - Wired into min‑gate CI alongside MacroCtx smoke (green)
  - Added using mix smoke (PyVM, using ON): `tools/test/smoke/selfhost/collect_prints_using_mixed.sh` — green
    - Fix: MiniVmBinOp.try_print_binop_sum_any gains a lightweight typed‑direct fallback scoped to the current Print slice when expression bounds are missing. Spec unchanged; only robustness improved.
    - Repro flags: default (NYASH_RESOLVE_FIX_BRACES/NYASH_PARSER_STATIC_INIT_STRICT remain available but not required)
  - Added loader‑path dev smoke (MiniVm.collect_prints focus): `tools/test/smoke/selfhost/collect_prints_loader.sh`（任意/開発用）
  - Added empty‑args using smoke (PyVM, using ON): `tools/test/smoke/selfhost/collect_empty_args_using_smoke.sh` (uses seam brace safety valve; default‑OFF)
- Runtime (Rust)
  - StringBox.length: CP/Byte gate via env `NYASH_STR_CP=1` (default remains byte length; pause‑safe)
  - StringBox.indexOf/lastIndexOf: CP gate via env `NYASH_STR_CP=1`（既定はByte index; PyVMはCP挙動）

Notes / Risks
- PyVM はプラグイン未連携のため、JsonBox は最小シムで対応（仕様不変、既定OFFの機能変更なし）。将来的に PyVM→Host ブリッジを導入する場合はデフォルトOFFで段階導入。
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
  - [x] Mini‑VM：me 呼びの全面撤去（関数呼びへ統一）
  - [x] `substring` 呼び前の index>=0 ガード徹底・`print_prints_in_slice` に番兵を追加
  - [x] 代表スモーク（int+int=46）を緑化

3) JSON ローダ分離（導線のみ・互換維持）
- [x] MiniJsonLoader（薄ラッパ）経由に集約の導線を適用（prints/単一intの抽出に使用）→ 後で `apps/libs/json_cur.nyash` に差し替え可能
- [x] スモークは現状維持（stdin/argv 供給とも緑）
 - [x] loader‑path の検証用スモーク追加（dev 任意）

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
1) JSON ローダの分離（`apps/libs/json_cur.nyash` 採用準備）
2) if/loop の代表スモークを 1–2 本追加（PyVM と出力一致）
 - [x] If（リテラル条件）: T/F の分岐出力（mini_vm_if_literal_branch_smoke）
 - [x] Loop 相当（連続 Print の順序）: a,b,c,1,2 の順序確認（mini_vm_print_sequence_smoke）
3) Mini‑VM を関数形式へ移行（トップレベル MVP から段階置換）
 - [x] Entry を薄く（args→json 決定→core.run へ委譲）
 - [x] core.run の単発 Literal（string/int）対応を内製
 - [x] 純関数 API 追加: MiniVm.collect_prints(json)（最小: literal対応）

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
- 追加候補（機能追加ポーズ遵守: libs 配下・任意採用・互換保持）
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
- Mini‑VM Stage‑B: JSON ローダ分離 + if/loop 代表スモーク（継続的に強化）
- UTF‑8 委譲: StringBox→Utf8CursorBox の段階置換（内部のみ；ゲート）
- UTF‑8 CP gate (Rust): indexOf/lastIndexOf env‑gated CP semantics（既定OFF）
- LLVM 重テスト: 手動/任意ジョブのみ（常時スキップ）

## 80/20 Plan（小粒で高効果）

JSON / Plugin v2（現状に追記）
- [x] v2 resolve→invoke 配線（TypeBox.resolve フォールバック + キャッシュ）
- [x] JsonBox methods を nyash.toml に登録
- [x] PyVM 最小シム（JsonDoc/JsonNode）を追加
- [x] JSON collect_prints スモーク追加（緑）
- [x] yyjson ベンダリング＋ノード操作実装（parse/root/get/size/at/str/int/bool）
- [x] ランタイム切替（env `NYASH_JSON_PROVIDER`）— 既定は serde。yyjson 専用 CI は追加しない。
- [ ] TLV void タグ整合（任意：共通ヘルパへ寄せる）
- [ ] method_id キャッシュの統一化（loader 内で Box単位の LRU/Hash で維持）
- [x] MiniVmPrints フォールバックは開発用トグルのみ（既定OFF）

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

## Repo / Branches (private)
- Active: `selfhost`（作業用）, `main`（既定）
- Clean‑up: 古い開発ブランチ（academic-papers / feat/match-type-pattern / selfhosting-dev）を整理。必要なら再作成可。

## Self‑Hosting — Stage A（要約）

Scope（最小）
- JSON v0 ローダ（ミニセット）/ Mini‑VM（最小命令）/ スモーク 2 本（print / if）

Progress
- [x] Mini‑VM MVP（print literal / if branch）
- [x] PyVM P1: `String.indexOf` 追加
- [x] Entry 統一（`Main.main`）/ ネスト関数リフト

- Next（クリーン経路）
- [x] Mini‑VM: 入口薄化（MiniVm.run 呼び一択）— `apps/selfhost-vm/mini_vm.nyash` を薄いラッパに再編（using 前置に依存）
- [x] Mini‑VM: collect_prints の混在ケース用スモーク追加（echo/itoa/compare/binop）→ `tools/test/smoke/selfhost/collect_prints_mixed.sh`
- [x] Mini‑VM: JSON ローダ呼び出しの段階統一（digits/quoted/whitespace を MiniJson に寄せる・第一弾完了）
- [x] Dev sugar: 行頭 @ = local のプリエクスパンドをランナー前処理に常時組込み（ゼロセマンティクス）
- [x] Mini‑VM ソースの @ 採用（apps/selfhost‑vm 配下の入口/補助を段階 @ 化）
- [x] Runner CLI: clap ArgAction（bool フラグ）を一通り点検・SetTrue 指定（panic 回避）
- [ ] Docs: invariants/constraints/testing‑matrix へ反映追加（前処理: using前置/@正規化）
 - [x] Docs: Using→Loader 統合メモ（短尺）— docs/design/using-loader-integration.md（READMEにリンク済）

### Guardrails（active）
- 参照実行: PyVM が常時緑、マクロ正規化は pre‑MIR で一度だけ
- 前展開: `NYASH_MACRO_SELFHOST_PRE_EXPAND=auto`（dev/CI）
- テスト: VM/goldens は軽量維持、IR は任意ジョブ

## Post‑Bootstrap Backlog（Docs only）
- Language: Scope reuse blocks（design） — docs/proposals/scope-reuse.md
- Language: Flow blocks & `->` piping（design） — docs/design/flow-blocks.md
- Guards: Range/CharClass sugar（reference） — docs/reference/language/match-guards.md
- Strings: `toDigitOrNull` / `toIntOrNull`（design note） — docs/reference/language/strings.md
 - Concurrency: Box model（Routine/Channel/Select/Scope） — docs/proposals/concurrency/boxes.md
 - Concurrency semantics（blocking/close/select/trace） — docs/reference/concurrency/semantics.md

## Nyash VM めど後 — 機能追加リンク（備忘）
- スコープ再利用ブロック（MVP 提案）: docs/proposals/scope-reuse.md
- 矢印フロー × 匿名ブロック（設計草案）: docs/design/flow-blocks.md
- Match Guard の Range/CharClass（参照・設計）: docs/reference/language/match-guards.md
- String 便利関数（toDigit/Int; 設計）: docs/reference/language/strings.md

Trigger: nyash_vm の安定（主要スモーク緑・自己ホスト経路が日常運用）。達成後に検討→MVP 実装へ。

## Next Up (Parity & Using Edge Smokes)

- Parity quick harness（任意）: `tools/smokes/parity_quick.sh` で代表 apps/tests を PyVM vs llvmlite で比較
- Using エッジケース: 相互依存/相対パス混在/alias のスモーク追加（apps/tests + tools/test/smoke/using/edge_cases.sh）
- Using 混在スモークの緑化仕上げ（PyVM）
  - MiniVmPrints.print_prints_in_slice の binop/compare/int リテラル境界（obj_end/p_obj_end）を調整
  - ArrayBox 経路の size/get 依存を避け、直接 print する経路の安定化を優先
  - 再現フラグ（開発のみ）: `NYASH_RESOLVE_FIX_BRACES=1 NYASH_PARSER_STATIC_INIT_STRICT=1`

### Next Up (JSON line)
- TLV void タグの統一（任意）
- method_id 解決キャッシュの整備・計測
- YYJSON backend のスケルトン追加（既定OFF・プラグイン切替可能設計）
- JSON smokes をCI最小ゲートへ追加
