# C ABIとの整合性：Phase 12スクリプトプラグインシステム

## 🚨 重要な発見

Phase 10.1で既に**C ABI v0**が定義されており、これとPhase 12の提案を整合させる必要があります。

## 📊 現状のC ABI（Phase 10.1）

### 既存のBID-FFI（プラグイン用）
```c
// 現在のプラグインFFI（TLVベース）
extern "C" fn nyash_plugin_invoke(
    type_id: u32,
    method_id: u32,
    instance_id: u32,
    args: *const u8,      // TLVエンコード
    args_len: usize,
    result: *mut u8,      // TLVエンコード
    result_len: *mut usize,
) -> i32
```

### 新しいNyRT C ABI v0
```c
// コア関数
int32_t  nyrt_abi_version(void);
NyBox    nyrt_box_new(uint64_t typeid, uint64_t size);
void     nyrt_box_free(NyBox b);

// プラグイン関数（例：Array）
int32_t  nyplug_array_abi_version(void);
NyBox    nyplug_array_new(void);
int32_t  nyplug_array_get(NyBox arr, uint64_t i, NyBox* out);
```

## 🎯 Phase 12の修正案

### 問題点
- Gemini/Codexの提案した`BoxInterface`トレイトは**Rust専用**
- C ABIとの相互運用性が考慮されていない
- TLVエンコーディングとの整合性が不明

### 解決策：C ABIラッパー戦略

```rust
// ❌ 元の提案（Rust専用）
trait BoxInterface {
    fn invoke(&self, method_id: u32, args: NyashValue) -> NyashValue;
}

// ✅ 修正案（C ABI互換）
pub struct ScriptPluginWrapper {
    // Nyashスクリプトインスタンス
    script_box: NyashValue,
    
    // C ABI互換性のためのFFI関数
    ffi_invoke: extern "C" fn(
        type_id: u32,
        method_id: u32,
        instance_id: u32,
        args: *const u8,
        args_len: usize,
        result: *mut u8,
        result_len: *mut usize,
    ) -> i32,
}

impl ScriptPluginWrapper {
    // 既存のBID-FFIと完全互換
    pub extern "C" fn invoke_ffi(
        &self,
        type_id: u32,
        method_id: u32,
        instance_id: u32,
        args: *const u8,
        args_len: usize,
        result: *mut u8,
        result_len: *mut usize,
    ) -> i32 {
        // 1. TLVデコード
        let nyash_args = decode_tlv(args, args_len);
        
        // 2. Nyashスクリプト呼び出し
        let result_value = self.script_box.invoke(method_id, nyash_args);
        
        // 3. TLVエンコード
        encode_tlv(result_value, result, result_len)
    }
}
```

## 🔄 統合アーキテクチャ

```
[JIT/AOT] ---> C ABI (nyrt_*/nyplug_*) --+--> [ネイティブプラグイン]
                                          |
                                          +--> [ScriptPluginWrapper] --> [Nyashスクリプト]
```

### 利点
1. **完全な後方互換性** - 既存のプラグインがそのまま動作
2. **統一されたFFI** - JIT/AOT/プラグインすべて同じC ABI
3. **透過的な利用** - 呼び出し側はネイティブ/スクリプトを区別しない

## 📝 実装修正案

### Phase 12.1（修正版）
1. **ScriptPluginWrapperの実装**
   - BID-FFI互換のC関数エクスポート
   - TLVエンコード/デコード処理
   - Nyashスクリプトへの橋渡し

2. **プラグインレジストリ拡張**
   ```rust
   pub struct PluginRegistry {
       // 既存のネイティブプラグイン（C ABI）
       native_plugins: HashMap<u32, PluginHandle>,
       
       // スクリプトプラグイン（C ABIラッパー経由）
       script_plugins: HashMap<u32, ScriptPluginWrapper>,
   }
   ```

3. **export box構文の実装**
   ```nyash
   export box CustomMathPlugin {
       // BID-FFI互換のためのメタ情報
       __type_id__ = 100  // 動的割り当てor設定ファイル
       __methods__ = {
           "cached_sin": 1,
           "cached_cos": 2
       }
       
       // 通常のNyashコード
       init { ... }
       cached_sin(x) { ... }
   }
   ```

## 🚀 移行パス

### 段階1：既存プラグインの動作確認
- FileBox、NetBox等がC ABI経由で正常動作
- パフォーマンステスト

### 段階2：簡単なスクリプトプラグイン
- MathBoxの一部機能をNyashで再実装
- C ABIラッパー経由での動作確認

### 段階3：高度な統合
- ネイティブとスクリプトの混在
- 動的ロード/アンロード

## ⚡ パフォーマンス影響

```
呼び出しチェーン:
1. JIT → C ABI関数呼び出し（既存）
2. C ABI → ScriptPluginWrapper（追加）
3. Wrapper → TLVデコード（追加）
4. Wrapper → Nyashスクリプト実行（追加）
5. Wrapper → TLVエンコード（追加）

予想オーバーヘッド: 100-500ns/呼び出し
```

## 🎯 結論

Phase 12のスクリプトプラグインシステムは、**C ABIを尊重**しつつ実装可能です。

- BoxInterfaceトレイトは内部実装詳細に留める
- 外部インターフェースは既存のC ABI（BID-FFI）を維持
- ScriptPluginWrapperがブリッジとして機能

これにより、**「Everything is Plugin」**の哲学を保ちながら、スクリプトプラグインを実現できます。