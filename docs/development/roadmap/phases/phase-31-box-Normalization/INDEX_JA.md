# Phase‑31 — Box Normalization（Static→Singleton 正規化）

最終更新: 2025‑10‑19（Map router 回帰テスト + Type ID SSOT + トランポリン撤退振り返り）

## サマリ
- ねらい: すべてのメソッド呼び出し形を「me + args」に統一し、static box を型ごとのシングルトンインスタンス（`Type.singleton`）に正規化する。
- 効果: ルータ分岐削減、receiver 有無によるバグ根絶（今回の ArrayBox(1) 類）、Extern/HostBridge 経路の単純化、AOP/計測の注入点の一元化。
- 方式: Builder で `Static(Type.method(args)) → Instance(Type.singleton.method(args))` に書換。ルータは常に receiver を渡す。Verifier で逸脱を Fail‑Fast。
- 導入: フラグ既定OFFで段階導入（A→B→C）。プラグイン側で `len`/`length` エイリアスを正規化（トランポリン撤退済み）。

### 今日の更新（P0仕上げ＋P1 部分）
- host anchors: `nyash_array_new_h` を既定ON（feature gate 撤去）。
- extern_adapter(Array/Map): `nyrt.array.size` 受領者の HostHandle unwrap／Map.keys/values/size の dev ログ追加。
- Router 再配線: `PluginBoxV2` の Array/Map スロットを表テーブル前提で正規に通す（builtin/plugin 両経路）。
- env 読み統一: runtime 配下の直 `std::env::var` を `env_gate_box` に寄せ（進行中）。
- Router/Adapter 回帰テスト: `map_host_keys_values_return_arrays` / `map_remove_returns_removed_array_len` などを追加し、HostSlot/Plugin 両経路で ArrayBox が返ることを固定。
- Type ID 単一起点: Router/extern/loader から直 `builtin_type_id("MapBox")` や固定値参照（11/12/13）を排除し、`crate::types::ids::{map,array,string,by_name}` へ統一。rg 監視で再発チェックを継続。
- Reentrant Guard（host slot 中の再入許可）: Map.values() → Array.set の再入を slot 配下のみ許可し、連鎖を安定化。
  - 実装: `host_api::nyrt_host_call_slot` 実行中に thread‑local `IN_HOST_SLOT=true` を設定し、`plugin_loader_unified` のガードを `recursed && !in_slot` に変更。
  - 効果: values→Array.set→Array.size の連鎖が正しく成立。代表スモーク `map_values_array_element_vm` は PASS。

#### Runtime meta 層（Callable/Future）の箱化（追加）
- ねらい: 言語機能の足場（Callable/Future）をホスト所有の薄い箱として分離し、プラグイン/外部I/Oの逆流を構造で防止。
- 実装: `src/runtime/meta/{callable,future}/` を新設。README と LAYER_GUARD で責務を明記。
- 互換: 既存の `runtime::{callable_box,future_box}` re-export は撤去済み。新規・既存ともに `runtime::meta::{callable,future}` を使用。
- 影響: plugin-only/legacy いずれのビルドでも Callable/Future が安定（router/scheduler への依存のみ）。

#### HostHandle -14 検知（境界テストの安定化・追加）
- ENV: `HAKO_HOSTHANDLE_TEST_RET_MISMATCH=1`（互換 `NYASH_HOSTHANDLE_TEST_RET_MISMATCH=1`）で String.len 経路を -14 として観測。
- 実装: VM の HostSlot/Extern で rc を stdout に `hosthandle-test rc=-14` として出力（テスト専用）。
- スモーク: `tools/smokes/v2/profiles/plugins/hosthandle_boundary_suite_vm.sh` は一時ファイルに退避してから `-14` を grep（PIPE 経由の出力欠落を回避）。

#### P1 安定化（今回分）
- builtin ルータ: ARRAY/MAP の host route 判定を dev‑only ログで観測可能に（`HAKO_DEBUG_HOST_SLOT=1`）。
- extern_adapter: Array/Map のレガシー分岐を撤去し、HostHandle unwrap をハブ化。`nyrt.array.size`/`nyrt.map.{keys,values,size}` の戻り値を正規化。
- 回帰テスト追加:
  - ParameterGuardBox（ENV ON/OFF）。
  - ループヘッダ PHI 即時挿入（先頭固定・更新 in‑place）。
  - DCE: Method 受け手／Closure captures の Copy 温存。
  - Router/Adapter（今回追記）:
    - `map_host_keys_values_return_arrays` — HostSlot 強制下で keys/values → ArrayBox → len が成立することを固定（ユニット）。
    - `map_remove_returns_removed_array_len` — remove が削除値（ArrayBox）を返し、直後に len を呼べることを固定（ユニット）。

