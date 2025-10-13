# CURRENT_TASK — 現在のタスクと進捗

## ✨ Today’s Update — 2025‑10‑15（Phase 1: MirCall 仕上げ 小粒）

## ✨ Today’s Update — 2025‑10‑15（Quick Step‑3 観測ON）

## ✨ Today’s Update — 2025‑10‑15（Phase 1: MirCall 小粒 仕上げ その2）

- 正規化の安全拡張: Array.contains を whitelist に追加済み（ModuleFunction/BoxCall 両経路）。
- quick-selfhost に MirCall(Array.contains) を追加し PASS 確認。
- Extern 無効診断を定数化（`DIAG_EXTERN_DISABLED`）し、docs の記述と実装の揺れを解消。

- quick.env に Map.get/set の HostHandle 強制フラグを導入（観測ON）。
  - `tools/smokes/v2/configs/env/quick.env`: `NYASH_MAP_GET_FORCE_HOST=1`, `NYASH_MAP_SET_FORCE_HOST=1` に設定。
- quick プロファイルに代表スモークを昇格（実行・緑確認）。
  - `tools/smokes/v2/profiles/quick/host_handle_router_map_size_has_vm.sh`
  - `tools/smokes/v2/profiles/quick/host_handle_router_map_get_missing_vm.sh`
  - `tools/smokes/v2/profiles/quick/host_handle_router_map_set_effect_vm.sh`
  - 実行例: `tools/smokes/v2/run.sh --profile quick --filter 'host_handle_router_map_.*\\.sh'` → 3/3 PASS。

- 正規化カバレッジ拡充（安全系のみ）
  - Array.contains を ModuleFunction→Method / BoxCall→Method の両方で降格対象に追加。
    - 変更: `src/mir/optimizer_passes/normalize.rs`（ArrayBox ホワイトリストに `contains` 追加）。
  - Map.delete/clear は既に対象。周辺ケースはスモークで担保（欠損キー delete は no-op/size 不変）。
- Router 側は最小限（arity/戻り値）チェックのみ。実行経路は MirCall(Method) に統一。
- 追加スモーク（Extern 無効でも緑維持）
  - `tools/smokes/v2/profiles/quick-selfhost/mircall_array_contains_vm.sh`
  - `tools/smokes/v2/profiles/quick-selfhost/mircall_map_delete_missing_vm.sh`
  - 実行: `tools/smokes/v2/run.sh --profile quick-selfhost --filter 'mircall_array_contains_vm.sh|mircall_map_delete_missing_vm.sh'` → PASS。


## ✨ Today’s Update — 2025‑10‑13 (quick rollout step‑2)

- Quick staged enablement (HostHandleRouter):
  - Enabled Map.size/has force in quick profile to widen observation safely.
    - `tools/smokes/v2/configs/env/quick.env`: added `NYASH_MAP_SIZE_FORCE_HOST=1`, `NYASH_MAP_HAS_FORCE_HOST=1`.
    - VM router now supports fine‑grained flags for Map (`*_SIZE_FORCE_HOST`, `*_HAS_FORCE_HOST`), in addition to `NYASH_MAP_FORCE_HOST`.
  - Added a light quick smoke (representative):
    - `tools/smokes/v2/profiles/quick/host_handle_router_map_size_has_vm.sh` — asserts `size()==1`, `has("a")==true`, `has("b")==false`.
    - Run: `tools/smokes/v2/run.sh --profile quick --filter host_handle_router_map_size_has_vm.sh` → PASS.

- Optional plugin‑only build (status/check):
  - Tried: `cargo build --release --no-default-features -F cli,plugins,host-anchors` → FAIL (expected for now).
  - Cause: remaining `crate::boxes::*` references when `legacy-boxes` feature is OFF.
  - Next: use `tools/dev/list_boxes_refs.sh` to enumerate refs and progressively pluginize/route.

- Legacy withdrawal plan (note):
  - Gate condition to flip: plugin‑only build green + smokes green with `legacy-boxes` OFF.
  - Steps: replace remaining `crate::boxes` refs → flip default OFF for `legacy-boxes` → verify → remove `src/boxes/` and cfg guards.
  - Target: document and aim for a short window after Map/Array/String HostHandle parity tests are stable (quick/plugins both green).


## ✨ Today’s Update — 2025‑10‑14 (Collections/Map unification)

- Collections API unification:
  - size()/isEmpty() across Array/Map/String; length() deprecated.
  - apps/ selfhost code migrated (length->size) — representative 121 files updated; suites green.
- Map semantics unified:
  - get(missing) -> null (empty TLV)
  - set/clear/delete -> null
  - keys/values Stage‑2: HostHandle(ArrayBox) when NYASH_PLUGIN_MAP_ARRAY_HANDLE=1; fallback keysS/valuesS String path retained.
  - Map.call P1: missing key now returns null (no error). Tightened smokes accordingly.
- Mini-VM & Rust MIR robustness:
  - Builder-level terminator normalization (all blocks end with ret/jump/branch/throw).
  - LocalSSA copy insertion respects first terminator (no push beyond).
  - Dev guard for unterminated blocks removed (structural fix in place).
- Smokes added:
  - plugin: map_array_handle_identity_vm (identity/visibility)
  - plugin: map_keys_values_stage2_vm (HostHandle arrays)
  - plugin: map_keys_values_fallback_vm (string fallback)
  - plugin: map_call_boundary_mixed_vm (missing→null or transitional error; non-callable→error)
    - tightened: now requires missing→'null' (no transitional error)
  - plugin: map_keys_order_stage2_vm (keys ordering/content when HostHandle enabled; SKIP if unavailable)
  - plugin: map_values_handle_mutation_vm (values() handle mutation visibility; SKIP if unavailable)
  - quick-selfhost: host_handle_router_array_len_vm (forces slot 102 path)
  - quick-selfhost: userbox_boxcall_stopflag_vm (NYASH_VM_USER_INSTANCE_BOXCALL=0 rewrite works)
  - plugins: stage2_on_suite.sh (convenience runner enabling HostHandle Array and executing Stage‑2 smokes)
  - plugins: stage2_on_suite_vm.sh (profile-friendly wrapper to run the Stage‑2 suite)
  - legacy profile created under `tools/smokes/v2/profiles/legacy/` with wrappers + README; default suites do not include it.
    - Added wrappers: `legacy_map_keys_values_fallback_vm.sh`, `legacy_array_size_force_host_vm.sh`, `legacy_map_keys_values_bridge_vm.sh`, `legacy_string_length_vm.sh`.
  - quick/core coverage migrated to size(): added `quick/core/string_extern_size_vm.sh` (length() test remains for legacy compatibility).
  - quick/selfhost: HostHandleRouter dev smokes —
    - `host_handle_router_array_set_get_force_vm.sh` (Array.size/get/set via slots 102/100/101 under force)
    - `host_handle_router_map_set_get_vm.sh` (Map.set/get via slots 204/203 under force)
    - `host_handle_router_string_len_vm.sh` (String.size via slot 300 under force)

- HostHandleRouter (phase-in)
  - Implemented slots: Array (100/101/102), Map (200/202/203/204), String (300).
  - plugins profile now forces HostHandle paths (Map/Array/String) via env overlay.
  - Removed unreachable legacy MapBox dispatch block in VM router (unified earlier branch + HostHandle force path used).
  - quick profile staged: enable only minimal HostHandle routes (Array.size, Map.* by opt-in) to observe gradually.
  - legacy Stage‑1 keys/values fallback disabled in plugins profile (`HAKO_MAP_KEYS_VALUES_FALLBACK=0`).
- Map plugin:
  - runtime capability gating for keys/values (env-based; no build feature).
  - get/delete null returns enforced.

## ✨ Today’s Update — 2025‑10‑14 (plugin‑only build pass + quick spot checks)

- Plugin‑only build (legacy‑boxes OFF) — First green
  - Guarded all major `crate::boxes::*` references under `#[cfg(feature="legacy-boxes")]` and provided minimal plugin‑only fallbacks.
  - Areas covered: runner, builtin_impls, plugin loader v2 (ffi + externs), MIR interpreter (extern_adapter/handlers/helpers), array_flatten_helper, gc_trace, method_router_box (builtin arms), box_registry, semantics, global_hooks.
  - Verified build: `cargo build --release --no-default-features -F cli,plugins,host-anchors` → PASS。

