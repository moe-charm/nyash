# CURRENT TASK (Compact) — Phase 15 / Self-Hosting（Ny→MIR→MIR-Interp→VM 先行）

このドキュメントは「いま何をすれば良いか」を最小で共有するためのコンパクト版です。詳細は git 履歴と `docs/`（phase-15）を参照してください。

— 最終更新: 2025‑09‑08 (LLVM Core‑13 P0完了/BitOps/Array/Echo/Map OK、VInvoke調査中)

【Quick Update — LLVM Core‑13 P0】
達成
- LLVM 18 環境でビルド安定。`env.box.new(_i64x)` 経路の統一と i64 正規化を実装。
- IntegerBox `.value()` 整合修正。
- BinOp（i64）を網羅（Add/Sub/Mul/Div/Mod/BitAnd/BitOr/BitXor/Shl/Shr/And/Or）。
- fmod 対応: 浮動剰余を LLVM `frem` で実装。
- MIR最適化に method_id 注入パス（`passes::method_id_inject`）を組み込み、BoxCall by‑id を安定化。

スモーク状況（AOT/LLVM）
- BitOps/Shift OK（Result: 48）: `NYASH_LLVM_BITOPS_SMOKE=1 ./tools/llvm_smoke.sh release`
- Arrays OK（Result: 3）: `NYASH_LLVM_ARRAY_SMOKE=1 ./tools/llvm_smoke.sh release`
- Echo OK: `NYASH_LLVM_ECHO_SMOKE=1 ./tools/llvm_smoke.sh release`
- Map(by‑id) OK（v=42/size=1）: `NYASH_LLVM_MAP_SMOKE=1 ./tools/llvm_smoke.sh release`
- VInvoke（可変長）未達（戻り 0）: `NYASH_LLVM_VINVOKE_SMOKE=1` は調査継続

残タスク（P0完了条件）
- `env.box.new`（new_i64x 側）の引数 i64 化クロージャを完全インライン化（lifetime エラー解消）。
- BinOp 未網羅の match 箇所をもう1か所整理（`_ => todo!()` か軽実装）。
- 再ビルド通過後、代表スモークの一致確認。
— 最終更新: 2025‑09‑08 (LLVM Core‑13 安定化 P0 ほぼ完了 + BitOps統合着手)

【Quick Update — LLVM Core‑13 P0（最新版）】
現状
- LLVM 18 環境でビルド実行。opaque pointer 由来の型照会（get_element_type）を除去し、`env.box.new` は NyRT shim（`nyash.env.box.new`, `nyash.env.box.new_i64x`）へ統一。
- `IntegerBox.value()` → `IntegerBox.value` に是正（フィールド参照）。
- BinOp（モック経路）を網羅実装（加算/減算/乗算/除算/剰余/AND/OR/XOR/SHL/SHR/論理And/Or）。
- 残りは BinaryOp の非網羅パターン検出が 1 箇所（同ファイル内の別 match）。ここに `_ =>` を追加してビルド通過予定。

直近タスク（P0完了）
1) `src/backend/llvm/compiler.rs` の BinaryOp match を総確認し、未網羅箇所に `_ =>` か軽実装を付与。
2) 再ビルド通過を確認。
3) 代表スモーク（VM一致）を1本通す。

【BitOps/Shift 取り込み（Phase 1）】
背景
- main 側でビット演算（& | ^）とシフト（<< >>）の構文と VM 実装を着地。Arrow(>>) は廃止、パイプラインは `|>` を利用。
- nyash_llvm 側の LLVM 降ろしは既に BitAnd/BitOr/BitXor/Shl/Shr を実装済みのため、構文経路が繋がれば AOT も動作。

やること（P1）
1) apps/tests/ny-llvm-bitops/main.nyash を構文経路で有効化（必要なら追加/修正）。
2) tools/llvm_smoke.sh に `NYASH_LLVM_BITOPS_SMOKE=1` での E2E を既定ジョブに追加（または README に手順追記）。
3) `tools/build_llvm.sh apps/tests/ny-llvm-bitops/main.nyash -o app_bit && ./app_bit` が `Result: 48` を出すことを確認。

注意
- 右シフト(>>) は Arrow ではなく ShiftRight として解釈される（パイプラインは `|>`）。
- 右シフトは現状論理シフト相当の実装（将来算術/論理の切替が必要なら追補）。

代表スモーク
代表スモーク（例）
- ビルド: `LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) cargo build --release --features llvm`
- 実行: `tools/build_llvm.sh apps/tests/mir-compare-multi/main.nyash -o app && ./app`
- 受け入れ: VM と同一の戻り値（Core‑13 正規化ON）

【VInvoke（by-name/by-id vector）— 戻り値マッピング問題／修正方針】

再現手順
- `NYASH_LLVM_VINVOKE_TRACE=1 NYASH_LLVM_VINVOKE_SMOKE=1 ./tools/llvm_smoke.sh release`

取得ログ（要点）
- nyrt: vinvoke.by_name: recv_h=1 argc=1 vals_ptr=0x... tags_ptr=0x...
- nyrt: vinvoke.by_name: type_id=11 method_id=2 nargs=1 tags[..1]=[3]
- nyrt: vinvoke.by_name: rc=0 out_len=16
- nyrt: vinvoke.by_name: ret_tag=3
- 標準出力: VInvokeRc: 0 / Result: 0

確認事項
- 受け側（NyRT）には正しく届いている：
  - recv_h=1（MapBox インスタンス）
  - type_id=11（MapBox）, method_id=2（get）
  - argc=1, tags=[3]（整数引数）
  - 呼び出し rc=0（成功）
  - 戻り TLV ret_tag=3（i64）
- にも関わらずアプリ側出力は 0 → NyRT→プラグインはOK。戻り値の扱いで欠落。

原因の見立て（LLVM 側の戻り値マッピング）
- 変長 vector 経路の戻りは常に i64：
  - tag=3 はそのまま数値、tag=8 はハンドルIDを i64 として返却。
- LLVM codegen（by-name/by-id vector）で dst の型が MirType::Unknown 等の場合、返り値 i64 を「ハンドル（i8*）として扱う」ために int_to_ptr でポインタ化して格納している箇所がある。
- 今回は「純粋な整数（42想定）」なので、i64=42 をポインタ 0x2a 扱い→無効ハンドルと見做され最終的に 0 になる可能性が高い。

短期対処（最小差分）
1) LLVM vector 経路の戻り格納時、dst が Unknown/Box/String 等でも「既知の原則プリミティブ返りメソッド」は整数として保持する特例を入れる。
   - 例：by-name では box_type + method_name、by-id では type_id + method_id で判定（MapBox.get 等）。
2) by-id vector（`nyash.plugin.invoke_tagged_v_i64`）も同様に統一。

正道（中期）
- 返り値の種別（primitive or handle）を codegen 側で判別可能にする。
  - 方式A：nyash.toml にメソッド戻り型ヒント（primitive/handle/f64/bool）を追加し、レジストリ→codegen へ供給。
  - 方式B：NyRT シムに「期待戻り形式（ハンドル期待か値期待か）」のフラグ引数を追加し、codegen から渡す（dst 型が Unknown なら handle期待=1 など）。NyRT は ret_tag=3 でも handle 期待なら IntegerBox を生成してハンドルで返す。

受け入れ基準
- `NYASH_LLVM_VINVOKE_SMOKE=1`（by-name）で `VInvokeRc: 42` を出力。
- `NYASH_LLVM_VINVOKE_RET_SMOKE=1`（by-id, return）で `Result: 42` を出力。

備考
- argc/ポインタ整合性（vals/tags の GEP([0,0])→i64* キャスト）は現状問題なし。

メモ
- 作業ディレクトリ: `/mnt/c/git/nyash-project/nyash_llvm`（branch: `llvm-dev`）
- 次の commit で P0 を締め、P1（ビット演算/Shift 実装）に移行する。

【Phase 17.1 — LLVM Core‑13 安定化（専用worktree/branch）】
目的
- Core‑13 正規化後の MIR を LLVM AOT に下ろし、VM と同値の代表ケースを安定動作させる。

作業環境
- worktree: /mnt/c/git/nyash-project/nyash_llvm （branch: llvm-dev, origin/llvm-dev 追従）
- LLVM 18 前提: `LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix)`

短期タスク（P0）
- Opaque Pointer 対応: Inkwell 0.6 + LLVM 18 に合わせ、`get_element_type()` 等を使用しない降ろしに修正。
  - `env.box.new` の引数ハンドリングをヒューリスティックに単純化（i8* は `nyash.box.from_i8_string`）
- IntegerBox API 整合: `.value()` → `.value` に是正（フィールド参照）。
- BinaryOp の網羅性: `BitAnd/BitOr/BitXor/Shl/Shr` 未対応を `_ => todo!()` で一旦回避（代表スモーク優先）。

