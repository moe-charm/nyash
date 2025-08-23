# HTTP Result Patterns (VM×Plugins)

目的: Netプラグイン（HTTP）の戻り値モデルをVM視点で明確化します。E2Eの実行方法と、典型ケースでのResultの形をまとめます。

## 実行方法（代表）
```bash
tools/run_vm_stats.sh local_tests/vm_stats_http_ok.nyash vm_stats_ok.json
tools/run_vm_stats.sh local_tests/vm_stats_http_err.nyash vm_stats_err.json
tools/run_vm_stats.sh local_tests/vm_stats_http_404.nyash vm_stats_404.json
tools/run_vm_stats.sh local_tests/vm_stats_http_500.nyash vm_stats_500.json
```

## 戻り値モデル
- unreachable（接続不可/タイムアウト等）: `Result.Err(ErrorBox)`
  - ErrorBoxには原因メッセージ（例: "connection refused", "timeout"）が入ります。
- HTTPステータス 404/500 等: `Result.Ok(Response)`
  - `response.status` が 404/500 を保持し、ボディやヘッダーは `response.body`, `response.headers` に格納されます。

## 使い分けの意図
- ネットワーク層の到達不能（transport）は「例外的」な失敗としてErr。
- アプリ層のHTTPステータスは「通常の結果」としてOkに包む（分岐の簡潔化・情報保持）。

## Tips
- デバッグ
  - `NYASH_NET_LOG=1 NYASH_NET_LOG_FILE=net_plugin.log` でNetプラグインの内部ログを記録。
  - `NYASH_DEBUG_PLUGIN=1` で VM→Plugin のTLV先頭情報をダンプ。
- 計測
  - `--vm-stats`/`--vm-stats-json` で命令プロファイルを取得し、BoxCallやNewBoxのホット度を把握。

関連ドキュメント
- `docs/reference/architecture/mir-to-vm-mapping.md`（Result/Handleの取り扱い）
- `docs/VM_README.md`（VM統計・既知の制約）