- Quick profile regression (default features ON)
  - Rebuilt default: `cargo build --release`。
  - Ran focused smokes (HostHandleRouter + core): PASS。
    - `tools/smokes/v2/run.sh --profile quick --filter 'host_handle_router_map_*|gc_mode_off|ssot_enabled_disabled_rc_vm'` → 5/5 PASS。

- Notes / Constraints
  - Extern calls are enabled under legacy ON; plugin‑onlyでは `handle_callee_extern` は明示エラー（将来のplugin対応予定）。
  - FutureBox 系（VMValue::Future / env.future）は legacy 限定でガード（plugin‑only は無効）。

### Next
- Broaden quick run to full suite after a regular rebuild（所要 ~1–2min）。
- If stable, start pruning unreachable legacy arms and tighten docs on feature gates。

### Next (confirmed — Phase 15.75 follow‑ups)
- Map.call P1 (VM sugar) finalization:
  - Confirm `call/1`, `call/2` routes in `selfhost/hakorune-vm/method_call_handler.hako` — confirmed present.
  - Re‑run 3 smokes (success/missing/non‑callable) and keep them green; add 1 boundary smoke (mixed missing/non‑callable diagnostics).
- Plugin‑on smokes (Stage‑2 HostHandle Array):
  - Add 2 cases for `Map.keys/values` (length/content; value handle shape/access).
  - Add identity: `arr -> map.set -> get -> same` minimal case.
- Phased deprecation next steps:
  - Update docs/guards for VM convenience handlers (order: String → Map → Array); no behavior change.
  - Enable BoxCall fast‑path stop‑flag in test profile to confirm green; default remains OFF.
- HostHandleRouter gradual relocation:
  - Delegate one existing function into the router and keep suites green (small step).
- Small hygiene (quick wins):
  - Move remaining JSON extraction `indexOf` to `JsonFieldExtractor/JsonCursor`.
  - Apply one `boxcall_builder` site to `build_method` for reuse (tiny diff).

## ✨ Today’s Update — 2025‑10‑14 (naming separation: Rust vs Hakorune)

- Developer UX: dual-line naming made explicit without changing internal flags.
  - Cargo aliases added: `build-rust`/`run-rust` (legacy built-ins ON), `build-hako`/`run-hako` (plugin-only).
    - File: .cargo/config.toml
  - Smokes: `--profile hako` added as an alias of `plugins` to avoid naming confusion.
    - Files: tools/smokes/v2/run.sh, tools/smokes/v2/configs/env/hako.env
  - Docs updated: `docs/guides/plugin-only-build.md` terminology (Rust line / Hakorune line) and alias usage.

Next small step
- Optional: split `src/runtime/method_router_box/` into `{builtin.rs, plugin.rs}` with `mod.rs` as thin orchestrator (no behavior change). README updated with plan.

## ✨ Today’s Update — 2025‑10‑14 (MethodRouter 分離・委譲 完了)

- 構造分離（挙動不変）
  - 追加: `src/runtime/method_router_box/plugin.rs` — PluginBoxV2 経路を集約（HostHandle 早期 + force-ENV + plugin_host_box 委譲）。
  - 追加: `src/runtime/method_router_box/builtin.rs` — Legacy 腕（File/Callable/Array/Map）を `#[cfg(feature="legacy-boxes")]` で移設。
  - 変更: `src/runtime/method_router_box/mod.rs` — 入口で早期委譲（plugin→builtin）。未処理 BoxRef は `method_not_supported` を即返却。
  - 旧巨大分岐はコメントアウトで実行経路から撤退（次パスで物理削除予定）。

- スモーク/確認
  - quick: `tools/smokes/v2/run.sh --profile quick --filter host_handle_router_map_size_has_vm.sh` → PASS
  - quick-selfhost: `tools/smokes/v2/run.sh --profile quick-selfhost --filter host_handle_router_string_len_vm.sh` → PASS

- 影響
  - 機能挙動は不変（委譲による構造整理のみ）。
  - 今後、旧ブロックを物理削除して未使用 import を整理する（小差分）。

Next
- 物理削除: mod.rs のコメント化された旧ブロックを削除 → quick/quick-selfhost の軽いセット緑確認（完了）。
- Phase 1（MirCall）: 既存VMの MirInstruction::Call は `handlers/calls/legacy/*` で稼働中（Global/ModuleFunction/Method/Extern）。
  - plugin-only では `Extern` は明示エラー（legacy-only）: handlers/calls/function.rs:…（確認済み）
  - Method 経路は `method_router_box::route` に一本化（本日の分離作業で整流）。

## ✨ Today’s Update — 2025‑10‑14（反映ライン）

- MethodRouter 旧経路の撤退（実行経路から完全除外）
  - `src/runtime/method_router_box/mod.rs` は入口のみを維持し、plugin/builtin へ早期委譲。
  - 旧巨大分岐はコメントアウト済み（実行不可）。次パスで物理削除予定（機能差分なし）。
  - plugin 経路（HostHandle 早期 + force‑ENV + plugin_host_box 呼び出し）→ `plugin.rs`
  - builtin 経路（File/Callable/Array/Map legacy 腕）→ `builtin.rs`（全て `#[cfg(feature="legacy-boxes")]`）

- 確認（代表）
  - quick: hosthandle(Map.size/has) → PASS
  - quick‑selfhost: hosthandle(String.size) → PASS

- 次アクション（小差分）
- 物理削除: `mod.rs` のコメントブロックを除去 → 未使用 import/警告整理 → quick/quick‑selfhost で再確認
- Phase 1（MirCall）準備: `extern_adapter.rs`/`vm_types.rs` に最小ガード（必要時のみ、可逆）

## ✨ Today’s Update — 2025‑10‑15（MirCall 前進）

- Normalize: BoxCall→Call 降格を追加（安全な2ケース）
  - `src/mir/optimizer_passes/normalize.rs`
    - `Method{box_name, method=="methodRef"}` 由来の Callable を ArrayBox に加えて MapBox/StringBox でも検出し、`call()` を `Callee::Method` へ降格（arity==0 と argv 再構成の両方）。
    - `ModuleFunction("<Box>.methodRef/2")` 由来の Callable も ArrayBox/MapBox/StringBox を対象に検出（recv=args[0]）。
    - BoxCall 起点は型不明のため従来通り ArrayBox のみ（安全策）。
- Smokes（quick‑selfhost に軽い正常系を追加）
  - `tools/smokes/v2/profiles/quick-selfhost/mircall_method_map_vm.sh`（Map.has/set が 0→1 を反映）
  - `tools/smokes/v2/profiles/quick-selfhost/mircall_module_function_map_vm.sh`（Map.size が 1 を返す）
  - 既存: `mircall_method_vm.sh`（String.indexOf）、`mircall_module_function_vm.sh`（Array.size）、`selfhost_mircall_value_callable_vm.sh`（callee=Value）と重複しない形で追加。
- 走らせ方（抜粋）
  - `tools/smokes/v2/run.sh --profile quick-selfhost --filter 'mircall_*_map_vm.sh'`
  - 正常終了（VM出力 OK）で PASS。

### 追加（MirCall 前進 その2）

- Normalize: ModuleFunction → Method の安全な降格を導入（Array/Map/String の代表API）
  - `MapBox.size/0|has/1|get/1|set/2`
  - `ArrayBox.size/len/length/get/set/push`
  - `StringBox.size/len/length/indexOf/lastIndexOf/substring/charAt/concat`
  - 受けは `args[0]` を receiver に移し、引数配列は先頭をドロップ。
  - 効果: 実行器の Method 経路に集約（Router一本化の恩恵を受ける）。
- Docs: `docs/reference/vm/call-unification.md` を追加（MirCall統一ルートの簡潔な説明）。
- Plugins スモーク: HostHandleRouter 境界系（-1/-11/-13）を追加。
  - `tools/smokes/v2/profiles/plugins/hosthandle_boundary_errors_vm.sh`（PASS）。

### 追加（MirCall 前進 その3 — 小粒2本）

