# Nyash ABI 統合設計図 v1.0 (2025-09-01)

## 🎯 概要

Nyash ABIは、既存のC ABIプラグインを維持しながら、より型安全で拡張性の高いプラグインシステムを実現する統一ブリッジ規格です。

### 設計原則
1. **後方互換性**: 既存のC ABIプラグインはそのまま動作
2. **最小侵襲**: MIR層の変更を最小限に
3. **段階的移行**: nyash.tomlで個別に移行可能
4. **ゼロコスト抽象化**: インライン値で不要なボクシング回避

## 📐 基本構造

### NyashValue - 3×u64統一表現

```c
// nyash_abi.h
typedef struct NyashValue {
    uint64_t type_id;     // 型識別子（上位16bit: カテゴリ、下位48bit: ID）
    uint64_t box_handle;  // Arc<dyn NyashBox>のポインタ or インライン値
    uint64_t metadata;    // フラグ・メソッドID・追加データ
} NyashValue;

// メタデータフラグ（上位16bit）
#define NYASH_META_INLINE     0x0001  // box_handleがインライン値
#define NYASH_META_ASYNC      0x0002  // 非同期結果
#define NYASH_META_WEAK       0x0004  // 弱参照
#define NYASH_META_BORROWED   0x0008  // 借用（参照カウント不要）
#define NYASH_META_ERROR      0x0010  // エラー値

// 型カテゴリ（type_idの上位16bit）
#define NYASH_TYPE_PRIMITIVE  0x0001  // i64/f64/bool/null
#define NYASH_TYPE_STRING     0x0002  // 文字列
#define NYASH_TYPE_ARRAY      0x0003  // 配列
#define NYASH_TYPE_MAP        0x0004  // マップ
#define NYASH_TYPE_CUSTOM     0x1000  // ユーザー定義Box
#define NYASH_TYPE_PLUGIN     0x2000  // プラグインBox
```

### NyashFunc - 統一関数シグネチャ

```c
typedef NyashValue (*NyashFunc)(
    uint32_t argc,         // 引数の数
    NyashValue* args,      // 引数配列（args[0]はレシーバー）
    void* context          // ランタイムコンテキスト
);

// プラグインエントリーポイント
typedef struct NyashPlugin {
    const char* name;              // プラグイン名
    const char* version;           // バージョン
    uint32_t    method_count;      // メソッド数
    const char** method_names;     // メソッド名配列
    NyashFunc*  method_funcs;      // メソッド関数配列
    NyashFunc   init;              // 初期化関数
    NyashFunc   drop;              // 破棄関数
} NyashPlugin;

// エクスポート関数
extern "C" const NyashPlugin* nyash_plugin_init(void);
```

## 🔄 既存C ABIとの共存

### トランポリン戦略

```rust
// src/runtime/abi_bridge.rs

// 既存のC ABI関数シグネチャ
type OldCFunc = extern "C" fn(*mut c_void, *const c_void) -> *mut c_void;

// 自動生成トランポリン
fn create_c_abi_trampoline(old_func: OldCFunc) -> NyashFunc {
    Box::into_raw(Box::new(move |argc, args, ctx| {
        // NyashValue → 旧C ABI形式に変換
        let old_args = convert_to_old_format(args);
        let old_result = old_func(old_args.as_ptr(), ctx);
        
        // 旧C ABI形式 → NyashValueに変換
        convert_from_old_format(old_result)
    })) as NyashFunc
}
```

### nyash.toml設定

```toml
# nyash.toml v2.1
[plugin.math]
path = "plugins/math.so"
abi = "c"           # 既存C ABI（デフォルト）

[plugin.advanced_math]  
path = "plugins/advanced_math.so"
abi = "nyash"       # 新Nyash ABI

[plugin.hybrid]
path = "plugins/hybrid.so"
abi = "auto"        # 自動検出（シンボル名で判定）
```

## 💨 インライン最適化

### 基本型のインライン表現

