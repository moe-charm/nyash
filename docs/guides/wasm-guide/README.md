# 🌐 Nyash WASM ガイド

Nyash WebAssembly（WASM）実行に関する包括的ガイド

## 📖 ドキュメント一覧

### 基本ガイド
- **[Rust依存性分析](rust-dependency-analysis.md)** - 実行時Rust依存性の詳細分析
- **[Phase比較](phase-comparison.md)** - 9.77手動実装 vs 9.8+FFI基盤の比較
- **[配布ガイド](deployment-guide.md)** - WASM配布・実行方法

### 技術仕様
- **[FFI/BIDチュートリアル](ffi-bid-tutorial.md)** - 外部API統合方法
- **[メモリ管理](memory-management.md)** - WASM メモリレイアウト・最適化

## 🚀 クイックスタート

### WASM コンパイル
```bash
# 基本コンパイル
./target/release/nyash --compile-wasm program.nyash

# AOT コンパイル（配布用）
./target/release/nyash --aot program.nyash
```

### ブラウザー実行
```html
<!DOCTYPE html>
<html>
<body>
    <script>
        WebAssembly.instantiateStreaming(fetch('program.wasm'), importObject)
            .then(instance => instance.exports.main());
    </script>
</body>
</html>
```

## 🎯 実行方式選択

| 用途 | 方式 | コマンド |
|------|------|----------|
| **開発・テスト** | インタープリター | `nyash program.nyash` |
| **高速実行** | VM | `nyash --backend vm program.nyash` |
| **Web配布** | WASM | `nyash --compile-wasm program.nyash` |
| **ネイティブ配布** | AOT | `nyash --aot program.nyash` |

## 📊 性能比較

| バックエンド | 実行速度 | 配布サイズ | 依存関係 |
|-------------|----------|------------|----------|
| インタープリター | 1x | - | Rust |
| VM | 20.4x | - | Rust |
| **WASM** | **13.5x** | **小** | **なし** |
| AOT | 目標1000x+ | 小 | なし |

## 🔗 関連ドキュメント
- [言語ガイド](../LANGUAGE_GUIDE.md)
- [実行バックエンド](../execution-backends.md)
- [ビルドガイド](../build/README.md)

---
**最終更新**: 2025-08-15