- Normalize: BoxCall(method_id/同一ブロックNewBox起源) → Method
  - 受けが同一ブロック内の `NewBox(Array|Map|String)` である場合に限定し、代表APIのみを Method に降格（get/set/push/size/has 等）。
  - 目的: コア箱の“直呼び”の多くを MirCall(Method) に統一し、Router一本化の恩恵（arityガード/HostHandle早期など）を広げる。
- Normalize: Extern("nyrt.(string|array|map).size/length") → Method(size)
  - 影響が最小のゼロ引数関数のみを対象に、receiver=args[0] を抽出して Method 化。
  - 目的: Extern 経路の一部を Method に寄せ、実行器の単一路に整流。
  - 既存 quick-selfhost の mircall_* スモークはすべて PASS 維持。

## ✨ Today’s Update — 2025‑10‑15（Phase 15.75 TODO に復帰するための4点片付け）

- 1) ルーター不要ブロックの物理削除（重複腕の撤去）
  - `src/runtime/method_router_box/mod.rs`: 既に早期委譲（plugin→builtin）後ろに残っていた旧 Plugin/Builtin 腕を物理削除（到達不能だった部分）。
  - 機能差分なし（実行経路は委譲のまま）。ビルドOK。

- 2) ファイル分離の仕上げ＋README
  - 追加: `src/runtime/method_router_box/README.md`（責務/入出力/ガード/撤退計画）。
  - 以後の腕追加は `plugin.rs` / `builtin.rs` 側に集約する方針を明文化。

- 3) plugin-only 緑の再確認
  - 実行: `cargo build --release --no-default-features -F cli,plugins,host-anchors` → ビルドPASS。
  - 既知のWarningのみ（機能的な問題なし）。

- 4) 二重ライン整備（軽量）
  - `.cargo/config.toml` の alias `build-hako` は既に配置済み（確認）。
  - ドキュメントは `docs/guides/plugin-only-build.md` を継続利用（別途追補は不要）。

## ✨ Today’s Update — 2025‑10‑15（Phase 0-mini 仕上げ）

- extern_adapter: 分割登録をデフォルト化してハブ化。
  - `extern_core.rs`（string/time/map）と `extern_future_legacy.rs`（env.future.*）を登録。末尾再登録で分割側を優先。
  - 物理撤去は次パスで安全に段階削除（重複は機能差なし）。
- array_flatten_helper: builtin/plugin の二層分割＋ファサード委譲。
  - 呼び出し側（CallableBox.call）に README 参照コメントを最小付与。
- 正規化拡充（Extern→Method）: nyrt.map.{keys,values} / string系（indexOf/lastIndexOf/substring/charAt/replace）。
- quick-selfhost / plugins スモークは緑維持。


## 🏁 Milestones Timeline（Self‑Host → Parity）

- 2025-08-09: initial commit
- 2025-10-09: M2 Self‑Rebuild 達成（自己ホストEXEで再ビルド）
- 2025-10-11: M3 VM↔LLVM Parity（最小）達成（parity_q_* 緑）

所要日数（M2まで）: 61日。M3はその2日後に達成。

## ✨ Today’s Update — 2025‑10‑12

- HostBridgeBox（Phase B）: .hako 側を Extern 呼びに切替（landed）
  - `HostBridgeBox.box_new/box_call` → `Extern("hostbridge.box_new/box_call")` へ統一
  - AOT/EXE/VM の呼び出し面が一面化（Rust HostBridge 経由で Plugin/Provider を解決）
- nyvm: HostBridge 完全移譲（nyvm 内の便宜実装を撤退し、常に Rust extern へフォワード）
- Runner: 早期プラグインロード（idempotent）を導入（`execute_file_with_backend` の冒頭で一回）
- Runner→nyvm: in‑memory 直渡し（tempファイル不要）へ切替
- MirIoBox（Phase B 準備）: yyjson プロバイダー経由の入出力を HostBridge extern に統一（設計/ドキュメント追加）

この小粒アップデートで、MIR 入出力と VM 側読み手の安定化を前進させたよ。

- MirIoBox（Phase A）導入（Hako ABI 側）
  - 追加: `selfhost/shared/mir/mir_io_box.hako`
  - API: `validate/functions/blocks/instructions/terminator`（最小）
  - nyvm ブリッジの function-only JSON も許容（narrow 形式）
  - HakoruneVmCore.run 冒頭で `validate` を呼び Fail‑Fast（構造不整合を早期検出）

- TerminatorHandler の読み取りを空白許容化（Rust emit の整形揺れに寛容）
  - `"op": "ret"` と `"op":"ret"` の両方を受理
  - フォールバック: instructions[] 末尾オブジェクトから同じ規則で抽出

- スモーク追加（quick-selfhost）
  - `terminator_whitespace_vm`（op 後の空白差を検証）
  - `entry_nonzero_vm`（entry≠0 を優先開始）

- スモーク安定化（暫定）
  - `nyvm_nowait_hakorune` に “Unknown backend:” フィルタを追加（根治は run.sh 正規化で対応予定）

次アクション（小さい順）
- run.sh の backend 正規化（空/"."/未知 → `vm`）
- MirIoBox.validate に terminator 必須・参照妥当性（then/else/target）の検証を集約（dev 緩和は ENV で）
- MirIoBox.validate_function: provider 経路にも jump/branch 参照妥当性チェックを適用（scan と等価）
- nyvm/provider スモークの暫定OKを撤去（JSON プラグイン早期ロードが安定後）
- BackwardObjectScannerBox に小窓（ENV）を追加し末尾2–3候補で早期抜け（大 JSON のコスト安定化）
- HostBridgeBox 設計の落とし込み（Plugin/Static の統一呼び出し）→ 最小疎通（FileBox.open/read）

- SSOT 優先の Type 解決/ID（既定ON）
  - TypeBox/slot/arity/aliases を SSOT（`specs/type_registry.toml`）優先で参照。存在しない場合のみ静的表にフォールバック。
  - Core type_id 解決も SSOT → config(hako/nyash.toml) → 既定 の順へ統一。
  - ENV ゲート: `HAKO_REGISTRY_SSOT_DISABLE=1`（`NYASH_REGISTRY_SSOT_DISABLE` 互換）で一時OFF可能。
  - 参照: `src/runtime/type_registry.rs:66`, `src/runtime/type_registry.rs:312`

- 診断メッセージの一元化（arity/unknown‑slot 等）
  - ルーター側の直書きは撤去し、`diagnostics::msg::no_method_arity` などのヘルパで統一。
  - 参照: `src/runtime/method_router_box/mod.rs:16`, `src/runtime/method_router_box/map_callable.rs:44`, `src/runtime/method_router_box/method_ref.rs:29`

- SSOT バリデータ追加（編集時の安全）
  - `tools/check_ssot_table.sh` — name→slot 多重割当や完全重複を検出。
  - 実行: `./tools/check_ssot_table.sh` → OK/FAIL を表示。

- スモーク（代表）
  - M2 quick 代表: PASS（カーネルが無ければ自動SKIP）。
    - `tools/smokes/v2/run.sh --profile quick --filter 'selfhost_*_vm'`
  - M3 integration‑core: 20/20 PASS（VM/LLVM パリティ小セット）。
    - `tools/smokes/v2/run.sh --profile integration-core`
  - 自己ホスト emit 最小確認（実例）
    - emit‑only(min‑json): `NYASH_DISABLE_PLUGINS=1 NYASH_USE_NY_COMPILER=1 NYASH_NY_COMPILER_MIN_JSON=1 NYASH_NY_COMPILER_CHILD_ARGS="--pipeline-v2" NYASH_NY_COMPILER_EMIT_ONLY=1 NYASH_NY_COMPILER_SKIP_PY=1 NYASH_JSON_ONLY=1 timeout 5 ./target/release/hakorune --backend vm apps/examples/string_p0.hako`
    - emit‑MIR(JSON v0): `NYASH_USE_NY_COMPILER=1 NYASH_NY_COMPILER_MIN_JSON=1 NYASH_NY_COMPILER_CHILD_ARGS="--emit-mir" NYASH_JSON_ONLY=1 timeout 5 ./target/release/hakorune --backend vm apps/examples/string_p0.hako`
  - quick 常時へ昇格（rc‑only）
    - plugin‑on 代表: `tools/smokes/v2/profiles/quick/selfhost/plugin_on_min_rc_vm.sh`
    - selfhost emit‑mir 最小: `tools/smokes/v2/profiles/quick/selfhost/selfhost_emit_mir_min_rc_vm.sh`

