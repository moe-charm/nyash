# E2E Tests Documentation

最終更新: 2025-08-23

## E2E Plugin Net Tests (e2e_plugin_net.rs)

### HTTP Tests

#### 基本的なHTTP通信
- `e2e_http_ok_test` - 基本的なHTTP通信の成功ケース
- `e2e_http_post_and_headers` - POSTリクエストとヘッダー処理
- `e2e_http_empty_body` - 空ボディのレスポンス処理
- `e2e_http_multiple_requests_order` - 複数リクエストの順序処理

#### HTTPステータスコード処理
- `e2e_vm_http_status_404` - 404 Not Found レスポンス処理
  - サーバーが404ステータスで応答 → クライアントは `Result.Ok(HttpResponseBox)` を受信
  - `response.getStatus()` で 404、`response.readBody()` で "NF" を取得
- `e2e_vm_http_status_500` - 500 Internal Server Error レスポンス処理
  - サーバーが500ステータスで応答 → クライアントは `Result.Ok(HttpResponseBox)` を受信
  - `response.getStatus()` で 500、`response.readBody()` で "ERR" を取得

#### HTTPエラー処理
- `e2e_vm_http_client_error_result` - 接続失敗時のエラー処理
  - ポート8099への接続失敗 → `Result.Err(ErrorBox)` を返す
  - エラーメッセージ: "connect failed for 127.0.0.1:8099/nope"

### Socket Tests
- `e2e_socket_echo` - ソケットエコーサーバー
- `e2e_socket_timeout` - ソケットタイムアウト処理

## E2E Plugin Net Additional Tests (e2e_plugin_net_additional.rs)

### 高度なHTTPエラー処理
- `e2e_vm_http_client_error_result` - VM環境での接続エラー処理
- `e2e_vm_http_empty_body` - VM環境での空ボディ処理

## HTTPエラーハンドリングの重要な区別

### 接続失敗（Transport Layer Error）
- **状況**: サーバーに到達できない（unreachable）
- **結果**: `Result.Err(ErrorBox)`
- **例**: ポートが閉じている、ホストが存在しない

### HTTPステータスエラー（Application Layer Error）
- **状況**: サーバーに到達したがアプリケーションエラー
- **結果**: `Result.Ok(HttpResponseBox)`
- **例**: 404 Not Found、500 Internal Server Error
- **理由**: トランスポート層は成功、HTTPプロトコルとして正常な応答

この区別により、ネットワークレベルのエラーとアプリケーションレベルのエラーを適切に処理できます。