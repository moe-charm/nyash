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

## 2025-10-16 追記（Stage‑3: boxes 剥がしの完了条件と現在地）

Stage‑3 DoD（Definition of Done）
- 参照ゼロ（外縁）: `crate::boxes::*` が `src/boxes/**` と `#[cfg(feature="legacy-boxes")]` ガード配下以外に存在しない
- ビルド緑:
  - Legacy（既定ON）: `cargo build --release`
  - Plugin‑only（検証線）: `cargo build --release --no-default-features -F cli,plugins,host-anchors`
  - スモーク: quick と plugins（build‑only）が PASS
- 実行経路の分離: Router（builtin/plugin）と extern_adapter（core/future‑legacy）の二分体制が成立
- ドキュメント/CI: CURRENT_TASK と guides を更新、CIに plugin‑only build（build‑only）を追加（最小）

現在地（2025‑10‑16）
- refs 棚卸し: 外縁の直参照は `#[cfg(feature="legacy-boxes")]` で封じ込め済み（runtime/backend/tests）。
  - 代表: runner/dispatch の FloatBox、box_operators の FloatBox、result_conv の FloatBox、ast の Float は cfg 下で分岐。
  - messaging/transport はファイル/モジュール単位で legacy‑only。
- 検証: plugins プロファイルの build‑only スモーク（個別タイムアウト導入）→ PASS を継続確認。
- 追加作業（継続）: tests で legacy 直参照するファイルは先頭に `#![cfg(feature="legacy-boxes")]` を寄せて統一。

次の一手（Stage‑3 締め）
- by‑dir 上位から `crate::boxes::*` 参照を再確認（helper: `tools/dev/list_boxes_refs.sh --by-dir`）。
- 参照ゼロ化が確認できたら、CIに plugin‑only build（build‑only）ジョブを追加（docsのサンプルどおり）。
- 既定フラグOFF/`src/boxes/` 物理撤去は、観測安定後の独立PRで（可逆運用を優先）。

## Phase 2 — Parser/Resolver（構造→導線→段階移設）

目的
- 既存実装の挙動を変えずに“箱と境界”を先に整える。薄いFacadeで導線を用意し、ミニスモークで観測可能にする。

状態（2025-10-15）
- [x] Interfaces: `src/layers/interfaces.rs` に `ParserOutput`/`ResolverInput`/`FrontendError`
- [x] Guards: `src/front/{parser_layer,resolver_layer}/LAYER_GUARD.rs`
- [x] Facades: `front::parser_layer::facade::parse_source_to_ast`, `front::resolver_layer::facade::resolve_passthrough`
- [x] Impl適合: `ast::ASTNode` を `ParserOutput`/`ResolverInput` に適合
- [x] Doc: `docs/reference/frontend-layers.md`
- [x] Opt-in導線: Runner で `HAKO_FRONT_USE_FACADE=1` のとき Facade 経由でパース
- [x] Smoke（最小）: `quick-selfhost/parser_facade_min_vm.sh`

次アクション（P2-1）
1) README強化（I/O例・最小AST例）
   - `src/front/parser_layer/README.md` / `src/front/resolver_layer/README.md`
2) 追加スモーク（必要なら）
   - 空プログラム/単一関数などの極小ケースを1本
3) Facade採用の範囲拡大（任意・段階）
   - ランナーの他経路（例: demos/jit）でも dev-flag 下で Facade を採用
4) 本移設（別タスク）
   - 純関数ヘルパから段階移設。既存APIは薄い委譲に変更（挙動不変）

受け入れ基準
- quick/quick-selfhost 緑。既定挙動は不変（dev-flagのみ影響）。
- 仕様/契約が `docs/reference/frontend-layers.md` と README に明記されている。

## Phase 3 — Boxes Migration（レガシー撤退の実務）

目的
- `src/boxes/` の参照を段階的に削減し、plugin/HostHandleRouter 経路に一本化する。

現状（2025-10-15）
- plugin-only build は build-only 緑（警告あり）。refs は残っている（`tools/dev/list_boxes_refs.sh` 参照）。
- type_registry の core factory は legacy-OFF で plugin 優先（Array/Map/String）。
- router/extern/array helper は分割済み（builtin/plugin）。