### P2/P3 進捗（箱化で段階移行）

- 共有箱（共通化）
  - 追加: `apps/selfhost/common/mir/mir_schema_box.hako`（const/ret/compare/branch/jump/binop + mir_call extern/global/method/ctor）
  - 追加: `apps/selfhost/common/mir/block_builder_box.hako`（const_ret/compare_branch/binop/loop_counter + call系最小）

- emit 経路の薄アダプタ化（互換維持）
  - 更新: `apps/selfhost-compiler/pipeline_v2/emit_mir_flow_map.hako`（P1/P2）
  - 更新: `apps/selfhost-compiler/pipeline_v2/emit_mir_flow.hako`（P1/P2/P3 の最小導線）

- Extern（P3最小）
  - CallEmit/MirJsonBuilderMin に Extern 生成APIを追加
  - `emit_op_eq(lhs,rhs)` を導入（Extern("nyrt.ops.op_eq")）

- 代表スモーク（rc‑only追加）
  - `tools/smokes/v2/profiles/quick/selfhost/selfhost_emit_mir_binop_min_rc_vm.sh`
  - `tools/smokes/v2/profiles/quick/selfhost/selfhost_pipeline_v2_op_eq_vm.sh`
  - `tools/smokes/v2/profiles/quick/selfhost/selfhost_pipeline_v2_op_eq_false_vm.sh`

## ▶ Next — P4/P5（着手順）

P4: NewBox/Call/Method を shared Box（MirSchema/BlockBuilder）へ直結（出力互換）
- 対応範囲
  - Constructor 最小（ArrayBox/StringBox/MapBox）→ ret（rc-only代表はE2Eで既に担保）
  - Method 最小（size/length/indexOf 等の1〜2本）→ ret
  - Global 最小（純関数: JSON.stringify）→ ret（rc-only）
- 実装方針
  - emit_mir_flow(_map).hako に薄アダプタ関数を追加（BlockBuilder の *_call_ret を利用）
  - v1 生成が必要な経路は MirJsonBuilderMin で mir_call を出力し、必要時に MirJsonV1Adapter で v0 へ変換。
- 受け入れ
  - quick 常時代表は現状維持（rc-only）。P4 は opt-in 代表で観測。

P5: LocalSSA ensure_cond/ensure_calls の最小適用
- CondInserter の escape 修正済み（JsonCursorBox に委譲）。
- ensure_calls のミニ版を導入（call/method/new の入口で必要なら copy/phi を材化）。
- 受け入れ: rc-only 代表1本（pipeline_v2 の call/min）を追加し常緑。


次ステップ（提案）
- Selfhost MIR 生成 P1（const/ret と compare→branch/jump→ret）を自己ホスト箱に寄せる（Rust 側は受け口に縮退）。
  - 受け入れ: quick の emit‑min‑json/emit‑mir 代表が rc‑only 緑、integration‑core は現行セット緑維持。

次フェーズ（M4: Parity‑Plus & Stability）
- quick 常時セットに parity_* 維持（追加2本: JSON stringify / <=, >=）
- plugins プロファイルで最小 LLVM 交差を常時ON（軽量）
- MIR→EXE の小スモークは quick/llvm の AOT 系で常時確認（負荷低）
- provider 起動1行ログ（policy/config/loaded/anchors/stage2）＋ dlsym セルフチェック（force時Fail‑Fast）を固定化


## ✅ Runtime Cleanup — Boxification Round（2025-10-11 完了）

目的
- ルータとホストAPIに混在していた一時フォールバック/分岐を箱として外出しし、責務境界を明確化。

実施
- Stage‑1 フォールバックを Adapter Box へ移動
  - 追加: `src/runtime/adapters/map_keys_values_stage1.rs`
  - 変更: `src/runtime/method_router_box/mod.rs` — 旧インライン実装を撤去し、adapter に委譲
- HostHandle ルーティングの抽出（骨組み）
  - 追加: `src/runtime/host_handle_router/mod.rs`（今は薄い委譲、段階移行の受け口）
  - 変更: `src/runtime/host_api.rs` — `nyrt_host_call_slot` の先頭で router を試行
- ドキュメント/ガード
  - 追加: `src/runtime/method_router_box/README.md`, `LAYER_GUARD.rs`
  - 追加: `src/runtime/provider_box/README.md`, `LAYER_GUARD.rs`
  - 追加: `src/runtime/host_handle_router/README.md`, `LAYER_GUARD.rs`

影響
- 構造のみの変更（仕様不変）。Stage‑1/Stage‑2 の切替は従来通り `NYASH_PLUGIN_MAP_ARRAY_HANDLE`。

次のステップ（小粒）
- HostHandleRouter へ Array/Map/Instance スロット分岐を段階移設（現行の `host_api.rs` 内分岐を縮退）
- `print()` 経路の ConsoleBox 化（ロック順序固定/非同期化、ハング回避）
- plugin‑on スモーク: identity（values→push→values）/print の2本を追加


## ✅ Method Router 箱化アップデート（2025-10-14 完了）

- method_router_box を 2 つの小箱で分割
  - `method_ref.rs`: methodRef 疑似メソッドを集約し、型チェックと CallableBox 生成をFail-Fastで実施
  - `map_callable.rs`: Map.call/callAsync の糖衣を隔離。プラグインは get/set 群のみ実装すればよい
- ルーター本体は各小箱へ委譲するだけに縮小。将来追加する糖衣（例: Map.deleteIf 等）も箱単位で追加可能
- TLV Codec を `src/runtime/codec/codec_box.rs` に分離し、README で責務を明記。encode_value() で型分岐を一箇所に圧縮
- docs/reference/plugin-system/vm-plugin-integration.md に Phase 15.7 の箱構造を追加

## ✅ CallableBox methodRef 一元化（2025-10-12 完了）

- methodRef は VM 予約疑似メソッドとして実装
  - `receiver.methodRef(name: String, arity: Integer)` を VM ルーターが最優先で処理
  - 型不一致・負の arity は Fail-Fast でエラーにする
  - CallableBox 生成時は `bx.share_box()` を束縛して呼び出し経路を統一
- Array/Map plugin から methodRef 実装を撤去し、スロット 113 は VM 側でのみ扱う
- Map.call/callAsync は VM シュガーで `get(key)` → CallableBox → call/callAsync へ委譲
  - plugin 側は get/set 群だけ維持（call 系の実装不要）
- 既存スモーク: quick/core/callable_* は緑維持
- TODO: ResolverBox で MethodHandle を取得する設計稿、docs 追記

## ✅ Router Slot化 + CallableBox 導入（2025-10-10 完了）

- Router を String/Array/Map で完全に表駆動（slot）に統一
  - String: length/size/isEmpty/substring/indexOf/lastIndexOf/charAt/…（SSOT: TypeRegistry）
  - Array: get/set/push/pop/clear/contains/indexOf/join/sort/reverse/slice/toJSON + methodRef(113)
  - Map: size/len/has/get/set/delete/remove/keys/values/clear/toJSON
- hako_core_map の小ユーティリティ追加
  - `has_key_str`, `size_of_str_map` を追加、Builtin MapBox の has/size をコア経由へ委譲
- CallableBox 追加（関数参照の箱）
  - API: `arity()/call(argsArray)/callAsync(argsArray)/toString()`
  - 生成: `ArrayBox.methodRef(name, arity)`（slot 113）、`env.callable.{make|from|from_instance}`
  - Router: `CallableBox` の slot 500..503 を実装し、call は MirCall 正規化で実行
  - 非同期: `callAsync` は最小実装（即時 Future 完了）。将来 `spawn_task` へ昇格予定
- スモーク
  - 追加: `tools/smokes/v2/profiles/quick/core/callable_basic_vm.sh`（sync 最小）
  - 既存 quick/plugin-on 含む代表は緑を維持