検証（P0）
- ビルド: `LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) cargo build --release --features llvm`
- 代表スモーク: `tools/build_llvm.sh apps/tests/mir-compare-multi/main.nyash -o app && ./app`（VM一致）

後続（P1）
- ビット演算/シフトの実装と検証（LLVM 降ろし/IR 生成/実行互換）。
- AOT 実行の戻り値と型変換経路の整理（i64/handle→Box materialize）。
- 受け入れ: VM と同一の戻り値（Core‑13 正規化 ON。既定ON）

ブランチ/作業場所
- worktree: `/mnt/c/git/nyash-project/nyash_llvm`（branch: `llvm-dev` → origin/llvm-dev セット）
短期タスク（P1/継続）
1) VInvoke（可変長）調査と修正
   - NyRT 変長invokeで type_id/method_id/argc/tags/rc をログ出力（限定的）
   - LLVM 降ろし（vector/by‑name vector）の argc/配列ポインタ（GEP [0,0]）の妥当性確認
   - PluginHost 側の戻りTLV(tag: 3=int/8=handle/5=float) との整合確認
2) README 追記（BitOps/Array/Echo/Map の手順は追記済。VInvoke は解決後に追記）

【ハンドオフ（2025‑09‑06 final）— String.length 修正 完了／JIT 実行を封印し四体制へ】

概要
- 目的: AOT/JIT‑AOT で発生していた `StringBox.length/len` が 0 になる不具合の是正（Lower の二段フォールバック：`nyash.string.len_h` → `nyash.any.length_h`）。
- 結果: 当該不具合は修正・確認完了（AOT/VM で期待値）。JIT 直実行の継続調査は打ち切り、実行モードは「インタープリター／VM／Cranelift(EXE)／LLVM(EXE)」の4体制へ移行。

【Phase 15.17 追記 — Core‑13 純化モード 実装/稼働・AOT shim 拡張】
概要
- Core‑13 純化モード（厳格）を導入: `NYASH_MIR_CORE13_PURE=1`
  - Builder 段: `new`→`ExternCall(env.box.new, [type, …args])` を直接生成（`NewBox/birth` 非生成）
  - Unary(−/!/~): 直接展開（`Const+BinOp/Compare`）
  - WeakRef(weak_new/load): 純化ONでは生成回避（パススルー）
  - Optimizer 安全網: Load/Store→`env.local.get/set` 変換、最終MIRで13命令以外はエラー
- 実行系:
  - VM: `env.local.get/set` と `env.box.new` を実装（BoxFactoryRegistry 連動）
  - LLVM AOT: `env.local.get/set` の Lowering と `env.box.new` shim を追加
    - NyRT 追加: `nyash.env.box.new` / `nyash.env.box.new_i64x` (最大4引数)
    - NyRT 追加: `nyash.box.from_i8_string`（i8*→StringBox）、`nyash.box.from_f64`（f64→FloatBox）
    - Lowering: 引数を i64 ハンドルに正規化（int/ptr→i64、i8*→from_i8_string、f64→from_f64）
  - Cranelift （スケルトン）: `env.local.get/set` / `env.box.new` を実行ループに最小実装（将来のAOT出力の布石）

検証
- `cargo test`: 206 passed / 0 failed / 24 ignored（通常）
- 純化ON（選択的 skip あり）: グリーン。`src/tests/mir_pure_envbox.rs` 追加で `env.box.new` 生成を検証

実装（済）
- LowerCore: 二段フォールバック実装を追加（Param/Local/リテラル）。
  - `emit_len_with_fallback_param`/`_local_handle`/`_literal`
  - `core.rs` の `len/length` で二段フォールバックを使用。結果をローカルスロットへ保存（Return で拾えるように）
- BoxCall 共通（ops_ext）:
  - StringBox の `len/length` を最優先で処理（Param/Local/リテラル/handle.of）。
  - リテラル `new StringBox("...")` → `length` は即値畳み込み（const 返却）。
- Hostcall registry 追補: `nyash.string.len_h` を ReadOnly 登録 + 署名 `(Handle)->I64` 追加。
- JIT ブリッジ: `extern_thunks.rs` に `nyash_string_len_h` を追加、`cranelift` ビルダーに `SYM_STRING_LEN_H` を登録。
- ポリシー: `StringBox.length/len` マッピングを `nyash.any.length_h` → `nyash.string.len_h` に是正。
- デッドコード整理: 旧 `lower_boxcall_simple_reads` を削除（conflict 回避）。
- ツール/スモーク: `tools/aot_smoke_cranelift.sh` 追加、`apps/smokes/jit_aot_string_length_smoke.nyash` 追加。

— 15.17 追加 実装（済）
- Core‑13 純化モード: `NYASH_MIR_CORE13_PURE=1`（Builder/Optimizer/Verifier 連携）
- VM: `env.local.get/set`, `env.box.new` を実装
- LLVM AOT: `env.box.new` shim（new/new_i64x）+ 引数 i8*/f64 のハンドル化 helper 追加
- Cranelift: ExternCall の最小実装（get/set/new）を追加（スケルトン）
- テスト: `src/tests/mir_pure_envbox.rs` 追加（純化 new→env.box.new の生成確認）
- テスト: `src/tests/mir_pure_e2e_vm.rs` 追加（純化ONで VM 実行: new StringBox + length の e2e）
- テスト: `src/tests/mir_pure_locals_normalized.rs` 追加（locals が env.local.get/set に正規化されることを確認）
- テスト: `src/tests/mir_pure_llvm_build.rs` 追加（feature=llvm 時に純化ONで .o を正常生成できることを確認。実行の同値性はAOT実装拡張後に別途追加予定）
- テスト: `src/tests/mir_pure_e2e_arith.rs` 追加（純化×VMで加算の e2e）
- テスト: `src/tests/mir_pure_e2e_branch.rs` 追加（純化×VMで条件分岐の e2e）
- テスト: `src/tests/mir_pure_only_core13.rs` 追加（最終MIRが13命令のみで構成されることを静的検査）
- CI: Core‑13 純化(LLVM) ワークフロー追加（`.github/workflows/core13-pure-llvm.yml`）。LLVM 18 をセットアップし、`--features llvm` で純化ONテストを実行。
- CI: Core‑13 純化モード専用ワークフローを追加（`.github/workflows/core13-pure.yml`）。`NYASH_MIR_CORE13_PURE=1 cargo test --all-targets` を実行。

確認状況（最終）
- `apps/smokes/jit_aot_string_min.nyash`（concat/eq）: AOT で `Result: 1`（OK）。
- `apps/smokes/jit_aot_string_length_smoke.nyash`: AOT .o 生成/リンク・実行とも良好（稀発の segfault 調査は「低優先」に移行）。
- `apps/smokes/jit_aot_any_len_string.nyash`: AOT で `Result` が期待値（0 問題は解消）。
- 備考: JIT 直実行の既知の不安定性は、JIT 実行封印に伴い調査終了とする（アーカイブ扱い）。

残課題（方針更新後）
P0: 実行モード整理（JIT 実行封印）
- ランタイム実行は「Interpreter／VM」に限定。ネイティブ配布は「Cranelift AOT(EXE)／LLVM AOT(EXE)」。JIT 関連のランタイムフラグ説明は docs で封印明記。

P1: AOT 安定化（低頻度 segfault の追跡：低優先）
- 稀な DT_TEXTREL 警告・segfault は PIE/LTO/relro/TLS/extern 登録順の再確認を残課題として維持（優先度は下げる）。

P2: リファクタ（Phase A）継続（振る舞い不変）
- Hostcall シンボル `SYM_*` 統一、`core/string_len.rs` への集約、観測フックの整理は継続。JIT 実行依存の観測は停め、VM/AOT 観測を優先。

P3: Core‑13 純化 仕上げ（今回の続き）
- Cranelift AOT: 実オブジェクト出力で `env.local/env.box` のシンボル連携（現状は実行スケルトンのみ）
- LLVM AOT: `env.box.new_i64x` の引数拡張（>4, TLV支援）と型復元の精度UP（文字列/浮動/配列 等）
- Builder 純化の徹底: Load/Store/WeakRef を完全非生成（全経路点検）
- E2E 純化スモーク: `apps/smokes_pure13/` を追加（VM/LLVM EXE の結果一致）
- CI: 純化ON ジョブを常時実行（最終MIR 13命令チェック含む）

進捗（2025‑09‑06 終了報告）
- ops_ext: StringBox.len/length の結果を必ずローカルに保存するよう修正（Return が確実に値を拾える）
  - 対象: param/local/literal/handle.of 各経路。`dst` があれば `local_index` に slot を割当てて `store_local_i64`。
- デバッグ計測を追加
  - JIT Lower 追跡: `NYASH_JIT_TRACE_LOWER=1`（BoxCall の handled 判定／box_type／dst 有無）
  - Return 追跡: `NYASH_JIT_TRACE_RET=1`（known/param/local の命中状況）
  - ローカルslot I/O: `NYASH_JIT_TRACE_LOCAL=1`（`store/load idx=<N>` を吐く）
  - String.len_h 実行: `NYASH_JIT_TRACE_LEN=1`（thunk 到達と any.length_h フォールバック値を吐く）
