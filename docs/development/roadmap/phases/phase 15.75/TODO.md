# Phase 15.75 — 脱Rust 大作戦 TODO (P1/P2 入口)

**📚 [← INDEX.md に戻る](./INDEX.md)** | **🎯 今すぐやること（1-2週間）** | **🗂️ [長期バックログは TODO_ROADMAP.md](./TODO_ROADMAP.md)**

---

目的
- Rust 実装を薄い橋(HostBridge/Router/ABI)に縮退し、Hakorune(.hako) 側へ段階移譲。
- すべて段階導入・スモーク緑維持・実行時 capability で進める。

優先タスク（P1 スコープ / 次の1〜2週間）
1. VM シュガー: Map.call P1（同期のみ） — [DONE]
   - 仕様: docs/guides/map-callable.md
   - 実装: VM ルーターに薄い分岐（get→call）。プラグイン変更なし。
   - スモーク: 成功/欠損/非Callable の3本
   - ゲート: HAKO_MAP_CALL_ENABLE=1（devオン、既定OFF）

2. Fallback アダプタの箱化（keysS/valuesS） — [DONE]
   - 目的: Rust 内のフォールバック文字列→配列化を .hako 側に移設（HostBridge 呼び出しの一本化）
   - 実装: adapter.hako（String→Array変換）＋ Runner 側の軽接続
   - スモーク: 既存 fallback/stage2/identity の3本で緑維持

3. HostHandleRouter 段階移設（薄いリダイレクト） — [P1: stub DONE]
   - 現状: host_api.rs 内の分岐
   - 目標: src/runtime/host_handle_router へ委譲、router内で Box 種別毎の slot に分散
   - スモーク: stage2(keys/values) の緑維持で Accept

4. Parser/Tokenizer 小片の先行移行（純関数領域）
   - 方針: Token utils/小スキャナ等の純関数を .hako へ移し、Rust 側は Adapter 経由に縮退
   - スモーク: JSON/文字列/mini-parser の軽ケース（quick）

受け入れ基準
- quick/full スイート緑（新規/既存スモークふくめ）。
- 新機能は実行時 capability で既定OFF（devでON）。
- docs（guides/proposals）に仕様/使い方/戻し方が明記されていること。

参考
- 仕様草案: docs/guides/map-callable.md
- 統一API: docs/guides/collections-api.md
- 脱Rust計画群: docs/development/proposals/phase-15.75/

## 2025-10-14 追記（実行ラインの名称統一・導線整備）

- [x] bin ラッパー追加（名称は hakorune 系で統一、実体は nyash）
  - `bin/hakorune` → `target/release/nyash` ラッパー
  - `bin/hakorune-stage0` → Rust VM ライン（nyash デフォルト）
  - `bin/hakorune-stage1` → LLVM ライン（`nyash --backend llvm`）
- [x] ドキュメント整合
  - TOOLCHAIN_REQUIREMENTS.md に alias/ラッパー対応を追記
  - BOOTSTRAP_STRATEGY_MIR_ROLLBACK.md に bin 構成・備考を追記
- [x] mod.rs（router）のコメント化ブロックを物理削除（差分衝突回避後の小パッチ）
- [ ] Phase 1（MirCall）着手準備：必要時のみ `extern_adapter.rs`/`vm_types.rs` に最小ガード（可逆）

## 2025-10-15 追記（Phase 0-mini 仕上げ）

- [x] extern_adapter 分割とハブ化（core/future を分割登録）
  - core: `extern_adapter/extern_core.rs`
  - future: `extern_adapter/extern_future_legacy.rs`
- [x] extern_adapter のインライン重複ハンドラを物理削除（hub最小化）
  - 削除: `nyrt.time.now_ms`, `nyrt.string.*`, `nyrt.array.size`, `nyrt.map.{size,keys,values}`, `env.future.*`
  - 残置: 分割モジュールの `register(..)` のみ（hubは登録呼び出しとその他最小機能）
- [x] array_flatten_helper の二分割（builtin/plugin）とファサード委譲
  - builtin: `array_flatten_helper_builtin.rs` / plugin: `array_flatten_helper_plugin.rs`
  - 呼び出し箇所に README 参照コメントを最小付与
