# Net Plugin (HTTP over TCP PoC)

最終更新: 2025-08-22

## 概要
- `nyash-net-plugin` は Socket/HTTP をプラグインとして提供します。
- HTTP は最小限の HTTP/1.1 実装（GET/POST、ヘッダ、Content-Length）を実ソケットで処理します。

## 提供Box
- `SocketServerBox`, `SocketClientBox`, `SocketConnBox`
- `HttpServerBox`, `HttpRequestBox`, `HttpResponseBox`, `HttpClientBox`

`nyash.toml` 定義例（抜粋）はリポジトリ直下の `nyash.toml` を参照。

## 動作仕様（HTTP）
- Server
  - `start(port)`: TCP待受を開始
  - `accept()`: 接続受理＋リクエスト簡易パース（path/body）→ `HttpRequestBox` を返す
  - `HttpRequestBox.respond(resp)`: `resp` の status/header/body を HTTP/1.1 として送出（`Connection: close`）
- Client
  - `get(url)`, `post(url, body)`: host:port に接続し、HTTP/1.1 リクエストを送出
  - `HttpResponseBox` は遅延受信。`readBody()/getStatus()/getHeader()` 呼び出し時に受信・パース

制限:
- Chunked/Keep-Alive/HTTP/2は非対応（PoC）。Content-Length のみ対応。

## エラーモデル（PoC）
- BID-FFI の負値（例: `-5` PluginError）は Nyash 側で例外（`RuntimeFailure`）として扱われます。
- タイムアウト系（Socket の `acceptTimeout/recvTimeout`）はエラーではなく、以下の値を返します:
  - `acceptTimeout(ms)`: タイムアウト時は `void`
  - `recvTimeout(ms)`: タイムアウト時は 空 `bytes`（長さ0の文字列）

### HTTPエラーハンドリング（重要）
- **接続失敗（unreachable）**: `Result.Err(ErrorBox)` を返す
  - 例: ポート8099に接続できない → `Err("connect failed for 127.0.0.1:8099/...")`
- **HTTPステータスエラー（404/500等）**: `Result.Ok(HttpResponseBox)` を返す
  - 例: 404 Not Found → `Ok(response)` で `response.getStatus()` が 404
  - トランスポート層は成功、アプリケーション層のエラーとして扱う

将来の整合:
- `ResultBox` での返却に対応する設計（`Ok(value)`/`Err(ErrorBox)`）を検討中。
- `nyash.toml` のメソッド宣言に戻り値型（Result）を記載し、ランタイムで自動ラップする案。

## 並列E2E運用の注意
- 各テストで異なるポートを使用（例: 8080/8081/8090/8091...）
- サーバの `stop()` と再起動時はポート開放の遅延に注意（短い待機を挟むと安定）
- ログを有効化して競合や順序の問題を診断可能

## ログ出力
環境変数で簡易ログを有効化できます。

```bash
export NYASH_NET_LOG=1
export NYASH_NET_LOG_FILE=net_plugin.log
```

stderr とファイルの両方に出力されます。

## 例
```nyash
local srv, cli, r, req, resp
srv = new HttpServerBox()
srv.start(8080)

cli = new HttpClientBox()
r = cli.get("http://localhost:8080/hello")

req = srv.accept()
resp = new HttpResponseBox()
resp.setStatus(200)
resp.setHeader("X-Test", "V")
resp.write("OK")
req.respond(resp)

// client side
print(r.getStatus())
print(r.getHeader("X-Test"))
print(r.readBody())
```