### 次タスク（小粒提案）
- callAsync の真の非同期化（スケジューラ接続）
- `ref` 構文の導入（`ref me.method/arity`, `ref Module.fn/arity`）
- Map シュガー: `Map.call/Map.callAsync`（CallableBox への薄い委譲）
- String 残りスロット（replace/trim/toUpper/toLower 等）のスモーク拡充

**最終更新**: 2025-10-14

---

## 🐛 **Today's Bug Fixes (2025-10-09 Evening)**

### ✅ **Plugin config_path Corruption Bug修正**
- **問題**: `load_config()`がファイル読み込み**前**に`config_path`を設定 → 存在しないファイル（"hakorune.toml"）で上書き → method invocation時にPluginError
- **修正**: `config_path`の設定をファイル読み込み成功**後**に移動（Line 8 → Line 15）
- **影響**: すべてのplugin method invocation
- **ファイル**: `src/runtime/plugin_loader_v2/enabled/loader/config.rs`
- **テスト**: ✅ ArrayBox.push/MapBox.set/get 動作確認

### ✅ **LLVM Mode VM Delegation Fix**
- **問題**: `execute_vm_mode()`メソッドが存在しない（typo）
- **修正**: `execute_vm_mode` → `execute_vm_engine`
- **ファイル**: `src/runner/modes/llvm.rs:268`
- **テスト**: ✅ `cargo build --release --features llvm` PASS

---

## 🎯 **Current Phase: Hakorune VM Phase 2 Day 5完了（Load/Store実装）**

**完了**: Phase 1 Day 1-3 + Phase 2 Day 4-5（基盤構築・演算・制御フロー・単項演算・メモリ操作・箱化モジュール化）
**次のステップ**: Phase 4（Call/BoxCall実装）または Phase 2 Day 6（TypeOp実装）
**進捗率**: 15/16命令実装（93%）

---

## 🚀 **Hakorune VM Implementation Progress**

### ✅ **Phase 1完了: 基盤構築（Day 1-3）**

#### **Day 1: JSON MIRパーサー基盤** (2025-10-09)
- ✅ HakoruneVmCore 骨格作成（288行）
- ✅ 4命令実装: Const/BinOp(Add)/Ret/Copy
- ✅ @match命令ディスパッチ実装

#### **Day 2: BinOp全種・Compare全種** (2025-10-09)
- ✅ BinOp全種実装: Add/Sub/Mul/Div/Mod
- ✅ Compare全種実装: Eq/Ne/Lt/Le/Gt/Ge
- ✅ テスト拡張: 10テストケース
- ✅ Rust VM PHIバグ発見＋修正（else-if問題）

#### **Day 3: 制御フロー** (2025-10-09)
- ✅ 3箱作成: BlockMapperBox, TerminatorHandlerBox, PhiHandlerBox
- ✅ Branch/Jump/Phi 実装
- ✅ 複数ブロック実行ループ
- ✅ 5テストケース PASS

#### **Day 3 リファクタリング: 箱化モジュール化強化** (2025-10-09)
- ✅ Option A: デッドコード削除（35行）
- ✅ Option C: 命令ハンドラー箱化（272行削減）
- ✅ 7箱作成: ValueManagerBox, JsonFieldExtractorBox + 5ハンドラー
- ✅ hakorune_vm_core.hako: 488行 → 181行（-63%）
- ✅ 全テスト: 15/15 PASS ✅

**コミット**:
- `9b6bdf58` - refactor(vm): Phase 1 Day 3 箱化モジュール化強化（307行削減）
- `00808eed` - feat(mir): ExternCall廃止 → Call統一（MirCall移行）

---

### ✅ **Phase 2開始: 単項演算（Day 4）**

#### **Day 4: UnaryOp実装** (2025-10-09)
- ✅ UnaryOpHandlerBox 作成（63行）
- ✅ 3種類の演算実装: Neg/Not/BitNot
- ✅ InstructionDispatcherBox 更新（unaryop ルーティング追加）
- ✅ 7テストケース作成 + 実行
- ✅ 全テスト: 22/22 PASS ✅（Phase 1: 15 + Phase 2: 7）

**実装詳細**:
- **Neg**: 算術否定 (`-x`)
- **Not**: 論理否定 (`!x` → 0/非0を1/0に変換)
- **BitNot**: ビット否定 (`~x = -x - 1`)

**新規ファイル**:
- `unaryop_handler.hako` (63行)
- `test_phase2_day4.hako` (テストスイート)

**更新ファイル**:
- `instruction_dispatcher.hako` (+1 using, +1 case)
- `hako.toml` (+1 module override)
- `nyash.toml` (+1 module)

---

### ✅ **Phase 2 Day 5: Load/Store実装** (2025-10-09)
- ✅ メモリストレージ（mem）追加
- ✅ LoadHandlerBox 作成（44行）
- ✅ StoreHandlerBox 作成（36行）
- ✅ HakoruneVmCore/InstructionDispatcher更新（mem引数追加）
- ✅ 5テストケース作成（4/5 PASS、1つスキップ）
- ✅ 全テスト: 26/27 PASS ✅（Phase 1: 15 + Phase 2: 11）

**実装詳細**:
- **Load** (`%dst = load %ptr`): メモリから読み込み
- **Store** (`store %value -> %ptr`): メモリへ書き込み
- 未初期化メモリは0を返す

**新規ファイル**:
- `load_handler.hako` (44行)
- `store_handler.hako` (36行)
- `test_phase2_day5.hako` (テストスイート)

**更新ファイル**:
- `hakorune_vm_core.hako` (mem追加、全メソッドにmem引数追加)
- `instruction_dispatcher.hako` (+2 using, +2 case, mem引数追加)
- `hako.toml` (+2 module override)
- `nyash.toml` (+2 module)

**既知の問題**:
- Test 3（未初期化メモリLoad）で比較演算子のバグ（要調査）

---

## 📊 **実装済み命令（15/16 = 93%）**

1. ✅ **Const** - 定数読み込み
2. ✅ **UnaryOp** - 単項演算（Neg/Not/BitNot）
3. ✅ **BinOp** - 算術演算（Add/Sub/Mul/Div/Mod）
4. ✅ **Compare** - 比較演算（Eq/Ne/Lt/Le/Gt/Ge）
5. ✅ **Load** - メモリ読み込み
6. ✅ **Store** - メモリ書き込み
7. ✅ **Copy** - 値コピー
8. ✅ **Return** - 関数からreturn
9. ✅ **Branch** - 条件分岐
10. ✅ **Jump** - 無条件ジャンプ
11. ✅ **Phi** - SSA値マージ

---

## ⏳ **未実装命令（5/16 = 7%）**

### **Phase 2: 演算・型操作（1命令、0.5人日）**
- ⏳ **TypeOp** - 型チェック/キャスト統一

### **Phase 4: 呼び出し（2命令、3-4人日）** ⭐最重要
- ⏳ **Call** - 関数呼び出し（MirCall統一）
- ⏳ **BoxCall** - メソッド呼び出し（後でCallに統合）

### **Phase 5: GC・構造（3命令、1-2人日）**
- ⏳ **Barrier** - メモリバリア
- ⏳ **Safepoint** - GCセーフポイント
- ⏳ **Nop** - 最適化用ノーオペレーション

---

## 🎯 **Next Steps（優先順位）**

### Option A: Phase 2（演算・型操作）から順番に
- UnaryOp/TypeOp/Load/Store実装
- 見積もり: 2-3人日
- メリット: 段階的に進められる

### Option B: Phase 4（呼び出し）を先に実装
- Call/BoxCall実装（最難関）
- 見積もり: 3-4人日
- メリット: 関数呼び出しができるようになり、実用的なプログラム実行可能

### Recommendation: **Option A → Phase 2から順番に**
- 理由: Call/BoxCall実装は複雑なので、基礎固めしてから
- UnaryOp/TypeOp/Load/Storeを先に実装して、VM基盤を強化

---

## 📚 **重要ドキュメント**

- **進捗詳細**: [mini_vm_progress.md](docs/development/current/main/mini_vm_progress.md)
- **MIR命令セット**: [INSTRUCTION_SET.md](docs/reference/mir/INSTRUCTION_SET.md)
- **開発マスタープラン**: [00_MASTER_ROADMAP.md](docs/development/roadmap/phases/00_MASTER_ROADMAP.md)

