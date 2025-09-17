# Nyash ABIをC実装TypeBoxで提供する設計仕様書

## 🎯 概要

「Everything is Box」哲学の究極形：**ABIすらBoxとして扱う**

本設計は、Nyash ABIをTypeBox（C ABI）として実装することで、Rust依存を排除し、完全なセルフホスティングを実現する革新的アプローチです。

**3大AI専門家の評価：**
- **Gemini**: 「技術的妥当性が高く、言語哲学とも合致した、極めて優れた設計」
- **Codex**: 「Feasible and attractive: ABI-as-Box completes the idea」
- **ChatGPT5**: 「実装に耐える設計。10の改善点で完璧」（統合ABI設計に反映済み）

## 📐 技術設計

### 基本アーキテクチャ

```c
// nyash_abi_provider.h - Codex推奨の16バイトアライメント版
typedef struct __attribute__((aligned(16))) {
    // TypeBox標準ヘッダ（バージョニング対応）
    uint32_t abi_tag;           // 'NABI' (0x4942414E)
    uint16_t api_version;       // APIバージョン
    uint16_t struct_size;       // 構造体サイズ（互換性チェック用）
    const char* name;           // "NyashABIProvider"
    void* (*create)(void);      // プロバイダインスタンス生成
    
    // Nyash ABI専用拡張（コンテキストベース）
    struct {
        // セレクターキャッシング対応
        nyash_status (*lookup_selector)(nyash_ctx*, uint64_t type_id, const char* name, nyash_selector* out);
        nyash_status (*call)(nyash_ctx*, void* obj, nyash_selector sel, nyash_value* argv, int argc, nyash_value* out);
        
        // 基本操作
        nyash_status (*create_value)(nyash_ctx*, uint64_t type_id, void* data, uint64_t flags, nyash_value* out);
        void (*retain)(void* obj);
        void (*release)(void* obj);
        
        // 型情報
        uint64_t (*type_of)(nyash_value value);
        bool (*is_null)(nyash_value value);
        bool (*eq)(nyash_value a, nyash_value b);
        uint64_t (*hash)(nyash_value value);
    } nyash_ops;
    
    // 機能ビット（将来拡張用）
    uint64_t capabilities;      // NYASH_CAP_ASYNC | NYASH_CAP_WEAKREF等
    void* reserved[4];          // 将来拡張用
} NyashABITypeBox;
```

### NyashValue - 16バイト最適化構造（Codex提案）

```c
// JIT/LLVM最適化を考慮した16バイトアライメント
typedef struct __attribute__((aligned(16))) {
    uint64_t type_id;     // 型識別子（上位16bit: vendor, 8bit: kind）
    uint64_t payload;     // ポインタまたはインライン値
    uint64_t metadata;    // フラグ・追加情報（下位3bit: タグ）
} nyash_value;

// メタデータタグ（Gemini提案のTagged Pointers）
#define NYASH_TAG_POINTER    0x0  // ヒープオブジェクトへのポインタ
#define NYASH_TAG_SMALL_INT  0x1  // 61ビット整数（符号付き）
#define NYASH_TAG_BOOL       0x2  // 真偽値
#define NYASH_TAG_NULL       0x3  // null
#define NYASH_TAG_SMALL_ENUM 0x4  // 小さな列挙型
```

### メモリ管理 - Intrusive参照カウント（Gemini詳細提案）

```c
// すべてのヒープBoxの共通ヘッダ
typedef struct {
    atomic_size_t ref_count;        // アトミック参照カウント
    const nyash_obj_vtable* vtable; // メソッドテーブル
    uint64_t type_id;               // 型識別子
    uint64_t flags;                 // THREAD_AFFINE等のフラグ
    
    // 弱参照サポート（循環参照対策）
    struct weak_ref_list* weak_refs; // 弱参照リスト（オプション）
} nyash_obj_header;

// Gemini推奨：弱参照による循環参照回避
typedef struct {
    nyash_obj_header* target;  // 対象オブジェクト（NULL可能）
    atomic_bool is_valid;      // 有効性フラグ
} nyash_weak_ref;
```

### コンテキストベースAPI（Codex推奨）