### 次の一手（B段）— プラグイン互換トランポリン撤退（完了）
- 旧互換ラッパ（`HAKO_PLUGIN_TRAMPOLINE`）は 2025-10-18 時点で撤去。
- len/length エイリアスは各プラグイン（ArrayBox/MapBox/StringBox）が直接解決済み。
- 追加ガードは不要。以降はプラグイン本体と Router テーブルで整合を維持する。

## スコープ / 非対象
- 対象
  - 呼び出し形状の統一（MIR/Builder/Router/Verifier）
  - プラグイン/Extern/HostBridge の呼び出し規約整合
  - selfhost 側の静的メソッド（Hako）→ instance 形の糖衣
- 非対象（今回やらない）
  - 言語仕様の新規拡張（デフォルト引数・可変長シンタックスなど）
  - 既存最適化のチューニング（導入後に計測して別フェーズ）

## ゴール（受け入れ基準）
- すべての static 呼び出しが MIR 上で `receiver=Type.singleton` に正規化される。
- ルータは receiver を常に受ける前提で動作し、static/instance の分岐が無い。
- Verifier が「receiver 欠落」や「static 直呼び」を検出して Fail‑Fast（開発時）。
- プラグイン ABI は各 Box resolver が直接エイリアスを提供（トランポリンなし）。既存スモーク（quick→plugins→full）緑。
- HostBridge/Extern の呼び出しは `me + args` 規約に統一。引数正規化（プリミティブ化）で再発無し。

## 非機能（性能/安定）
- LLVM/VM: `me` 未使用は inlining + DCE で最終的に消える（同一バイナリ/LTO 前提）。
- 動的リンク（.so）越し: 追加 1 引数のコストは微小。必要箇所は旧 ABI のエイリアスを維持。

---

## 設計（構造）
### 1) 呼び出し正規化（Builder）
- 変換規則（疑似）:
  - `Call ModuleFunction("Type.method/N", args)` かつ `method != birth` → `Callee::Method { box_name: "Type", receiver: Some(Type.singleton), method }`
  - 既に instance の場合は変換無し。
- 影響ファイル（予定）:
  - `src/mir/builder/calls/method_resolution.rs`
  - `src/mir/builder/builder_calls/*`（ModuleFunction 経路）

### 2) ルータ（VM/ランタイム）
- 常に `receiver` を渡す前提に一本化（static/instanceの分岐を削除）。
- 不変: `birth()` の取り扱い（自動/明示）は既存契約維持。
- 影響（例）:
  - `src/backend/mir_interpreter/handlers/calls/function.rs`（ModuleFunction ブリッジ）
  - `src/backend/mir_interpreter/handlers/boxes/legacy/mod.rs`（BoxCall ディスパッチ）
  - `src/runtime/method_router_box/*`（外部/Plugin 経路の期待形）

### 3) Verifier（Fail‑Fast）
- ルール:
  - ModuleFunction 直呼び（static 形）が残っていたらエラー。
  - Method の `receiver=None` は禁止。
  - static 正規化の `me` は観測不可（反射禁止）…観測を試みるパスを警告/エラー。
- 影響: `src/mir/verify/*`

### 4) プラグイン ABI 互換（直接 alias）
- len/length などの互換は各プラグインの `resolve` 実装で提供。
- 追加トランポリンや自動生成スクリプトは不要。

### 5) HostBridge/Extern（橋渡しの一元化）
- 規約: `Extern(iface.method)` のハンドラは常に `me + args` 形に揃える（必要なら内部で `singleton` を補う）。
- 引数正規化: BoxRef（ArrayBox 等）→プリミティブ化は既存の `normalize_extern_arg`（VM）へ集約。

---

## ガード/フラグ
- `HAKO_STATIC_AS_SINGLETON=1`（NYASH_* alias 可） … 既定OFF、A/B/C 段階で切替。
- CLI 既定は変更しない。ENV は短命（導入〜安定化まで）。

