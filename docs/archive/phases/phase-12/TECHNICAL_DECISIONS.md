# Phase 12: 統一TypeBox ABI - 技術的決定事項

## 📅 最終更新: 2025-09-02

このドキュメントは、3人のAI専門家（Gemini、Codex、ChatGPT5）の深い技術検討を経て決定された、統一TypeBox ABIの技術的決定事項をまとめたものです。

## 🎯 核心的決定事項

### 1. 統一TypeBox構造体の確定

```c
typedef struct NyashTypeBox {
    // === ヘッダー情報（不変のバイナリ契約）===
    uint32_t abi_tag;               // 'TYBX' (0x54594258) - 必須
    uint16_t version;               // APIバージョン（現在: 1）
    uint16_t struct_size;           // 構造体サイズ（前方互換性）
    const char* name;               // Box型名："StringBox"
    
    // === 基本操作（COM互換）===
    void* (*create)(void* args);    // インスタンス生成
    void (*destroy)(void* self);    // インスタンス破棄
    void (*retain)(void* self);     // 参照カウント増加（新規）
    void (*release)(void* self);    // 参照カウント減少（新規）
    
    // === 高速メソッドディスパッチ ===
    uint32_t (*resolve)(const char* name);    // メソッド名→ID変換
    NyResult (*invoke_id)(void* self,         // ID指定の高速呼び出し
                         uint32_t method_id, 
                         NyValue* args, 
                         int argc);
    
    // === メタ情報 ===
    const char* (*get_type_info)(void);       // JSON形式の型情報
    uint64_t capabilities;                    // 機能フラグ
    
    // === 将来拡張用 ===
    void* reserved[4];                        // NULL初期化必須
} NyashTypeBox;
```

### 2. Type ID管理戦略

64-bit構造: `[8b domain | 8b vendor | 16b category | 32b serial]`

- **0x00xxxxxx**: Core/Built-in types
- **0x40xxxxxx**: First-party plugins
- **0x80xxxxxx**: User-defined boxes (動的登録)
- **0xC0xxxxxx**: Experimental/Debug

### 3. メモリ管理統一方針

#### 参照カウント（必須）
- アトミック操作（`_Atomic uint64_t`）
- retain/releaseはCOM互換の固定位置
- 循環参照対策：Trial Deletion + Weak Boundary

#### GC協調（オプション）
- traceメソッドの提供（プラグイン必須）
- 世代別GCとの協調動作

### 4. パフォーマンス目標値

- **メソッド呼び出し**: 旧50-150ns → 新1-3ns（最大50倍高速化）
- **インラインキャッシング**: Monomorphic IC標準装備
- **JIT統合**: ウォームアップ後の直接インライン化

### 5. セキュリティレベル定義

1. **trusted**: 同一プロセス内、直接関数呼び出し
2. **sandboxed**: Wasm/別プロセスで実行
3. **restricted**: 限定的な権限のみ

## 🔧 実装上の決定事項

### メソッドディスパッチ戦略

1. **新規プラグイン**: invoke_id優先（高速パス）
2. **既存プラグイン**: method経由でフォールバック
3. **移行期間**: 両方サポート、段階的に新方式へ

### NyValue統一表現

```c
typedef struct __attribute__((aligned(16))) {
    uint64_t type_tag;   // 型識別子
    union {
        int64_t  i64;    // 整数
        double   f64;    // 浮動小数点
        void*    ptr;    // ポインタ（Box/String等）
        uint64_t bits;   // ビットパターン（NaN-box/SMI）
    } payload;
} NyValue;
```

### API互換性保証

- `NyBoxHeader.header_size`による前方互換
- vtableの先頭3メソッドは不変（COM準拠）
- 新機能は`capabilities`フラグで判定

## 📊 移行戦略

### Phase 1: 基礎実装（1週間）
- NyashTypeBox構造体定義
- 基本的なregister_user_box実装
- 互換レイヤー（既存C ABI→TypeBox）

### Phase 2: 最適化（2週間）
- メソッドID解決機構
- インラインキャッシング基礎
- JIT統合準備

### Phase 3: 完全移行（1ヶ月）
- すべてのビルトインBoxをTypeBox化
- パフォーマンスベンチマーク
- 移行ツール完成

## ✅ 合意済み事項

1. **Everything is Box哲学の完全実現**
   - すべてのBox（ビルトイン、プラグイン、ユーザー定義）が統一ABI
   - NyValue経由ですべて扱える

2. **段階的移行の保証**
   - 既存プラグインは動作継続
   - 新規開発は新ABIを推奨

3. **パフォーマンス優先**
   - メソッドIDによる高速化は必須
   - JIT/AOTとの密接な統合

## 🚫 却下された提案

1. **完全な動的型システム**: 静的解析を困難にするため
2. **プロセス間通信ベース**: オーバーヘッドが大きすぎる
3. **Rustネイティブ統合**: C ABIの安定性を優先

## 📚 参考資料

- [AI先生たちの技術的検討](./ai-consultation-unified-typebox.md)
- [統一TypeBox ABI仕様](./unified-typebox-abi.md)
- [ユーザー定義Box統合](./unified-typebox-user-box.md)
- [Nyash ABI Minimal Coreと進化戦略](../../../../reference/abi/NYASH_ABI_MIN_CORE.md)