- 再現確認
  - `apps/smokes/jit_aot_any_len_string.nyash` は依然 Result: 0（JIT-direct）。
  - 追跡ログ（要 `NYASH_JIT_TRACE_LOWER=1 NYASH_JIT_TRACE_RET=1 NYASH_JIT_TRACE_LOCAL=1`）
    - `BoxCall ... method=length handled=true box_type=Some("StringBox") dst?=true`
    - ローカル slot の流れ: `idx=0` recv(handle) → `idx=1` string_len → `idx=2` any_len → `idx=3` cond → select → `idx=4` dst 保存 → Return で `load idx=4`
    - つまり lowering/Return/ローカル材化は正しく配線されている。
JIT 直実行に関する未解決点（import 解決先の差異疑い 等）は封印に伴いアーカイブ化。必要時に `docs/development/current/` へ復元して再開する。

暫定変更（フォールバック強化）
- `ops_ext` の StringBox.len で「リテラル復元（NewBox(StringBox, Const String))」を param/local より先に優先。
  - JIT-AOT 経路で文字列リテラルの length は常に即値化（select/hostcall を経由せず 3 を返す）。
  - ただし今回のケースでは local 経路が発火しており、まだ 0 のまま（hostcall 実行が 0 を返している疑い）。

未解決/次アクション（デバッグ指針）
- [ ] `nyash.string.len_h` が実際にどの関数へリンクされているかを確認
  - Cranelift JIT: `src/jit/lower/builder/cranelift.rs` の `builder.symbol(...)` 群は設定済みだが、実行時に thunk 側の `eprintln` が出ない。
  - 追加案: `emit_host_call` で宣言した `func_id` と `builder.symbol` 登録可否の整合をダンプ（シンボル直列化や missing import の検知）。
- [ ] `extern_thunks::nyash_string_len_h` へ確実に到達させるため、一時的に `emit_len_with_fallback_*` で `SYM_STRING_LEN_H` を文字列リテラル直書きではなく定数経由に統一。
- [ ] `nyash.string.from_u64x2` の呼び出し可否を同様にトレース（`NYASH_JIT_TRACE_LOCAL=1` の直後に `NYASH_JIT_TRACE_LEN=1` が見えるか）
- [ ] ワークアラウンド検証: `NYASH_JIT_HOST_BRIDGE=1` 強制でも 0 → host-bridge 経路の呼び出しが発火していない可能性。bridge シンボル登録も再確認。

メモ/所見
- lowering と Return 材化（ローカルslot への保存→Return で load）は動いている。値自体が 0 になっているので hostcall 側の解決/戻りが疑わしい。
- AOT .o の生成は成功。segv は今回は再現せず。

実行コマンド（デバッグ用）
- `NYASH_JIT_TRACE_LOWER=1 NYASH_JIT_TRACE_RET=1 NYASH_JIT_TRACE_LOCAL=1 NYASH_AOT_OBJECT_OUT=target/aot_objects/test_len_any.o ./target/release/nyash --jit-direct apps/smokes/jit_aot_any_len_string.nyash`
- 追加で thunk 到達確認: `NYASH_JIT_TRACE_LEN=1 ...`（現状は無出力＝未到達の可能性）

Phase A 進捗（実施済）
- A‑1: Hostcall シンボルの定数化（直書き排除）完了

【ハンドオフ（2025‑09‑06 4th）— GUI/egui 表示テスト計画（Cranelift/LLVM × Windows/WSL）】

目的
- 以前は Windows exe で egui ウィンドウ表示を確認できていたが、現状で再現が不安定な報告あり。Cranelift/LLVM と OS 組み合わせ別に手順と期待結果を明文化し、再現性を担保する。

前提・重要ポイント
- プラグイン版 EguiBox は Windows 専用で実ウィンドウ分岐（`#[cfg(all(windows, feature = "with-egui"))]`）。Linux/WSL では `run()` はスタブ（void）でウィンドウは出ない（X 転送の有無に関係なく）。
- Windows でウィンドウ表示を行うには、`nyash-egui-plugin` を `--features with-egui` でビルドし、`nyash.toml` の `plugin_paths`（または `NYASH_PLUGIN_PATHS`）に DLL のパスが解決できること。
- Linux でウィンドウ表示を確認したい場合は「Rust 例（gui_simple_notepad）」または「ビルトイン EguiBox（nyash 本体を `--features gui` でビルドし、専用 Nyash スクリプトを使用）」を利用する。

テストマトリクス（手順と期待結果）
1) Windows × Cranelift（JIT-direct/EXE 相当）
   - 準備: `cargo build --release --features cranelift-jit`
   - プラグイン: `cargo build -p nyash-egui-plugin --release --features with-egui`
   - 実行（JIT-direct 経路）: `powershell -ExecutionPolicy Bypass -File tools\egui_win_smoke.ps1`
   - 期待: Egui ウィンドウが表示される（アプリ終了までブロッキング）。

2) Windows × LLVM（EXE/直実行）
   - LLVM 準備: `tools\windows\ensure-llvm18.ps1 -SetPermanent`
   - プラグイン: `cargo build -p nyash-egui-plugin --release --features with-egui`
   - Nyash 本体: `cargo build --release --features llvm`
   - 直実行: ` .\target\release\nyash.exe --backend llvm apps\egui-hello\main.nyash`
   - AOT EXE: `tools\build_llvm.ps1 apps\egui-hello\main.nyash -Out egui_hello.exe` → ` .\egui_hello.exe`
   - 期待: どちらの経路でも Egui ウィンドウが表示される（DLL が `plugin_paths` に解決可能であること）。

3) WSL × Cranelift（JIT-direct/EXE 相当）
   - 準備: `cargo build --release --features cranelift-jit`
   - 実行: `./target/release/nyash --jit-direct apps/egui-hello/main.nyash`
   - 期待: 実ウィンドウは出ない（プラグインの `run()` はスタブ）。エラーなく終了（void）。
   - 備考: GUI 表示が必要な場合は Rust 例（`cargo run --features gui-examples --example gui_simple_notepad --release`）を利用。

4) WSL × LLVM（EXE/直実行）
   - 準備: `./tools/llvm_check_env.sh` → `LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) cargo build --release --features llvm`
   - 実行: `./target/release/nyash --backend llvm apps/egui-hello/main.nyash`
   - 期待: 実ウィンドウは出ない（スタブ）。
   - 備考: GUI 表示が必要な場合は Rust 例、もしくは nyash 本体を `--features gui` でビルドし、ビルトイン EguiBox 用の Nyash スクリプトを別途用意（要サンプル整備）。

診断スイッチ（共通）
- `NYASH_DEBUG_PLUGIN=1`: プラグインのロードパス/解決状況を表示
- `NYASH_CLI_VERBOSE=1`: プラグインホスト初期化やランタイムの詳細ログ
- Windows/プラグインの DLL 解決が怪しい場合は `NYASH_PLUGIN_PATHS` で DLL ディレクトリを明示（`;` 区切り）

既知の制約 / TODO
- Linux/WSL でプラグイン版 EguiBox の `run()` は現在スタブ（実ウィンドウなし）。将来的に `#[cfg(target_os = "linux")]` 分岐で eframe 実装を追加し、X11/Wayland でも表示可能にする。
- ビルトイン EguiBox（`src/boxes/egui_box.rs`）は `--features gui` でビルド時に有効。Nyash スクリプト側の API はプラグイン版と異なる（`setTitle/setSize/addText/run`）。Linux GUI 確認用にビルトイン用サンプル（apps/egui-builtin/）を追加して整備する。
- Windows AOT リンクは MSVC `link.exe` を既定（fall back: clang）。lld へのスイッチ（速度優先）を将来オプション化検討。

次アクション（引き継ぎ TODO）
- [ ] Windows: 4通りの実行（Cranelift 直/JIT、LLVM 直、AOT EXE）で `apps/egui-hello/main.nyash` のウィンドウ表示を再確認（`NYASH_DEBUG_PLUGIN=1` でログ採取）。
- [ ] WSL: Cranelift/LLVM の直実行は「スタブ終了」が期待値であることを README/ガイドに明記。GUI が必要なら Rust 例 or ビルトイン EguiBox 経路を案内。
- [ ] Linux 向けプラグイン `with-egui` 実装の導入可否を検討（`plugins/nyash-egui-plugin/src/lib.rs` の `winrun` と類似の `linrun` を追加）。
- [ ] ビルトイン EguiBox 用の Nyash サンプルを `apps/egui-builtin/` として追加し、`--features gui` での手順を `dev/selfhosting/` または `docs/` に追記。
  - `nyash.handle.of` / `nyash.string.len_h` / `nyash.console.birth_h` を `SYM_*` に統一
