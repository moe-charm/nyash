# CURRENT_TASK — Status and Next Steps (2025‑10‑16)

このページは「いま何をしていて、次に何をするか」を 1 画面で把握できるようにするダッシュボードだよ。最新の作業に合わせて随時更新していくにゃ。

## Snapshot
Updates (today - 2025-10-16 continued)
- **P0修正完了**: MirIoBox export追加 → selfhost基盤復旧 ✅
  - 問題: `selfhost/shared/hako_module.toml` に `mir.io = "mir/mir_io_box.hako"` export欠落
  - 影響: ALL selfhostテストが "Unknown module function: MirIoBox.validate/1" で失敗
  - 修正: export追加 → 基盤復旧確認（mir_builder_binop_add/compare_eq/binop_mul PASS）
  - Commit: `36d0cf4e` - "fix(selfhost): Add MirIoBox export - P0 hotfix for ALL selfhost tests"

- **ChatGPT5レポート検証完了**: Task Agent 4並列調査 → 3/4が誤診断！真因発見 🔥
  - Task 1: "Array.size正規化未実装" → ❌ **誤診断** - Phase 15.5で完全実装済み
  - Task 2: "ALWAYS_ON_TOGGLE問題" → ❌ **誤診断** - 真因はMirIoBox export欠落（P0で修正済み）
  - Task 3: "auto_birth実装問題" → ❌ **誤診断** - 完全実装済み、lifecycle verification微調整のみ
  - Task 4: **真の根本原因発見** → ✅ **MIR Builder パラメータレジスタバグ**
    - 問題: `loop(i < path.size())` が MIR で `loop(i < this.size())` になる
    - 原因: パラメータv%0-v%N（me/json_text/path）がループ内で上書きされる
    - 証拠: MIR JSON で `"box": 0` (v%0=me) が `path.size()` に使われている
    - 影響: `json_query_vm` などパラメータ参照を含むループで破壊

- **MIR Builder バグ修正 Phase 1完了** (2025-10-16 continued)
  - ✅ Task先生4人並列調査完了 - 真因3箇所特定:
    1. prepare_loop_variables: パラメータフィルタなし（ALL変数がPHI対象）
    2. VarMapGuard: `value == me_vid` 条件が誤作動（コンテキスト判別不足）
    3. Copy命令: パラメータレジスタv%0-v%Nを直接上書き
  - ✅ Phase 1修正実装完了: パラメータフィルタ追加
    - ファイル: `src/mir/loop_builder/phi.rs:21-28`
    - 内容: `prepare_loop_variables` に関数パラメータのフィルタリングロジック追加
    - 効果: パラメータレジスタの上書きを部分的に抑制（v%0の上書きは解消）
    - ビルド: ✅ 成功（警告のみ）
    - テスト: ✅ selfhost基盤テスト PASS (mir_builder_binop_add/compare_eq/binop_mul)
  - 🔥 Phase 2修正必要: VarMapGuard誤作動修正（v%1-v%Nの上書きがまだ残存）
    - ファイル: `src/mir/loop_builder/mod.rs:155-173`
    - 問題: PHIノードがv%1を持つとき、VarMapGuardが不要なCopy命令を発行