- [x] 正規化の拡充（Extern→Method）: nyrt.map.{keys,values} / string系を Method に降格

### 今後1週間（詳細タスク / 明確化）
- [x] Router README の最小追記（HostHandle slot/ENV 一覧と責務）
  - `src/runtime/method_router_box/README.md`
- [x] Quick ロールアウト Step‑3（観測拡大）
  - `quick.env` に `NYASH_MAP_GET_FORCE_HOST=1`, `NYASH_MAP_SET_FORCE_HOST=1` を導入（観測ON）
  - 代表スモークを quick に昇格（get_missing / set_effect / size_has）→ 緑確認
- [x] Plugin‑only CI 枠（build‑only）を docs に定義
  - `docs/guides/plugin-only-build.md` に minimal CI サンプル追記（build 緑のみ）
- [x] Extern disabled 診断の安定化メモ
  - `docs/reference/vm/call-unification.md` に「plugin‑only で Extern は明示エラー」の注記を補足（定数化: DIAG_EXTERN_DISABLED）
- [ ] 既知の軽警告の後始末（機会があれば）
  - `host_api_anchors/mod.rs` の属性整列、未使用 import の整理

## Next — Phase 1（MirCall 小粒・箱化方針で進行）

- [x] 安全メソッドの whitelist を箱化（一元化）
  - 実装: `normalize.rs` に `is_safe_core_method(&str,&str)` を追加し、ModuleFunction/BoxCall 降格の両方から参照
- [x] Callable(argv 再構成) のパリティ確認（軽スモーク）
  - quick-selfhost に arity>0 の `methodRef.call(argv)` 正常系を1本追加（Extern無効でも緑）
- [x] HostHandleRouter 境界スモーク（-14: 返却型不一致）
  - plugins プロファイルに1本追加（型不一致を明示検出）。ENVフック `HAKO_HOSTHANDLE_TEST_RET_MISMATCH=1` で再現
- [ ] 正規化の安全拡張（必要時のみ）
  - 既存正常系（Array.slice/join、Map.delete/clear）を維持。追加が必要なら `is_safe_core_method` にのみ追記（箱化原則）
- [x] ドキュメントの参照強化（設計の導線統一）
  - `docs/reference/vm/call-unification.md` に `is_safe_core_method`（安全判定の箱）への短い言及を追加
- [ ] plugin-only 警告のスポット掃除（任意）
  - 未使用変数/関数に限定して抑制（機能差なし）

Status Notes (2025‑10‑13)
- plugins: keys()/values() は Stage‑2 既定ON（HostHandle(Array)）。values 要素は PluginHandle(Array) 直返し可能。
- Router: keysS/valuesS→Array の文字列シムは撤退済み。
- Map.call P1 実装＆スモーク完了。

---

次アクション（確定 / 短期）

1) Map.call P1 最終確定（VMシュガー）
- selfhost/hakorune-vm/method_call_handler.hako の `call/1`, `call/2` ルートを最終確認（存在確認済）
- 既存スモーク3本（成功/欠損/非Callable）を再実行し緑確認
- 境界スモークを1本追加（例: 欠損キー＋非Callableの混在ケースで診断メッセージ確認）

2) plugin-on スモーク拡充（Stage‑2: HostHandle Array）
- Map.keys/Map.values の Stage‑2（HostHandle Array）検証を2本追加
  - keys: 配列長・要素内容の最小確認
  - values: 型（Handle/Box）の期待形状とアクセス可否
- identity ケース追加: `arr -> map.set -> get -> 同一` の最小確認

3) 段階撤退の次ステップ（便宜ハンドラの縮退）
- VM 便宜ハンドラの次候補を順にドキュメント/ガード更新（String → Map → Array の順）
- BoxCall fast‑path の段階停止フラグをテストプロファイルでONにして緑確認（既定はOFFを維持）

4) HostHandleRouter の段階移設（小粒・安全）
- 既存関数を1つだけ router に委譲して緑維持（小ステップで進める）

整備タスク（短時間で高効果）
- JSON抽出の残り `indexOf` を JsonFieldExtractor/JsonCursor に寄せる（10–15分×数箇所）
- boxcall_builder に `build_method` を使う呼び出しを1箇所適用（再利用性向上・差分小）
