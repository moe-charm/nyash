# CURRENT TASK (Compact) — Phase 15 / Self-Hosting（Ny→MIR→MIR-Interp→VM 先行）

このドキュメントは「いま何をすれば良いか」を最小で共有するためのコンパクト版です。詳細は git 履歴と `docs/`（phase-15）を参照してください。

— 最終更新: 2025‑09‑06 (Phase 15.16 反映, AOT/JIT-AOT 足場強化)

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