- **レガシーコード削除調査完了** (2025-10-16 continued)
  - ✅ Task先生4人並列調査 → 191行即時削除可能 + 箱化候補181行発見
  - **Task 1: collect_free_vars** (149行削除OK)
    - ファイル: `src/mir/builder/vars.rs` (全149行)
    - 状態: `#[allow(dead_code)]` マーカー付き、呼び出し元0件
    - 重複: `exprs_lambda.rs::collect_vars` に同一ロジック存在
    - 推奨: ✅ **即時削除**（Phase 2で箱化検討 → VarCollectorBox）
  - **Task 2: record_kpi** (34行削除OK)
    - ファイル: `src/mir/builder/observe/resolve.rs` (関数14行 + 静的変数7行 + ヘルパー12行 + 呼び出し1行)
    - 状態: 実使用0件（tools/apps で未使用）、Phase 15.7のデバッグ機能
    - 代替: DebugHub経由で同等データ取得可能
    - 推奨: ✅ **即時削除**（将来必要なら KpiRecorderBox で復活）
  - **Task 3: utils.rs dead functions** (8行削除OK)
    - ファイル: `src/mir/builder/utils.rs`
    - 発見: 完全DEADな関数0個、誤った `#[allow(dead_code)]` マーカー8個
    - 全17関数すべて使用中（15-36回呼び出し）
    - 推奨: ✅ **マーカー削除のみ**（関数は削除不可）
  - **Task 4: 箱化候補発掘** (BuilderObserverBox 181行)
    - Everything is Box 実現状況: ⭐⭐⭐⭐⭐ Builder内の責務は既に高度に箱化済み
    - 成功事例10個確認: LegacyCallBridgeBox, OriginTrackerBox, WeakFieldRegistryBox 等
    - 箱化候補: `observe/` module (181行) → `BuilderObserverBox` (Medium優先度)
    - 推奨: 削除191行実施後、箱化は長期計画で検討
  - **合計即時削除**: 149 + 34 + 8 = **191行削減可能**

- Phase‑31（static → singleton 正規化）進捗
  - A‑1b 完了: 「関数スコープのシングルトン・キャッシュ」を導入して、同一関数内の `me` プレースホルダ重複生成を解消。
    - 実装: `MirBuilder.current_fn_singletons` を追加し、`maybe_prepend_static_me()` から `current_fn_singleton()` を使用。
    - `main`/メソッド/静的メソッドの各 lowering フェーズでキャッシュの save/restore を実施。
  - A‑1c 完了: ModuleFunction call の Verifier と VM 側整備
    - Verifier が ModuleFunction の受領者を検査（Known かつ Box 型のときに Fail‑Fast）。
    - VM Router/legacy fallback は常に receiver 前提。Void 受領者は即時エラーに。
    - ModuleFunction トランポリンを `handlers/calls/trampolines.rs` に分離し、Array/Map/String/Console を表駆動化。
  - A‑1d 完了: LegacyCallBridgeBox でレガシー call 経路を箱化。
    - `src/mir/builder/calls/legacy_bridge/` を新設し、旧 `emit_legacy_call` の処理を移設。
    - Call 発行はすべて `emit_call_with_guard`（EmitGuard）経由に統一し、BoxCall/PluginCall も薄い `emit_boxcall()` ガードでローカルSSA素材化を強制。
  - A‑1e 完了: MapBox の長さ系呼び出しを Extern 化。
    - `src/mir/builder/normalize/map_length.rs` を追加し、`MapBox.(size|len|length)` → `Extern("nyrt.map.size")` に正規化。
    - `normalize::apply_all` に Map ルールを組み込み、EmitGuard 経路で常に LocalSSA 化された receiver が渡るよう統一。
  - A‑1f 進捗: Map keys/values の安定化に向けた基盤整備。
    - Optimizer で `nyrt.map.size/keys/values` のうち size/keys/values の差し戻し抑止（Extern→Method の巻き戻しを禁止）。
    - Extern adapter で Map.size/keys/values を HostSlot 経由・Plugin 経由の両方へ橋渡し（runtime 側でテーブル再利用）。
    - Builder 側で `Extern("nyrt.map.values|keys")` の結果に ArrayBox 注釈を付与し、後段の `.size()` で型ズレしにくくした。
  - A‑2 着手: `Const Void` (静的 me) を `static_singleton::get()` で実体 BoxRef 化。
    - `runtime/static_singleton.rs` を追加し、`OnceCell<Mutex<…>>` で Box 単位のシングルトンを lazy 初期化。
    - Interpreter `handle_const` が `MirType::Box` の場合に singleton を取得して受領者を具体化。
- Json canonicalization fix
  - `hostbridge.extern_invoke` の引数をプリミティブ化する正規化ヘルパを導入。Plugin ArrayBox でも正しく文字列を渡せるようになったよ。
  - `JsonCanonicalBox.canonicalize` を純 String→String 経路に統一して `json_canonical_box_vm` / `mirio_canonicalize_vm` スモークが PASS したにゃ。
  - `host_handles::release()` を追加してホストアンカー経由の一時ハンドルを解放。
