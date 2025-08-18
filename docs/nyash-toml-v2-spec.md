# nyash.toml v2 仕様 - 究極のシンプル設計

## 🎯 概要
**革命的シンプル設計**: nyash.toml中心アーキテクチャ + 最小限FFI

## 📝 nyash.toml v2形式

### マルチBox型プラグイン対応
```toml
[libraries]
# ライブラリ定義（1つのプラグインで複数のBox型を提供可能）
"libnyash_filebox_plugin.so" = {
    boxes = ["FileBox"],
    path = "./target/release/libnyash_filebox_plugin.so"
}

# 将来の拡張例: 1つのプラグインで複数Box型
"libnyash_network_plugin.so" = {
    boxes = ["SocketBox", "HTTPServerBox", "HTTPClientBox"],
    path = "./target/release/libnyash_network_plugin.so"
}

# FileBoxの型情報定義
[libraries."libnyash_filebox_plugin.so".FileBox]
type_id = 6
abi_version = 1  # ABIバージョンもここに！

[libraries."libnyash_filebox_plugin.so".FileBox.methods]
# method_id だけで十分（引数情報は実行時チェック）
birth = { method_id = 0 }
open = { method_id = 1 }
read = { method_id = 2 }
write = { method_id = 3 }
close = { method_id = 4 }
fini = { method_id = 4294967295 }  # 0xFFFFFFFF
```

## 🚀 究極のシンプルFFI

### プラグインが実装する関数

#### 必須: メソッド実行エントリーポイント
```c
// 唯一の必須関数 - すべてのメソッド呼び出しはここから
extern "C" fn nyash_plugin_invoke(
    type_id: u32,      // Box型ID（例: FileBox = 6）
    method_id: u32,    // メソッドID（0=birth, 0xFFFFFFFF=fini）
    instance_id: u32,  // インスタンスID（0=static/birth）
    args: *const u8,   // TLVエンコード引数
    args_len: usize,   
    result: *mut u8,   // TLVエンコード結果バッファ
    result_len: *mut usize  // [IN/OUT]バッファサイズ
) -> i32              // 0=成功, 負=エラー
```

#### オプション: グローバル初期化
```c
// プラグインロード時に1回だけ呼ばれる（実装は任意）
extern "C" fn nyash_plugin_init() -> i32 {
    // グローバルリソースの初期化
    // 設定ファイルの読み込み
    // ログファイルのオープン
    // 0=成功, 負=エラー（プラグインは無効化される）
}
```

### 廃止されたAPI
```c
// ❌ これらは全部不要！
nyash_plugin_abi_version()     // → nyash.tomlのabi_version
nyash_plugin_get_box_count()   // → nyash.tomlのboxes配列
nyash_plugin_get_box_info()    // → nyash.tomlから取得
NyashHostVtable               // → 完全廃止！
```

## 📊 設計原則

### 1. **Single Source of Truth**
- すべてのメタ情報はnyash.tomlに集約
- プラグインは純粋な実装のみ

### 2. **Zero Dependencies**
- Host VTable廃止 = 依存関係ゼロ
- プラグインは完全に独立

### 3. **シンプルなライフサイクル**
- `init` (オプション): プラグインロード時の初期化
- `birth` (method_id=0): インスタンス作成
- 各種メソッド: インスタンス操作
- `fini` (method_id=0xFFFFFFFF): 論理的終了

### 4. **ログ出力**
```rust
// プラグインは自己完結でログ出力
eprintln!("[FileBox] Opened: {}", path);  // 標準エラー

// または専用ログファイル
let mut log = File::create("plugin_debug.log")?;
writeln!(log, "{}: FileBox birth", chrono::Local::now())?;
```

### 5. **init関数の活用例**
```rust
static mut LOG_FILE: Option<File> = None;

#[no_mangle]
pub extern "C" fn nyash_plugin_init() -> i32 {
    // ログファイルを事前に開く
    match File::create("filebox.log") {
        Ok(f) => {
            unsafe { LOG_FILE = Some(f); }
            0  // 成功
        }
        Err(_) => -1  // エラー → プラグイン無効化
    }
}
```

## 🔧 実装の流れ

### Phase 1: nyash.toml v2パーサー
1. 新形式の読み込み
2. Box型情報の抽出
3. メソッドID管理

### Phase 2: プラグインローダー簡素化  
1. `nyash_plugin_init`（オプション）と`nyash_plugin_invoke`（必須）をロード
2. nyash.tomlベースの型登録
3. Host VTable関連コードを削除
4. init関数が存在し失敗した場合はプラグインを無効化

### Phase 3: プラグイン側の対応
1. abi/get_box_count/get_box_info関数を削除
2. init関数は必要に応じて実装（グローバル初期化）
3. invoke関数でメソッド処理
4. ログ出力を自己完結に

## 🎉 メリット

1. **究極のシンプルさ** - 基本的にFFI関数1つ（initはオプション）
2. **保守性向上** - 複雑な相互依存なし
3. **テスト容易性** - モック不要
4. **移植性** - どの言語でも実装可能
5. **拡張性** - nyash.toml編集で機能追加
6. **初期化保証** - init関数で早期エラー検出可能

## 🚨 注意事項

- プラグインのログは標準エラー出力かファイル出力で
- メモリ管理はプラグイン内で完結
- 非同期処理はNyash側でFutureBoxラップ

---

**革命完了**: これ以上シンプルにできない究極の設計！