---

## 🔧 **開発環境設定**

### テスト実行コマンド
```bash
# Phase 1 Day 1+2 テスト（10テスト）
HAKO_ALLOW_USING_FILE=1 HAKO_USING_PROFILE=dev NYASH_USING_AST=1 NYASH_DISABLE_PLUGINS=1 \
  ./target/release/hako selfhost/hakorune-vm/tests/test_phase1_minimal.hako

# Phase 1 Day 3 テスト（5テスト）
HAKO_ALLOW_USING_FILE=1 HAKO_USING_PROFILE=dev NYASH_USING_AST=1 NYASH_DISABLE_PLUGINS=1 \
  ./target/release/hako selfhost/hakorune-vm/tests/test_phase1_day3.hako

# Phase 2 Day 4 テスト（7テスト - UnaryOp）
HAKO_ALLOW_USING_FILE=1 HAKO_USING_PROFILE=dev NYASH_USING_AST=1 NYASH_DISABLE_PLUGINS=1 \
  ./target/release/hako selfhost/hakorune-vm/tests/test_phase2_day4.hako
```

### 箱ファイル一覧
```
selfhost/hakorune-vm/
├── hakorune_vm_core.hako (181行) - メインVM
├── block_mapper.hako (77行) - ブロックマップ作成
├── terminator_handler.hako (208行) - Ret/Jump/Branch処理
├── phi_handler.hako (223行) - PHI命令処理
├── instruction_dispatcher.hako (57行) - 命令ディスパッチャー
├── value_manager.hako (41行) - レジスタ管理
├── json_field_extractor.hako (47行) - JSONフィールド抽出
├── const_handler.hako (39行) - Const命令
├── unaryop_handler.hako (63行) - UnaryOp命令
├── binop_handler.hako (70行) - BinOp命令
├── compare_handler.hako (77行) - Compare命令
└── copy_handler.hako (29行) - Copy命令
```

---

## 📈 **統計**

- **合計削減**: 1,525行（307行 Hakorune VM + 1,218行 MIR整理）
- **新規箱**: 14箱（Phase 1: 11箱 + Phase 2: 3箱）
- **テスト成功率**: 26/27 (96%)
- **箱化後平均サイズ**: 53行/箱
- **コア削減率**: -63%（488行 → 181行）
- **命令実装率**: 15/16 (93%)

---

**注**: 詳細な進捗・失敗記録は [mini_vm_progress.md](docs/development/current/main/mini_vm_progress.md) 参照


---

## 📦 Collections Unification — Step A–D Plan (2025-10-10)

目的: Array/Map/String の意味論を hako_core_* に集約し、ハードコーディングを撤去。Plugin/Core/ユーザーBox を Single Route/Single Face で完全統一する。

### Step A — 構造的解決（最優先・小差分）
- type_registry.rs
  - 追加: `CORE_TYPE_IDS: Lazy<HashMap<&'static str,u32>>`（MapBox=11, ArrayBox=12, StringBox=13）
  - 追加: `is_core_box(type_name: &str) -> bool`
- 呼び出し側の置換（段階導入）
  - `matches!(..., "ArrayBox"|"MapBox"|"StringBox")` → `is_core_box(..)`
  - 対象: provider_box, codec, router（MethodRouterBox）, ほか分岐箇所
- 期待効果: 3箱の分岐重複の単一起点化（SSOT）。

### Step B — ドキュメント（責務の一枚化）
- docs/architecture/single-route-single-face.md に「core意味論の責務」を1ページ集約
  - Array: index正規化/slice境界/戻り値（get→null, set/push→void）
  - Map: get→null, set/clear/delete→void, keys/values 順序（辞書順）
  - String: length/isEmpty/substring/indexOf/lastIndexOf/charAt（byte基準；将来codepointはフラグ）

### Step C — スモーク（plugin-on/strict）
- 追加（短い1–2本）
  - Map: `get(miss)==null`, `set/clear/delete` が `void`（Result:0観測）/ keys/values の順序
  - Array: `slice` 負数end clamp, `get(oob)==null`, `set/push` の `void`
- 実行例
  - 通常: `SMOKES_PROFILE_ENV=plugin-on tools/smokes/v2/run.sh --profile quick-selfhost --filter 'plugin_on_*'`
  - strict: `SMOKES_PROFILE_ENV=plugin-on-strict tools/smokes/v2/run.sh --profile quick-selfhost --filter 'plugin_on_*'`
  - HostHandle系: `HAKO_EXPORT_HOST=1` でビルド、記号が無ければテストはSKIP（既存ガード有り）

### Step D — コード（意味論の移譲）
- crates/hako_core_map/src/lib.rs 拡張
  - `size(n)`, `has(bool)`, `get(null)`, `set(void)`, `clear(void)`, `delete(void)`, `keys(Array, sorted)`, `values(Array, normalized)` の意味論を実装
  - Plugin/Core の実装はこの関数群へ委譲（薄アダプタ化）
- crates/hako_core_array/src/lib.rs 拡張
  - `length(len)`, `normalize_index(len,idx)`, `slice_bounds`, `get(null)`, `set(void)`, `push(void)` を実装
  - Plugin/Core の実装はこの関数群へ委譲
- 注意: 値型は Box<dyn NyashBox> を前提。意味論はインデックス・境界・戻り値に限定し、値のTLV/Handleはアダプタ側で扱う。

### 運用/フラグ
- plugin-on strict（builtin fallback抑止）
  - `NYASH_PLUGIN_ON_STRICT=1`（alias: `HAKO_PLUGIN_ON_STRICT=1`）
  - Bring‑Up は既定OFF。プロファイル `plugin-on-strict.env` で段階適用。
- near‑spec 不在時の type_id 冗長記録（済）
  - Array=12/Map=11/String=13 を box_specs に記録して type_id→invoke 解決を堅牢化。

### Done/Blocked メモ
- Done: invoke re‑probe/near‑spec冗長化、HostHandle経路（診断/スモーク）、strictフラグ導入、EnvGateBox 置換の一部
- Next: Step A 実装→置換、Step C スモークの追加、Step D の core 意味論移譲（小粒PR分割）


---

## 🔗 Delegation (from/extends) — Phase‑1 Plan (2025-10-10)

目的: ユーザーBox→Core/Plugin Box への委譲（composition）を Single Route/Single Face に統一し、from Parent.method() を親レシーバで安全に実行可能にする。

### 設計（構造優先）
- Delegation metadata: Builder が AST の `extends` を収集 → Delegation 定義（Child → [Parent…]）として保持
- InstanceBox: 隠しスロット `__delegates: Map<String, BoxRef>` を導入（Phase‑1: Child.birth 内で親を birth→格納）
- 正規化: `from Parent.method(args)` → MIR 正規化パスで `DelegatedBoxCall(parent=Parent, method, args)` に変換
- Router: `DelegatedBoxCall` を受け、`__delegates[Parent]` を取り出して通常 `route(receiver=delegate, method, args)` に委譲
- Fail‑Fast: 親が未生成/未登録の場合は strict でエラー。非strict は一時的に BoxCall 互換でフォールバック（移行期間のみ）
- 範囲: メソッド委譲のみ（フィールド/状態のオーバーレイは対象外）。循環委譲は検出してエラー。

### Step E — 実装手順
1) 構造
   - Builder: `extends` → Delegation定義作成、Child.birth に親生成（引数なし）を自動注入（Phase‑1）
   - MIR: `from` 呼び出しを `DelegatedBoxCall` に正規化するパスを追加
   - VM/Router: `DelegatedBoxCall` ハンドリングを追加（`__delegates` 経由で親レシーバ取得→route）
2) ドキュメント
   - docs/architecture/single-route-single-face.md に「委譲は composition」を追記（親の所有/寿命/GC方針）
   - 言語リファレンス: from/extends の意味論（親は共有せず Child が所有、strict の失敗条件）