次アクション（P3-1）
1) 参照の棚卸しと優先順位付け（fanout 高→低）
   - runner/extern/MIR handlers を先に cfg で囲い、plugin-only ビルドの阻害要因を解消
2) plugins プロファイルの代表スモークを必要に応じて追加（経路変更時）
3) plugin-only build を定期確認（build-only）
4) refs が 0 に近づいたら、短期ブランチで `legacy-boxes` 既定OFF→ `src/boxes/` 削除

受け入れ基準
- legacy（デフォルト）+ quick: 緑維持
- plugin-only: build-only 緑
- plugins プロファイル代表スモーク: 緑

ドキュメント
- 詳細計画: `PHASE_3_BOXES_MIGRATION.md`

## Phase 4 — Dual Parser Harness（入口の一本化・観測の強化）

目的
- Rust ラインと Hakorune 自己ホストラインの両方で同じスモークを回し、最小同等性を段階保証する。

範囲（最小）
- ハーネス: `SMOKES_PARSER_MODE=rust|hako|both`（既定=rust）
- 仕様: 最小 JSON v0 ヘッダ（`{"version":"0","kind":"Program","stats":{"stmts":N}}`）
- 判定: `version/kind/stats.stmts` の比較のみ（AST 完全一致は要求しない）

受け入れ基準
- `both` で Phase‑A スモーク（セミコロン受理/if‑else/ブロック終端/using 最小）が緑
- 既定（rust）の速度・安定は不変

備考
- セミコロンは既定受理（Rust/ハコ双方）。無効化は `NYASH_PARSER_ALLOW_SEMICOLON=0`（開発用）

進捗メモ（2025-10-15 P3-5〜P3-9 抜粋）
- WASM v2 backend を legacy 前提にゲート（plugin-only との交差を回避）
- LLVM interpreter の Float/Null を cfg 分岐（plugin-only は String/Void 代替）
- tests の legacy 直参照に軽ガード（build-with-tests 互換）
  - box_tests/refcell_assignment_test/vm_functionbox_call/policy_mutdeny などを legacy 前提に
  - host_reverse_slot は plugin-only で PluginBox 経路に切替

### 2025-10-16 追記（Stage‑4 完了ライン TODO）
- [x] C‑ABI ハーネス（2関数）配置（feature `parser-c-abi` 配下）
- [x] Rust 薄層（ヘッダ出力・C文字列ユーティリティ）実装
- [x] both モード最小スモーク追加（ヘッダ一致／ビルド失敗時は SKIP）
- [x] hako ヘッダは当面 Rust パーサを再利用して parity を確保（後に Hako Parser に差替）
- [x] Docs 整理（Start Here/DoD/Status と 15.76 への導線）
- [x] ランナー導線（任意）：`SMOKES_PARSER_MODE=both|hako` の時に C‑ABI を呼ぶ薄い分岐（bin 依存を解消）
- [ ] CI（任意）：`parser-c-abi` を build‑only で1本だけ回す軽量ジョブ（既定 OFF）

## Next — Phase 1（MirCall 小粒・箱化方針で進行）

- [x] 安全メソッドの whitelist を箱化（一元化）
  - 実装: `normalize.rs` に `is_safe_core_method(&str,&str)` を追加し、ModuleFunction/BoxCall 降格の両方から参照
- [x] Callable(argv 再構成) のパリティ確認（軽スモーク）
  - quick-selfhost に arity>0 の `methodRef.call(argv)` 正常系を1本追加（Extern無効でも緑）
- [x] HostHandleRouter 境界スモーク（-14: 返却型不一致）
  - plugins プロファイルに1本追加（型不一致を明示検出）。ENVフック `HAKO_HOSTHANDLE_TEST_RET_MISMATCH=1` で再現
- [ ] 正規化の安全拡張（必要時のみ）
  - 既存正常系（Array.slice/join、Map.delete/clear）を維持。追加が必要なら `is_safe_core_method` にのみ追記（箱化原則）
  - 2025-10-15: StringBox.replace を安全リストに追加済み（Extern/ModuleFunction → Method 降格）。
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
