# プラグインBoxは既に箱を引数に取れる！

## 🎯 重要な発見

**プラグインBoxは既にC ABIレベルで箱を引数に取ることができます！**

## 📊 実装の詳細

### 1. TLVプロトコルでのハンドルサポート

```rust
// TLVタグ定義
const TAG_HANDLE: u8 = 8;  // プラグインハンドル用

// ハンドルエンコード関数
pub fn plugin_handle(buf: &mut Vec<u8>, type_id: u32, instance_id: u32) {
    buf.push(TAG_HANDLE);
    buf.push(0u8); // reserved
    buf.extend_from_slice(&(8u16).to_le_bytes());  // size = 8
    buf.extend_from_slice(&type_id.to_le_bytes());      // 4 bytes
    buf.extend_from_slice(&instance_id.to_le_bytes());  // 4 bytes
}
```

### 2. プラグイン呼び出し時の処理

```rust
// Nyashコード
box1.process(box2, box3)

// ↓ VM/プラグインローダーでの処理
for arg in args {
    if let Some(p) = arg.as_any().downcast_ref::<PluginBoxV2>() {
        // 箱引数はハンドルとしてエンコード
        encode::plugin_handle(&mut tlv, p.type_id, p.instance_id);
    }
    // ... 他の型の処理
}

// ↓ C ABIプラグイン側
int32_t nyash_plugin_invoke(
    uint32_t type_id,
    uint32_t method_id,
    uint32_t instance_id,
    const uint8_t* args,    // TLVエンコードされた引数
    size_t args_len,
    uint8_t* result,
    size_t* result_len
) {
    // TLVデコード
    uint8_t tag;
    uint32_t arg_type_id, arg_instance_id;
    
    if (decode_handle(args, &tag, &arg_type_id, &arg_instance_id)) {
        // ハンドル引数を処理
        // arg_type_id と arg_instance_id で箱を特定
    }
}
```

## 🔄 実際の使用例

### Nyashレベル
```nyash
// FileBoxがStringBoxを引数に取る例
local file = new FileBox()
local path = new StringBox("/tmp/test.txt")
file.open(path)  // StringBox（プラグインBox）を引数に！

// ArrayBoxがMapBoxを引数に取る例
local array = new ArrayBox()
local map = new MapBox()
array.push(map)  // MapBox（プラグインBox）を引数に！
```

### プラグイン間の相互運用
```nyash
// NetBoxがJSONBoxを引数に取る例
local net = new NetBox()
local json = new JSONBox()
json.set("url", "https://api.example.com")
net.post(json)  // JSONBoxを引数として渡す
```

## 💡 重要なポイント

### 1. ハンドルによる間接参照
- 箱の実体は渡さない（メモリ安全性）
- `(type_id, instance_id)`のペアで識別
- プラグイン側でハンドルから実体にアクセス

### 2. 型安全性
- `type_id`で型を識別可能
- 不正な型の場合はエラー返却

### 3. 所有権管理
- インスタンスIDで参照管理
- プラグイン間でも安全に共有

## 🎯 結論

**C ABIの制約があっても、ハンドル機構により箱は箱を引数に取れる！**

これは既に実装済みの機能であり、プラグイン間での高度な連携が可能です。

### 埋め込みVMへの示唆

既存のTLVハンドル機構をそのまま使えば、埋め込みVMでも同じように箱引数をサポートできます：

1. Nyashスクリプト内で箱を引数に使用
2. MIRバイトコードにBoxCall命令を含める
3. 埋め込みVMがTLVエンコードでC ABIプラグインを呼び出し
4. ハンドル経由で箱を渡す

**Everything is Box、そしてC ABIでも箱は箱を扱える！**