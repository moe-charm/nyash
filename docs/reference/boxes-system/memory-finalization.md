# 🧠 Nyash メモリ管理 & finiシステム

**最終更新: 2025年8月19日 - 統合仕様書**

## 📋 概要

Nyashは「Everything is Box」哲学のもと、統一的なメモリ管理と予測可能なリソース解放を実現しています。

## 🏗️ 基本アーキテクチャ

### Arc<Mutex>一元管理

```rust
// インタープリターレベルでの統一管理
type NyashObject = Arc<Mutex<dyn NyashBox>>;
```

すべてのBoxは、インタープリターレベルで`Arc<Mutex>`によって管理されます。

#### 利点
- **スレッドセーフティ**: 自動的に保証
- **参照カウント**: 自動的なメモリ解放
- **統一的アクセス**: すべて同じ方法で操作

#### 設計原則

```rust
// ✅ 正しい設計（シンプルなフィールド）
pub struct GoodBox {
    data: String,
    count: i32,
}

// ❌ アンチパターン（Box内部での二重ロック）
pub struct BadBox {
    data: Arc<Mutex<String>>,  // 内部でロック管理しない
}
```

### プラグインシステムのメモリ管理

BID-FFIプラグインシステムでは、**HostVtable**を通じて安全なメモリ管理を実現：

```rust
pub struct NyashHostVtable {
    pub alloc: unsafe extern "C" fn(size: usize) -> *mut u8,
    pub free: unsafe extern "C" fn(ptr: *mut u8),
    pub wake: unsafe extern "C" fn(handle: u64),
    pub log: unsafe extern "C" fn(level: i32, msg: *const c_char),
}
```

**重要**: プラグインが割り当てたメモリはプラグインが解放する原則

## 🔥 finiシステム - 論理的解放フック

### 核心コンセプト

`fini()`は**物理的メモリ破棄ではなく論理的使用終了**を宣言する革新的システムです。

```nyash
box MyResource {
    init { name, file }
    
    fini() {
        print("Resource " + me.name + " is being finalized")
        // ファイルクローズなどのクリーンアップ処理
        // 物理的メモリは共有参照が残っていても論理的には「終了」
    }
}
```

**重要**: `fini()`は「このオブジェクトをもう使わない」という宣言であり、物理的な即時破棄ではありません。

### 実行順序（確定仕様）

#### 自動カスケード解放
```nyash
box Pipeline {
    init { r1, r2, r3, weak monitor }
    
    fini() {
        // 1) ユーザー定義処理（柔軟な順序制御可能）
        me.r3.fini()  // 依存関係でr3→r2の順
        me.r2.fini()
        
        // 2) 自動カスケード: 残りのr1がinit宣言順で自動解放
        // 3) weakフィールドは対象外（lazy nil化）
    }
}
```

#### 決定的な解放順序
1. **finalized チェック** - 既に解放済みなら何もしない（idempotent）
2. **再入防止** - `in_finalization`フラグで再帰呼び出し防止
3. **ユーザー定義fini()実行** - カスタムクリーンアップ処理
4. **自動カスケード** - `init`宣言順で未処理フィールドを解放
5. **フィールドクリア** - 全フィールドを無効化
6. **finalized設定** - 以後の使用を禁止

### weak参照による循環参照回避

```nyash
box Node {
    init { id, weak next } // 'next'は弱参照
}

local node1 = new Node("A", null)
local node2 = new Node("B", node1) // node2はnode1への弱参照を持つ
node1.next = node2 // node1はnode2への強参照を持つ
// 循環参照を回避し、安全に解放される
```

#### weak参照の特性
- **所有権なし**: オブジェクトの生存期間に影響しない
- **自動nil化**: 参照先が解放されると自動的に`null`になる
- **fini()対象外**: 弱参照フィールドはfini()カスケードでスキップ

### 不変条件（重要）

- **weak参照**: `weak`フィールドに対して`fini()`を直接呼ぶことはできません
- **finalized後禁止**: `fini()`呼び出し後は、そのオブジェクトの使用はすべて禁止
- **カスケード順序**: `init`宣言順の**逆順**で実行、`weak`フィールドはスキップ

## 🌟 実用例

### リソース管理
```nyash
box FileHandler {
    init { file, buffer }
    
    fini() {
        // オブジェクト削除時に自動呼び出し
        if me.file != null {
            me.file.close()
            console.log("File closed automatically")
        }
        // bufferは自動カスケードで解放
    }
}
```

### プラグインリソース
```nyash
box PluginResource {
    init { plugin_handle }
    
    fini() {
        // プラグイン側のリソース解放を呼び出し
        me.plugin_handle.cleanup()
        console.log("Plugin resource finalized")
    }
}
```

## 🛡️ メモリ安全性保証

### valgrind検証済み
- **セグフォルト回避**: HostVtableの生存期間問題解決済み
- **メモリリーク検出**: プラグインシステムでのメモリ管理検証済み
- **二重解放防止**: idempotentなfini()実装

### プラグインメモリ安全性
- **プラグイン分離**: プラグインメモリはプラグインが管理
- **境界明確化**: HostVtableを通じた安全なインターフェース
- **ライフサイクル管理**: birth/finiによる明確な生存期間

## 🚀 Phase 9.75g-0での進化

- **BID-FFIプラグインシステム**: 外部ライブラリの安全な統合
- **型情報管理**: nyash.tomlによる型安全な変換
- **HostVtable**: プラグイン↔ホスト間の安全なメモリ管理
- **plugin-tester**: メモリ安全性診断ツール

---

**関連ドキュメント**: 
- [Box型リファレンス](box-reference.md)
- [プラグインシステム](../plugin-system/)
- [BID-FFI仕様](../plugin-system/ffi-abi-specification.md)