- A‑2: string_len ヘルパ抽出（共通化）完了
  - `src/jit/lower/core/string_len.rs` 新設、`emit_len_with_fallback_*` を移設
  - 呼び出し元はそのまま（挙動は不変）
- A‑3: 観測の統一（第一弾）
  - string_len 内で `observe::lower_hostcall` を発火（len_h/any.length_h）
  - Cranelift/ObjectBuilder の `emit_host_call[_typed]` に `NYASH_JIT_TRACE_IMPORT=1` によるインポート解決ログを追加

観測結果（A‑3 導入後）
- `NYASH_JIT_TRACE_IMPORT=1` で `nyash.string.len_h` / `nyash.any.length_h` の import 呼び出しを確認（JIT/AOT 両方）
- それでも `NYASH_JIT_TRACE_LEN=1` の thunk 到達ログは出ず → 依然解決先に差異がある疑い（要継続調査）

■ 実行系の最終方針（Phase 15 着地）
- ランタイム: Interpreter / VM
- 配布: Cranelift AOT (EXE) / LLVM AOT (EXE)
- JIT 直実行: 封印（ドキュメント上も「実験的/無効」へ集約）

■ 検証チェックリスト（更新）
- VM: `./target/release/nyash --backend vm apps/smokes/jit_aot_string_min.nyash` → Result:1
- AOT(Cranelift): `./tools/build_aot.sh apps/smokes/jit_aot_string_length_smoke.nyash -o app` → `./app` 実行 → 期待結果
- AOT(Windows one‑shot): `pwsh -File tools/windows/build_egui_aot.ps1 -Input apps/egui-hello-plugin/main.nyash -Out app_egui` → 画面表示


— Phase A（無振る舞い変更）リファクタ方針（継続）
- A‑1: Hostcall シンボルを定数に統一（直書き排除）
  - `"nyash.handle.of"` → `jit::extern::handles::SYM_HANDLE_OF`
  - `"nyash.string.len_h"` → `jit::extern::collections::SYM_STRING_LEN_H`
  - `"nyash.console.birth_h"` → 既存の定数へ（なければ `extern::...` に追加して使用）
- A‑2: 長さ取得の共通化
  - 新規: `src/jit/lower/core/string_len.rs`
  - 既存の `emit_len_with_fallback_{param,local,literal}` をこのモジュールへ抽出し、`core.rs`/`ops_ext.rs` から呼び出すだけにする（挙動は据え置き）。
  - 目的: 重複と分岐のばらけを解消し、シンボル差し替えや観測フックを一点で行えるようにする。
※ Phase A は「振る舞いを変えない」ことを厳守する。

2) 診断イベントの追加（軽量）
   - `emit_len_with_fallback_*` と `lower_box_call(len/length)` に `observe::lower_hostcall` を追加し、
     Param/Local/リテラル/handle.of どの経路か、select の条件（string_len==0）をトレース可能にする（`NYASH_JIT_EVENTS=1`）。

3) AOT segfault (稀発) の追跡（低優先）
   - `tools/aot_smoke_cranelift.sh` 実行中に稀に segv（`.o` 生成直後/リンク前後）。
   - `nyash.string.from_u64x2` 載せ替えと DT_TEXTREL 警告が出るので、PIE/LTO/relro 周りと TLS/extern の登録順を確認。

4) 警告のノイズ低減（低優先）
   - `core_hostcall.rs` の unreachable 警告（case 統合の名残）。
   - `jit/lower/*` の unused 変数/unused mut の警告。

影響ファイル（今回差分）
- `src/jit/lower/core.rs`（len/length 二段フォールバック呼出し、保存強化）
- `src/jit/lower/core/ops_ext.rs`（StringBox len/length 優先処理、リテラル即値畳み込み、保存）
- `src/jit/hostcall_registry.rs`（`nyash.string.len_h` 追補）
- ドキュメント: README(ja/en) の実行モード更新、ガイド（egui AOT）に one‑shot スクリプト反映
- `src/jit/extern/collections.rs`（`SYM_STRING_LEN_H` 追加）
- `src/jit/lower/extern_thunks.rs`（`nyash_string_len_h` 追加）
- `src/jit/lower/builder/cranelift.rs`（`SYM_STRING_LEN_H` のシンボル登録）
- `tools/aot_smoke_cranelift.sh`（新規）
- `apps/smokes/jit_aot_string_length_smoke.nyash`（新規）

再現/確認コマンド
- ビルド（JIT/AOT）: `cargo build --release --features cranelift-jit`
- JIT‑AOT（.o出力）: `NYASH_AOT_OBJECT_OUT=target/aot_objects/test_len_any.o ./target/release/nyash --jit-direct apps/smokes/jit_aot_any_len_string.nyash`
- AOT 連結〜実行: `bash tools/aot_smoke_cranelift.sh apps/smokes/jit_aot_string_min.nyash app_str`

次アクション（引き継ぎ TODO）
- [ ] Return の後方走査材化を実装（BoxCall/Call/Select 等の定義→保存→Return 接続）。
- [ ] `emit_len_with_fallback_*` / `lower_box_call(len/length)` にイベント出力を追加（選択分岐/経路ログ）。
- [ ] AOT segv の最小再現収集（PIE/relro/TLSの前提確認）→ `nyrt` 側エクスポート/リンカフラグ点検。
- [ ] `NYASH_USE_PLUGIN_BUILTINS=1` 時の `length` も robust path を常に使用することを E2E で再確認。
- [ ] Cranelift AOT: `env.local/env.box` を実オブジェクト出力に反映（link/name 解決の道付け）
- [ ] LLVM AOT: `nyash.env.box.new_i64x` の引数≥5およびTLV化の検討、引数型の復元精度UP
- [ ] Builder 純化の網羅化（Load/Store/WeakRef 非生成の全経路テスト追加）
- [ ] 純化ON E2E スモーク（VM/LLVM）と CI 常時ジョブの追加

メモ
- `jit_aot_any_len_string.nyash` は `return s.length()` の Return 経路解決が決め手。材化を強化すれば `3` が期待値。
- 既存の Array/Map 経路・他の smokes は影響なし（len/size/get/has/set の HostCall/PluginInvoke は従来どおり）。

■ 進捗サマリ
- Phase 12 クローズアウト完了。言語糖衣（12.7-B/P0）と VM 分割は反映済み。
- Phase 15（Self-Hosting: Cranelift AOT）へフォーカス移行。
  - 設計/仕様ドキュメントとスモーク雛形を追加済み。
    - 設計: `docs/backend-cranelift-aot-design.md`
    - API案: `docs/interfaces/cranelift-aot-box.md`
    - LinkerBox: `docs/interfaces/linker-box.md`
    - スモーク仕様: `docs/tests/aot_smoke_cranelift.md`
    - 雛形スクリプト: `tools/aot_smoke_cranelift.sh`, `tools/aot_smoke_cranelift.ps1`
- README にセルフホスト到達の道筋を明記（C ABI を Box 化）。

【ハンドオフ（2025‑09‑06 3rd）— String.length: const‑fold→Return 材化の不一致 調査ログとTODO】

概要（現象）
- 目標: JIT/JIT‑AOT で `StringBox.length/len` が 3 を返すべき箇所で 0 になるケースを解消。
- 現状: Lower 中の早期 const‑fold で `length = 3` を確実に計算（[LOWER] early const‑fold ... = 3 が出力）。Return 時点でも `ValueId(3)` が `known_i64=3` と認識される（[LOWER] Return known_i64?=true）。にもかかわらず最終結果（実行結果）は 0。

重要な観測（再現とログ）
- MIR ダンプ（プリンタ仕様上、BoxCall は `call %box.method()` として表示）
  0: `%1 = const "abc"`
  1: `%2 = new StringBox(%1)`
  2: `call %2.birth(%1)`  // birth は通常 call（dst なし）
  3: `%3 = call %2.length()` // これも通常 call 表記（内部は BoxCall）
  4: `ret %3`
- Lower ログ:
  - `[LOWER] early const-fold StringBox.length = 3` が出る（const‑fold 成功）
  - `[LOWER] Return value=ValueId(3) known_i64?=true param?=false local?=true`
  - それでも実行結果は `Result: 0`
- `nyash.jit.dbg_i64`（[JIT‑DBG]）の出力が実行時に出ていない（import は宣言されるが call が観測されず）。

今回入れた変更（実装済・該当ファイル）
- Return/材化の強化（known を最優先）
  - `src/jit/lower/core_ops.rs`: `push_value_if_known_or_param` を「known_i64 最優先」に変更。
  - `src/jit/lower/core.rs` の `I::Return` でも `known_i64` を最優先で積むように変更。
- Call/ArrayGet の戻り値の保存
  - `src/jit/lower/core.rs`: `I::Call` 戻り値を dst が無い場合もスクラッチローカルへ保存（栈不整合の防止）。`I::ArrayGet` 戻り値も dst スロットへ保存。