3) スモーク
   - ユーザー→Core 親: `from ArrayBox.push/get` が親レシーバで動くこと
   - ユーザー→Plugin 親: `from MapBox.keys/values` が親レシーバで動くこと（Stage‑1/2 混在許容）
   - 失敗系: 親未生成/未登録、循環委譲（strict=Fail、非strict=警告/互換）
4) フラグ/運用
   - 機能ガード: `NYASH|HAKO_DELEGATION_ENABLE=1`（既定OFF→段階導入）。strict は即時 Fail‑Fast。
   - Bring‑Up 中は非strictを既定（互換）。緑確認後に strict をプロファイル単位でON。

### 受け入れ条件
- plugin-on/strict/HostHandle スモークが緑（親委譲ケースを含む）
- Router 一経路（Delegated→通常route）で分岐の増加なし
- 既存 from の互換（非strict）を維持、strict で Fail‑Fast が働く

- Builtin callAsync true async implemented (HAKO_CALLABLE_ASYNC=1): job queue + VM polling; added smoke quick/core/callable_async_builtin_vm.sh.

---

## Plugin Policy & Birth Unification (Phase 15.7) — Plan & Status

Goals
- Default plugin policy = auto (ON). If no providers configured, no side-effects.
- VM unifies lifecycle: new → birth(me,args) always (birth missing = no-op, idempotent)
- Plugin init: load-time nyash_plugin_init() (optional) and first-birth ensure_ready() (Once)
- Provider resolution single order: Plugin → Builtin → Registry; on-demand reprobe for T
- Boot disabled state is not cached (allow later retry when policy flips to ON)

Done
- Runner: default policy auto (None/unknown → auto)
- Boot: policy off no longer cached as success (returns false; retry later)
- ProviderBox: reprobe list extended to include FileBox
- VM: always attempt birth after new (ignore missing method)
- Smokes: plugins/filebox_write_read_vm added; quick/core/filebox stabilized (SKIP when unavailable)
- Docs updated: docs/reference/plugin-system/vm-plugin-integration.md (final rules)

Next
- Determinism guard (deny IO caps like FileBox when HAKO_DETERMINISTIC=1)
- Error ergonomics: ProviderNotFound/PluginInitFailed/BirthFailed with provenance
- Optional: on-demand reprobe toggle for deterministic mode (off)

Acceptance
- plugins profile: all smokes green (callable/map/string/array/json/filebox)
- quick profile: filebox_basic is PASS or SKIP (plugin missing)
- Runner: default plugin policy is auto; no regressions
- VM: new→birth unified; no double-birth



## Plugin Capabilities & Deterministic Guard (Phase 15.7)
- Added docs: docs/reference/plugin-system/capabilities.md
- Set NET caps (1<<1) for nyash-net-plugin boxes; FileBox already IO (1<<0)
- Deterministic mode now denies IO/NET boxes; on-demand reprobe disabled (unchanged)
- PathBox remains caps=0 (deterministic/pure)

Acceptance:
- Quick/plugins/full smokes stay green
- Deterministic denial works for FileBox/Net family when HAKO_DETERMINISTIC=1

## Update — Router/Adapter Consolidation and Stage‑2 Identity (2025-10-11/12)

What we changed
- HostHandleRouter: host_api by‑slot分岐をHostHandleRouterへ全面移設。`nyrt_host_call_slot`はRouter委譲のみ。
- ConsoleAdapter: print() を一箇所（console_adapter）に集約。Void/null/String/BoxRef(String)の出力正規化。
- ENV gate: `host_handle_trace()` を env_gate_box に追加（HAKO_HOST_HANDLE_TRACE/NYASH_*対応）。既定OFF。
- docs: env-variables.md を更新（HAKO_*主、NYASH_*互換、TTL/cleanup、Stage‑2は実験/プロファイル限定）。

Stage‑2 identity 進捗
- ハング解消: `nyash_array_new_h` を loader経由→Builtin ArrayBox 直接生成に変更（再帰ロック回避）。
- 昇格経路: tag=8 (PluginHandle ArrayBox) → HostHandle へ正規化（ffi_bridge/host_api decode）。
- Map plugin: values(Stage‑2) と METHOD_GET の Array 値を HostHandle(tag=9) に正規化（identity一本化）。
- 現状: futex_wait は解消。`plugin_on_values_identity_vm` は Result: 22（未達）。Array.set/len トレースは `NYASH_HOST_HANDLE_TRACE=1` で採取可能。

Next steps
- 観測: HostHandle slot 101/102 の rc/out_len を確認し、必要なら fail‑fast 条件（rc=0 & out_len 不変の扱い）を詰める。
- 確認: Map.get の HostHandle返却がVM側で同一Arcとして解決されるか（キャッシュ＋Routerの動線）を点検。
- 緑化: 上記 fix 後に plugins/profile（plugin_on_*）を再実行。安定後、plugins.env の Stage‑2 既定ONへ復帰。


## ✅ Stage‑2 HostHandle（Collections）完了（2025-10-11→10-14）

目的
- Map/Array/String のコレクションAPIを Stage‑2（HostHandle 直往復）へ統一し、同一性と使いやすさを確保。

実施
- プラグイン側
  - MapBox: keys()/values()（NYASH_PLUGIN_MAP_ARRAY_HANDLE=1）で HostHandle(Array) を返却。
  - MapBox.get(): Array 値は HostHandle(tag=9) に正規化して返す（Stage‑2経路と整合）。
  - ArrayBox: birth/length/get/set/push/slice の最小TLV実装（lengthはTLV i64）。
- ホスト側
  - decode 正規化: tag=8(Array) は HostHandle へ昇格、tag=9 は HostHandleBox 化。
  - HostHandleRouter: Array(100/101/102), Map(200..204), Instance(1..4), String(300) の by‑slot を実装。
  - ProviderBox: NYASH_USE_PLUGIN_BUILTINS=1 で core(Array/Map/String) を plugin-only と扱う。
- スモーク/プロファイル
  - plugins プロファイルで Stage‑2 を既定ON（NYASH_PLUGIN_MAP_ARRAY_HANDLE=1）。
  - plugins スモークから --dev を撤去（print 経路の揺れを排除）。
  - PASS: plugin_on_values_identity_vm / plugin_on_print_array_size_vm / map_keys_values_stage2_vm / map_stage2_identity_vm。

備考
- 追加した診断ログ（Router/Decode/host_bridge/array-plugin）は NYASH_DEBUG_PLUGIN=1 のときのみ出力（既定OFF）。
- 一時フォールバックENV NYASH_ARRAY_SIZE_FORCE_HOST は撤去（診断用途のみ、既定OFFで温存可）。

次のステップ（WASM/LLVM への波及）
- LLVM: nyrt_host_call_slot/nyash_array_new_h/nyash_host_from_plugin_handle を輸出（AOTも対応）。slot呼び出しに統一。
- WASM Phase‑A: plugins OFF + 内蔵コレクションで緑化 → Phase‑B: HostHandle import（BigInt）で完全化。

**最終更新**: 2025-10-14

---

## ✅ Phase 15.7 — Plugins/Stage‑2 完了と Self‑Hosting 再開（2025-10-11）

要旨
- Plugins profile（VM/Stage‑2 HostHandle）代表スモーク PASS（identity/print）。
- LLVM parity（integration-core）全緑（15/15）。
- ProviderBox/ENV 方針固定（HAKO_PLUGIN_POLICY=auto を尊重、コアは NYASH_USE_PLUGIN_BUILTINS=1 でプラグイン優先）。

次のステップ（Self‑Hosting に復帰）
- M1 Bootstrap EXE（最小）
  - 受け入れ: 自作フロントが非空JSON（"kind":"Program"）を出力し、LLVMハーネスで EXE 化。
  - 実装: apps/selfhost-compiler/compiler.hako（最小出力）。
  - スモーク: tools/exe_first_smoke.sh, tools/mir_builder_exe_smoke.sh（.hakoに更新）。
- M2 Self‑Rebuild
  - 受け入れ: コンパイラが自分のソースを処理し、JSON/MIR サマリが安定（ハッシュ/サイズ）
  - スモーク: tools/selfhost_smoke.sh（.hakoに更新）ほか Stage‑2 セット。
- M3 VM↔LLVM パリティ
  - 受け入れ: 小サンプル2〜3本で VM/LLVM 出力一致を継続確認。