```c
// グローバル状態を避けるコンテキスト構造
typedef struct {
    nyash_allocator* allocator;     // カスタムアロケータ
    nyash_scheduler* scheduler;     // スケジューラ（async用）
    void* tls;                      // スレッドローカルストレージ
    uint64_t feature_flags;         // 機能フラグ
    nyash_error* last_error;        // 最後のエラー
} nyash_ctx;

// ステータスコード（例外を使わない）
typedef enum {
    NYASH_OK = 0,
    NYASH_E_TYPE,      // 型エラー
    NYASH_E_NULL,      // NULL参照
    NYASH_E_BOUNDS,    // 境界外アクセス
    NYASH_E_MEMORY,    // メモリ不足
    NYASH_E_NOT_FOUND, // メソッド/プロパティ未定義
} nyash_status;
```

## 🚀 実装戦略（3段階）

### Phase 1: C Shim実装（1-2週間）
既存Rust実装へのCラッパーとして開始：

```c
// plugins/nyash_abi_c/src/shim.c
nyash_status nyash_call_shim(nyash_ctx* ctx, void* obj, 
                             nyash_selector sel, nyash_value* argv, 
                             int argc, nyash_value* out) {
    // Rust FFI経由で既存実装を呼び出し
    extern nyash_status rust_nyash_call(/* ... */);
    return rust_nyash_call(ctx, obj, sel, argv, argc, out);
}
```

### Phase 2: フルC実装（1ヶ月）
- 基本型（Integer/String/Bool/Array/Map）の完全実装
- セレクターキャッシング機構
- アトミック参照カウント + 弱参照
- 型情報レジストリ

### Phase 3: Nyash再実装（2ヶ月）
- C実装をNyashで書き直し
- AOTコンパイルで同じC ABIを公開
- セルフホスティング達成！

## 🔧 パフォーマンス最適化

### セレクターキャッシング（Codex提案）
```c
// 文字列ルックアップは初回のみ
typedef struct {
    uint32_t vtable_slot;  // 解決済みvtableスロット
    uint64_t cache_cookie; // インラインキャッシュ用
} nyash_selector;

// JITは直接vtable_slotを使用可能
```

### Tagged Pointers（Gemini提案）
- 64bitポインタの下位3bitをタグとして活用
- 小さな整数（61bit）、真偽値、nullをヒープ確保なしで表現
- `metadata & 0x7`でタグ判定、分岐なしアンボックス可能

## 📊 テスト戦略

### 適合性テストスイート
```bash
tests/abi/
├── conformance/     # ABI適合性テスト
├── memory/         # 参照カウント・弱参照テスト
├── threading/      # マルチスレッド安全性
├── performance/    # ベンチマーク（1.5x以内目標）
└── compatibility/  # Rust/C実装の差分テスト
```

### ファジング（Codex提案）
- `nyash_value`エンコード/デコード
- `type_id`正規化
- 循環参照パターン

## 🌟 技術的革新性

### 1. ABI as a Box
「Everything is Box」哲学の完成形。ABIそのものがBoxとして扱われることで：
- 実行時のABI切り替えが可能
- デバッグ用ABI、プロファイリング用ABIの動的ロード
- 異なるメモリ管理モデル（GC等）の実験が容易

### 2. セルフホスティングへの明確な道筋
1. **Rust製コンパイラ** → **C ABI上のNyashコンパイラ**をビルド
2. **Nyashコンパイラ** → **自分自身を再コンパイル**
3. Rust依存完全排除！

### 3. 長期的保守性
- C ABIは最も安定した普遍的インターフェース
- プラットフォーム非依存
- 30年後も動作する設計

## 📝 配布と使用

### nyash.toml設定
```toml
[plugins.nyash_abi_provider]
path = "plugins/nyash_abi_provider.so"
abi = "c"
types = ["NyashABIProvider"]
capabilities = ["async", "weakref", "selector_cache"]
```

### Nyashコードでの使用
```nyash
// ABIプロバイダーの取得
local abiType = getTypeBox("NyashABIProvider")
local abi = abiType.create()

// セレクターキャッシングを活用した高速呼び出し
local strType = abi.lookupType("StringBox")
local lengthSel = abi.lookupSelector(strType, "length")

local str = abi.createValue(strType, "Hello World")
local len = abi.call(str, lengthSel, [])
print(len)  // 11
```

## 🎯 まとめ

本設計により、Nyashは：
- **Rust非依存**でセルフホスティング可能に
- **「Everything is Box」哲学**を完全実現
- **高性能**（Tagged Pointers、セレクターキャッシング）
- **安全**（アトミック参照カウント、弱参照）
- **拡張可能**（バージョニング、機能ビット）

これは単なる実装の変更ではなく、言語の哲学を技術で体現する、美しいアーキテクチャの完成です。