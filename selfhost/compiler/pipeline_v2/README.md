Pipeline V2 — Box‑First Extract→Emit

Scope
- Selfhost compilerの emit‑only 経路（Stage‑1 JSON → MIR(JSON v0/v1)）を、箱の責務で明確化する。
- Parser/Resolver/Runtime には影響しない。既定挙動は不変（devフラグ/引数でのみ起動）。

Modules（責務）
- compare_extract_box.hako
  - 目的: Stage‑1 JSON から Compare(lhs, rhs, op) を堅牢に抽出（整数のみ）。
  - API:
    - extract_return_compare_ints(ast_json) -> {cmp,lhs,rhs} | null
    - extract_if_compare_ints(ast_json) -> {cmp,lhs,rhs} | null
  - 失敗時: null（呼び出し側でフォールバック）。

- emit_compare_box.hako
  - 目的: Compare の MIR(JSON v0) 生成。
  - API:
    - emit_compare_ret(lhs,rhs,cmp,trace) -> JSON v0（compare→ret）
    - emit_compare_cfg3(lhs,rhs,cmp,materialize,trace) -> JSON v0（branch/jump/ret; materialize=2でcopy材化想定）
  - 失敗時: なし（入力は抽出済み前提）。

- pipeline.hako（flow PipelineV2）
  - 役割: Extract系→Emit系の配線。Call/Method/New/Compare/If(Compare) を段階的に対応。
  - フラグ:
    - prefer_cfg=0: Return‑only（compare→ret）
    - prefer_cfg=1: CFG（branch/jump/ret）
    - prefer_cfg=2: CFG+材化（将来の copy after PHI を想定; 現状は等価分岐）
  - trace: 1で最小トレース（[trace]）を出力。既定は0（静音）。

I/O（最小仕様）
- 入力: Stage‑1 JSON（Return/If/Call/Method/New の最小形）。負数/空白は RegexFlow で吸収。
- 出力: MIR(JSON v0)。将来 v1(MirCall) への直出力は lower_stage1_to_mir_v1 を併設（dev用途）。

Fail‑Fast & Fallback
- 抽出箱は見つからない場合 null を返す。pipeline は legacy extractor（Stage1ExtractFlow）でフォールバックする。
- 既定ONは変えない（dev引数でのみ有効）。

Testing
- quick/selfhost に compare/binop/call/method/new の代表スモークがある。Compare系は Return‑only と CFG をそれぞれ確認。
- Mini‑VM（apps/selfhost/vm/boxes/mir_vm_min.nyash）は最小仕様。算術/比較/CFGのみのスモークで品質を担保。

Notes
- 追加の Extract 箱（Call/Method/New）を段階導入し、Stage1ExtractFlow の責務を縮小する計画。
- trace は既定OFF。--emit-trace 指定時のみ出力する。CI/quick は既定で静音。

WASM 開発ラインとの取り込み方針（注意）
- wasm-development ブランチは独立開発ライン。Selfhost 側には以下のみ注意して取り込む。
  - 共有仕様（MIR JSON 形状、PHI invariants、v1/v0 変換ポリシ）に関するドキュメントの同期。
  - Python LLVM ハーネスの仕様更新点（値配線/PHI 正規化）を docs に反映し、実装取り込みは最小・可逆。
  - Selfhost（本フォルダ）の箱 API/入出力は変更せず、adapter で吸収（MirJsonV1Adapter など）。
  - 実装取り込みは小粒・局所・既定OFFのフラグ配下。quick が緑のままになる粒度で実施。
  - 互換チェックは quick/integration の代表スモークで行い、重い検証は wasm ライン側で継続。