- String.length/len の早期 const‑fold を二段に強化
  - `src/jit/lower/core.rs`: BoxCall 入り口で `StringBox.literal` の length/len を即値化（最優先）。
  - `src/jit/lower/core/ops_ext.rs`: 同様の const‑fold を堅牢化（NewBox(StringBox, Const) から復元）。
  - `src/jit/lower/core.rs`: lowering 前に `known_str` を事前シード、`string_box_literal` マップ（NewBox → リテラル文字列）を構築し、Copy 伝播も対応。
- トレース導線
  - `src/jit/lower/core/string_len.rs`: 二段フォールバック（param/local/literal）にデバッグフック（タグ 110x/120x/130x 系）追加。
  - `src/jit/lower/builder/cranelift.rs`: ローカル slot の store/load トレース（`NYASH_JIT_TRACE_LOCAL=1`）。
  - `crates/nyrt/src/lib.rs`: AOT 側の `nyash.string.len_h` / `nyash.any.length_h` に `[AOT-LEN_H]` を追加。
  - `src/jit/lower/extern_thunks.rs`: `nyash_string_from_u64x2` に `[JIT-STR_H]` を追加（JIT のハンドル生成観測）。`nyash_handle_of` に `[JIT-HANDLE_OF]` を追加。

仮説（根本原因）
- const‑fold で 3 を積めているにも関わらず、Return 時の実返却が 0。優先順位の修正により `known_i64` から 3 を積むよう修正済みだが、compiled JIT 関数内での Return 材化導線（ret_block への引数配線/最後の return）が値 0 に擦り替わる経路が残っている可能性。
  - ret_block/ジャンプ引数の材化不整合
  - 後続命令でスタックが上書きされる経路
  - birth の dst なし call で残留値が生じていた可能性（Call 戻り値スクラッチ保存で対策済）

次アクション（TODO）
1) Return の後方走査材化（優先・CURRENT_TASK 既存 TODO の実装）
   - BoxCall/Call/Select/Const/Copy/Load に遡って、Return が値を確実に拾う材化パスを補強する。
   - 既に known_i64 最優先化は実施済み。残りは ret_block 引数配線の最終確認（CraneliftBuilder の ret 経路）。

2) 実行時の値トレース強化（短期）
   - `emit_return`（CraneliftBuilder）で、ret_block へ jump 直前の引数 `v` を `nyash.jit.dbg_i64(299,v)` で確実に呼ぶ（env でON）。
   - ret_block 入口パラメータの `return_` 直前でも `dbg_i64(300,param0)` 呼び出しを足し、どこで 0 になるかを確定する。

3) BoxCall(length/len) の早期 fold 命中率最終確認
   - `NYASH_JIT_TRACE_LOWER=1` で `[LOWER] early const-fold ... = 3` が必ず出ることを確認。
   - 既に出ているが、Return までの導線で 3 が 0 に化ける起点を 2) で特定する。

4) AOT/JIT‑AOT 観測の整備（参考）
   - `[AOT-LEN_H]` で AOT 側 len_h/any.length_h の handle 解決有無をログ化。JIT‑AOT smoke での差異を収集。

再現/確認コマンド（更新）
- 早期 fold と Return 導線ログ:
  - `NYASH_JIT_TRACE_LOWER=1 NYASH_JIT_TRACE_RET=1 NYASH_AOT_OBJECT_OUT=target/aot_objects/test_len_any.o ./target/release/nyash --jit-direct apps/smokes/jit_aot_any_len_string.nyash`
- ローカル slot 観測:
  - `NYASH_JIT_TRACE_LOCAL=1 NYASH_AOT_OBJECT_OUT=... --jit-direct ...`
- AOT 側の handle 解決ログ:
  - `NYASH_JIT_TRACE_LEN=1 bash tools/aot_smoke_cranelift.sh apps/smokes/jit_aot_string_min.nyash app_str`

補足メモ
- MirPrinter は BoxCall を `call %box.method()` と出力する仕様（今回は BoxCall 経路で const‑fold が呼ばれていることは [LOWER] ログで確認済み）。
- `call %2.birth(%1)` の戻り値残留に備え、I::Call の dst なし呼び出しでもスクラッチ保存して栈を消費するよう修正済み（回帰に注意）。

担当者への引き継ぎポイント
- まず 2) の dbg を CraneliftBuilder の ret 経路（jump to ret_block と return_）に追加し、`v`/`param0` が 0 になる箇所を特定してください。
- 次に 1) の Return 後方走査材化を入れて、BoxCall/Select/Copy 等いずれの経路でも Return が安定して値を拾えるようにしてください。
- その後、smoke `apps/smokes/jit_aot_any_len_string.nyash` が `Result: 3` で通ることを確認し、const‑fold のログと一致することをもってクローズ。

【将来計画（バグ修正後）— JIT を exec 専用にし、VM 連携を段階的に廃止】

目的
- JIT は「コンパイル＋実行（exec）」に一本化し、VM 依存のレガシー経路（param-index/TLS参照）を撤去する。
- 値の材化・ハンドル管理・hostcall を JIT 側で一貫させ、境界の不整合を根本から減らす。

ロードマップ（段階移行）
1) 実行モードの明確化（設定）
   - 環境変数 `NYASH_JIT_MODE=exec|compile|off` を導入。
   - 既存の `NYASH_JIT_STRICT` は非推奨化し、`MODE=compile` に集約。

2) JIT ABI の一本化
   - `src/jit/lower/extern_thunks.rs` などから `with_legacy_vm_args` を撤去。
   - `nyash_handle_of` を含む extern は「JIT引数/ハンドルのみ」を受け付ける設計に変更。
   - ランタイム境界で `VMValue -> JitValue(Handle)` へのアダプタを用意。

3) レガシー撤去（JIT/AOT側）
   - `crates/nyrt/src/lib.rs` の `nyash.string.len_h`/`nyash.any.length_h` から param-index フォールバックを削除。
   - lowering の `-1` センチネルや VM 依存の fallback を廃止し、`handle.of` または既存ローカルハンドルに統一。

4) フォールバック方針（移行期間）
   - 関数単位で `unsupported>0` の場合のみ VM にフォールバック。
   - オプション `NYASH_JIT_TRAP_ON_FALLBACK=1` を追加し、移行時の漏れを検出可能に。

5) Return 導線の強化（本タスクの延長）
   - Cranelift 生成の ret 経路に dbg を常設（envでON）。
   - Return の後方走査材化を標準化し、const-fold/BoxCall/Select いずれでも Return が値を確実に拾うように。

6) ドキュメント/テスト更新
   - README/CURRENT_TASK にモード説明と運用方針を追記。
   - CI の smoke は `MODE=exec` を常態化し、compile-only はAOT出力/ベンチのみで使用。

影響範囲（主な修正ポイント）
- `src/jit/manager.rs`（モード/実行ポリシー）
- `src/jit/lower/extern_thunks.rs`（レガシーVM依存排除、JIT ABI専用化）
- `src/jit/lower/core.rs` / `src/jit/lower/core_ops.rs`（-1センチネル削除・ハンドル材化徹底）
- `crates/nyrt/src/lib.rs`（dotted名hostcallのレガシー経路削除）
- ドキュメント（README/CURRENT_TASK）

ロールアウト/リスク
- フラグ駆動で段階的に切替（デフォルト `exec`）。
- リスク: plugin経路/hostcall registry/ハンドルリーク。
  - 緩和: `handles::begin_scope/end_scope_clear` によりハンドル回収を徹底、registryの検証を追加。

【本日更新】
- VM if/return 無限実行バグを修正（基本ブロック突入時に `should_return`/`next_block` をリセット）。include 経路のハングも解消。
- ArrayBox プラグイン生成失敗に対し、v2 ローダへパス解決フォールバック（`plugin_paths.search_paths`）を追加し安定化。
- std/string の P0 関数を Ny 実装で追加（length/concat/slice/index_of/equals）。index_of は substring ループで代替。
- 残課題: string_smoke で `fails` 累積の else 側に φ が入らず未定義値参照（MIR Builder 側の SSA/φ 振る舞い）。別タスク化。

【ハンドオフ（2025‑09‑06）— AOT/JIT‑AOT 足場と箱下寄せリファクタ】
- 変更サマリ
  - nyrt: AOT 連携の dotted 名を追加（Map/String/Any/birth）
    - `nyash.map.{size_h,get_h,get_hh,set_h,has_h}`
    - `nyash.string.{len_h,charCodeAt_h,concat_hh,eq_hh,lt_hh}` / `nyash.any.{length_h,is_empty_h}`
    - NewBox/文字列: `nyash.instance.birth_name_u64x2`, `nyash.string.from_u64x2`
  - JIT‑AOT(ObjectBuilder):
    - 文字列リテラル→ハンドル生成（u64x2 パック → `nyash.string.from_u64x2`）
    - 出力関数を `ny_main` としてエクスポート
    - 最小 Store/Load（i64）を StackSlot で実装
  - Lower（箱を下に寄せる最小整理）:
    - Map: param 不在でもローカルハンドルがあれば `_H` シンボルで直呼び
    - Any.length: StringBox は `nyash.string.len_h` を優先。ローカル/再構築/旧 index の順にフォールバック
    - Copy/Load でローカルハンドルを dst 側 slot に伝播
    - Array.length は ArrayBox 受けに限定（ops_ext ガード）

