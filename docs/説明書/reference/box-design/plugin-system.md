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

### 🔄 Boxライフサイクル管理

```yaml
lifecycle:
  # コンストラクタ（生命を与える）
  - sig: "FileBox#birth(path: string, mode?: string)"
    doc: "Box creation - called after memory allocation"
    
  # デストラクタ（生命を終える）  
  - sig: "FileBox#fini()"
    doc: "Box destruction - called before memory deallocation"
```

**重要な原則**：
- `birth()` - Boxインスタンス作成時に呼ばれる（メモリ割り当て後）
- `fini()` - Boxインスタンス破棄時に呼ばれる（メモリ解放前）
- プラグインが割り当てたメモリはプラグインが解放する責任を持つ

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

```c
// plugins/filebox/src/filebox.c
#include "nyash_plugin_api.h"

// インスタンス管理
typedef struct {
    FILE* fp;
    char* buffer;  // プラグインが管理するバッファ
} FileBoxInstance;

// birth - Boxに生命を与える
i32 filebox_birth(u32 instance_id, const u8* args, size_t args_len) {
    // 引数からpath, modeを取得
    const char* path = extract_string_arg(args, 0);
    const char* mode = extract_string_arg(args, 1);
    
    // インスタンス作成
    FileBoxInstance* instance = malloc(sizeof(FileBoxInstance));
    instance->fp = fopen(path, mode);
    instance->buffer = NULL;
    
    // インスタンスを登録
    register_instance(instance_id, instance);
    return NYB_SUCCESS;
}

// fini - Boxの生命を終える
i32 filebox_fini(u32 instance_id) {
    FileBoxInstance* instance = get_instance(instance_id);
    if (!instance) return NYB_E_INVALID_HANDLE;
    
    // プラグインが割り当てたメモリを解放
    if (instance->buffer) {
        free(instance->buffer);
    }
    
    // ファイルハンドルをクローズ
    if (instance->fp) {
        fclose(instance->fp);
    }
    
    // インスタンス自体を解放
    free(instance);
    unregister_instance(instance_id);
    
    return NYB_SUCCESS;
}

// read - バッファはプラグインが管理
i32 filebox_read(u32 instance_id, i32 size, u8** result, size_t* result_len) {
    FileBoxInstance* instance = get_instance(instance_id);
    
    // 既存バッファを解放して新規割り当て
    if (instance->buffer) free(instance->buffer);
    instance->buffer = malloc(size + 1);
    
    // ファイル読み込み
    size_t read = fread(instance->buffer, 1, size, instance->fp);
    instance->buffer[read] = '\0';
    
    // プラグインが所有するメモリを返す
    *result = instance->buffer;
    *result_len = read;
    
    return NYB_SUCCESS;
}
```

## 🔐 メモリ管理の原則

### 所有権ルール
1. **プラグインが割り当てたメモリ**
   - プラグインが`malloc()`したメモリはプラグインが`free()`する
   - `fini()`メソッドで確実に解放する
   - Nyash側は読み取りのみ（書き込み禁止）

2. **Nyashが割り当てたメモリ**
   - Nyashが提供したバッファはNyashが管理
   - プラグインは読み書き可能だが解放禁止
   - 引数として渡されたメモリはread-only

3. **ライフサイクル保証**
   - `birth()` → 各メソッド呼び出し → `fini()` の順序を保証
   - `fini()`は必ず呼ばれる（GC時またはプログラム終了時）
   - 循環参照による`fini()`遅延に注意

### Nyash側の実装
```rust
impl Drop for PluginBox {
    fn drop(&mut self) {
        // Boxが破棄される時、必ずfiniを呼ぶ
        let result = self.plugin.invoke(
            self.handle.type_id,
            FINI_METHOD_ID,  // 最大値のmethod_id
            self.handle.instance_id,
            &[],  // no arguments
            &mut []
        );
        
        if result.is_err() {
            eprintln!("Warning: fini failed for instance {}", self.handle.instance_id);
        }
    }
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