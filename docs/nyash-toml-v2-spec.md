# nyash.toml v2 仕様 - マルチBox型プラグイン対応

## 🎯 概要
1つのプラグインライブラリが複数のBox型を提供できるように拡張した仕様。

## 📝 基本構造

### 1. 後方互換性のある現行形式（単一Box型）
```toml
[plugins]
FileBox = "nyash-filebox-plugin"

[plugins.FileBox.methods]
read = { args = [] }
write = { args = [{ from = "string", to = "bytes" }] }
```

### 2. 新形式：マルチBox型プラグイン
```toml
# ライブラリ定義
[plugins.libraries]
"nyash-network" = {
    plugin_path = "libnyash_network.so",
    provides = ["SocketBox", "HTTPServerBox", "HTTPRequestBox", "HTTPResponseBox"]
}

# 各Box型の詳細定義
[plugins.types.SocketBox]
library = "nyash-network"
type_id = 100
methods = {
    bind = { args = [
        { name = "address", from = "string", to = "string" },
        { name = "port", from = "integer", to = "u16" }
    ]}
}

[plugins.types.HTTPServerBox]
library = "nyash-network"
type_id = 101
methods = {
    bind = { args = [
        { name = "address", from = "string", to = "string" },
        { name = "port", from = "integer", to = "u16" }
    ]},
    route = { args = [
        { name = "path", from = "string", to = "string" },
        { name = "handler", from = "box", to = "box" }
    ]}
}
```

## 🔧 型システム

### サポートする型（基本型のみ）
```toml
{ from = "string", to = "string" }      # 文字列
{ from = "integer", to = "i64" }        # 整数
{ from = "float", to = "f64" }          # 浮動小数点
{ from = "bool", to = "bool" }          # 真偽値
{ from = "bytes", to = "bytes" }        # バイト配列
```

**重要**: プラグインとNyash本体間では基本型のみやり取り。Box型の受け渡しは行わない（箱は箱で完結）。

### 2. プラグインFFI拡張
```c
// 既存: 単一Box型
nyash_plugin_abi_version()
nyash_plugin_init()

// 新規: 複数Box型
nyash_plugin_get_box_count()  // 提供するBox型の数
nyash_plugin_get_box_info(index)  // 各Box型の情報
```

## 📊 実装優先順位

1. **Phase 1**: nyash.toml v2パーサー実装
   - 後方互換性維持
   - 新形式の読み込み

2. **Phase 2**: plugin-tester拡張
   - 複数Box型の検出
   - 各Box型のメソッド検証

3. **Phase 3**: ローダー拡張
   - 複数Box型の登録
   - 型ID管理

## 🎯 HTTPServerBox依存問題の解決

この設計により、以下が可能になります：

```toml
[plugins.libraries]
"nyash-network" = {
    plugin_path = "libnyash_network.so",
    provides = ["SocketBox", "HTTPServerBox", "HTTPRequestBox", "HTTPResponseBox"]
}

# HTTPServerBoxはプラグイン内でSocketBoxを直接使用可能
# MapBoxへの依存は以下のように解決：
# - HTTPResponseBoxは内部でHashMapを使用
# - get_header("name") で個別アクセス
# - get_all_headers() は文字列配列として返す
```