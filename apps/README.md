# Nyash Applications Showcase

このディレクトリには、Nyashの実力を示す実用的なアプリケーションが含まれています。

## 🚀 実装済みアプリケーション

### 🎮 ゲーム・エミュレータ

#### CHIP-8エミュレータ
**場所**: `chip8_nyash/chip8_emulator.nyash`  
**特徴**: 完全なゲーム機エミュレータ、グラフィック表示対応
```bash
./target/release/nyash apps/chip8_nyash/chip8_emulator.nyash
```

### 📝 エディタ・開発ツール

#### Enhanced Kilo Editor
**場所**: `kilo_nyash/enhanced_kilo_editor.nyash`  
**特徴**: テキストエディタ（kilo改良版）、実用的なファイル編集機能
```bash
./target/release/nyash apps/kilo_nyash/enhanced_kilo_editor.nyash
```

### 🌐 ネットワークアプリ

#### TinyProxy
**場所**: `tinyproxy_nyash/proxy_server.nyash`  
**特徴**: HTTPプロキシサーバー、Netプラグイン活用
```bash
./target/release/nyash apps/tinyproxy_nyash/proxy_server.nyash
```

### 🛠️ ユーティリティ・ベンチマーク

#### ny-echo - 最小CLI実装
**場所**: `ny-echo/main.nyash`
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

## 🎯 予定アプリケーション（論文・ベンチマーク用）

### 📊 CLBG標準ベンチマーク
AI先生たちの推奨により、論文説得力向上のため以下を実装予定：

#### 1. binary-trees - メモリ・GC性能測定
**目的**: GC性能、メモリ割り当て速度測定
**期待性能**: Interpreter(1x) → VM(8x) → LLVM(20x)
```nyash
// 二分木大量生成・破棄でGC性能測定
box TreeNode {
    init { left, right, value }
    birth(depth, value) { ... }
}
```

#### 2. n-body - 数値計算の王道
**目的**: 浮動小数点演算、ループ最適化効果測定
**期待性能**: Interpreter(1x) → VM(10x) → LLVM(50x)
```nyash
// 太陽系シミュレーション、重力計算
// MathBoxを活用した数値計算ベンチマーク
```

#### 3. mandelbrot - 計算+画像出力
**目的**: 純粋計算性能、ファイル出力確認
**期待性能**: Interpreter(1x) → VM(15x) → LLVM(80x)
```nyash
// フラクタル計算、PPM/PNGファイル出力
// 視覚的にJIT/LLVM効果を確認可能
```

### 🌟 Nyash特色ベンチマーク

#### 4. JSON Stream Aggregator
**目的**: プラグイン統一性、「Everything is Box」実証
**特徴**: File/Netプラグインから同じコードで処理
```nyash
// FileBoxとNetBoxから同じAPIでJSONを読み取り
// 同一コードでローカルファイルとHTTP APIに対応
```

## 📊 性能指標（現在の実測値）

| アプリ | Interpreter | VM | LLVM(予定) | 用途 |
|--------|-------------|----|-----------| -----|
| ny-echo | 1.0x | 13.5x | 50x | I/O性能 |
| ny-array-bench | 1.0x | 13.5x | 40x | 計算性能 |
| chip8_emulator | 1.0x | 13.5x | 60x | ゲーム性能 |
| enhanced_kilo_editor | 1.0x | 13.5x | 45x | エディタ性能 |
| tinyproxy | 1.0x | 13.5x | 35x | ネットワーク性能 |

## 🎯 実装ロードマップ

### ✅ 完了済み
- [x] ny-echo（基本I/O検証）
- [x] ny-array-bench（性能基準）
- [x] chip8_emulator（ゲーム・グラフィック）
- [x] enhanced_kilo_editor（実用ツール）
- [x] tinyproxy（ネットワーク）

### 🚧 実装予定（論文・ベンチマーク用）
- [ ] binary-trees（GC性能測定）
- [ ] n-body（数値計算）
- [ ] mandelbrot（視覚的ベンチマーク）
- [ ] JSON Stream Aggregator（プラグイン統一）

### 🔮 将来候補
- [ ] レイトレーサー（CPU集約的）
- [ ] Lispインタープリター（言語実装）
- [ ] 静的サイトジェネレータ（実用性）

## 🤝 貢献方法

新しいアプリケーションのアイデアや改善提案は大歓迎です！

1. 新しいアプリディレクトリを作成
2. main.nyashとtest.shを実装
3. このREADMEに追加
4. PRを送信

すべてのアプリケーションは「Everything is Box」哲学に従い、プラグインシステムを活用することを推奨します。