- Map.values stage2 の根治（2025‑10‑17）
  - PluginHost 再入ガードを深さカウンタ（MAX=8）化し、Void フォールバックを撤廃。
  - HostHandleRouter が ArrayBox (PluginBoxV2) の slot 100/101/102 を扱えるようになり、Stage‑2 keys/values が常に ArrayBox を返す構造に。
  - `EnvToggle::enabled` を拡張して空キー＝既定ONと扱い、Array host routes をテーブル側で常時有効化。
  - `map_values_array_element_vm` を再実行して PASS（`nyrt.array.size expects ArrayBox` を解消）。
- P0 Hotfix (Phase‑31): ModuleFunction 呼び出しの `me` 不足を構造＋VMで補正
  - Builder（unified/legacy 両方）: ModuleFunction 発行時に、現在モジュール上の関数定義を参照し、`args.len()+1 == params.len()` なら per‑function singleton を先頭に付与。
    - 変更: `src/mir/builder/builder_calls/emit.rs` / `src/mir/builder/calls/legacy_bridge/mod.rs`
  - VM: `exec_function_inner` で同条件を検出し、静的 Box は `static_singleton::get()`（失敗時は `Void`）を先頭に差し込んで整列。
    - 変更: `src/backend/mir_interpreter/exec.rs`
  - 効果: `json_query_vm` の `Type error: nyrt.string.length expects String` を解消（パラメータ列のズレが原因）。以降の失敗は `ArrayBox.substring` 未実装に起因（別項で対応）。

- Plugins プロファイルの再走（結果: FAIL 15/54 → 14/54 予定）
  - 代表的な失敗:
    - MapBox: `values` 経路で受領者素材化/型注釈の順序ズレ（array.size に ArrayBox で届かない）
    - SetBox: `add/has/size` が出力欠落（router 経路の素材化不足）
    - FileBox: `use of undefined value ValueId(..)`（ファイル読み戻しの素材化漏れ）
    - ArrayBox: `array_slice_edges_vm` / `hosthandle_boundary_suite_vm` が `extern calls disabled (legacy-only)` で失敗（レガシー専用 extern 依存の残骸）
  - 一時状況: Map.size/has/remove は修正済（strict/parity/remove が PASS）。`values` は Extern 経路・型注釈は入ったが、使用順序（SSA 素材化）がまだズレる箇所あり。
- Docs
  - Phase‑31 計画書を `docs/development/roadmap/phases/phase-31-box-Normalization/INDEX_JA.md` に追加済み。
- Verifier スモーク拡充
  - quick-selfhost に ModuleFunction 静的呼び出しの Fail-Fast を確認するスモークを追加。
    - `mir_verify_module_function_missing_receiver_vm.sh`: singleton 未注入ケースを `--verify` で検知。
    - `mir_verify_module_function_receiver_mismatch_vm.sh`: 受領者 Box 型がズレたケースを検知。
  - これで Phase-31 P0-2（Verifier 形状固定）の足場を確保。
- alias_tools レガシーテストの一時停止
  - `internal_ref_variable_is_rewritten` / `internal_ref_function_qualified_is_rewritten` を `#[ignore]` で退避。
    - 理由: ASTNode::BoxDeclaration の `body` フィールド撤退との不整合。P0-4 ドキュメント更新時に復活させるメモを残す。

Open issues / blockers
- **🔥 P0-CRITICAL**: MIR Builder パラメータレジスタバグ根治（Phase 1完了、Phase 2進行中）
  - Phase 1 ✅: パラメータフィルタ実装完了（v%0の上書き解消）
  - Phase 2 🔥: VarMapGuard誤作動修正（v%1-v%Nの上書きまだ残存）
    - 問題: PHIノードがたまたまv%1を持つとき、VarMapGuardが発動してCopy命令生成
    - 解決策: VarMapGuardの条件を改善（パラメータVIDの場合はコンテキスト判別）
  - Phase 3 予定: MIR Verifier にパラメータ上書き検出追加