## ドキュメント / LAYER_GUARD
- `docs/development/proposals/` に本設計の背景と対処（この文書を索引）。
- LAYER_GUARD（意図）
  - Router 層: 「receiver 必須」。ModuleFunction の直参照禁止。
  - Builder 層: static→singleton 正規化必須。抜けはテストで遮断。
  - Extern 層: プリミティブ引数化と `me+args` 契約。

---

## 段階導入（A/B/C）
### Phase A（実験・既定OFF）
- 実装
  - Builder 正規化（static→singleton）
  - ルータ分岐の掃除（receiver 常時）
  - Verifier 追加（開発時のみ Fail）
  - プラグイン resolver の alias 対応を確認
- スモーク
  - static/instance 同名メソッドの一致
  - HostBridge 経路（extern）での等価性
- 成果: quick 緑 + plugins 代表 PASS

### Phase B（互換・計測）
- （完了）プラグインが直接 alias を提供することを確認。追加トランポリンは不要。

### Phase C（既定ON 検討）
- （完了）CI/Docs は新規約に統一済み。旧 `HAKO_PLUGIN_TRAMPOLINE` は撤退済み。

---

## 変更対象（予定ファイル・開始行）
- Builder
  - `src/mir/builder/calls/method_resolution.rs`（静的→受領者注入）
- VM/Router
  - `src/backend/mir_interpreter/handlers/calls/function.rs`
  - `src/backend/mir_interpreter/handlers/boxes/legacy/mod.rs`
  - `src/runtime/method_router_box/*`
- Verifier
  - `src/mir/verify/*`（新規/既存拡張）
- プラグイン/Extern
  - `src/backend/mir_interpreter/handlers/calls/legacy/extern_handler.rs`（規約コメント追加・整合）

---

## テスト計画
- 単体
  - static→singleton 正規化の生成確認（MIR 形状）
  - receiver 欠落時の Verifier エラー
- 結合
  - Router（builtin/plugin）での一貫ディスパッチ
  - HostBridge（extern）経路の文字列/数値/配列の正規化
- スモーク
  - quick→plugins→full の差分比較（代表）
  - selfhost 側の静的 API（Hako）
  - meta 層の代表確認（plugins プロファイル）:
    - `tools/smokes/v2/profiles/plugins/callable_async_plugin_vm.sh`（Future 表示安定）
    - `tools/smokes/v2/profiles/plugins/plugin_map_len_vm.sh`（len エイリアス整合）
    - `tools/smokes/v2/profiles/plugins/set_bad_arity_vm.sh`（arity ガード Fail‑Fast）
- 受け入れ基準
  - 既存スモーク緑 + 新規スモーク（static/instance 等価）緑
  - LLVM/VM 出力差異がゼロ/許容範囲

---

## リスクと対策
- 互換性リスク（プラグイン）: resolver alias を Fail-Fast 方針で維持。段階導入。
- 反射の観測: LAYER_GUARD と Verifier。ドキュメントで未定義化を宣言。
- 性能劣化: ベンチで観測し、必要ならホットパスのみ旧 ABI 直呼びエントリを併存。

## ロールバック
- `HAKO_STATIC_AS_SINGLETON=0` で旧挙動へ即時復帰。

---

## 実装 TODO（P0→）
1. Builder に static→singleton 正規化を実装（最小: String/Array/Map 代表）
2. Router の receiver 常時渡しを再点検（不要分岐の撤去）
3. Verifier 追加（ModuleFunction 直呼び/receiver 欠落の Fail）
4. スモーク追加（static/instance 等価、extern 経路）
5. Docs 反映（ガイド/リファレンス/ENV 記載）

### 本日の優先P0（追記）
- Array.size 正規化の徹底（Builder normalize → `Extern("nyrt.array.size")` 固定、Optimizer で巻き戻し禁止）
- EmitGuard 後の LocalSSA 素材化の再確認（values→size 連鎖で未定義参照を出さない）


## 付録（インターフェース最小定義）
- `Type.singleton(): &Type`（内部/once 初期化、外部観測不可）
- 呼び出し正規化: `Static(Type.method(args)) → Method(Type.singleton, args)`
- Verifier: ModuleFunction 直呼び/receiver=None を禁止

---

> 補足: 今回のバグ（ArrayBox(1) 漏れ）の根は「受け渡し規約が2通りあった」こと。構造で1通りに揃えるのが再発防止の最短路だよ。

