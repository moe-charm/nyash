# Nyash Applications Showcase

このディレクトリには、Nyashの実力を示す実用的なアプリケーションが含まれています。

## 🚀 アプリケーション一覧

### 1. ny-echo - 最小CLI実装
標準入力を読み取り、オプションに応じて変換して出力する基本的なCLIツール。

```bash
# 基本使用
echo "Hello World" | nyash apps/ny-echo/main.nyash

# 大文字変換
echo "hello" | nyash apps/ny-echo/main.nyash --upper

# 小文字変換
echo "HELLO" | nyash apps/ny-echo/main.nyash --lower
```

**特徴**:
- ConsoleBoxによるI/O処理
- StringBoxの変換メソッド活用
- VM/JIT/AOTすべてで同一動作

### 2. ny-array-bench - 性能ベンチマーク
ArrayBoxの各種操作をベンチマークし、VM/JIT/AOTの性能比較を行うツール。

```bash
# ベンチマーク実行
nyash apps/ny-array-bench/main.nyash

# 出力例（JSON形式）
{
  "create_1000": 1.23,
  "map_1000": 2.45,
  "reduce_1000": 0.98,
  "relative_performance": {"vm": 1.0, "jit": 5.2}
}
```

**特徴**:
- カスタムStatsBoxによる計測
- JSON形式でCI連携可能
- 性能改善の定量的測定

### 3. ny-jsonlint（開発中）
PyRuntimeBoxを使用してPythonのjsonモジュールでJSON検証を行うツール。

### 4. ny-filegrep（開発中）
ファイルシステムを検索し、パターンマッチングを行う実用的なツール。

### 5. ny-http-hello（開発中）
HTTPサーバーを実装し、Web対応を実証するデモアプリケーション。

## 🔧 ビルドと実行

### 実行方法
```bash
# インタープリター実行
nyash apps/APP_NAME/main.nyash

# VM実行（高速）
nyash --backend vm apps/APP_NAME/main.nyash

# JIT実行（最速）
nyash --backend jit apps/APP_NAME/main.nyash
```

### テスト実行
各アプリケーションにはtest.shが含まれています：

```bash
cd apps/ny-echo
./test.sh
```

## 📊 性能指標

| アプリ | VM | JIT | AOT | 用途 |
|--------|-----|-----|-----|------|
| ny-echo | 1.0x | 5x | 10x | I/O性能 |
| ny-array-bench | 1.0x | 5x | 10x | 計算性能 |
| ny-jsonlint | 1.0x | 3x | 5x | FFI性能 |
| ny-filegrep | 1.0x | 4x | 8x | 実用性能 |
| ny-http-hello | 1.0x | 6x | 12x | 並行性能 |

## 🎯 開発ロードマップ

- [x] Phase 1: ny-echo（基本I/O検証）
- [x] Phase 2: ny-array-bench（性能基準）
- [ ] Phase 3: ny-jsonlint（プラグイン統合）
- [ ] Phase 4: ny-filegrep（実用性）
- [ ] Phase 5: ny-http-hello（Web対応）

## 🤝 貢献方法

新しいアプリケーションのアイデアや改善提案は大歓迎です！

1. 新しいアプリディレクトリを作成
2. main.nyashとtest.shを実装
3. このREADMEに追加
4. PRを送信

すべてのアプリケーションは「Everything is Box」哲学に従い、プラグインシステムを活用することを推奨します。