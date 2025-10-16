# Phase‑31 — Box Normalization（Static→Singleton 正規化）

最終更新: 2025‑10‑17（Map.values stage2 fix 反映）

## サマリ
- ねらい: すべてのメソッド呼び出し形を「me + args」に統一し、static box を型ごとのシングルトンインスタンス（`Type.singleton`）に正規化する。
- 効果: ルータ分岐削減、receiver 有無によるバグ根絶（今回の ArrayBox(1) 類）、Extern/HostBridge 経路の単純化、AOP/計測の注入点の一元化。
- 方式: Builder で `Static(Type.method(args)) → Instance(Type.singleton.method(args))` に書換。ルータは常に receiver を渡す。Verifier で逸脱を Fail‑Fast。
- 導入: フラグ既定OFFで段階導入（A→B→C）。プラグイン ABI はトランポリン自動生成で互換維持。

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
- プラグイン ABI はトランポリンで互換維持。既存スモーク（quick→plugins→full）緑。
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

### 4) プラグイン ABI 互換（トランポリン）
- 自動生成方針:
  - 旧: `extern "C" Foo_canonicalize(json)` → 内部 `Foo::canonicalize(Foo::singleton, json)` 呼び。
  - 逆方向（必要時）: 新 ABI を旧 ABI に委譲する薄いラッパ。
- 配置: 生成器は `build.rs` or 専用小モジュール（`tools/` 発）で最初は静的表を元に作成。

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
  - トランポリン生成（最小箇所）
- スモーク
  - static/instance 同名メソッドの一致
  - HostBridge 経路（extern）での等価性
- 成果: quick 緑 + plugins 代表 PASS

### Phase B（互換・計測）
- 旧 ABI 域のトランポリンを網羅
- ベンチ/サイズ/ビルド時間の観測（リグレッション無し）
- full プロファイル PASS（WARN 非致命）

### Phase C（既定ON 検討）
- CI/ドキュメントの既定を新規約に統一
- 旧 ABI の非推奨化（1 リリース告知→削除計画）

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
  - 生成: `tools/` or `build.rs`（トランポリン）

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
- 受け入れ基準
  - 既存スモーク緑 + 新規スモーク（static/instance 等価）緑
  - LLVM/VM 出力差異がゼロ/許容範囲

---

## リスクと対策
- 互換性リスク（プラグイン）: トランポリンで遮断。段階導入。
- 反射の観測: LAYER_GUARD と Verifier。ドキュメントで未定義化を宣言。
- 性能劣化: ベンチで観測し、必要ならホットパスのみ旧 ABI 直呼びエントリを併存。

## ロールバック
- `HAKO_STATIC_AS_SINGLETON=0` で旧挙動へ即時復帰。
- 生成トランポリンは残しても害無し（削除は安定後）。

---

## 実装 TODO（P0→）
1. Builder に static→singleton 正規化を実装（最小: String/Array/Map 代表）
2. Router の receiver 常時渡しを再点検（不要分岐の撤去）
3. Verifier 追加（ModuleFunction 直呼び/receiver 欠落の Fail）
4. トランポリン生成の雛形（最小 1 箇所）
5. スモーク追加（static/instance 等価、extern 経路）
6. Docs 反映（ガイド/リファレンス/ENV 記載）

### Known issues（2025‑10‑17 時点）
- MapBox.remove が戻り値 `Some(value)` を返さないケースが残っている（Stage‑2 values は復旧済み）。
  - 対応方針: extern adapter で remove の戻り値を HostHandle 化し、EmitGuard 経路の素材化を統一。
- me 注入の誤適用により plugin ModuleFunction へ影響する懸念。
  - 対処: `method_index.static_signature()` に該当する静的シグネチャのみ `me` 合成を許可（Builder/VM 双方でガード）。
- FileBox 系の `use of undefined value` は引数素材化の漏れが原因。
  - 対処: mir_call 作成前後の LocalSSA を全発行経路に適用済みかスイープし、欠落箇所を補強（EmitGuard で統一）。