- 追加スモーク（JIT‑AOT）
  - `apps/smokes/jit_aot_string_min.nyash`（concat+eq）→ PASS
  - `apps/smokes/jit_aot_any_isempty_string.nyash` → PASS
  - `apps/smokes/jit_aot_any_len_string.nyash` → 現状 Result: 0（後述の未解決）
  - `apps/smokes/jit_aot_map_min.nyash` → 環境により MapBox 生成が必要

- 実行例
  - 文字列ミニ（AOT）:
    - `NYASH_AOT_OBJECT_OUT=target/aot_objects/test_str.o ./target/release/nyash --jit-direct apps/smokes/jit_aot_string_min.nyash`
    - `cc target/aot_objects/test_str.o -L target/release -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive -lpthread -ldl -lm -o app_str && ./app_str` → `Result: 1`
  - isEmpty（AOT）:
    - 同様に `app_empty` → `Result: 1`
  - Map 最小（AOT）:
    - `.o` 生成/リンクは通る。`new MapBox()` はプラグイン/設定に依存（`nyash.toml` と `.so` の配置を確認）

- 未解決 / 既知の課題（優先度高）
  1) String.length の AOT 実行が 0 になるケース
     - 症状: `s = new StringBox("abc"); return s.length()` → `Result: 0`
     - 現状の対処: Any.length を String.len_h 優先にし、ローカル/再構築/旧 index の順でフォールバック。Const fold も追加済み。
     - 追加方針: 受け型伝播（Copy/Load→dst へ型共有）をより堅牢化。最終手段として、ローカルハンドル時に `string.len_h`→`any.length_h` の二段呼び分け（0 返りのときだけ後者）で保険を張る。
  2) MapBox 生成（AOT 実行バイナリ）
     - 環境によりプラグイン解決が必要。`nyash.toml` のあるディレクトリで実行し、必要なら各プラグインを `target/release` に配置。

- 次アクション（引き継ぎ TODO）
  - [ ] Any.length の 0 問題を完全解消
    - [ ] 受けの型/ハンドル伝播（Copy/Load/Store）を統一ヘルパ化し、length/len/charCodeAt で確実にハンドルを積む
    - [ ] StringBox(Const) は定数畳み込みを最優先（len を即値化）
    - [ ] 保険: `string.len_h`→0→`any.length_h` の順にフォールバック（ローカルハンドル時）
  - [ ] メソッド→シンボル/引数規約の集中表を作成（Array/Map/String/Any）
    - [ ] ops_ext/core の分岐重複を縮減（箱の責務を「下」に寄せる）
  - [ ] AOT スモーク拡充
    - [ ] String/Array の length/len を追加、select/分岐のミニ例も用意
    - [ ] Map.get/has/set（プラグインあり環境用）

- 影響ファイル（主要）
  - 追加/更新: `crates/nyrt/src/lib.rs`（dotted エクスポート多数）、
    `src/jit/lower/builder/{object.rs,cranelift.rs}`、
    `src/jit/lower/{core.rs,core/ops_ext.rs,core_hostcall.rs}`、
    スモーク: `apps/smokes/jit_aot_*.nyash`

■ ハンドオフ（JIT AOT / LLVM の現状と次アクション）
- 現状サマリ
  - Array fast‑path: VM 側 len/length を最前段に早期化（Void→0 も確認）。
  - Null 互換: NullBox→VMValue::Void へ統一（比較の整合確保）。
  - std/array smoke: `NYASH_DISABLE_PLUGINS=1` で PASS（len/push/pop/slice）。
  - LLVM AOT: 復活（nyrt の read lock 寿命修正、build_llvm.sh のリンクパス `-L target/release` 追加）。
  - JIT AOT(ObjectBuilder): P0 安定化＋P1 実装済（const/return、i64 binop、compare、select、branch/jump、hostcall 基本、PHI最小化ブロック引数）。
    - jit-direct で .o 生成確認: `apps/smokes/jit_aot_arith_branch.nyash` → Result 13、.o 出力 OK。
    - build_aot.sh は既定で STRICT=0、出力 `target/aot_objects/main.o` に固定。
  - nyrt: AOT 連携用 dotted 名 alias を Array に追加（`nyash.array.{len_h,get_h,set_h,push_h}`）。

- 優先TODO（次にやること）
  1) JIT AOT P2: hostcall 拡張（規約ベースの最小集合）
     - Map: `nyash.map.{size_h,get_h,has_h,set_h}` の dotted 名を nyrt に追加（既存実装へ forward）
     - String: 代表メソッド（len/concat/substring/indexOf 等）で必要なシンボルを dotted 名として追加
     - ObjectBuilder から `emit_host_call_typed` で呼び出し（Lower の対応表に従う）
  2) LowerCore: slot/name→hostcall マッピング（by‑slot を優先、by‑name は互換フォールバック）
     - Array/Map/String の最小セット（len/get/set/push、size/get/has/set、len/concat など）
  3) 後続（必要時）: JIT AOT スモークを追加（分岐あり最小、Array/Map の各1本）

- 実行コマンド（確認用）
  - JIT AOT（jit-direct + .o）:
    - `NYASH_DISABLE_PLUGINS=1 NYASH_JIT_EVENTS=1 NYASH_AOT_OBJECT_OUT=target/aot_objects/jit_aot_arith.o ./target/release/nyash --jit-direct apps/smokes/jit_aot_arith_branch.nyash`
  - LLVM AOT（emit+link）:
    - `LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) tools/build_llvm.sh apps/tests/ny-llvm-smoke/main.nyash -o app`

■ 現在のフォーカス（JITオンリー／一旦の着地）
1) Core 緑維持（完了）
   - `tools/jit_smoke.sh` / Roundtrip(A/B) / Bootstrap(c0→c1→c1') / Using E2E = PASS

【P1 進捗 — LLVM Core-13: ビット演算/シフト】
実装
- LLVM 降ろしで `BinaryOp::{BitAnd,BitOr,BitXor,Shl,Shr}` を i64 経路に実装済み（既存）。
- `compile_and_execute` の MIR インタプリタにも同演算を実装し、パリティを確保。

検証
- 単体テスト追加: `src/tests/llvm_bitops_test.rs`
  - MIR を直接構築して `1=(5&3),7=(5|2),4=(5^1),32=(1<<5),4=(32>>3)` の合計 48 を検証。
  - VM 実行で 48、LLVM `compile_to_object` がエラーなく emit、`compile_and_execute` でも 48 を確認（フォールバック実行）。
- AOT スモーク（任意）: `tools/llvm_smoke.sh` に `NYASH_LLVM_BITOPS_SMOKE=1` で有効化する項目を追加（入力は Nyash ソース制約のため現状 skip 既定）。

注意
- Nyash ソースパーサが `&|^<<>>` を未サポートのため、ビット演算の E2E は当面 MIR 直構築テストで担保。
  将来 `grammar/` の演算子追加後に `apps/tests/ny-llvm-bitops/` を有効化予定。
2) CI 分離（完了）
   - Core（常時）: `tools/jit_smoke.sh` + Roundtrip
   - Plugins（任意）: `NYASH_SKIP_TOML_ENV=1 ./tools/smoke_plugins.sh`（strict既定OFF、`NYASH_PLUGINS_STRICT=1`でON）
3) Self‑host E2E（完了）
   - ny_plugins 有効 + `NYASH_USE_NY_COMPILER=1` の自己ホストE2Eをオプションゲートで運用
4) クリーンアップ（完了）
   - 未使用import/変数の整理、runner責務分割、tools出力体裁の統一

■ ブランチ/構成（Phase 15）
- 実装ブランチ: `phase-15/self-host-ny-mir`
- 既存 Workspace は維持（`crates/*`）。
- 方針: crates 側は変更せず「Nyash スクリプト + nyash.exe」だけで実装・運用（Windows優先）。
  - 例: `C:\git\nyash-project\nyash_self\nyash` 直下で `target\release\nyash` 実行。
- Nyash 製パーサは `apps/ny-parser-nyash/`（Nyashコード）として配置（最初は最小サブセット）。
- MIR 解釈層は既存 `backend/mir_interpreter.rs` と `runner/modes/mir_interpreter.rs` を拡充。
- AOT 関連の雛形は `src/backend/cranelift/` に維持（feature gate: `cranelift-aot`）。

■ 再開TODO（優先順）
1) std Ny実装の実体化（P0/P1）
   - string: length/concat/slice/indexOf/equals → P0 完了（string_smoke PASS）
   - array: len/push/pop/slice を内蔵経路で先行（次着手）
   - map: get/set/len/keys (+values/entries/forEach)
   - jit_smoke に機能検証を常時化（Coreは `NYASH_DISABLE_PLUGINS=1`）