---

## 進捗（実装済み）

### Phase A‑1b — 関数スコープのシングルトン・キャッシュ導入（完了）
- 目的: `emit_static_me_placeholder()` を各所で都度呼ぶ“分散生成”をやめ、関数内で 1 回だけ生成して再利用する。
- 実装:
  - `MirBuilder.current_fn_singletons: HashMap<String, ValueId>` を追加。
  - 関数開始時にスコープを作り、関数終了時に復元（キャッシュは関数単位で生存）。
  - `maybe_prepend_static_me()` から `current_fn_singleton(box_name)` を利用し、ModuleFunction 経由の静的呼び出しに一貫して `me` を付与。
- 変更点（主な参照）:
  - `src/mir/builder.rs:176, 448-468` — キャッシュ本体とプレースホルダ生成の単一ルート。
  - `src/mir/builder/builder_calls/build.rs:39-46, 138-149, 232-241, 249-258, 265-274` — `maybe_prepend_static_me()` による受領者注入。
  - `src/mir/builder/builder_calls/lowering.rs` — メソッド/静的メソッドを関数化する際にキャッシュを save/restore。
  - `src/mir/builder/lifecycle.rs:50` — `main` 構築時のキャッシュ初期化。
- 効果:
  - 同一関数内の静的メソッド連続呼び出しで `me` プレースホルダの重複生成が解消。
  - 将来の OnceLock 実体置換を 1 箇所で行える導線ができた。

### Phase A‑1c — ModuleFunction alias 整備 & Verifier 補強（完了）
- 目的: ModuleFunction 経由の静的呼び出しで receiver 欠落を早期検出し、VM 側も常に receiver あり前提に揃える。
- 実装:
  - Verifier: `ModuleFunction(box.method/arity)` で Known かつ Box 型の受領者が無い場合に Fail‑Fast。
  - Runtime: MethodRouter が Void 受領者を即時 InvalidInstruction とし、legacy fallback も receiver 前提に統一。
  - ModuleFunction alias: `handlers/calls/trampolines.rs` を追加し、Array/Map/String/Console の ModuleFunction → Method 変換を表駆動化。

### Phase A‑1d — LegacyCallBridgeBox 導入（完了）
- 目的: `emit_instruction(MirInstruction::Call)` の直叩きを Builder から排除し、ガード済みの入口に集約する。
- 実装:
  - `src/mir/builder/calls/legacy_bridge/` に LegacyCallBridgeBox を新設し、旧 `emit_legacy_call` の本体を移設。
  - すべてのレガシー Call 発行を `emit_call_with_guard`（EmitGuard）経由に統一するヘルパ `emit_call()` を追加。
  - BoxCall/PluginCall も `emit_boxcall()` でローカルSSA素材化→`emit_box_or_plugin_call` へ流すようガード化。
  - `src/mir/builder/builder_calls/emit.rs` の `emit_legacy_call` は Box への委譲のみとし、モジュール境界でレガシー経路を閉じ込めた。
- 効果:
  - Guard を通さない Call が発生しなくなり、Extern/Method/Global いずれも素材化漏れを検出可能に。
  - レガシー経路が単一ファイルに箱化されたことで、段階的削除（将来的なフェーズB/C）や差分追跡が容易に。

### Phase A‑2 — singleton 実体（lazy 初期化）導入（完了）
- 目的: `emit_static_me_placeholder()` が生成する Void const を実体 BoxRef に置き換える。
- 実装:
  - `runtime/static_singleton.rs` を追加し、`OnceCell<Mutex<…>>` で Box 名ごとのシングルトンを lazy 作成。
  - Interpreter `handle_const` が `MirType::Box(..)` を検知した場合に `static_singleton::get()` を参照し、`VMValue::BoxRef` を登録。
- 効果:
  - MIR 側は Box 型として `me` を扱い、VM 実行時に実体 Arc<NyashBox> が注入される。後続のマクロ/Verifier 強化に備えて受領者が具体化された。

### Phase A‑3 — ループヘッダー PHI の「真のループキャリア変数」化（完了）
- 目的: ループ条件で一時文字列（例: `ch`）が PHI 化され、`i < s.size()` が誤って `ch < s.size()`（String vs Integer）になる問題を根治する。
- 実装:
  - `build.rs` 側で preheader の変数スナップショットを取得し、事前スキャンで得た `assigned_vars` との積を取って PHI 対象を決定。
  - 実装箇所: `src/mir/loop_builder/build.rs:36` 付近（preheader_vars キャプチャと loop_carried 生成）、`src/mir/loop_builder/phi.rs:14-48`（PHI 対象の retain）。
