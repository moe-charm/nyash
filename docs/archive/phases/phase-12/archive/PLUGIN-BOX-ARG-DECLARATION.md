# プラグインBoxの箱引数宣言方法

## 📊 nyash_box.tomlでの宣言

### 1. 基本的な箱引数の宣言

```toml
[HttpRequestBox.methods.respond]
id = 3
args = [ { name = "resp", type = "box" } ]  # type = "box" で箱引数を宣言
returns = { type = "void" }
```

### 2. 引数の型一覧

| 型指定 | 説明 | TLVタグ |
|--------|------|---------|
| `"i64"` | 64ビット整数 | 3 |
| `"f64"` | 64ビット浮動小数点 | 5 |
| `"string"` | UTF-8文字列 | 6 |
| `"bool"` | 真偽値 | 1 |
| `"box"` | **箱（ハンドル）** | 8 |

### 3. 実際の使用例

#### NetプラグインのHttpRequestBox
```toml
[HttpRequestBox]
type_id = 21

[HttpRequestBox.methods.respond]
id = 3
args = [ { name = "resp", type = "box" } ]  # HttpResponseBoxを受け取る
returns = { type = "void" }
```

使用方法（Nyash）：
```nyash
local request = server.accept()      // HttpRequestBox
local response = new HttpResponseBox()  // 別のプラグインBox
response.setStatus(200)
request.respond(response)  // 箱を引数として渡す！
```

#### 戻り値が箱の例
```toml
[HttpServerBox.methods.accept]
id = 3
args = []
returns = { type = "box" }  # HttpRequestBoxを返す
```

## 🔧 C実装側での処理

### TLVデコード
```c
// HttpRequestBox.respondの実装例
case 3: { // respond
    // 引数をデコード
    if (args_len < 12) return -1;  // header(4) + handle(8)
    
    // TLVタグチェック
    uint8_t tag = args[4];
    if (tag != 8) return -1;  // TAG_HANDLE = 8
    
    // ハンドルデータ取得
    uint32_t resp_type_id = *(uint32_t*)&args[8];
    uint32_t resp_instance_id = *(uint32_t*)&args[12];
    
    // HttpResponseBox（type_id=22）であることを確認
    if (resp_type_id != 22) return -1;
    
    // レスポンス処理...
}
```

## 💡 重要なポイント

### 1. 型安全性
- `type = "box"`は任意の箱を受け取れる
- 実装側で`type_id`チェックにより型安全性を確保

### 2. 相互運用性
- 異なるプラグイン間でも箱の受け渡しが可能
- ハンドル（type_id + instance_id）により参照

### 3. 宣言の簡潔さ
```toml
# シンプルな宣言
args = [ { name = "box_arg", type = "box" } ]

# 複数の箱引数も可能
args = [ 
    { name = "box1", type = "box" },
    { name = "box2", type = "box" },
    { name = "count", type = "i64" }
]
```

## 🎯 結論

プラグインBoxは`nyash_box.toml`で`type = "box"`と宣言するだけで、他の箱を引数に取ることができます。C ABIレベルではTLVハンドル（タグ8）として処理され、完全な相互運用性が実現されています。