2) NyコンパイラMVPのsubset拡張
   - let/call/return に続き if/ブロック/関数引数まで拡張し、`NYASH_USE_NY_COMPILER=1` スモークを充実
3) Self‑host E2E ゲートの昇格
   - 連続N回グリーン後にCI optional→requiredへ昇格（trace/hash基準）
4) Plugins厳格ONの段階移行
   - Core‑13準拠サンプルへ置換し、`NYASH_PLUGINS_STRICT=1` ゲートで順次ONに復帰

【優先追加 — JIT AOT（ObjectBuilder）安定化・拡張】
- P0: 安定化（完了）
  - switch_to_block なしでの命令発行panic対策（emit_const系）
  - 終端命令なしVerifierエラー対策（emit_return 実装）
  - build_aot.sh の STRICT 緩和（デフォルト0）＋ obj 直指定
- P1: 最小命令カバレッジ（今すぐ実装）
  - i64 binop: add/sub/mul/div/mod を実コード生成
  - compare: eq/ne/lt/le/gt/ge → b1→i64(0/1) へ正規化してpush
  - 分岐/ジャンプ: br_if_top_is_true/jump_to 実装（ブロック遷移とCFG整合）
  - select: emit_select_i64 実装（cond, then, else の順）
- P2: hostcall 系の型付き発行（必要最小限）
  - array/map/string/integer の代表 extern を ObjectBuilder に実装
  - ny-llvm-smoke 等に相当する JIT AOT smoke 追加
- P3: CI スモーク
  - `tools/jit_smoke.sh` に AOT(JIT)最小タスクを追加（STRICT=0 で .o 生成確認）

## ブロッカー/暫定対応（2025‑09‑05 更新）
- 影響範囲（Backend差）
  - JIT(cranelift) → 影響なし。
  - VM(backends=vm) → if/return 無限ループは修正済み（基本ブロック突入時に CF リセット）。
  - 結論: include ハングの根因は VM の制御フロー残存フラグ。修正により解消。

- 事象A: include ハング → 解消
  - `apps/tmp_len_min.nyash`/`apps/tmp_len_test.nyash` 正常完走を確認。

- 事象B: ArrayBox プラグイン生成エラー → 解消
  - v2 ローダにフォールバック探索（`plugin_paths.search_paths`）を追加し、workspace の `./target/release/*.so` を自動解決。
  - DEBUG 時に birth 戻り `code/out_len` をロギング。

- 事象C: std/string_smoke の最終段で未定義値参照 → 解消
  - MIR Builder の if 降ろしで φ を必ず生成（then のみ代入・else 未代入時は pre 値と then 値で合流）。
  - string_smoke PASS を確認。

## 次アクション（デバッグ計画）
- A1: includeハング最小化再現を固定（VM経路優先で調査）
  - `apps/tmp_len_test.nyash` 固定、`NYASH_DEBUG=1` で `execute_include_expr` → `ensure_static_box_initialized` までの経路にログを追加。
  - `included_files`／`include_stack` の push/pop と RwLock/RwLock の取り回しを確認。ポップ忘れ/二重ロックがないか検査。
  - `apps/std/string.nyash` 内のメソッドを段階的に無効化して最小原因を特定（現状 length のみでも再現）。

- A2: VM if/return 無限実行（VM限定）を優先修正
  - 症状: JITは1回then→Return→終了。VMはthenのprintが際限なく繰り返される。
  - 再現最小: `apps/tmp_if_min.nyash`
    ```nyash
    static box Main {
      main() {
        local x
        x = 3
        if x == 3 {
          print("ok3")
          return 0
        }
        print("bad")
        return 1
      }
    }
    ```
    - JIT: `./target/release/nyash apps/tmp_if_min.nyash` → 1回だけ ok3, Result:0
    - VM: `timeout 4s ./target/release/nyash --backend vm apps/tmp_if_min.nyash` → ok3 が無限に出続け TIMEOUT
  - MIRダンプ（`NYASH_VM_DUMP_MIR=1`）では if 降下は正しく、then/else 各ブロックは `ret` を含む。
    - 例: bb1 に `extern_call log("ok3")` の後 `ret 0`。bb2 に `ret 1`。
  - 観測ログ（`NYASH_VM_DEBUG_EXEC=1`）では Print/Const が繰り返し実行。Return の終端処理が機能していない疑い。
  - 仮説: VM 実行ループの制御フロー（`execute_function`）で `ControlFlow::Return` を受け取った後の関数脱出が何らかの理由で無効化/上書き/再入している。
  - 着手案:
    - `execute_function` に短期ログ: 現在ブロックID/terminator種別/`should_return` セット→関数戻りの分岐をeprintln（NYASH_VM_DEBUG_EXEC=1時）
    - `execute_instruction` で `Return` ディスパッチ時に明示ログ（val_id/値）を出す（現状VTトレースも可）。
    - `previous_block`/`loop_executor`/`record_transition` で自己遷移が起きていないか確認。
    - `BasicBlock::add_instruction` にて terminator設定/Successorsの更新は正常（コード・MIR上はOK）。処理後の `next_block` 決定ロジックを再点検。

## ハンドオフ（変更点・補助情報）
- 追加ファイル（std MVP + smokes）
  - `apps/std/string.nyash`, `apps/std/array.nyash`
  - `apps/smokes/std/string_smoke.nyash`, `apps/smokes/std/array_smoke.nyash`
- スクリプト/設定の更新
  - `tools/jit_smoke.sh`: Std smokes に `timeout 15s`、ArrayBox未提供時は `SKIP` を出力
  - `tools/smoke_plugins.sh`: `NYASH_PLUGINS_STRICT=1` のON/OFF表示
  - `nyash.toml`: `ny_plugins` に std 2件を追加
  - `src/runner/modes/vm.rs`: `NYASH_VM_DUMP_MIR=1` でVM実行前にMIRをダンプ
  - `src/mir/builder/stmts.rs`: 末尾 `return/throw` 後に同ブロックへ更に命令を積まないための早期breakを追加（安全強化）
- 再現とログ
  - VM再現: `timeout 4s ./target/release/nyash --backend vm apps/tmp_if_min.nyash`
  - JIT対照: `./target/release/nyash apps/tmp_if_min.nyash`
  - MIRダンプ: `NYASH_VM_DUMP_MIR=1 --backend vm ...`
  - 命令トレース: `NYASH_VM_DEBUG_EXEC=1 --backend vm ...`
- プラグイン/ArrayBox注意
  - 既定でプラグイン経由に迂回するため、未ビルドだと ArrayBox 生成に失敗。
  - 回避: `NYASH_USE_PLUGIN_BUILTINS=0` または `NYASH_PLUGIN_OVERRIDE_TYPES` から `ArrayBox,MapBox`を除外。もしくはプラグインをビルド。

## すぐ着手できるTODO（VM側）
- [ ] `execute_function` にブロック遷移/Return検出ログ（NYASH_VM_DEBUG_EXEC=1時のみ）
- [ ] Return発生時に確実に `Ok(return_value)` で関数を抜けることを確認（`should_return`/`next_block` の上書き防止）
- [ ] `record_transition`/`loop_executor` の副作用で自己遷移が起きていないか確認
- [ ] 修正後、`apps/tmp_if_min.nyash` が VM/JIT 両方で一発終了することを確認（MIRダンプ上は既に正しい）
- B1: ArrayBox 経路の選択を明示
  - 手元では `NYASH_USE_PLUGIN_BUILTINS=0` で内蔵にフォールバックするか、プラグインを `cargo build -p nyash-array-plugin --release` で用意。
  - CIは当面 `SKIP` 維持。

## 実行メモ（暫定）
- Std smokes（手元で回す）
  - `NYASH_LOAD_NY_PLUGINS=1 NYASH_USE_PLUGIN_BUILTINS=0 ./tools/jit_smoke.sh`
  - またはプラグインをビルドしてから `NYASH_LOAD_NY_PLUGINS=1 ./tools/jit_smoke.sh`

■ 予定（R5 拡張: Ny Plugins → Namespace）
- Phase A（最小）: 共有レジストリ `NyModules` を追加し、`env.modules.set/get` で exports を登録/取得。
  - `[ny_plugins]` は戻り値（Map/StaticBox）を「ファイルパス→名前空間」に変換して登録。
  - 名前空間導出: ルート相対・区切りは `.`、拡張子除去・無効文字は `_`。予約 `nyashstd.*` 等は拒否。
- Phase B（範囲）: 共有Interpreterオプション（`NYASH_NY_PLUGINS_SHARED=1`）で静的定義を共有。ログに REGISTERED を出力。
- Phase C（言語結線）: `using <ns>` を `NyModules` 参照→未解決時にファイル/パッケージ解決（nyash.link）へフォールバック。

