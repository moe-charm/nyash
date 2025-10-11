# CURRENT_TASK — 現在のタスクと進捗


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
  ./target/release/hako apps/selfhost/hakorune-vm/tests/test_phase1_minimal.hako

# Phase 1 Day 3 テスト（5テスト）
HAKO_ALLOW_USING_FILE=1 HAKO_USING_PROFILE=dev NYASH_USING_AST=1 NYASH_DISABLE_PLUGINS=1 \
  ./target/release/hako apps/selfhost/hakorune-vm/tests/test_phase1_day3.hako

# Phase 2 Day 4 テスト（7テスト - UnaryOp）
HAKO_ALLOW_USING_FILE=1 HAKO_USING_PROFILE=dev NYASH_USING_AST=1 NYASH_DISABLE_PLUGINS=1 \
  ./target/release/hako apps/selfhost/hakorune-vm/tests/test_phase2_day4.hako
```

### 箱ファイル一覧
```
apps/selfhost/hakorune-vm/
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