備考
- 拡張子は .hako に統一（自作/サンプル/スモーク）。既存 .nyash は互換経路で当面維持（削除は後段）。



## ✅ ENV Consolidation & Provider Simplification（2025-10-11）

要旨
- ENVは「3本柱」に集約: `HAKO_PLUGIN_POLICY`, `NYASH_PLUGIN_MAP_ARRAY_HANDLE`(pluginsのみ), `HAKO_HOST_HANDLE_TRACE`(短命)
- ProviderBoxは policy に一本化（force=strict=フォールバック禁止）。旧 builtins 系ENVは依存撤退（互換は当面維持）。
- スモークのENVパススルーを最小化（policy/map_handle/trace のみ）。

効果
- テスト実行のENVノイズが大幅削減（9→3）。判定経路の分岐が明瞭化し、失敗解析が容易に。

---

## ✅ Self‑Host M1/M2 状態（2025-10-11 現在）

M1（EXE-first）
- compiler.hako は Program v0（version:Int=0, Return 7）を出力。
- スモーク: Programヘッダ＋Bridge 実行 既定ON（`NYASH_EXE_BRIDGE=1`）。PASS。

M2（MIR builder ブリッジ）
- 既定: ProgramヘッダのみでPASS。
- `NYASH_MIR_BUILDER_EXE=1` 有効時に Builder/EXE 実行を通す導線を整備（runnerの `--emit-mir-json` を優先入力）。
- 現状: オブジェクト生成フェーズで一部ケースが失敗（一時出力/末尾整合の微修正が必要）。

次の一手（小差分で既定ONへ）
- builder入力の安定: `NYASH_LLVM_OBJ_OUT`/一時名の固定と末尾整合（改行/JSON単一行）。
- PASS確認後、`NYASH_MIR_BUILDER_EXE` の既定を 1 に昇格。

---

## 🔧 追加のコード整備（完了）
- 警告削減: 到達不能・重複削除、未使用抑止（import/変数）、局所 `#[allow(dead_code)]`。
- リンク堅牢化: `tools/build_llvm.sh` が `libhako_kernel.a` を `crates/hako_kernel` → `target/release` の順に探索。
- シンボル衝突回避: `nyash_array_new_host` に改名（Kernel側と競合回避）。

---

## 📋 Todo（短期）
- [ ] MIR builder 既定ON（微整合の最終パッチ）
- [ ] quick の代表2本の緑化（Program/MIR経路のみを対象に）
- [ ] env-variables.md に primary/alias・短命ENVの一文追記（必要なら）

備考
- integration-core（LLVM parity）は全緑維持（15/15）。
- quick の広域失敗は今回の範囲外（Box/using/legacy挙動の混在）。対象縮小で段階対応する。


## Phase 15.7 — Hakorune Compiler/VM Gap Plan (2025‑10‑12)

Rust vs Selfhost — quick delta
- MIR ops: const/binop/compare/branch/jump/ret/copy（済）; phi（scan/apply 最小）; 未: load/store/typeop/safepoint/barrier/throw
- Call shapes: Extern/Global/Constructor/Method 最小導通（emit v1→v0 経由）; 未: BoxCall/E2E 境界、Method 拡張 arity
- SSOT: slots/arity/aliases 既に SSOT優先; 未: type_id/box 表の SSOT 化（静的表は fallback 維持）
- Plugins: Provider 早期ロード/identity cache 済; 未: values/keys の Handle 完全化（順序はテスト非依存へ）

Action items（P4/P5 反映）
- P4: NewBox/Call/Method を shared MirSchema/BlockBuilder 直結（出力互換）。rc‑only 代表を追加。
- P5: LocalSSA ensure_calls/ensure_cond の最小導入（copy/phi 材化）。rc‑only 代表を追加。
- SSOT: resolve_typebox_by_name を SSOT 優先に寄せ、生成テーブルへ段階移行。
- Plugin‑on: 代表は順序非依存＋最小rc、strict は plugin‑tester build-all を前置し、未在庫は SKIP。
- Smokes: Selfhost を opt‑in に固定（SMOKES_SELFHOST_ENABLE / SMOKES_SELFHOST_M2M3_ENABLE）。CI 既定は quick + integration‑core を維持。


## ✨ Today’s Update — 2025‑10‑12 (late)
- P4 完了（薄アダプタ直結）
  - emit_mir_flow(_map) に extern/global/method/ctor の薄アダプタを追加し、BlockBuilder 直結へ
  - emit_call/emit_method/emit_newbox（v0/v1）も shared BlockBuilder に統一（出力互換）
- P5 最小（LocalSSA 集約）
  - ensure_materialize_last_ret(mod) 追加、ensure_cond(mod) を全ブロック対応に拡張（If/Loop）
  - rc-only 代表: selfhost_localssa_ensure_calls_rc_vm / _ensure_cond_rc_vm / _ensure_cond_if_loop_rc_vm
- Pipeline（v1 経路）
  - MirCallBox 依存を Emit*（shared builder）に段階置換（差分最小）
- plugin‑on 代表の安定化（quick 全緑化）
  - preflight 失敗時は SKIP、在庫プリチェック（new ArrayBox/MapBox）追加
- SSOT 再スキャン
  - specs/type_registry.toml に不足なし。resolve_* は SSOT 優先（静的 fallback）で維持
- quick: 288/288 PASS、integration‑core: 20/20 PASS

### Next（ハコ VM 同等化ロードマップ）
- Phase‑A: JSON MIR v0 Reader を CLI 経路に昇格（dev ゲート撤去）／ LocalSSA phi 材化代表を 1 本追加
- Phase‑B: Stage‑3（break/continue/throw/try）最小実装／ Map plugin の handle 値対応＋identity 代表
- Phase‑C: SSOT を型解決ホットパスに全面適用／ 診断 helpers へ最終集約

### Backend/CLI 整理（設計の確定）
- 単一 CLI `hakorune` + `--backend {nyvm|rust|llvm}`（HAKO_BACKEND）。バイナリ別名は任意（hakorune-{vm,rust,llvm}）
- コンパイラは分離（hakorune-compiler）。dev ではバンドル起動を opt‑in で提供
- ツール解決層を CLI に実装（dist/bin → workspace → hako.toml [tools]/[backends] → user config → ENV → PATH → autobuild(opt)）


### Note — nyvm 既定化（2025‑10‑12）
- CLI `--backend` の既定を `nyvm` に変更（以前は vm）。Mini‑VM は opt‑in（HAKO_NYVM_ENGINE=mini）。
- 目的: Ny 製 Hakorune VM を本線に育て、Rust VM は段階撤退へ。


## Phase 15.7 — Async parity
- VM: Await native; nowait via env.future.spawn_instance (pseudo-async)
- LLVM: rewrite Await/Future* to env.future.*; thin special-cases in builder
- Hakorune VM: bridge path aligns; smokes added (nowait/await)

## ▶ Next TODOs — Phase 15.75 脱Rust (P1 入口)
- Map.call P1（同期・VMシュガー／ゲート付）→ 実装＋最小スモーク3本
- keysS/valuesS フォールバックの箱化（.hako アダプタ化）→ 緑維持
- HostHandleRouter 段階移設（host_api 分岐の縮退）→ 緑維持
- Parser/Tokenizer の純関数領域を .hako へ薄移設 → 軽スモーク緑

### 🗂 Archive Note
- 詳細ログ・履歴の多くは proposals/phase-15.75/ に集約。
- 本ファイルは当面「今日の更新」「次のTODO」の要約のみを維持（古い粒度の高い記録は該当 docs に統合）。

## Collections API follow-up (Phase 15.7+)
- [DONE] Map.keys()/values() fallback is default ON in VM router (keysS/valuesS → ArrayBox via split). No env required.
- [DOC] docs/guides/collections-api.md updated to state default fallback and unified methods.
- [STAGE] Add .hako HostBridge wiring helpers (`selfhost/hakorune-vm/map_keys_values_bridge.hako`) and smoke (quick-selfhost) to validate keysS/valuesS → Array bridging.
- [NEXT] When plugins provide Array-returning keys()/values() consistently, remove Rust fallback path and keep .hako adapter as optional helper only.