■ 直近で完了したこと（主要抜粋／JIT）
- R1: JSON v0 ブリッジ（`--ny-parser-pipe`/`--json-file`）、変換器 `src/runner/json_v0_bridge.rs`、スモーク追加
- R2: ラウンドトリップ E2E（`tools/ny_roundtrip_smoke.{sh,ps1}`）
- R3: 直結ブリッジ v0（`--parser ny`/`NYASH_USE_NY_PARSER=1`、`NYASH_DUMP_JSON_IR=1`）→ `return (1+2)*3` で 9
- R5: Ny スクリプトプラグイン（[ny_plugins]）列挙＋実行（OK/FAIL 出力・列挙のみガード付き）
  - NyModules登録/名前空間導出/Windows正規化の仕様確定・回帰スモーク
  - using/namespace（ゲート）・nyash.link最小・resolverキャッシュ・実行時フック（提案付き診断）
- AOT P2(step‑1): RUN スモーク配線（最小オブジェクト生成＋実行ログ）

- ■ 直近で完了したこと（主要抜粋）
- T0: MIRインタープリタ強化（分岐/比較/PHI/extern/Box最小）＋ Runner 観測ログ
- T1: Nyash製ミニパーサ（整数/四則/括弧/return）→ JSON IR v0 出力
- T2: JSON IR v0 → MIRModule 受け口（`--ny-parser-pipe`）
- T3: CLI 切替/ヘルプ（`--ny-parser-pipe`/`--json-file`、mirヘルプ追補）
- T4: Docs/Samples/Runner scripts（apps/ny-mir-samples, tools/*, README 追補）
- Phase 15 起点準備
  - CLIに `--backend cranelift-aot` と `--poc-const` を追加（プレースホルダ動作）。
  - `src/backend/cranelift/{mod.rs,aot_box.rs,linker_box.rs}` の雛形追加（feature gate）。
  - MIR解釈層スケルトン（`semantics/eval.rs` と `backend/mir_interpreter.rs`）の確認

■ 再開用クイックメモ（JITのみ）
- ビルド
  - VM/JIT: `cargo build --release --features cranelift-jit`
  - LLVM（必要時）: `LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) cargo build --release --features llvm`
  - AOT（導入後）: `cargo build --release --features cranelift-aot`
- スモーク（JIT/VM）
  - Core: `NYASH_DISABLE_PLUGINS=1 NYASH_CLI_VERBOSE=1 ./tools/smoke_vm_jit.sh`
  - Parser Bridge: `./tools/ny_parser_bridge_smoke.sh`
  - Roundtrip: `./tools/ny_roundtrip_smoke.sh`（A/B）
  - Plugins: `NYASH_SKIP_TOML_ENV=1 ./tools/smoke_plugins.sh`（厳格は `NYASH_PLUGINS_STRICT=1` 時のみON）
  - Bootstrap: `./tools/bootstrap_selfhost_smoke.sh`
  - Using/Resolver: `./tools/using_e2e_smoke.sh`

■ 状態
- JIT自己ホストMVP: 到達（E2E/ブートストラップ/ドキュメント/CI分離まで完了）
- リファクタ: Step1/2/3 完了（未使用掃除・runner分割・tools体裁統一）
- 次回は「std実装の実体化」と「Nyコンパイラsubset拡張」から再開
- 参照
  - Phase 15 概要/ロードマップ: `docs/development/roadmap/phases/phase-15/README.md`, `docs/development/roadmap/phases/phase-15/ROADMAP.md`
  - ハンドオフ: `docs/handoff/phase-15-handoff.md`
  - 設計/API: `docs/backend-cranelift-aot-design.md`, `docs/interfaces/*`

■ 合否基準（P0: Ny→MIR→MIR-Interp→VM 最小成立）
- 自作Nyashパーサ（最小サブセット）が Nyash で動作し、テスト入力から中間形式(JSON暫定)を生成できる。
- Runner が中間形式を MIRModule に変換し、MIR 解釈層で実行して既知の結果（例: `Result: 42`）を出力する。
- 代表ケース（整数四則演算/括弧/return）で往復が安定。

■ JSON IR v0（暫定スキーマ）
- version: 整数（例: 0）
- kind: 固定 "Program"
- body: 配列（Stmt[]）
- Stmt（最小）
  - { "type": "Return", "expr": Expr }
- Expr（最小）
  - { "type": "Int", "value": 123 }
  - { "type": "Binary", "op": "+"|"-"|"*"|"/", "lhs": Expr, "rhs": Expr }
- error（失敗時）
  - { "version":0, "kind":"Error", "error": { "message": "...", "span": {"start":N, "end":M} } }
- 例
  - `return 1+2*3` → {"version":0,"kind":"Program","body":[{"type":"Return","expr":{"type":"Binary","op":"+","lhs":{"type":"Int","value":1},"rhs":{"type":"Binary","op":"*","lhs":{"type":"Int","value":2},"rhs":{"type":"Int","value":3}}}}]}
  - `return (1+2)*3` → `Binary('*', Binary('+',1,2), 3)` の形で生成

■ 補足（優先/範囲）
- 先行するのは Ny→MIR→MIR-Interp→VM の自己ホスト経路（AOTはP2以降）。
- OS 優先: Windows →（後続で Linux/macOS）。
- メモリ/GC: P0は整数演算/定数返し中心でNyRT拡張不要。
- Codex 非同期運用: `tools/codex-async-notify.sh`／`tools/codex-keep-two.sh` 継続利用。

## 実行コマンド（サマリ）
- VM/JIT 実行例
  - `printf "Hello\n" | NYASH_CLI_VERBOSE=0 ./target/release/nyash apps/ny-echo/main.nyash`
  - `printf "Hello\n" | NYASH_CLI_VERBOSE=0 ./target/release/nyash --backend vm apps/ny-echo/main.nyash`
- AOT/LLVM 系は後段（当面OFF）

- JSON v0 ブリッジ（R1 Quick Start）
  - パイプ実行（Unix/WSL）: `printf '{"version":0,"kind":"Program","body":[{"type":"Return","expr":{"type":"Binary","op":"+","lhs":{"type":"Int","value":1},"rhs":{"type":"Binary","op":"*","lhs":{"type":"Int","value":2},"rhs":{"type":"Int","value":3}}}}]}' | ./target/release/nyash --ny-parser-pipe`
  - ファイル指定（Unix/WSL）: `./target/release/nyash --json-file sample.json`
  - スモーク（Unix/Windows）: `./tools/ny_parser_bridge_smoke.sh` / `pwsh -File tools/ny_parser_bridge_smoke.ps1`

- E2E ラウンドトリップ（R2）
  - Unix/WSL: `./tools/ny_roundtrip_smoke.sh`
  - Windows: `pwsh -File tools/ny_roundtrip_smoke.ps1`
  - tmux通知で並列実行（例）:
    - `CODEX_ASYNC_DETACH=1 ./tools/codex-async-notify.sh "./tools/ny_roundtrip_smoke.sh" codex`
    - `CODEX_ASYNC_DETACH=1 ./tools/codex-async-notify.sh "pwsh -File tools/ny_roundtrip_smoke.ps1" codex`

- Ny プラグイン列挙（R5）
  - 有効化: `--load-ny-plugins` または `NYASH_LOAD_NY_PLUGINS=1`
  - `nyash.toml` 例:
    ```toml
    ny_plugins = [
      "apps/std/ny-config.nyash",
      "apps/plugins/my-helper.nyash"
    ]
    ```
  - 実行: 列挙に加え、Interpreterで順次実行（ベストエフォート）。
  - ガード: `NYASH_NY_PLUGINS_LIST_ONLY=1` で列挙のみ（実行しない）
  - 注意: プラグインスクリプトは副作用の少ない初期化/登録処理に限定推奨。
  - Std Ny スモーク実行（任意）: `NYASH_LOAD_NY_PLUGINS=1 ./tools/jit_smoke.sh`

## トレース/環境変数（抜粋）
- AOT/Link: `NYASH_LINKER`, `NYASH_LINK_FLAGS`, `NYASH_LINK_VERBOSE`
- ABI: `NYASH_ABI_VTABLE=1`, `NYASH_ABI_STRICT=1`
- VM/JIT: `NYASH_VM_PIC_STATS`, `NYASH_JIT_DUMP` など従来通り

---
詳細な履歴や議事録は docs 配下の Phase 15 セクションを参照してください。
 - ny-config（R4）
  - `./target/release/nyash apps/std/ny-config.nyash`
  - 現状: Interpreter 経路のプラグイン初期化順序により FileBox/TOMLBox を使うには Runner 側の微調整が必要（VM 経路への移行 or プラグイン登録の早期化）。スクリプト本体は追加済み。
- 直結ブリッジ v0（R3 Quick Start）
  - `printf 'return (1+2)*3\n' > t.ny && NYASH_USE_NY_PARSER=1 NYASH_DUMP_JSON_IR=1 ./target/release/nyash t.ny`
