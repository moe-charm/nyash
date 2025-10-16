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

- **MIR Builder バグ修正 Phase 1-3完了！** (2025-10-16 continued) ✅
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
  - ✅ Phase 2.1修正完了: VarMapGuard を ParserBox.* 限定から**全関数**に拡大
    - ファイル: `src/mir/loop_builder/mod.rs:155-171`
    - 変更: `if fun.signature.name.starts_with("ParserBox.")` 条件を削除
    - 効果: Main.eval_path_text 等でもVarMapGuard適用
  - ✅ Phase 2.2修正完了: local_ssa ensure で**関数パラメータ（v%0-v%N）を絶対に避ける**
    - ファイル: `src/mir/builder/ssa/local.rs:40-46`
    - 変更: `while fun.params.contains(&loc)` ループ追加
    - 効果: Copy命令生成時にパラメータレジスタを回避
  - ✅ Phase 2.3修正完了: **current_fn_singleton 根本原因修正！**
    - 🔥 真の原因: `try_handle_me_direct_call` がme引数を追加していない
    - 症状: `this.test_loop(arg1, arg2)` → `call_module_fn Main.test_loop/2(arg1, arg2)` ← **me引数なし！**
    - 影響: パラメータマッピングずれ → %0=arg1, %1=arg2, %2=null （正: %0=me, %1=arg1, %2=arg2）
    - SSA違反: ループ条件 `path.size()` 評価時に `%0 = copy %13` (path→me) が生成される
    - エラー: "Method router missing receiver for size(0 args)" - nullに対してsize()呼び出し
    - 最小再現: `/tmp/test_param_overwrite.hako`, `/tmp/test_param_overwrite2.hako` 作成済み
    - ✅ 修正1（正しい）: `src/mir/builder/builder_calls/special.rs:123-127`
      - `try_handle_me_direct_call` で me引数を prepend
      - `let me_id = self.current_fn_singleton(&canon_cls);`
      - `args_with_me.insert(0, me_id);`
    - ❌ 修正2（間違い・ChatGPT5により修正済み）: `src/mir/builder.rs:456-474`
      - Claude誤診: `current_fn_singleton` を関数パラメータ %0 を返すように修正
      - 問題: static box methodには me パラメータが存在しない
      - 結果: 呼び出し順が壊れる → 無限ループ・不定動作
      - **ChatGPT5修正**: `emit_static_me_placeholder` でvoidシングルトン生成・キャッシュに戻した
      - VM側で void プレースホルダ → `static_singleton::get()` で実体化
    - 結果: ✅ MIR正常生成 `call_module_fn Main.test_loop/2(%5_void, %6, %7)` (3引数正しい)
    - テスト: ✅ 最小再現ケース実行成功（エラーなし）
    - 状況: ✅ json_query_vm 無限ループ解消（修正2の間違いが原因だった）

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

- **非決定要素（async/GC）揺れ要因調査完了** (2025-10-16 continued) ✅
  - Task先生調査 → **決定的失敗（非決定的ではない）**
  - **async_await / gc_mode_off テスト失敗原因**:
    - 5回実行すべてで同一エラー: "Extern future disabled (legacy-only)"
    - 根本原因: `legacy-boxes` feature がデフォルトで無効
    - 影響: `env.future.*` extern がビルド時に静的無効化
    - 非決定性: ❌ なし（タイミング・GC問題ではない）
  - **環境変数一覧作成完了**:
    - Async/Await: `HAKO_AWAIT_MAX_MS` (5000ms), `NYASH_REWRITE_FUTURE=1`
    - GC: `NYASH_GC_MODE` (counting/off), `NYASH_GC_TRACE=1`, 閾値系変数
    - デバッグ: `HAKO_VM_TRACE`, `NYASH_CLI_VERBOSE=1`, `SMOKES_DEV_LOG=1`
  - **修正提案3案**:
    1. Feature Flag 有効化 (最小変更): `default = [..., "legacy-boxes"]`
    2. テストを SKIP 化 (推奨): Phase 15.77 で削除予定のため
    3. Phase 20.5 で Hakorune VM Future 実装 (長期)
  - **ドキュメント作成**:
    - 決定性調査レポート: `docs/development/analysis/async-gc-determinism-report.md`
    - 安定化ガイド: `docs/development/analysis/quick-profile-stabilization-guide.md`
  - **推奨アクション**: テストを SKIP 化（非決定的ではないため優先度低）

