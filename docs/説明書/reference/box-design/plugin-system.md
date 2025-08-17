# Nyash Box プラグインシステム設計

## 概要

Nyashの「Everything is Box」哲学を維持しながら、Boxの実装をプラグイン化できるシステム。ビルトインBoxとプラグインBoxを透過的に切り替え可能。

## 🎯 設計原則

1. **シンプル** - 設定ファイル1つで切り替え
2. **透過的** - Nyashコードの変更不要
3. **統一的** - ビルトインもプラグインも同じBox

## 📋 プラグイン定義（YAML署名DSL）

```yaml
# filebox.plugin.yaml
schema: 1
plugin:
  name: filebox
  version: 1
  
apis:
  # 静的メソッド（::）
  - sig: "FileBox::open(path: string, mode?: string) -> FileBox"
    doc: "Open a file with optional mode"
    
  - sig: "FileBox::exists(path: string) -> bool"
    doc: "Check if file exists"
    
  # インスタンスメソッド（#）
  - sig: "FileBox#read(size?: int) -> string"
    doc: "Read file content"
    
  - sig: "FileBox#write(content: string) -> int"
    doc: "Write to file"
    
  - sig: "FileBox#close() -> void"
    doc: "Close file handle"
```

### 署名DSL仕様

- **静的メソッド**: `Type::method()` - C++風の`::`記法
- **インスタンスメソッド**: `Type#method()` - Ruby風の`#`記法
- **オプショナル引数**: `arg?: type` - `?`サフィックス
- **戻り値**: `-> type` - 矢印記法

## 🔧 設定ファイル（nyash.toml）

```toml
# プロジェクトルートのnyash.toml
[plugins]
FileBox = "filebox"      # FileBoxはプラグイン版を使用
# StringBox = "mystring" # コメントアウト = ビルトイン使用
```

## 🏗️ アーキテクチャ

### 1. Boxレジストリ

```rust
// 起動時の動作
let mut registry = HashMap::new();

// 1. ビルトインBoxを登録
registry.insert("FileBox", BoxProvider::Builtin(native_filebox));
registry.insert("StringBox", BoxProvider::Builtin(native_stringbox));

// 2. nyash.toml読み込み
let config = parse_nyash_toml()?;

// 3. プラグイン設定で上書き
for (box_name, plugin_name) in config.plugins {
    registry.insert(box_name, BoxProvider::Plugin(plugin_name));
}
```

### 2. 透過的なディスパッチ

```nyash
# Nyashコード（変更不要！）
local file = new FileBox("test.txt")
file.write("Hello, plugin!")
local content = file.read()
```

内部動作:
1. `new FileBox` → レジストリ検索
2. `BoxProvider::Plugin("filebox")` → プラグインロード
3. BID-FFI経由で実行

### 3. PluginBoxプロキシ

```rust
// すべてのプラグインBoxの統一インターフェース
pub struct PluginBox {
    plugin_name: String,
    handle: BidHandle,  // プラグイン内のインスタンス
}

impl NyashBox for PluginBox {
    // NyashBoxトレイトの全メソッドを
    // FFI経由でプラグインに転送
}
```

## 📦 プラグイン実装例

```rust
// plugins/filebox/src/lib.rs
#[no_mangle]
pub extern "C" fn filebox_open(
    path: *const c_char,
    mode: *const c_char
) -> BidHandle {
    // ファイルを開いてハンドルを返す
}

#[no_mangle]
pub extern "C" fn filebox_read(
    handle: BidHandle,
    size: i32
) -> *const u8 {
    // ファイルを読む
}
```

## 🚀 段階的導入計画

### Phase 1: 基本実装（現在）
- [x] BID-FFI基盤
- [x] FileBoxプラグイン実装
- [ ] nyash.tomlパーサー
- [ ] PluginBoxプロキシ
- [ ] 手動プラグインロード

### Phase 2: 開発体験向上
- [ ] YAMLからFFIコード自動生成
- [ ] エラーメッセージ改善
- [ ] プラグインテンプレート

### Phase 3: エコシステム
- [ ] プラグインレジストリ
- [ ] バージョン管理
- [ ] 依存関係解決

## 🎉 利点

1. **ビルド時間短縮** - 使わないBoxはコンパイル不要
2. **動的拡張** - 再コンパイルなしで新Box追加
3. **Everything is Box維持** - 哲学は変わらない
4. **段階的移行** - 1つずつBoxをプラグイン化

## 📚 関連ドキュメント

- [BID-FFI仕様](./ffi-abi-specification.md)
- [Everything is Box哲学](./everything-is-box.md)
- [実装タスク](../../../予定/native-plan/issues/phase_9_75g_0_chatgpt_enhanced_final.md)