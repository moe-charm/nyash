# 可選: ResultBox 正規化（returns_result）

最終更新: 2025-08-22

## 概要
`nyash.toml` のメソッド定義に `returns_result = true` を付けると、そのメソッドの戻りが `ResultBox` で正規化されます。
- 成功: `Ok(value)`（voidは `Ok(void)`）
- 失敗: `Err(ErrorBox("... (code: N)"))`（BID負エラーコードをErr化）

これは「おすすめルール」で、強制ではありません。段階的に、必要なメソッドから選んで導入できます。

## 使い方
`nyash.toml` を編集し、対象メソッドに `returns_result = true` を付けます。

```toml
[libraries."libnyash_net_plugin.so".HttpClientBox.methods]
# birth = { method_id = 0 }
# デフォルト（従来通り）
get  = { method_id = 1 }
post = { method_id = 2 }
# 推奨: Result正規化（有効化する場合）
# get  = { method_id = 1, returns_result = true }
# post = { method_id = 2, returns_result = true }
```

## 呼び出し側パターン
```nyash
res = http.get(url)
if res.is_ok() {
    resp = res.get_value()
    print(resp.readBody())
} else {
    err = res.get_error()
    print("HTTP error: " + err.toString())
}
```

## 推奨の導入順序
- Stage 1: `HttpClientBox.get/post`, `SocketClientBox.connect`
- Stage 2: `HttpServerBox.start/stop/accept`（起動・待受系）
- Stage 3: 失敗が起きうる便宜メソッド（必要なところだけ）

## 注意
- `returns_result` を付けたメソッドのみ Result化されます。未指定メソッドは従来動作（生値/void/例外）。
- タイムアウトをErrにしたい場合は、プラグイン側がBID負エラーを返すよう拡張してください（現状は空bytes/void）。
- 段階導入により、既存コードを壊さずに移行できます。