- 効果:
  - ループ条件の比較型が安定。`tools/smokes/v2/profiles/quick/apps/json_query_vm.sh` が PASS（以前の `String("0") < 1` 失敗が消失）。

### 残タスク（Phase A 継続）
- ✅ **ParameterGuardBox 拡張**  
  Builder（pending entry copy を含む）と optimizer `repair_*` で `dst ∈ params` を禁止済み。v%0 上書き事故を構造的に遮断。
- ✅ **Verifier 層の追加**  
  `check_no_parameter_reassignment` を追加し、MIR 完成後もパラメータ再定義を Fail‑Fast。
- **EmitGuard 後の ArrayBox.size 固定（継続中）**  
  `map.values()` → `.size()` の連鎖を `nyrt.array.size` へ確実に正規化し、EmitGuard の一度だけで素材化が済むよう整理する（Normalizer で Method 巻き戻し禁止）。現在 json_query_vm で `String("0") < 1` という残バグを追跡中。
- **RouterPolicy のログ拡張**  
  `Route::BoxCall` 強制時の理由を mat-trace に記録し、Singleton 正規化後の逸脱調査を容易にする。

### Phase A‑1e — Map.size 正規化 & Extern 安定化（完了）
- 目的: `MapBox.(size|len|length)` を `Extern("nyrt.map.size")` に統一し、Method 形の残存で起きる受領者素材化漏れを排除。
- 実装:
  - `src/mir/builder/normalize/map_length.rs` 新設。`normalize::apply_all` に組み込み。
  - Optimizer 側で `nyrt.map.size` の Method への巻き戻しを抑止。
  - Lowering で `MapBox.size/0` → `nyrt.map.size(recv)` の直下ろしを追加。
- 効果: plugins プロファイルの `parity_map_size_has_vm` / `strict_plugin_map_size_vm` が PASS。

### Phase A‑1f — Map.keys/values と remove の整備（完了）
- 目的: `Map.values()` の戻り値を ArrayBox として統一し、`.size()` 連鎖で未定義値／HostHandle 漏れが発生しないよう構造化。
- 実装:
  - `extern_adapter/collections.rs` を新設し、HostHandle unwrap をハブ化。`nyrt.map.{keys,values}` / `nyrt.array.size` が常に実体 Box を扱うよう統一。
  - Extern adapter を host slot テーブル経由＋ PluginV2 経路の二本で正規化し、両経路で `VMValue::BoxRef` を返却。
  - Map プラグインの `remove()` を HostHandle/TLV 返却へ統一し、削除した値を呼び出し元で即利用できるよう調整。
- テスト:
  - `map_host_keys_values_return_arrays` / `map_remove_returns_removed_array_len` を追加し、HostSlot 強制と plugin route の双方で ArrayBox が返る・`len()` が成立することを固定。
  - `map_values_array_element_vm.sh` / `map_remove_returns_value_vm.sh` / `plugin_map_len_vm.sh` で代表スモークを常時監視。

### Known issues（2025‑10‑19 時点）
- me 注入の誤適用により plugin ModuleFunction へ影響する懸念。
  - 対処: `method_index.static_signature()` に該当する静的シグネチャのみ `me` 合成を許可（Builder/VM 双方でガード）。
- FileBox 系の `use of undefined value` は引数素材化の漏れが原因。
  - 対処: mir_call 作成前後の LocalSSA を全発行経路に適用済みかスイープし、欠落箇所を補強（EmitGuard で統一）。
- Plugin ABI: 既存 C ABI 互換エントリを registry に登録し、外部呼び出し経路を段階移行（resolver alias で完結）。
- Docs/Tests: Phase-31 変更点のドキュメント整備とスモーク追加（singleton/Verifier 回り）。

### 現在の既知問題（dev）
- quick→plugins→full でのカテゴリ 2/3（出力差・モジュール解決）の再確認が未実施。LegacyCallBridgeBox 導入後、差分の再スキャンが P0‑5 で残っている。
- Plugin ABI の追加トランポリンは不要。残りは resolver/Router 側の整合性チェックのみ。
