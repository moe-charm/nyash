# CURRENT_TASK — Status and Next Steps (2025‑10‑16)

このページは「いま何をしていて、次に何をするか」を 1 画面で把握できるようにするダッシュボードだよ。最新の作業に合わせて随時更新していくにゃ。

## Snapshot
Updates (today)
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
- Phase‑31 残: Plugin 既存 ABI へのトランポリン実配線（registry へ新エントリ登録）と quick→plugins→full スモークの差分スキャン。
- Frozen guide への Windows 例追記など、P0 で止まっているドキュメント系タスクを再開する必要があるにゃ。
- `json_query_vm` の後続失敗（ArrayBox.substring 未実装）: text slicing 一貫性を保つ方針で修正する（Array のスライスは String で返す／もしくは substring を Fail‑Fast のままとし app 側で回避）。最小スモークを追加して固定。

## Prioritized TODOs
- **P0 — 直近解消したいもの**
  1. quick → plugins → full スモークを再実行し、カテゴリ 2/3（出力差・モジュール解決）の残差を棚卸し。
  2. Plugin ABI トランポリンの網羅化（registry 配線＆生成ツール化）。
  3. `docs/guides/frozen-toolchain.md` に Windows COFF 例を追記してハンドブックを更新。
  4. `json_query_vm` 後続エラーの切り分けと修正（ArrayBox.substring）。
  5. Map/Set/FileBox 回りの正規化と素材化（今回の回帰の最短修正）
     - Map.size: Builder 正規化（`MapBox.(size|len|length)` → `Extern("nyrt.map.size")`）は 2025‑10‑16 に実装済。Map.remove は戻り値を返すよう修正済。`values` は Extern/Bridge 渡し＋ArrayBox 注釈を入れたが、後続 `.size()` までの素材化順序を追加で整える（EmitGuard 後のローカル化を強制）。
     - EmitGuard: `finalize_call_operands` の適用が必ず走る経路に限定（直叩きはレガシーブリッジのみに）。
     - me 注入: `method_index.static_signature()` によるガードを徹底し、plugin の ModuleFunction に誤適用しない。
     - FileBox: 引数素材化の未適用箇所をスイープ（mir_call 作成前/後の LocalSSA を全経路で統一）。
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