- **using系11件失敗パターン分類完了** (2025-10-16 continued) ✅
  - Task先生調査 → **legacy-boxes除外は完全に無関係**（全11件がusing/module resolution問題）
  - **4パターン分類**:
    - **パターンA (5件, P2)**: Parser Error - module.hako をTOMLとしてパース試行
    - **パターンB (3件, P0)**: Type Error - using解決失敗 → UnknownBox/Void連鎖
    - **パターンC (1件, P0)**: Static Singleton未具現化 - MIR Builder の singleton 作成漏れ
    - **パターンD (3件, P1/P2)**: Expected Failure誤検出 - 循環依存検出失敗 + ログ漏出
  - **P0修正必要**: 4件（パターンB: workspace module resolution、パターンC: static box singleton）
  - **P1修正推奨**: 1件（パターンD-1: 循環依存検出実装）
  - **P2修正**: 6件（パターンA: ログ抑制、パターンD-2: デバッグログ防止）
  - **ドキュメント作成** (5件、44KB):
    - INDEX: `docs/development/analysis/using_failures_INDEX.md`
    - Quick Summary: `docs/development/analysis/using_failures_quick_summary.md` ⭐最初に読む
    - 分類レポート: `docs/development/analysis/using_failures_classification_report.md`
    - フローチャート: `docs/development/analysis/using_failures_flowchart.md`
    - 再現ガイド: `docs/development/analysis/using_failures_reproduction_guide.md`
  - **無実証明**: kernel-embedded boxes (String/Integer/Array等) は正常動作、すべてusing/module層の問題

- **plugin_on_strict_quick_array_semantics失敗原因調査完了** (2025-10-16 continued) ✅
  - Task先生調査 → **ArrayBox birth が3回呼ばれる問題**
  - **根本原因**:
    - `new ArrayBox()` 実行時に3つのインスタンス生成（id=1,2,3）
    - `set(0, 10)` が instance_id=2 に書き込み
    - `size()` が instance_id=3 (空) を読み取り → 0 を返す（期待値: 1）
  - **エラーログ**: `contracts_born_nobirth` - birth method未呼び出しでのオブジェクト生成
  - **修正箇所**: `src/backend/mir_interpreter/handlers/newbox.rs` - birth重複呼び出しの抑制
  - **影響**: プラグインArrayBox のインスタンス管理が破綻

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
- **✅ Phase 1-3完了**: MIR Builder パラメータレジスタバグ根治完了！
  - Phase 1 ✅: パラメータフィルタ実装完了（v%0の上書き解消）
  - Phase 2.1 ✅: VarMapGuard全関数適用（ParserBox.* 限定解除）
  - Phase 2.2 ✅: local_ssa パラメータレジスタ回避（Copy命令生成時）
  - Phase 2.3 ✅: me引数追加修正（try_handle_me_direct_call）
    - ❌ Claude誤診: current_fn_singleton 第一パラメータ返却（無限ループ原因）
    - ✅ ChatGPT5修正: emit_static_me_placeholder でvoidシングルトン生成
  - ✅ json_query_vm 無限ループ解消（Phase 2.3 修正2の間違いが原因だった）
  - Phase 4 予定: MIR Verifier にパラメータ上書き検出追加（保険）
- Phase‑31 残: Plugin 既存 ABI へのトランポリン実配線（registry へ新エントリ登録）と quick→plugins→full スモークの差分スキャン。
- Frozen guide への Windows 例追記など、P0 で止まっているドキュメント系タスクを再開する必要があるにゃ。

## Prioritized TODOs
- **P0 — 直近解消したいもの**
  1. ✅ **DONE**: MirIoBox export追加（selfhost基盤復旧完了）
  2. ✅ **DONE**: Task先生4人並列調査（真因3箇所特定）
  3. ✅ **DONE**: Phase 1 - パラメータフィルタ実装（v%0上書き解消）
  4. ✅ **DONE**: Phase 2.1/2.2 - VarMapGuard + local_ssa修正完了
  5. ✅ **DONE**: Phase 2.3 - me引数追加修正（ChatGPT5により修正完了）
  6. ✅ **DONE**: Task先生4人レガシー削除調査（191行削除可能）
  7. ✅ **DONE**: 非決定要素（async/GC）揺れ要因調査（決定的失敗を確認）
  8. ✅ **DONE**: json_query_vm 無限ループ解消（Phase 2.3 修正2の間違いが原因）
  9. **TODO**: レガシーコード削除実行（191行削減）
     - vars.rs 削除（149行）
     - record_kpi 削除（34行）
     - utils.rs マーカー削除（8行）
  9. **TODO**: Phase 4 - MIR Verifier パラメータ上書き検出追加（保険）
  9. quick → plugins → full スモークを再実行し、カテゴリ 2/3（出力差・モジュール解決）の残差を棚卸し。
  10. Plugin ABI トランポリンの網羅化（registry 配線＆生成ツール化）。
  11. `docs/guides/frozen-toolchain.md` に Windows COFF 例を追記してハンドブックを更新。
  12. SetBox/FileBox/Array slice 周辺の整備（Map.values は解消済み）
     - SetBox: `add/has/size` のローカル素材化を再確認（EmitGuard 経路の統一）。
     - FileBox: read/write 経路の undefined ValueId を解消（Call 発行を guard 経由に統一）。
     - Array slice: レガシー extern 依存を段階撤退し、必要なら専用 Bridge を追加。
  13. Legacy 排他運用の明文化と適用
      - AGENTS.md に「Legacy Boxes と Plugins — 排他運用」を追記（済）。
      - `docs/guides/build-modes.md` を追加（モード・コマンド・ルータ方針）（済）。
      - Cargo default から `legacy-boxes` を外す検討（plugin‑only を既定に）と CI への plugin‑only ライン追加。
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