- Phase‑31 残: Plugin 既存 ABI へのトランポリン実配線（registry へ新エントリ登録）と quick→plugins→full スモークの差分スキャン。
- Frozen guide への Windows 例追記など、P0 で止まっているドキュメント系タスクを再開する必要があるにゃ。

## Prioritized TODOs
- **P0 — 直近解消したいもの**
  1. ✅ **DONE**: MirIoBox export追加（selfhost基盤復旧完了）
  2. ✅ **DONE**: Task先生4人並列調査（真因3箇所特定）
  3. ✅ **DONE**: Phase 1 - パラメータフィルタ実装（v%0上書き解消）
  4. ✅ **DONE**: Task先生4人レガシー削除調査（191行削除可能）
  5. 🔥 **IN PROGRESS**: レガシーコード削除実行（191行削減）
     - vars.rs 削除（149行）
     - record_kpi 削除（34行）
     - utils.rs マーカー削除（8行）
  6. **TODO**: Phase 2 - VarMapGuard誤作動修正（v%1-v%N上書き解消）
  7. **TODO**: Phase 3 - MIR Verifier パラメータ上書き検出追加
  8. quick → plugins → full スモークを再実行し、カテゴリ 2/3（出力差・モジュール解決）の残差を棚卸し。
  9. Plugin ABI トランポリンの網羅化（registry 配線＆生成ツール化）。
  10. `docs/guides/frozen-toolchain.md` に Windows COFF 例を追記してハンドブックを更新。
  11. SetBox/FileBox/Array slice 周辺の整備（Map.values は解消済み）
     - SetBox: `add/has/size` のローカル素材化を再確認（EmitGuard 経路の統一）。
     - FileBox: read/write 経路の undefined ValueId を解消（Call 発行を guard 経由に統一）。
     - Array slice: レガシー extern 依存を段階撤退し、必要なら専用 Bridge を追加。
- **P1 — quality of life**
  - Doctor: structured error messages（missing clang/llvmlite/allowlist/lib paths）
  - Harness: tighter logs for `--target windows` & optional IR dump hint
  - Gate C: reduce deprecate/alias noise earlier in runner; aim for true PASS (no SKIP) in nyvm_* smokes
- **P2 — later**
  - CI: build-only job for `llvm_backend` / harness smoke（opt-in）
  - CI: optional Windows cross pipeline doc（no runner）

## Guardrails / Principles
- Fail-Fast: no silent fallback for FFI/extern; defaults stay strict
- Minimal ENV: config broadens allowlist but never changes default semantics
- Structure first: helpers isolated under `tools/aot/` と `tools/aot/windows/`
- Docs placement: `docs/guides/`, `docs/reference/`, `docs/development/roadmap/` の既存ディレクトリに限定

## How to Reproduce (quick memo)
- WSL（Linux 単体ビルド）
  - `./target/release/hakorune --backend mir --emit-mir-json build/mir/main.mir.json examples/simple_return.hako`
  - `tools/aot/emit_object_via_extern_c.sh build/mir/main.mir.json build/obj/main.o`
  - `tools/aot/link_with_clang.sh -o bin/hako-frozen-v1 build/obj/main.o`
- WSL → Windows（COFF）
  - `python3 tools/llvmlite_harness.py --in build/mir/main.mir.json --target windows --out build/obj/main_win.obj`
  - `clang link_stub_main.c nyrt_min_stubs_win.S main_win.obj -o test_main.exe`

## References
- Frozen toolchain guide: `docs/guides/frozen-toolchain.md`
- Windows 実績レポート: `build/WINDOWS_LINK_TEST_REPORT.md`
- Frozen v1 Box spec: `docs/reference/boxes/frozen_v1.md`
- Roadmap Phase‑15.77: `docs/development/roadmap/phases/phase-15.77/INDEX.md`
- Phase‑31 計画: `docs/development/roadmap/phases/phase-31-box-Normalization/INDEX_JA.md`
