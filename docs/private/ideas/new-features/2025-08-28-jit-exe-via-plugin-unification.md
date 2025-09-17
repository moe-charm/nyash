# JIT→EXE実現: プラグインBox統一化による革新的アプローチ

Status: Pending
Created: 2025-08-28
Priority: High
Related: Phase 10.x (JIT), Plugin System (BID-FFI)

## 💡 核心的洞察

既存のプラグインシステム（C ABI）を活用することで、JIT→EXE変換の道が開ける。

## 現状分析

### JIT実行の仕組み
```rust
// 現在: 2つの異なる呼び出し方法
JIT → HostCall → Rustビルトイン (ArrayBox, StringBox等)
JIT → PluginInvoke → プラグインBox (FileBox, NetBox等)
```

### プラグインシステムの特徴
```c
// すでに完全なC FFI！
extern "C" fn nyash_plugin_invoke(
    type_id: u32,
    method_id: u32,
    instance_id: u32,
    args: *const u8,      // TLV encoded
    args_len: usize,
    result: *mut u8,      // TLV encoded
    result_len: *mut usize,
) -> i32
```

## 🚀 提案: すべてをプラグイン化

### 1. ビルトインBoxもプラグインとして実装

```rust
// ArrayBoxをプラグイン化
plugins/nyash-array-plugin/
├── Cargo.toml
├── src/
│   └── lib.rs  // nyash_plugin_invoke_array実装
└── tests/
```

### 2. 統一されたJIT呼び出し

```rust
// LowerCore内で統一
match box_type {
    "ArrayBox" | "StringBox" | "FileBox" => {
        // すべて同じプラグインAPIを使用
        b.emit_plugin_invoke(type_id, method_id, ...);
    }
}
```

### 3. スタティックリンクによるEXE生成

```bash
# プラグインを静的ライブラリとしてビルド
cargo build --release --crate-type=staticlib

# すべてをリンク
ld -o nyash.exe \
   compiled.o \                    # JIT生成コード
   libnyash_array_plugin.a \       # ArrayBox
   libnyash_string_plugin.a \      # StringBox  
   libnyash_file_plugin.a \        # FileBox
   libnyash_runtime_minimal.a      # 最小ランタイム
```

## 🎯 実装ステップ

### Phase 1: プロトタイプ（1週間）
- [ ] ArrayBoxのプラグイン版実装
- [ ] JITからプラグイン呼び出しテスト
- [ ] 性能比較（HostCall vs Plugin）

### Phase 2: 段階的移行（1ヶ月）
- [ ] 主要ビルトインBoxのプラグイン化
  - StringBox
  - IntegerBox
  - BoolBox
  - MapBox
- [ ] JIT lowering層の統一

### Phase 3: EXE生成（2ヶ月）
- [ ] プラグインの静的リンク対応
- [ ] 最小ランタイム作成
- [ ] リンカースクリプト整備

### Phase 4: 最適化（継続的）
- [ ] インライン展開
- [ ] LTO（Link Time Optimization）
- [ ] プロファイルガイド最適化

## 📊 利点

1. **既存資産の活用**
   - プラグインシステムは実績あり
   - C ABIは安定している
   - TLVエンコーディング確立済み

2. **段階的移行可能**
   - ビルトインを一つずつ移行
   - 既存コードとの共存
   - リスク分散

3. **統一されたアーキテクチャ**
   - Everything is Box → Everything is Plugin
   - JIT/AOT/インタープリターで同じAPI
   - メンテナンス性向上

4. **自然なEXE生成**
   - プラグインは元々ネイティブコード
   - リンク処理が簡単
   - デバッグ情報も保持

## 🚨 検討事項

### パフォーマンス
- TLVエンコード/デコードのオーバーヘッド
- → 頻繁に呼ばれるメソッドは最適化版を用意

### メモリ管理
- ハンドル（instance_id）による間接参照
- → JIT側でキャッシュ機構を実装

### 互換性
- 既存のHostCall方式との共存期間
- → フラグで切り替え可能に

## 💡 将来展望

### 自己ホスティング
```nyash
// NyashでNyashコンパイラを書く
box NyashCompiler {
    compile(source) {
        local ast = Parser.parse(source)
        local mir = MirBuilder.build(ast)
        local obj = CraneliftBackend.emit(mir)
        return Linker.link(obj, plugins)
    }
}
```

### プラグインエコシステム
- ユーザーが作ったBoxもEXEに含められる
- プラグインマーケットプレイス
- 動的/静的リンクの選択

## 結論

プラグインシステムの**C ABI統一**により、JIT→EXE変換が現実的に。
「Everything is Plugin」という新たな設計哲学で、Nyashの未来が開ける。

## 参考リンク
- src/bid/plugin_api.rs - プラグインFFI定義
- plugins/ - 既存プラグイン実装
- docs/reference/plugin-system/ - プラグイン仕様書