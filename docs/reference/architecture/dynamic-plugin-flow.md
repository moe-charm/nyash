# Dynamic Plugin Flow (VM × Registry × PluginLoader v2)

最終更新: 2025-08-23

目的
- Nyash 実行時に、MIR→VM→Registry→Plugin の呼び出しがどう流れるかを図解・手順で把握する。
- TLVエンコード、ResultBoxの扱い、Handleのライフサイクル、nyash.tomlとの連携を1枚で理解する。

## ハイレベル流れ（シーケンス）

```
Nyash Source ──▶ MIR (Builder)
                  │  (BoxCall/NewBox/…)
                  ▼
               VM Executor
                  │  (BoxCall dispatch)
                  ├─ InstanceBox → Lowered MIR 関数呼び出し
                  ├─ BuiltinBox  → VM内ディスパッチ
                  └─ PluginBoxV2 → PluginLoader v2
                                  │  (nyash.toml を参照)
                                  ▼
                           Invoke (TLV)
                                  │
                                  ▼
                           Plugin (lib*.so)
                                  │  (戻り値をTLVで返却)
                                  ▼
                           Loader でデコード
                                  │  (returns_result/Handle/型)
                                  ▼
                           NyashBox (ResultBox/PluginBoxV2/基本型)
                                  │
                                  ▼
                              VM に復帰
```

## 主要構成要素
- MIR: `MirInstruction::{BoxCall, NewBox, …}` で外部呼び出し箇所を明示。
- VM: `src/backend/vm.rs`
  - InstanceBoxは `{Class}.{method}/{argc}` のLowered関数へ呼び出し
  - BuiltinはVM内の簡易ディスパッチ
  - PluginBoxV2は Loader v2 へ委譲
- Registry/Runtime: `NyashRuntime` + `box_registry` + `plugin_loader_v2`
  - `nyash.toml` の `libraries.*` を読み込み、Box名→ライブラリ名、type_id、method_id等を集約

## NewBox（生成）
1) MIRの `NewBox { box_type, args }`
2) VM: `runtime.box_registry` に `box_type` を問い合わせ
3) PluginBoxの場合、Loader v2が `birth(method_id=0)` を TLV で呼び出し
4) Pluginは `type_id` と新規 `instance_id` を返却 → Loader は `PluginBoxV2` を構築
5) VMは `ScopeTracker` に登録（スコープ終了で `fini` を呼ぶ）

## BoxCall（メソッド呼び出し）
- InstanceBox: Lowered関数 `{Class}.{method}/{argc}` を MIR/VM内で実行
- Builtin: VM内の `call_box_method` で対応（StringBox.length 等）
- PluginBoxV2: Loader v2 の `invoke_instance_method` で TLV を組み立てて呼び出し

## TLV（Type-Length-Value）
- ヘッダ: `u16 ver=1`, `u16 argc`
- 各引数: `u8 tag`, `u8 reserved`, `u16 size`, `payload`
- 主な tag:
  - 2 = i32 (size=4)
  - 6 = string, 7 = bytes
  - 8 = Handle(BoxRef) → payload = `u32 type_id || u32 instance_id`
  - 9 = void (size=0)

## 戻り値のマッピング（重要）
- `returns_result=false`
  - tag=8 → PluginBoxV2（Handle）
  - tag=2 → IntegerBox、tag=6/7 → StringBox、tag=9 → void
- `returns_result=true`（ResultBoxで包む）
  - tag=8/2 → `Result.Ok(value)`
  - tag=6/7 → `Result.Err(ErrorBox(message))`（Netプラグインなどがエラー文字列を返却）
  - tag=9 → `Result.Ok(void)`

補足
- VM内で ResultBox の `isOk/getValue/getError` をディスパッチ済み
- `toString()` フォールバックにより任意の Box を安全に文字列化可能

## Handle（BoxRef）のライフサイクル
- Loaderは `(type_id, instance_id)` を `PluginBoxV2` としてラップ
- `share_box()` は同一インスタンス共有、`clone_box()` はプラグインの birth を呼ぶ（設計意図による）
- `fini` は `ScopeTracker` または Drop で保証（プラグインの `fini_method_id` を参照）

## 具体例（HttpClientBox.get）
1) Nyash: `r = cli.get(url)`
2) MIR: `BoxCall`（returns_result=true）
3) VM→Loader: TLV（url = tag=6）
4) Loader→Plugin: `invoke(type_id=HttpClient, method_id=get)`
5) Plugin:
   - 接続成功: `Handle(HttpResponse)` を返す → Loaderは `Result.Ok(PluginBoxV2)`
   - 接続失敗: `String("connect failed …")` を返す → Loaderは `Result.Err(ErrorBox)`
6) Nyash: `if r.isOk() { resp = r.getValue() … } else { print(r.getError().toString()) }`

## nyash.toml 連携
- 例: `libraries."libnyash_net_plugin.so".HttpClientBox.methods.get = { method_id = 1, args=["url"], returns_result = true }`
- Loaderは `method_id` と `returns_result` を参照し、TLVと戻り値のラップ方針を決定
- 型宣言（args/kind）により、引数のTLVタグ検証を実施（不一致は InvalidArgs）

## デバッグTips
- `NYASH_DEBUG_PLUGIN=1`: VM→Plugin の TLV ヘッダと先頭64バイトをプレビュー
- `NYASH_NET_LOG=1 NYASH_NET_LOG_FILE=net_plugin.log`: Netプラグイン内部ログ
- `--dump-mir --mir-verbose`: if/phi/return などのMIRを確認
- `--vm-stats --vm-stats-json`: 命令使用のJSONを取得（hot pathの裏取りに）

## 将来の整合・改善
- nyash.toml に ok側の戻り型（例: `ok_returns = "HttpResponseBox"`）を追加 → Loader判定の厳密化
- Verifier強化: use-before-def across merge の検出（phi誤用を早期に発見）
- BoxCall fast-path の最適化（hot path最優先）

関連
- `docs/reference/plugin-system/net-plugin.md`
- `docs/reference/architecture/mir-to-vm-mapping.md`
- `docs/reference/architecture/mir-26-instruction-diet.md`