```c
// インライン値のエンコーディング（metadataにINLINEフラグ必須）

// 整数（i64）: box_handleに直接格納
NyashValue inline_i64(int64_t val) {
    return (NyashValue){
        .type_id = NYASH_TYPE_PRIMITIVE | (1 << 16),  // subtype=1 (i64)
        .box_handle = (uint64_t)val,
        .metadata = NYASH_META_INLINE
    };
}

// 浮動小数点（f64）: ビットパターンをbox_handleに
NyashValue inline_f64(double val) {
    union { double d; uint64_t u; } conv = { .d = val };
    return (NyashValue){
        .type_id = NYASH_TYPE_PRIMITIVE | (2 << 16),  // subtype=2 (f64)
        .box_handle = conv.u,
        .metadata = NYASH_META_INLINE
    };
}

// Bool: box_handleの最下位ビット
NyashValue inline_bool(bool val) {
    return (NyashValue){
        .type_id = NYASH_TYPE_PRIMITIVE | (3 << 16),  // subtype=3 (bool)
        .box_handle = val ? 1 : 0,
        .metadata = NYASH_META_INLINE
    };
}
```

## 🏗️ 実装フェーズ

### Phase 1: 基盤整備（1週間）
- [x] nyash_abi.h ヘッダー定義
- [ ] NyashValue ↔ Arc<dyn NyashBox> 変換関数
- [ ] C ABIトランポリン自動生成
- [ ] nyash.toml v2.1パーサー拡張

### Phase 2: ランタイム統合（2週間）
- [ ] 統一レジストリ実装（abi種別管理）
- [ ] VM層でのNyashFunc呼び出し
- [ ] インライン値の高速パス
- [ ] エラーハンドリング統一

### Phase 3: プラグイン移行（3週間）
- [ ] 既存プラグイン1つをNyash ABIに移行
- [ ] パフォーマンスベンチマーク
- [ ] 移行ガイドライン作成
- [ ] デバッグツール整備

### Phase 4: バインディング生成（4週間）
- [ ] Rust: #[nyash_abi]マクロ
- [ ] C++: NYASH_PLUGIN()マクロ
- [ ] Python: @nyash_plugin デコレータ
- [ ] JavaScript: NyashPlugin基底クラス

## 🔍 型レジストリ設計

### 型IDの生成戦略

```rust
// 型ID = カテゴリ(16bit) + ハッシュ(48bit)
fn generate_type_id(category: u16, type_name: &str) -> u64 {
    let hash = xxhash64(type_name.as_bytes());
    ((category as u64) << 48) | (hash & 0xFFFF_FFFF_FFFF)
}

// 既知の型は事前定義
const TYPE_ID_STRING: u64 = 0x0002_0000_0000_0001;
const TYPE_ID_ARRAY:  u64 = 0x0003_0000_0000_0002;
const TYPE_ID_MAP:    u64 = 0x0004_0000_0000_0003;
```

## ⚡ 最適化戦略

### JIT/AOT統合

```rust
// JIT時の特殊化
if method_id == KNOWN_METHOD_ADD && is_inline_i64(arg1) && is_inline_i64(arg2) {
    // 直接的な整数加算にコンパイル
    emit_add_i64(arg1.box_handle, arg2.box_handle);
} else {
    // 通常のNyashFunc呼び出し
    emit_nyash_func_call(func_ptr, args);
}
```

### メモリ管理

```rust
// 参照カウント最適化
if metadata & NYASH_META_BORROWED != 0 {
    // 借用フラグ付き → Arc::clone不要
    use_without_clone(box_handle);
} else {
    // 通常の参照カウント
    Arc::clone(box_handle);
}
```

## 📊 互換性マトリックス

| 機能 | C ABI | Nyash ABI | 自動変換 |
|------|-------|-----------|----------|
| 基本型引数 | ✅ | ✅ | 自動 |
| Box引数 | ポインタ | ハンドル | トランポリン |
| 戻り値 | malloc | NyashValue | トランポリン |
| エラー処理 | NULL | ERRORフラグ | 変換可能 |
| 非同期 | ❌ | ASYNCフラグ | - |
| メソッドID | 文字列 | u32 | ハッシュ |

## 🚀 次のステップ

1. **nyash_abi.hの作成**（crates/nyrt/include/）
2. **最小実装プラグイン作成**（SimpleMathの両ABI版）
3. **ベンチマーク測定**（オーバーヘッド評価）
4. **移行判断**（データに基づく方向性決定）

---

*既存を壊さず、新しい世界を開く - これがNyash ABIの道*