---

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

### Phase A‑1c — ModuleFunction トランポリン整備 & Verifier 補強（完了）
- 目的: ModuleFunction 経由の静的呼び出しで receiver 欠落を早期検出し、VM 側も常に receiver あり前提に揃える。
- 実装:
  - Verifier: `ModuleFunction(box.method/arity)` で Known かつ Box 型の受領者が無い場合に Fail‑Fast。
  - Runtime: MethodRouter が Void 受領者を即時 InvalidInstruction とし、legacy fallback も receiver 前提に統一。
  - Trampoline: `handlers/calls/trampolines.rs` を追加し、Array/Map/String/Console の ModuleFunction → Method 変換を表駆動化。

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

### 残タスク（Phase A 継続）
### Phase A‑1e — Map.size 正規化 & Extern 安定化（完了）
- 目的: `MapBox.(size|len|length)` を `Extern("nyrt.map.size")` に統一し、Method 形の残存で起きる受領者素材化漏れを排除。
- 実装:
  - `src/mir/builder/normalize/map_length.rs` 新設。`normalize::apply_all` に組み込み。
  - Optimizer 側で `nyrt.map.size` の Method への巻き戻しを抑止。
  - Lowering で `MapBox.size/0` → `nyrt.map.size(recv)` の直下ろしを追加。
- 効果: plugins プロファイルの `parity_map_size_has_vm` / `strict_plugin_map_size_vm` が PASS。

### Phase A‑1f — Map.keys/values と remove の整備（進行中）
- 目的: `Map.values()` の結果を ArrayBox として安定化し、直後の `.size()` などで型不一致や未定義値を起こさない。
- 実装:
  - Extern adapter で `nyrt.map.{keys,values}` を HostSlot 経由（テーブル駆動）＋ Plugin 経由の橋渡しに対応。
  - Builder で `Extern("nyrt.map.{values,keys}")` の結果に ArrayBox 注釈を付与（後段の利用で型ヒント）。
  - Map プラグインで `remove()` が削除した値を返すよう修正（i64/文字列/配列ハンドルを TLV で返却）。
- 残: `values()` 直後に `.size()` を呼ぶケースで、ArrayBox 注釈は付くものの、`.size()` 側の素材化順序が先行して未定義値参照になるパターンが残存。EmitGuard 後のローカル化を強制する追加ルールで詰める。

### Known issues（2025‑10‑16 plugins 再走後の把握）
- MapBox.values の直後に `.size()` を呼ぶケースで、ArrayBox 注釈は付くものの、`.size()` 側の受け素材化が先行して未定義値参照になるパターンが残存。
  - 対応: Builder 側の fast-path を拡張（ArrayBox 由来の `.size()` は `nyrt.array.size` に直下ろしし、EmitGuard 直後にローカル化）。Normalizer/Optimizer は Extern を保持。
- me 注入の誤適用により plugin ModuleFunction へ影響する懸念。
  - 対処: `method_index.static_signature()` に該当する静的シグネチャのみ `me` 合成を許可（Builder/VM 双方でガード）。
- FileBox 系の `use of undefined value` は引数素材化の漏れが原因。
  - 対処: mir_call 作成前後の LocalSSA を全発行経路に適用済みかスイープし、欠落箇所を補強（EmitGuard で統一）。
- Plugin ABI: 既存 C ABI 互換のトランポリンを registry に登録し、外部呼び出し経路を段階移行。
- Docs/Tests: Phase-31 変更点のドキュメント整備とスモーク追加（singleton/Verifier 回り）。

### 現在の既知問題（dev）
- quick→plugins→full でのカテゴリ 2/3（出力差・モジュール解決）の再確認が未実施。LegacyCallBridgeBox 導入後、差分の再スキャンが P0‑5 で残っている。
- Plugin ABI トランポリンの網羅化・自動生成は継続作業。現状は代表ケースのみ箱化されており、残りは TODO に記載のとおり。
