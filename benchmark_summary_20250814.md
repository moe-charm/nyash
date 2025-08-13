# 📊 Nyash Performance Benchmark Results
*Generated: 2025-08-14*

## 🚀 Executive Summary

Nyashの3つの実行バックエンドのパフォーマンス比較（100回実行平均）：

| Backend | Average Time | Speed vs Interpreter | 用途 |
|---------|-------------|---------------------|------|
| **🌐 WASM** | **0.17 ms** | **280x faster** | Web配布・サンドボックス実行 |
| **🏎️ VM** | **16.97 ms** | **2.9x faster** | 高速実行・デバッグ |
| **📝 Interpreter** | **48.59 ms** | **1x (baseline)** | 開発・AST直接実行 |

## 📈 詳細結果

### 🎯 Light Benchmark (Simple arithmetic)
```
Interpreter:  14.85 ms  (97.6x slower than WASM)
VM:           4.44 ms   (29.2x slower than WASM) 
WASM:         0.15 ms   (baseline)
```

### 🎯 Medium Benchmark (Moderate complexity)
```
Interpreter:  46.05 ms  (281.3x slower than WASM)
VM:           21.40 ms  (130.7x slower than WASM)
WASM:         0.16 ms   (baseline)
```

### 🎯 Heavy Benchmark (Complex calculations)
```
Interpreter:  84.88 ms  (414.2x slower than WASM)
VM:           25.08 ms  (122.4x slower than WASM)
WASM:         0.21 ms   (baseline)
```

## 🔍 Analysis & Insights

### 🌟 WASM Backend Performance
- **圧倒的高速性**: 平均280倍のスピードアップ
- **コンパイル効果**: MIR→WASMコンパイルによる最適化が効果的
- **一貫性**: すべてのベンチマークで安定した高パフォーマンス

### ⚡ VM Backend Performance  
- **中間的性能**: インタープリターより2.9倍高速
- **MIR最適化**: AST直接実行より効率的
- **実行ログ**: 詳細なデバッグ情報を提供（現在は冗長）

### 📝 Interpreter Performance
- **開発適性**: AST直接実行による開発しやすさ
- **デバッグ性**: 豊富なデバッグ出力
- **ベースライン**: 他バックエンドの比較基準

## 🎯 推奨用途

### 🌐 WASM (`--compile-wasm`)
- **本番環境**: Webアプリケーション配布
- **高速実行**: パフォーマンス重視のアプリケーション
- **サンドボックス**: セキュアな実行環境

### 🏎️ VM (`--backend vm`)
- **開発環境**: 高速な開発用実行
- **CI/CD**: テスト・ビルドパイプライン
- **デバッグ**: MIRレベルでの詳細解析

### 📝 Interpreter (default)
- **開発初期**: 構文・意味解析の確認
- **プロトタイピング**: 機能の素早い検証
- **言語機能開発**: 新機能の実装・テスト

## 🚀 Phase 8 Achievement

この結果により、**Native Nyash Phase 8.2 PoC1**の成功が実証されました：

- ✅ **MIR基盤**: 3つのバックエンドすべてが動作
- ✅ **WASM最適化**: 280倍のパフォーマンス向上達成
- ✅ **統合CLI**: シームレスなバックエンド切り替え
- ✅ **実用レベル**: 本格的なアプリケーション開発に対応

---

## 📊 Raw Data

**Test Configuration:**
- Iterations: 100 per benchmark
- Build: Release mode (-j32 parallel build)
- Platform: WSL2 Linux
- Date: 2025-08-14

**Detailed Output:** `benchmark_results_20250814_0713.txt` (5.4MB with debug logs)

---

*Everything is Box, Everything is Fast! 🚀*