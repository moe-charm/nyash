# Nyash VM 実行基盤ガイド（更新）

- プラグインBox引数の最小対応を追加（TLV: BoxRef）
- TLVタグ: 1=Bool, 2=I32, 3=I64, 4=F32, 5=F64, 6=String, 7=Bytes, 8=Handle(BoxRef)
  - BoxRefはプラグインBox参照（type_id:u32, instance_id:u32）を8バイトでエンコード
  - ユーザー定義Box・複雑なビルトインは当面非対応（toStringフォールバック）

現状のルーティング:
- User-defined: MIR関数（{Box}.{method}/{N}) にCall化（関数存在時）。それ以外はBoxCall。
- Builtin: BoxCall → VM内の簡易ディスパッチ。
- Plugin: BoxCall → PluginLoaderV2.invoke_instance_method。

今後のタスク:
- VM側のfrom Parent.method対応（Builder/VM両対応）
- TLVの型拡張（Float/配列/BoxRef戻り値など）

## 🧮 VM実行統計（NYASH_VM_STATS / JSON）

VMは命令カウントと実行時間を出力できます。

使い方（CLIフラグ）:
```bash
# 人間向け表示
nyash --backend vm --vm-stats program.nyash

# JSON出力
nyash --backend vm --vm-stats --vm-stats-json program.nyash
```

環境変数（直接指定）:
```bash
NYASH_VM_STATS=1 ./target/debug/nyash --backend vm program.nyash
NYASH_VM_STATS=1 NYASH_VM_STATS_JSON=1 ./target/debug/nyash --backend vm program.nyash
# 代替: NYASH_VM_STATS_FORMAT=json
```

出力は `total`（総命令数）, `elapsed_ms`（経過時間）, `counts`（命令種別→回数）, `top20`（上位20種）を含みます。

## 既知の制約とTips（VM×プラグイン）
- Netプラグイン（HTTP）
  - unreachable（接続不可/タイムアウト）は `Result.Err(ErrorBox)`。
  - HTTP 404/500 は `Result.Ok(Response)`（アプリ側で `response.status` を確認）。
  - デバッグ: `NYASH_NET_LOG=1 NYASH_NET_LOG_FILE=net_plugin.log`。
- FileBox
  - `close()` は `Ok(Void)`。`match Ok(_)` で受けるか、戻り値を無視してよい。
- Handle（BoxRef）戻り
  - TLV tag=8（type_id:u32, instance_id:u32）。Loaderが返り値typeに対応する `fini_method_id` を設定し `PluginBoxV2` を構築。
  - `scope_tracker` がスコープ終了時に `fini()` を呼ぶ（メモリ安全）。
- 大きいボディ/多ヘッダー/タイムアウト
  - 逐次拡張中。異常時の挙動は上記Result規約に従う。実行ログと `--vm-stats` を併用して診断。
- 反復タイムアウト: `local_tests/socket_repeated_timeouts.nyash` で `acceptTimeout/recvTimeout` の連続ケース確認
- BoxCallデバッグ: `NYASH_VM_DEBUG_BOXCALL=1` でBoxCallの受け手型・引数型・処理経路（enter/fastpath/unified）・結果型をstderr出力
  - 例: `NYASH_VM_DEBUG_BOXCALL=1 ./target/release/nyash --backend vm local_tests/test_vm_array_getset.nyash`

## 🔧 BoxCallの統一経路（Phase 9.79b）

### method_id（スロット）によるBoxCall
- Builderが受け手型を推論できる場合、`BoxCall`に数値`method_id`（スロット）を付与。
- 低スロットはユニバーサル予約（0=toString, 1=type, 2=equals, 3=clone）。
- ユーザー定義Boxは宣言時にインスタンスメソッドへスロットを4番から順に予約（決定論的）。

### VMの実行経路（thunk + PIC）
- ユニバーサルスロット（0..3）はVMのfast-path thunkで即時処理。
  - toString/type/equals/cloneの4種は受け手`VMValue`から直接評価。
- それ以外は以下の順で処理:
  1. Mono-PIC（モノモーフィックPIC）直呼び: 受け手型×method（またはmethod_id）のキーでホットサイトを判定し、
     `InstanceBox`は関数名キャッシュを使って `{Class}.{method}/{arity}` を直接呼び出す（閾値=8）。
  2. 既存経路: `InstanceBox`はMIR関数へCall、それ以外は各Boxのメソッドディスパッチへフォールバック。

環境変数（デバッグ）:
```bash
NYASH_VM_DEBUG_BOXCALL=1   # BoxCallの入出力と処理経路を出力
NYASH_VM_PIC_DEBUG=1       # PICヒットのしきい値通過時にログ
```

今後の拡張:
- 一般`method_id`（ユーザー/ビルトイン/プラグイン）に対するvtableスロット→thunk直呼び。
- PICのキャッシュ無効化（型version）と多相PICへの拡張（Phase 10）。
 - SocketBox（VM）
   - 基本API: `bind/listen/accept/connect/read/write/close/isServer/isConnected`
   - タイムアウト: `acceptTimeout(ms)` は接続なしで `void`、`recvTimeout(ms)` は空文字を返す
   - 簡易E2E: `local_tests/socket_timeout_server.nyash` と `socket_timeout_client.nyash`
 - Void 比較の扱い（VM）
   - `Void` は値を持たないため、`Eq/Ne` のみ有効。`Void == Void` は真、それ以外の型との `==` は偽（`!=` は真）。
   - 順序比較（`<, <=, >, >=`）は `TypeError`。

## E2E 実行例（HTTPのResult挙動）

代表ケースを `tools/run_vm_stats.sh` で実行できます。`--vm-stats-json` により命令プロファイルも取得可能です。

```bash
# 別ターミナルでサーバ起動
./target/release/nyash local_tests/http_server_statuses.nyash

# クライアント（別ターミナル）
tools/run_vm_stats.sh local_tests/vm_stats_http_ok.nyash vm_stats_ok.json
tools/run_vm_stats.sh local_tests/vm_stats_http_404.nyash vm_stats_404.json
tools/run_vm_stats.sh local_tests/vm_stats_http_500.nyash vm_stats_500.json

# 到達不能（サーバ不要）
tools/run_vm_stats.sh local_tests/vm_stats_http_err.nyash vm_stats_err.json
```

期待されるResultモデル
- unreachable（接続不可/タイムアウト）: `Result.Err(ErrorBox)`
- 404/500 等のHTTPエラー: `Result.Ok(Response)`（アプリ側で `response.status` を評価）

詳細: `docs/reference/architecture/mir-to-vm-mapping.md` と `docs/examples/http_result_patterns.md` を参照。
